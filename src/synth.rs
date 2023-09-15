extern crate fluidsynth;

use fluidsynth::{audio, settings, synth};
use log::{info, warn};

use crate::alsa;

pub fn try_init(
    sf2file: &str,
    banknum: i32,
) -> (synth::Synth, settings::Settings, audio::AudioDriver) {
    let mut settings = settings::Settings::new();
    // try to optimize for low latency
    if settings.setstr("audio.driver", "alsa") {
        warn!("Setting audio.driver in fluidsynth failed");
    }
    if settings.setint("audio.periods", 3) {
        warn!("Setting audio.periods in fluidsynth failed");
    }
    if settings.setint("audio.period-size", 64) {
        warn!("Setting audio.period-size in fluidsynth failed");
    }

    let alsa_dev = alsa::get_device();
    if alsa_dev.is_err() {
        panic!("Failed to find audio output device");
    }
    let alsa_dev = alsa_dev.unwrap();
    if settings.setstr("audio.alsa.device", &alsa_dev) {
        warn!("Failed to attach synth to headphone output {}", &alsa_dev);
    }
    if settings.setint("audio.realtime-prio", 99) {
        info!("Setting audio.realtime-prio in fluidsynth failed");
    }
    let mut syn = synth::Synth::new(&mut settings);
    if !syn.set_polyphony(1) {
        warn!("Failed to set polyphony to 1");
    }
    const FSYNTH_GAIN: f32 = 1.0;
    syn.set_gain(FSYNTH_GAIN);
    if syn.get_gain() != FSYNTH_GAIN {
        warn!("Failed to set gain to {}", FSYNTH_GAIN);
    }

    let adriver = audio::AudioDriver::new(&mut settings, &mut syn);
    let sf2 = syn.sfload(sf2file, 1);

    if sf2 == None {
        warn!("Failed to load sound font file {}", sf2file);
    }
    // select bank number
    syn.program_change(0, banknum);
    println!("Synth created");
    (syn, settings, adriver)
}
