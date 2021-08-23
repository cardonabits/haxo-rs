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

#[allow(dead_code)]
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

    let mut notemap = notemap::NoteMap::generate();
    if opt.record {
        notemap.start_recording();
    }

    let mut last_note = 0;
    loop {
        thread::sleep(Duration::from_millis(50));

        let keys = keyscan::scan()?;
        let pressure = sensor.read()?;
        let vol = max(0, pressure);
        const MIDI_CC_VOLUME: i32 = 7;
        synth.cc(0, MIDI_CC_VOLUME, vol);

        if notemap.is_recording() {
            notemap.record(keys, pressure);
        }

        if let Some(note) = notemap.get(&keys) {
            if last_note != *note {
                if log_enabled!(Level::Debug) {
                    debug!(
                        "Note: {} Pressure: {} Key {:032b}: {}",
                        midinotes::get_name(note).unwrap_or("Unknown?"),
                        pressure,
                        keys,
                        keys
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
        }
    }
}
