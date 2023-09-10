use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};

use std::error::Error;

use log::info;

struct MidiOut {
    conn_out: MidiOutputConnection,
}

impl MidiOut {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let midi_out = MidiOutput::new("Midi Output")?;

        let out_ports = midi_out.ports();

        let out_port: &MidiOutputPort = match out_ports.len() {
            0 => return Err("no output port found".into()),
            1 => {
                info!(
                    "Choosing the only available midi output port: {}",
                    midi_out.port_name(&out_ports[0]).unwrap()
                );
                &out_ports[0]
            }
            _ => {
                info!("\nAvailable output ports:");
                let mut i = 0;
                for p in out_ports.iter() {
                    info!("{}: {}", i, midi_out.port_name(p).unwrap());
                    i += 1;
                }
                info!("Picked the last one");
                &out_ports[i - 1]
            }
        };
        let conn_out = midi_out.connect(out_port, "haxophone")?;
        Ok(MidiOut { conn_out })
    }

    pub fn noteon(&mut self, note: i32, vol: i32) {
        const NOTE_ON_MSG: u8 = 0x90;
        let _ = self.conn_out.send(&[NOTE_ON_MSG, note as u8, vol as u8]);
    }
    pub fn cc(&mut self, msg: i32, val: i32) {
        const CC_MSG: u8 = 0xB0;
        let _ = self.conn_out.send(&[CC_MSG, msg as u8, val as u8]);
    }
    pub fn noteoff(&mut self, note: i32, vol: i32) {
        const NOTE_OFF_MSG: u8 = 0x80;
        let _ = self.conn_out.send(&[NOTE_OFF_MSG, note as u8, vol as u8]);
    }
}

#[cfg(test)]
mod tests {

    use std::thread::sleep;
    use std::time::Duration;
    use test_log::test;

    use super::*;

    #[test]
    fn test_new() -> Result<(), Box<dyn Error>> {
        MidiOut::new()?;
        Ok(())
    }

    #[test]
    fn test_note() -> Result<(), Box<dyn Error>> {
        let mut midi = MidiOut::new()?;
        midi.noteon(66, 100);
        sleep(Duration::from_millis(1000));
        midi.noteoff(66, 100);
        Ok(())
    }

    #[test]
    fn test_cc() -> Result<(), Box<dyn Error>> {
        let mut midi = MidiOut::new()?;
        midi.noteon(66, 100);
        sleep(Duration::from_millis(1000));
        const MIDI_CC_VOLUME: i32 = 7;
        midi.cc(MIDI_CC_VOLUME, 10);
        sleep(Duration::from_millis(1000));
        midi.cc(MIDI_CC_VOLUME, 100);
        sleep(Duration::from_millis(1000));
        midi.cc(MIDI_CC_VOLUME, 10);
        sleep(Duration::from_millis(1000));
        midi.noteoff(66, 100);
        Ok(())
    }
}
