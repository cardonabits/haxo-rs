// Map notes in the instrument to midi notes.  The note names are used for debug
// messages and when recording notemaps.  This assumes a Bb instrument
// TODO: Make this a command line flag, e.g. --instrument-key=Bb
pub const NOTES: &[(&str, i32)] = &[
    ("Low Bb", 44),
    ("Low B", 45),
    ("Low C", 46),
    ("Low C#", 47),
    ("Low D", 48),
    ("Low D#", 49),
    ("Low E", 50),
    ("Low F", 51),
    ("Low F#", 52),
    ("Low G", 53),
    ("Low Ab", 54),
    ("Low A", 55),
    ("Mid Bb", 56),
    ("Mid B", 57),
    ("Mid C", 58),
    ("Mid C#", 59),
    ("Mid D", 60),
    ("Mid D#", 61),
    ("Mid E", 62),
    ("Mid F", 63),
    ("Mid F#", 64),
    ("Mid G", 65),
    ("Mid Ab", 66),
    ("Mid A", 67),
    ("High Bb", 68),
    ("High B", 69),
    ("High C", 70),
    ("High C#", 71),
    ("High D", 72),
    ("High D#", 73),
    ("High E", 74),
    ("High F", 75),
    ("High F#", 76),
];

pub fn get_name(value: &i32) -> Option<&str> {
    for &n in NOTES {
        if *value == n.1 {
            return Some(n.0);
        }
    }
    None
}
