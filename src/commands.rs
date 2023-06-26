use std::cmp::{min,max};
use std::time::Duration;
use std::thread;

use log::info;
use fluidsynth::synth::Synth;

#[derive(Copy,Clone,PartialEq)]
enum CommandKeys {
    ChangeProgUp,
    ChangeProgFastUp,
    ChangeProgDown,
    ChangeProgFastDown,
    Unmapped,
}

const MIDI_CC_VOLUME: i32 = 7;

fn key2cmdkey(key: u32) -> CommandKeys {
    match key {
        0x10000 => CommandKeys::ChangeProgUp,
        0x90000 => CommandKeys::ChangeProgFastUp,
        0x400000 => CommandKeys::ChangeProgDown,
        0x480000 => CommandKeys::ChangeProgFastDown,
        // 0x800000=> CommandKeys::ChangeVolume,
        _ => CommandKeys::Unmapped,
    }
}

pub(crate) struct Command<'a> {
    synth: &'a Synth,
    prog_number: i32,
    last_cmd_key: CommandKeys,
}

impl<'a> Command<'a>{
    pub(crate) fn new(synth: &'a Synth, prog_number: i32) -> Self {
        Command {
            synth: synth,
            prog_number: prog_number,
            last_cmd_key: CommandKeys::Unmapped,
        }
    }
    pub(crate) fn process(self: &mut Self, key: u32) {
        let cmd_key = key2cmdkey(key);
        if cmd_key == self.last_cmd_key {
            return;
        }
        self.last_cmd_key = cmd_key;

        match cmd_key {
            CommandKeys::ChangeProgUp => self.change_program(1),
            CommandKeys::ChangeProgFastUp => self.change_program(10),
            CommandKeys::ChangeProgDown => self.change_program(-1),
            CommandKeys::ChangeProgFastDown=> self.change_program(-10),
            _ => (), 
        };
    }

    fn change_program(self: &mut Self, change:i32) {
            self.prog_number = max(0, min(127, self.prog_number + change));
            self.synth.program_change(0, self.prog_number);
            info!("New MIDI program number {}", self.prog_number);
            self.synth.noteon(0, 53, 60);
            self.synth.cc(0, MIDI_CC_VOLUME, 60);
            thread::sleep(Duration::from_millis(100));
            self.synth.noteoff(0, 53);
    }
}
