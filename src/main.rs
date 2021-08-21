extern crate time;

#[macro_use]
extern crate static_assertions;

use log::debug;

use std::cmp::max;
use std::error::Error;
use std::process::Command;
use std::thread;
use std::time::Duration;

mod keyscan;
mod notemap;
mod pressure;
mod synth;

fn shutdown() {
    debug!("Bye...");
    Command::new("/usr/bin/sudo")
        .arg("/usr/sbin/halt")
        .status()
        .expect("failed to halt system");
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let (synth, _settings, _adriver) = synth::try_init();

    println!("Starting haxophone...");

    keyscan::init_io().expect("Failed to initialize scan GPIO");
    let mut sensor = pressure::Pressure::init().expect("Failed to initialize pressure sensor");

    // Note: Assuming Bb instruments
    let notes = [
        ("Low Bb", 44),
        ("Low B", 45),
        ("Low C", 46),
        ("Low C#", 47),
        ("Low D", 48),
        ("Low D#", 49),
        ("Low E", 50),
        ("Low F", 51),
        ("Low F#", 52),
        ("Low G", 53),
        ("Low Ab", 54),
        ("Low A", 55),
        ("Mid Bb", 56),
        ("Mid B", 57),
        ("Mid C", 58),
        ("Mid C#", 59),
        ("Mid D", 60),
        ("Mid D#", 61),
        ("Mid E", 63),
        ("Mid F", 64),
        ("Mid F#", 65),
        ("Mid G", 66),
        ("Mid Ab", 67),
        ("Mid A", 68),
        ("High Bb", 69),
        ("High B", 70),
        ("High C", 71),
        ("High C#", 72),
        ("High D", 73),
        ("High D#", 74),
        ("High E", 75),
        ("High F", 76),
        ("High F#", 77),
    ];
    // Recording variables
    let mut recording = true;
    let mut recording_index = 0;
    let mut last_keys = 0;
    let mut last_recorded = 0;
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
                notemap.insert(keys, notes[recording_index].1);
                last_recorded = keys;
                println!("Keymap {} recorded for {}", keys, notes[recording_index].0);
                notemap::save(&notemap);
            }

            if pressure < -10 && keys == 0 && last_keys != 0 {
                recording_index += 1;
                if recording_index == notes.len() {
                    recording = false;
                    recording_index = 0;
                    println!("Done recording keymaps");
                } else {
                    println!("Next note is {}", notes[recording_index].0);
                }
            }

            if keys != last_keys {
                if pressure < 10 && pressure > -10 {
                    println!(
                        "Blow to record this keymap ({}) for {}",
                        keys, notes[recording_index].0
                    );
                    println!("Release all keys while sucking to move on to next note.");
                }
                last_keys = keys;
            }
        }

        if let Some(note) = notemap.get(&keys) {
            if last_note != *note {
                debug!("Pressure: {} Key {:032b}: {}", pressure, keys, keys);
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
