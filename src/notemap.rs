extern crate serde_json;

use std::collections::HashMap;
use std::fs;

pub fn generate() -> HashMap<u32, i32> {

    let mapfile = fs::read_to_string("notemap.json")
        .expect("Something went wrong reading the file");
    let notemap: HashMap<u32, i32> = serde_json::from_str(&mapfile).unwrap();
    notemap
}
