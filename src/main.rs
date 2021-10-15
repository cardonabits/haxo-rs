use log::{debug, info, log_enabled, Level};

#[cfg(feature = "instrumentation")]
use rppal::gpio::Gpio;

use std::cmp::{max, min};
use std::error::Error;
use std::process::Command;
use std::thread;
use std::time::Duration;

use schedule_recv::periodic;

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
    #[structopt(short, long, default_value = "/usr/share/sounds/sf2/FluidR3_GM.sf2")]
    sf2_file: String,
    #[structopt(short, long, default_value = "67")]
    bank_number: i32,
    #[structopt(short, long, default_value = "./notemap.json")]
    notemap_file: String,
}

#[allow(dead_code)]
fn shutdown() {
    debug!("Bye...");
    Command::new("/usr/bin/sudo")
        .arg("/usr/sbin/halt")
        .status()
        .expect("failed to halt system");
}

const TICK_USECS: u64 = 2_000;

// Limit the rate at which notes can be triggered to a really fast speed on a
// saxophone: 10 notes per second, or 16th notes at 150 bpm.  This
// avoids clicks and ugly artifacts.
const MIN_NOTE_DURATION: u64 = 100_000;

#[cfg(feature = "instrumentation")]
const GPIO_UART_RXD: u8 = 15;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let opt = Opt::from_args();
    debug!("{:?}", opt);

    let (synth, _settings, _adriver) = synth::try_init(&opt.sf2_file, opt.bank_number);
    let mut current_bank = opt.bank_number;

    let tick = periodic(Duration::from_micros(TICK_USECS));
    let mut rate_limit = 0;
    // Use UART RXD pin to monitor timing of periodic task.  This is easily
    // accessible on the haxophone HAT when the console is disabled.
    #[cfg(feature = "instrumentation")]
    let mut busy_pin = Gpio::new()?.get(GPIO_UART_RXD)?.into_output();

    println!("Starting haxophone...");

    keyscan::init_io().expect("Failed to initialize scan GPIO");
    let mut sensor = pressure::Pressure::init().expect("Failed to initialize pressure sensor");

    let mut notemap = notemap::NoteMap::generate(&opt.notemap_file);
    if opt.record {
        notemap.start_recording();
    }

    let mut last_note = 0;
    let mut control_command = false;
    loop {
        tick.recv().unwrap();
        #[cfg(feature = "instrumentation")]
        busy_pin.set_high();

        if rate_limit > 0 {
            rate_limit -= 1;
        }

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
                if vol > 0 && rate_limit == 0 {
                    synth.noteon(0, *note, 127);
                    rate_limit = MIN_NOTE_DURATION / TICK_USECS;
                    last_note = *note;
                    debug!("last_note changed to {}", last_note);
                }
            }
            if vol <= 0 && last_note > 0 {
                synth.noteoff(0, last_note);
                last_note = 0;
            }

            // Control commands
            if pressure < -10 {
                match midinotes::get_name(note) {
                    Some("Low Bb") => {
                        control_command = true;
                        info!("Prepared to receive for control command");
                    }
                    Some("Low F") => {
                        if control_command {
                            control_command = false;
                            current_bank = max(0, current_bank - 1);
                            synth.program_change(0, current_bank);
                            info!("New MIDI bank number {}", current_bank);
                            synth.noteon(0, 51, 127);
                            synth.cc(0, MIDI_CC_VOLUME, 127);
                            thread::sleep(Duration::from_millis(100));
                            synth.noteoff(0, 51);
                        }
                    }
                    Some("Low G") => {
                        if control_command {
                            control_command = false;
                            current_bank = min(128, current_bank + 1);
                            synth.program_change(0, current_bank);
                            info!("New MIDI bank number {}", current_bank);
                            synth.noteon(0, 53, 127);
                            synth.cc(0, MIDI_CC_VOLUME, 127);
                            thread::sleep(Duration::from_millis(100));
                            synth.noteoff(0, 53);
                        }
                    }

                    Some("Low C") => {
                        if control_command {
                            control_command = false;
                            info!("Shutting down");
                            synth.noteon(0, 46, 127);
                            synth.cc(0, MIDI_CC_VOLUME, 127);
                            thread::sleep(Duration::from_millis(100));
                            synth.noteoff(0, 46);
                            shutdown();
                        }
                    }
                    _ => {
                        control_command = false;
                    }
                }
            }
        } else {
            if log_enabled!(Level::Debug) {
                keyscan::debug_print(keys);
            }
        }
        #[cfg(feature = "instrumentation")]
        busy_pin.set_low();
    }
}
