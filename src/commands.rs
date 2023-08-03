use log::info;
use fluidsynth::synth::Synth;

#[derive(Copy,Clone,PartialEq)]
enum CommandKeys {
    Digit0 = 0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    ChangeBankKey,
    Enter,
    Unmapped,
}

#[derive(Debug,Copy,Clone,PartialEq)]
enum CommandEnum {
    ChangeBank,
    NoCommand,
}

const MAX_VAL_DIGITS: usize = 4;

fn key2cmdkey(key: u32) -> CommandKeys {
    match key {
        2 => CommandKeys::Digit1,
        16 => CommandKeys::Digit2,
        128 => CommandKeys::Digit2,
        1024 => CommandKeys::Digit3,
        8192 => CommandKeys::Digit4,
        65536 => CommandKeys::Digit5,
        524288 => CommandKeys::Digit6,
        4194304 => CommandKeys::Digit7,
        4096 => CommandKeys::Digit8,
        32768 => CommandKeys::Digit9,
        262144 => CommandKeys::Digit0,
        16384 => CommandKeys::ChangeBankKey,
        8388608 => CommandKeys::Enter,
        _ => CommandKeys::Unmapped,
    }
}

pub(crate) struct Command<'a> {
    synth: &'a Synth,
    values: [u32; MAX_VAL_DIGITS],
    val_idx: usize,
    last_cmd_key: CommandKeys,
    command: CommandEnum,
}

impl<'a> Command<'a>{
    pub(crate) fn new(synth: &'a Synth) -> Self {
        Command {
            synth: synth,
            values: [0; MAX_VAL_DIGITS],
            val_idx: 0,
            last_cmd_key: CommandKeys::Unmapped,
            command: CommandEnum::NoCommand,
        }
    }
    pub(crate) fn process(self: &mut Self, key: u32) {
        let cmd_key = key2cmdkey(key);
        if cmd_key == self.last_cmd_key {
            return;
        }
        self.last_cmd_key = cmd_key;

        self.command = match cmd_key {
            CommandKeys::ChangeBankKey => {
                info!("Starting ChangeBank command");
                self.val_idx = 0;
                CommandEnum::ChangeBank
            },
            CommandKeys::Enter => { info!("Executing {:?} with vals {:?}", self.command, self.values); CommandEnum::NoCommand },
            _ => self.command
        };

        if self.command == CommandEnum::ChangeBank {
            let digit_value = match cmd_key {
                CommandKeys::Digit0 => Some(0),
                CommandKeys::Digit1 => Some(1),
                CommandKeys::Digit2 => Some(2),
                CommandKeys::Digit3 => Some(3),
                CommandKeys::Digit4 => Some(4),
                CommandKeys::Digit5 => Some(5),
                CommandKeys::Digit6 => Some(6),
                CommandKeys::Digit7 => Some(7),
                CommandKeys::Digit8 => Some(8),
                CommandKeys::Digit9 => Some(9),
                _ => None,
            };
            if digit_value.is_some() && self.val_idx < MAX_VAL_DIGITS {
                self.values[self.val_idx] = digit_value.unwrap();
                self.val_idx += 1;
            }
            return;
        }
    }
}
                    // Some("Low F") => {
                    //     if control_mode {
                    //         control_mode = false;
                    //         current_bank = max(0, current_bank - 1);
                    //         synth.program_change(0, current_bank);
                    //         info!("New MIDI bank number {}", current_bank);
                    //         synth.noteon(0, 51, 127);
                    //         synth.cc(0, MIDI_CC_VOLUME, 127);
                    //         thread::sleep(Duration::from_millis(100));
                    //         synth.noteoff(0, 51);
                    //     }
                    // }
                    // Some("Low G") => {
                    //     if control_mode {
                    //         control_mode = false;
                    //         current_bank = min(128, current_bank + 1);
                    //         synth.program_change(0, current_bank);
                    //         info!("New MIDI bank number {}", current_bank);
                    //         synth.noteon(0, 53, 127);
                    //         synth.cc(0, MIDI_CC_VOLUME, 127);
                    //         thread::sleep(Duration::from_millis(100));
                    //         synth.noteoff(0, 53);
                    //     }
                    // }

                    // Some("Low C") => {
                    //     if control_mode {
                    //         control_mode = false;
                    //         info!("Shutting down");
                    //         synth.noteon(0, 46, 127);
                    //         synth.cc(0, MIDI_CC_VOLUME, 127);
                    //         thread::sleep(Duration::from_millis(100));
                    //         synth.noteoff(0, 46);
                    //         shutdown();
                    //     }
                    // }
                    // _ => {
                        // control_command = false;
                    // }
