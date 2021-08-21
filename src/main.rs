extern crate time;

#[macro_use]
extern crate static_assertions;
extern crate structopt;

use log::{debug, log_enabled, Level};

use std::cmp::max;
use std::error::Error;
use std::process::Command;
use std::thread;
use std::time::Duration;

use structopt::StructOpt;

mod keyscan;
mod midinotes;
mod notemap;
mod pressure;
mod synth;

#[derive(Debug, StructOpt)]
#[structopt(name = "haxo", about = "Make music on a haxophone")]
struct Opt {
    #[structopt(short, long)]
    record: bool,
}

fn shutdown() {
    debug!("Bye...");
    Command::new("/usr/bin/sudo")
        .arg("/usr/sbin/halt")
        .status()
        .expect("failed to halt system");
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let opt = Opt::from_args();
    println!("{:?}", opt);

    let (synth, _settings, _adriver) = synth::try_init();

    println!("Starting haxophone...");

    keyscan::init_io().expect("Failed to initialize scan GPIO");
    let mut sensor = pressure::Pressure::init().expect("Failed to initialize pressure sensor");

    // Recording variables
    let mut recording = opt.record;
    let mut recording_index = 0;
    let mut last_keys = 0;
    let mut last_recorded = 0;
    let mut record_next = false;
    // End of recording variables

    let mut notemap = notemap::generate();

    let mut last_note = 0;
    loop {
        thread::sleep(Duration::from_millis(50));

        let keys = keyscan::scan()?;
        let pressure = sensor.read()?;
        let vol = max(0, pressure);
        const MIDI_CC_VOLUME: i32 = 7;
        synth.cc(0, MIDI_CC_VOLUME, vol);

        if recording {
            if pressure > 10 && last_recorded != keys {
                notemap.insert(keys, midinotes::NOTES[recording_index].1);
                last_recorded = keys;
                println!(
                    "Keymap {} recorded for {}",
                    keys,
                    midinotes::NOTES[recording_index].0
                );
                notemap::save(&notemap);
                record_next = true;
            }

            if record_next {
                recording_index += 1;
                record_next = false;
                if recording_index == midinotes::NOTES.len() {
                    recording = false;
                    recording_index = 0;
                    println!("Done recording keymaps");
                } else {
                    println!("Next note is {}", midinotes::NOTES[recording_index].0);
                    println!("Draw to go back to previous note to add an alternate fingering.");
                }
            }

            if pressure < -10 && recording_index > 0 {
                recording_index -= 1;
                println!("Back to {}", midinotes::NOTES[recording_index].0);
                thread::sleep(Duration::from_millis(1000));
            }

            if keys != last_keys {
                if pressure < 10 && pressure > -10 {
                    println!(
                        "Blow to record this keymap ({}) for {}",
                        keys,
                        midinotes::NOTES[recording_index].0
                    );
                }
                last_keys = keys;
            }
        }

        if let Some(note) = notemap.get(&keys) {
            if last_note != *note {
                if log_enabled!(Level::Debug) {
                    debug!(
                        "Pressure: {} Key {:032b}: {} Note: {}",
                        pressure,
                        keys,
                        keys,
                        midinotes::get_name(note).unwrap_or("Unknown!?")
                    );
                };
                if vol > 0 {
                    synth.noteon(0, *note, 127);
                    last_note = *note;
                    debug!("last_note changed to {}", last_note);
                }
            }
            if vol <= 0 && last_note > 0 {
                synth.noteoff(0, last_note);
                last_note = 0;
            }
            if *note < 0 {
                // TODO: pick the right control messages.  For now, only one is supported
                shutdown();
                return Ok(());
            }
        }
    }
}
