extern crate fluidsynth;
extern crate rand;
extern crate time;

#[macro_use]
extern crate static_assertions;

use log::{debug, info, warn, error};

use fluidsynth::{synth, settings, audio};
use std::collections::HashMap;
use std::error::Error;
use std::thread;
use std::time::Duration;

use rppal::gpio::Gpio;
use rppal::gpio::Level;
use rppal::system::DeviceInfo;

// BCM pin numbering
const ROWS : [u8; 8] = [14,15,16,17,18,22,23,24];
const COLS : [u8; 4] = [25,26,27,4];

fn try_init_synth() -> (synth::Synth, settings::Settings, audio::AudioDriver) {
    let mut settings = settings::Settings::new();
    // try to optimize for low latency
    if !settings.setstr("audio.driver", "alsa") {
        warn!("Setting audio.driver in fluidsynth failed");
    }
    if !settings.setint("audio.periods", 3) {
        warn!("Setting audio.periods in fluidsynth failed");
    }
    if !settings.setint("audio.period-size", 444) {
        warn!("Setting audio.period-size in fluidsynth failed");
    }
    if !settings.setstr("audio.alsa.device", "hw:0") {
        warn!("Setting audio.alsa.device in fluidsynth failed");
    }
    if !settings.setint("audio.realtime-prio", 99) {
        warn!("Setting audio.realtime-prio in fluidsynth failed");
    }
    let mut syn = synth::Synth::new(&mut settings);
    // supposedly, assign tenor sax patch to midi channel 0
    syn.program_change(0, 67);
    if !syn.set_polyphony(1) {
        warn!("Failed to set polyphony to 1");
    }
    let adriver = audio::AudioDriver::new(&mut settings, &mut syn);
    //syn.sfload("/usr/share/sounds/sf2/FluidR3_GM.sf2", 1);
    syn.sfload("/usr/share/sounds/sf2/TimGM6mb.sf2", 1);
    println!("Synth created");
    (syn, settings, adriver)
}

fn init_scan_io() -> Result<(), Box<dyn Error>> {
    let gpio = Gpio::new()?;
    for col in &COLS {
        let mut pin = gpio.get(*col)?.into_input_pullup();
        pin.set_reset_on_drop(false);
    }
    for row in &ROWS {
        let mut pin = gpio.get(*row)?.into_output();
        pin.set_high();
        pin.set_reset_on_drop(false);
    }
    Ok(())
}

fn get_bit_at(input: u32, n: u8) -> bool {
    if n < 32 {
        input & (1 << n) != 0
    } else {
        false
    }
}

fn set_bit_at(output: &mut u32, n: u8) {
    if n < 32 {
        *output |= 1 << n; 
    }
}

fn clear_bit_at(output: &mut u32, n: u8) {
    if n < 32 {
        *output &= !(1 << n); 
    }
}

fn scan_keys() -> Result<u32, Box<dyn Error>> {
    const_assert!(ROWS.len() + COLS.len() <= 32); 
    let gpio = Gpio::new()?;
    let mut key_idx = 0;
    // a bit if set if the corresponding key is pressed
    let mut keymap :u32 = 0;
    for row in &ROWS {
        let mut row_pin = gpio.get(*row)?.into_output();
        row_pin.set_low();

        for col in &COLS {
            let col_pin = gpio.get(*col)?;
            let is_pressed = col_pin.read() == Level::Low;

            if get_bit_at(keymap, key_idx) != is_pressed {
                if is_pressed {
                    set_bit_at(&mut keymap, key_idx);
                } else {
                    clear_bit_at(&mut keymap, key_idx);
                }
            }
            key_idx += 1; 
        }
        row_pin.set_high();
    }
    Ok(keymap)
} 



fn gen_notemap() -> HashMap<u32,i32> {
    let mut notemap = HashMap::new();
    // silence
    notemap.insert(0,0);
    notemap.insert(656548352,46); // Bb
    notemap.insert(640819712,47); // B
    notemap.insert(639771136,48); // C
    notemap.insert(639836672,49); // C#
    notemap.insert(572662272,50); // D
    notemap.insert(39985664,51);  // Eb
    notemap.insert(35791360, 52); // E
    notemap.insert(2236928, 53);  // F
    notemap.insert(33694208, 54); // F#
    notemap.insert(2499072, 54);  // F#
    notemap.insert(139776, 55);   // G
    notemap.insert(143872, 56);   // G#
    notemap.insert(8704, 57);     // A
    notemap.insert(33554944, 58); // Bb
    notemap.insert(544, 58);      // Bb
    notemap.insert(2097664, 58);  // Bb
    notemap.insert(8960, 58);     // Bb
    notemap.insert(512, 59);      // B
    notemap.insert(8192, 60);     // C
    notemap.insert(528, 60);      // C
    notemap
}

fn main() -> Result<(), Box<dyn Error>> {

    env_logger::init();

    let (syn, _settings, _adriver) = try_init_synth();

    println!("Scanning haxophone a {}", DeviceInfo::new()?.model());

    init_scan_io().expect("Failed to initialize scan GPIO");

    let notemap = gen_notemap();

    let mut last_keys :u32 = 0;
    let mut last_note = 0;
    loop {
        thread::sleep(Duration::from_millis(10));

            let keys = scan_keys()?;
            if last_keys != keys {
                debug!("Key event {:032b}: {}", keys, keys);
                if let Some(note) = notemap.get(&keys) {
                    // until we have breadth control, assume all keys unpressed means silence
                    if *note > 0 {
                        syn.noteon(0, *note, 127);
                    }
                    // make before break
                    let ret = syn.noteoff(0, last_note);
                    last_note = *note;
                    debug!("last_note changed to {}", last_note);
                }
                last_keys = keys;
            }
        }
}
