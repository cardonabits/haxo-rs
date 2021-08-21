extern crate serde_json;

use std::collections::HashMap;
use std::fs;

const FILENAME: &str = "notemap.json";

pub fn generate() -> HashMap<u32, i32> {
    let mapfile = fs::read_to_string(FILENAME).expect("Something went wrong reading the file");
    let notemap: HashMap<u32, i32> = serde_json::from_str(&mapfile).unwrap();
    notemap
}

pub fn save(notemap: &HashMap<u32, i32>) {
    let notemap_json = serde_json::to_string_pretty(&notemap).unwrap();
    fs::write(FILENAME, notemap_json).expect("Unable to write file");
}

#[cfg(test)]
mod tests {
    // Import names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn update() {
        let mut notemap = generate();
        notemap.insert(1234567, 66);
        save(&notemap);
        let notemap2 = generate();
        assert_eq!(notemap2.get(&1234567), Some(&66));
        notemap.remove(&1234567);
        save(&notemap);
        let notemap2 = generate();
        assert_eq!(notemap2.get(&1234567), None);
    }
}
