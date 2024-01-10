use std::thread;
use std::time::Duration;

use fluidsynth::synth::Synth;
use log::info;

use crate::notemap::NoteMap;
use crate::synth::beep;

#[derive(Copy, Clone, PartialEq)]
enum TransposeCmd {
    HalfStepUp,
    HalfStepDown,
    Direct(i32),
    None,
}

// To set the transpose directly from a note we measure how far away from
// the reference the note is. This is then used as the offset. We use high C
// as the reference so we can fit soprano (High Bb -> -2), alto (Mid Eb -> -9),
// tenor (Mid Bb, -> -14) and bari (Low Eb -> -21) within the reachable range.
const TRANSPOSE_REFERENCE: i32 = 84;

// Get the command associated with the given key state and volume.
// For 'direct' transposition, the note must be played.
fn get_cmd(key: u32, vol: i32, notemap: &NoteMap) -> TransposeCmd {
    match (key, notemap.get_untransposed(&key), vol) {
        (_, Some(note), v) if v > 10 => TransposeCmd::Direct(note),
        (0x10000, None, _) => TransposeCmd::HalfStepUp,
        (0x400000, None, _) => TransposeCmd::HalfStepDown,
        _ => TransposeCmd::None,
    }
}

pub(crate) struct Transpose<'a> {
    synth: &'a Synth,
    // Minimum time keys must be held to compute transpose.
    countdown_init: u32,

    // The command that will be applied when the timer expires.
    cmd: TransposeCmd,
    // How much time remains until command will be applied.
    countdown: u32,
    // Has the command already been applied.
    applied: bool,
}

impl<'a> Transpose<'a> {
    pub(crate) fn new(synth: &'a Synth, countdown_init: u32) -> Self {
        Transpose {
            synth,
            countdown_init,
            cmd: TransposeCmd::None,
            countdown: 0,
            applied: false,
        }
    }

    pub(crate) fn process(self: &mut Self, key: u32, vol: i32, notemap: &mut NoteMap) {
        let cur_cmd = get_cmd(key, vol, notemap);

        // When the command changes, restart the countdown.
        if cur_cmd != self.cmd {
            self.cmd = cur_cmd;
            self.countdown = self.countdown_init;
            self.applied = false;
            return;
        }

        self.countdown = self.countdown.saturating_sub(1);
        if self.countdown > 0 || self.applied {
            return;
        }

        // Apply the command.
        match self.cmd {
            TransposeCmd::HalfStepUp => self.offset(1, notemap),
            TransposeCmd::HalfStepDown => self.offset(-1, notemap),
            TransposeCmd::Direct(note) => self.direct(note, notemap),
            TransposeCmd::None => (),
        };
        self.applied = true;
    }

    fn set_transpose(self: &mut Self, transpose: i32, notemap: &mut NoteMap) {
        notemap.transpose = transpose;
        info!("Set transpose to {transpose}");
        // Beep reference note then transposed note.
        beep(&self.synth, TRANSPOSE_REFERENCE, 50);
        thread::sleep(Duration::from_millis(100));
        beep(&self.synth, TRANSPOSE_REFERENCE + transpose, 50);
    }

    // Offset the tranpose amount by a number of half-steps.
    fn offset(self: &mut Self, offset: i32, notemap: &mut NoteMap) {
        self.set_transpose(notemap.transpose + offset, notemap);
    }

    // Jump directly to a transpose, based on the note played.
    fn direct(self: &mut Self, note: i32, notemap: &mut NoteMap) {
        // Compare note to reference to get transpose amount.
        let transpose = note - TRANSPOSE_REFERENCE;
        self.set_transpose(transpose, notemap);
    }
}
