use log::{debug, info, log_enabled, Level};

#[cfg(feature = "instrumentation")]
use rppal::gpio::Gpio;

use std::cmp::max;
use std::error::Error;
use std::process::Command;
use std::thread;
use std::time::Duration;

use schedule_recv::periodic;

use structopt::StructOpt;

mod alsa;
mod commands;
mod keyscan;
#[cfg(feature = "midi")]
mod midi;
mod midinotes;
mod notemap;
mod pressure;
mod synth;
mod transpose;

use crate::synth::beep;

#[derive(Debug, StructOpt)]
#[structopt(name = "haxo", about = "Make music on a haxophone", version = env!("VERGEN_GIT_DESCRIBE"), settings = &[structopt::clap::AppSettings::AllowNegativeNumbers])]
struct Opt {
    #[structopt(short, long)]
    record: bool,
    #[structopt(short, long, default_value = "/usr/share/sounds/sf2/FluidR3_GM.sf2")]
    sf2_file: String,
    #[structopt(short, long, default_value = "67")]
    prog_number: i32,
    #[structopt(short, long, default_value = "./notemap.json")]
    notemap_file: String,
    #[structopt(short, long, default_value = "-14")]
    transpose: i32,
}

#[derive(PartialEq)]
enum Mode {
    Play,
    Control,
    Transpose,
}

#[allow(dead_code)]
fn shutdown() {
    debug!("Bye...");
    Command::new("/usr/bin/sudo")
        .arg("/usr/sbin/halt")
        .status()
        .expect("failed to halt system");
}

const TICK_USECS: u32 = 2_000;

#[cfg(feature = "instrumentation")]
const GPIO_UART_RXD: u8 = 15;
#[cfg(feature = "instrumentation")]
const GPIO_UART_TXD: u8 = 14;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let opt = Opt::from_args();
    debug!("{:?}", opt);

    let (synth, _settings, _adriver) = synth::try_init(&opt.sf2_file, opt.prog_number);
    #[cfg(feature = "midi")]
    let mut midi_out = midi::MidiOut::new()?;

    let tick = periodic(Duration::from_micros(TICK_USECS as u64));
    // Use UART RXD pin to monitor timing of periodic task.  This is easily
    // accessible on the haxophone HAT when the console is disabled.
    #[cfg(feature = "instrumentation")]
    let mut busy_pin = Gpio::new()?.get(GPIO_UART_RXD)?.into_output();
    #[cfg(feature = "instrumentation")]
    let mut noteon_pin = Gpio::new()?.get(GPIO_UART_TXD)?.into_output();

    println!(
        "Starting haxophone (version {})...",
        env!("VERGEN_GIT_DESCRIBE")
    );

    keyscan::init_io().expect("Failed to initialize scan GPIO");
    let mut sensor = pressure::Pressure::init().expect("Failed to initialize pressure sensor");

    let mut notemap = notemap::NoteMap::generate(&opt.notemap_file, opt.transpose);
    if opt.record {
        notemap.start_recording();
    }

    let mut last_note = 0;
    let mut mode = Mode::Play;
    let mut cmd = commands::Command::new(&synth, opt.prog_number);

    const TRANSPOSE_COUNTDOWN_MS: u32 = 200u32;
    const TRANSPOSE_COUNTDOWN_TICKS: u32 = TRANSPOSE_COUNTDOWN_MS * 1000 / TICK_USECS;
    let mut transpose = transpose::Transpose::new(&synth, TRANSPOSE_COUNTDOWN_TICKS);

    const NEG_PRESS_COUNTDOWN_MS: u32 = 500u32;
    const NEG_PRESS_INIT_VAL: u32 = NEG_PRESS_COUNTDOWN_MS * 1000 / TICK_USECS;
    let mut neg_pressure_countdown: u32 = NEG_PRESS_INIT_VAL;
    let mut last_vol: i32 = 0;
    loop {
        tick.recv().unwrap();
        #[cfg(feature = "instrumentation")]
        busy_pin.set_high();

        let keys = keyscan::scan()?;
        let pressure = sensor.read()?;
        let vol = max(0, pressure);
        const MIDI_CC_VOLUME: i32 = 7;
        const MIDI_CC_BREATH: i32 = 2;
        if last_vol != vol {
            synth.cc(0, MIDI_CC_VOLUME, vol);
            #[cfg(feature = "midi")]
            midi_out.cc(MIDI_CC_BREATH, vol);
            last_vol = vol;
        }

        if notemap.is_recording() {
            notemap.record(keys, pressure);
        }

        if mode == Mode::Control {
            cmd.process(keys);
        } else if mode == Mode::Transpose {
            transpose.process(keys, vol, &mut notemap);
        }

        if mode != Mode::Play {
            // All three left hand palm keys pressed at once
            if keys == 0x124 {
                beep(&synth, 70, 50);
                mode = Mode::Play;
                info!("Return to Play Mode");
            }
            continue;
        }

        if let Some(note) = notemap.get(&keys) {
            if last_note != note {
                if log_enabled!(Level::Debug) {
                    debug!(
                        "Note: {} Pressure: {} Key {:032b}: {}",
                        notemap.get_name(&note).unwrap_or("Unknown?"),
                        pressure,
                        keys,
                        keys
                    );
                };
                if vol > 0 {
                    if last_note > 0 {
                        synth.cc(0, MIDI_CC_VOLUME, 0);
                        synth.noteoff(0, last_note);
                        #[cfg(feature = "midi")]
                        midi_out.noteoff(last_note);
                        #[cfg(feature = "instrumentation")]
                        noteon_pin.set_low();
                        synth.cc(0, MIDI_CC_VOLUME, vol);
                    }
                    synth.noteon(0, note, 127);
                    #[cfg(feature = "midi")]
                    midi_out.noteon(note, 127);
                    #[cfg(feature = "instrumentation")]
                    noteon_pin.set_high();
                    last_note = note;
                    debug!("last_note changed to {}", last_note);
                }
            }
            if vol <= 0 && last_note > 0 {
                synth.noteoff(0, last_note);
                #[cfg(feature = "midi")]
                midi_out.noteoff(last_note);
                #[cfg(feature = "instrumentation")]
                noteon_pin.set_low();
                last_note = 0;
            }

            // Negative pressure needs to hold for a minimum duration to trigger a mode change
            if pressure < -10 {
                neg_pressure_countdown = neg_pressure_countdown.wrapping_sub(1);
            } else {
                neg_pressure_countdown = NEG_PRESS_INIT_VAL;
            }

            // Enter Control Mode
            if neg_pressure_countdown == 0 {
                match notemap.get_name(&note) {
                    Some("Low Bb") => {
                        mode = Mode::Control;
                        beep(&synth, 71, 50);
                        info!("Enter Control Mode");
                    }
                    Some("Low B") => {
                        mode = Mode::Transpose;
                        beep(&synth, 71, 50);
                        thread::sleep(Duration::from_millis(20));
                        beep(&synth, 75, 50);
                        info!("Enter Transpose Mode");
                    }
                    _ => {}
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
