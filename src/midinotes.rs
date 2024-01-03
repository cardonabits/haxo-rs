// Map notes in the instrument to midi notes.  The note names are used for debug
// messages and when recording notemaps.  Notes are in concert pitch.
pub const NOTES: &[(&str, i32)] = &[
    ("Low Bb", 58),
    ("Low B", 59),
    ("Low C", 60),
    ("Low C#", 61),
    ("Low D", 62),
    ("Low D#", 63),
    ("Low E", 64),
    ("Low F", 65),
    ("Low F#", 66),
    ("Low G", 67),
    ("Low Ab", 68),
    ("Low A", 69),
    ("Mid Bb", 70),
    ("Mid B", 71),
    ("Mid C", 72),
    ("Mid C#", 73),
    ("Mid D", 74),
    ("Mid D#", 75),
    ("Mid E", 76),
    ("Mid F", 77),
    ("Mid F#", 78),
    ("Mid G", 79),
    ("Mid Ab", 80),
    ("Mid A", 81),
    ("High Bb", 82),
    ("High B", 83),
    ("High C", 84),
    ("High C#", 85),
    ("High D", 86),
    ("High D#", 87),
    ("High E", 88),
    ("High F", 89),
    ("High F#", 90),
];

pub fn get_name(value: i32) -> Option<&'static str> {
    for &n in NOTES {
        if value == n.1 {
            return Some(n.0);
        }
    }
    None
}
