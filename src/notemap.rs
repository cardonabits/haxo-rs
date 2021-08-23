extern crate serde_json;

use std::collections::BTreeMap;
use std::fs;
use std::thread;
use std::time::Duration;

use super::midinotes;

const FILENAME: &str = "notemap.json";

pub struct NoteMap {
    recording: bool,
    recording_index: usize,
    last_keys: u32,
    last_recorded: u32,
    record_next: bool,
    notemap: BTreeMap<u32, i32>,
}

impl NoteMap {
    pub fn generate() -> Self {
        let mapfile = fs::read_to_string(FILENAME);
        let mapfile = match mapfile {
            Ok(contents) => contents,
            Err(_error) => String::from("{}"),
        };
        let notemap: BTreeMap<u32, i32> = serde_json::from_str(&mapfile).unwrap();
        NoteMap {
            recording: false,
            recording_index: 0,
            last_keys: 0,
            last_recorded: 0,
            record_next: false,
            notemap: notemap,
        }
    }

    pub fn save(&self) {
        let notemap_json = serde_json::to_string_pretty(&self.notemap).unwrap();
        fs::write(FILENAME, notemap_json).expect("Unable to write file");
    }

    pub fn get(&self, key: &u32) -> std::option::Option<&i32> {
        self.notemap.get(key)
    }

    pub fn start_recording(&mut self) {
        self.recording = true;
    }

    pub fn is_recording(&self) -> bool {
        self.recording
    }

    pub fn record(&mut self, keys: u32, pressure: i32) -> () {
        if pressure > 10 && self.last_recorded != keys {
            self.insert(keys, midinotes::NOTES[self.recording_index].1);
            self.last_recorded = keys;
            println!(
                "Keymap {} recorded for {}",
                keys,
                midinotes::NOTES[self.recording_index].0
            );
            self.save();
            thread::sleep(Duration::from_millis(250));
            self.record_next = true;
        }

        if self.record_next {
            self.recording_index += 1;
            self.record_next = false;
            if self.recording_index == midinotes::NOTES.len() {
                self.recording = false;
                self.recording_index = 0;
                println!("Done recording keymaps");
            } else {
                println!("Next note is {}", midinotes::NOTES[self.recording_index].0);
                println!("Draw with keys pressed to go back to previous note to add an alternate fingering.");
                println!("Draw with no keys pressed to skip to next note.");
            }
        }

        if pressure < -10 {
            if self.recording_index > 0 && keys > 0 {
                self.recording_index -= 1;
                println!("Back to {}", midinotes::NOTES[self.recording_index].0);
            }
            if keys == 0 {
                self.record_next = true;
            }
            thread::sleep(Duration::from_millis(1001));
        }

        if keys != self.last_keys {
            if pressure < 10 && pressure > -10 {
                println!(
                    "Blow to record this keymap ({}) for {}",
                    keys,
                    midinotes::NOTES[self.recording_index].0
                );
            }
            self.last_keys = keys;
        }
    }

    fn insert(&mut self, key: u32, value: i32) {
        &self.notemap.insert(key, value);
    }

    #[allow(dead_code)]
    fn remove(&mut self, key: &u32) {
        &self.notemap.remove(key);
    }
}

#[cfg(test)]
mod tests {
    // Import names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn update() {
        let mut notemap = NoteMap::generate();
        notemap.insert(1234567, 66);
        notemap.save();
        let notemap2 = NoteMap::generate();
        assert_eq!(notemap2.get(&1234567), Some(&66));
        notemap.remove(&1234567);
        notemap.save();
        let notemap2 = NoteMap::generate();
        assert_eq!(notemap2.get(&1234567), None);
    }
}
