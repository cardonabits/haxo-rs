extern crate fluidsynth;

use fluidsynth::{audio, midi, settings, synth};
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

    const FSYNTH_GAIN: f32 = 1.0;
    syn.set_gain(FSYNTH_GAIN);
    if syn.get_gain() != FSYNTH_GAIN {
        warn!("Failed to set gain to {}", FSYNTH_GAIN);
    }

    let adriver = audio::AudioDriver::new(&mut settings, &mut syn);

    // Use FluidR3_GM.sf2 with multiple instruments for midi file
    let startup_sf2_filename = "/usr/share/sounds/sf2/FluidR3_GM.sf2";
    let sf2startup = syn.sfload(startup_sf2_filename, 1);

    if sf2startup == None {
        warn!("Failed to load sound font file {}", startup_sf2_filename);
    }

    // Enable polyphony for midi file
    if !syn.set_polyphony(16) {
        warn!("Failed to set polyphony to 16");
    }
    
    // Select bank 0
    syn.program_change(0, 0);

	// Play the midi file
    let player = midi::Player::new(&mut syn);
    player.add("/usr/share/haxo/Startup_Haxophone.mid");
    player.play();

    // Wait until midi file is finished
    while player.get_status() ==  midi::PlayerStatus::Playing {
        // wait ..
    }
    
    // Switch off polyphony for sax
    if !syn.set_polyphony(1) {
        warn!("Failed to set polyphony to 1");
    }

    let sf2 = syn.sfload(sf2file, 1);

    if sf2 == None {
        warn!("Failed to load sound font file {}", sf2file);
    }
    // select bank number
    syn.program_change(0, banknum);
    println!("Synth created");
    (syn, settings, adriver)
}
