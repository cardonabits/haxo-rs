extern crate rand;
extern crate time;

#[macro_use]
extern crate static_assertions;

use log::{debug, /* error, info, warn */};

use std::cmp::{max, min};
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

    let notemap = notemap::generate();

    let mut last_note = 0;
    loop {
        thread::sleep(Duration::from_millis(50));

        let keys = keyscan::scan()?;
        let pressure = sensor.read()?;
        let vol = min(max(0, pressure), 127);
        const MIDI_CC_VOLUME: i32 = 7;
        synth.cc(0, MIDI_CC_VOLUME, vol);

        if let Some(note) = notemap.get(&keys) {
            if last_note != *note {
                debug!("Pressure: {} Key {:032b}: {}", pressure, keys, keys);
                keyscan::debug_print(keys);
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
