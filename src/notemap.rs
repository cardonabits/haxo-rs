extern crate serde_json;

use std::collections::BTreeMap;
use std::fs;

const FILENAME: &str = "notemap.json";

pub fn generate() -> BTreeMap<u32, i32> {
    let mapfile = fs::read_to_string(FILENAME);
    let mapfile = match mapfile {
        Ok(contents) => contents,
        Err(_error) => String::from("{}"),
    };
    let notemap: BTreeMap<u32, i32> = serde_json::from_str(&mapfile).unwrap();
    notemap
}

pub fn save(notemap: &BTreeMap<u32, i32>) {
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
