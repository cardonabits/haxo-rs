use std::collections::HashMap;

pub fn generate() -> HashMap<u32, i32> {
    let mut notemap = HashMap::new();
    // silence
    notemap.insert(0, 0);
    notemap.insert(656548352, 46); // Bb
    notemap.insert(640819712, 47); // B
    notemap.insert(639771136, 48); // C
    notemap.insert(639836672, 49); // C#
    notemap.insert(572662272, 50); // D
    notemap.insert(576856576, 51); // Eb
    notemap.insert(35791360, 52); // E
    notemap.insert(2236928, 53); // F
    notemap.insert(33694208, 54); // F#
    notemap.insert(2499072, 54); // F#
    notemap.insert(139776, 55); // G
    notemap.insert(143872, 56); // G#
    notemap.insert(8704, 57); // A
    notemap.insert(33554944, 58); // Bb
    notemap.insert(544, 58); // Bb
    notemap.insert(2097664, 58); // Bb
    notemap.insert(8960, 58); // Bb
    notemap.insert(512, 59); // B
    notemap.insert(8192, 60); // C
    notemap.insert(528, 60); // C
                             // control messages
    notemap.insert(71565856, -1); // shutdown
    notemap
}
