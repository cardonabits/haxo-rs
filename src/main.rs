extern crate fluidsynth;
extern crate rand;
extern crate time;

use fluidsynth::*;
use std::error::Error;
use std::thread;
use std::time::Duration;

use rppal::gpio::Gpio;
use rppal::gpio::Level;
use rppal::system::DeviceInfo;

fn try_init_synth() -> (synth::Synth, settings::Settings, audio::AudioDriver) {
    let mut settings = settings::Settings::new();
    settings.setstr("audio.driver", "alsa");
    let mut syn = synth::Synth::new(&mut settings);
    // supposedly, assign tenor sax patch to midi channel 0
    syn.program_change(0, 67);
    let adriver = audio::AudioDriver::new(&mut settings, &mut syn);
    syn.sfload("/usr/share/sounds/sf2/FluidR3_GM.sf2", 1);
    println!("Synth created");
    (syn, settings, adriver)
}

fn main() -> Result<(), Box<dyn Error>> {

    let (syn, _settings, _adriver) = try_init_synth();

    // BCM pin numbering
    const ROWS : [u8; 5] = [3, 4, 14, 15, 18];
    const COLS : [u8; 1] = [2];
    let mut pressed = [[false; ROWS.len()]; COLS.len()];

    println!("Scanning haxophone a {}", DeviceInfo::new()?.model());

    for row in &ROWS {
        let _pin = Gpio::new()?.get(*row)?.into_input();
    }
    for col in &COLS {
        let _pin = Gpio::new()?.get(*col)?.into_input_pullup();
    }

    let mut last_note :i32 = 0;
    loop {
        let mut event = false;
        thread::sleep(Duration::from_millis(50));
        let mut rindex = 0;
        let mut cindex = 0;
        for row in &ROWS {
            let mut row_pin = Gpio::new()?.get(*row)?.into_output();
            row_pin.set_low();

            for col in &COLS {
                let col_pin = Gpio::new()?.get(*col)?.into_input();
                let is_pressed = col_pin.read() == Level::Low;

                if pressed[cindex][rindex] != is_pressed {
                    pressed[cindex][rindex] = is_pressed;
                    event = true;
                }
                cindex += 1;
                if cindex == COLS.len() {
                    cindex = 0;
                }
            }

            rindex += 1;
            if rindex == ROWS.len() {
                rindex = 0;
            }

            // compute a single note from all the pressed keys
            if event {
                let mut note :i32 = 50; 
                if last_note > 0 {
                    syn.noteoff(0, last_note);
                    println!("Note {} off", last_note);
                }
                for i in 0..ROWS.len() {
                    if pressed[0][i] {
                        note += 1 + i as i32;
                    }
                }

                // until we have breath control, assume no keys means silence
                if note != 50 {
                    syn.noteon(0, note, 80);
                    println!("Note {} on", note);
                }
                last_note = note;
            }

            // on drop row_pin will be reset to input
        }
    }
}
