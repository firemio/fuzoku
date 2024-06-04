use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
pub struct Girl {
    pub id: String,
    pub name: String,
    pub profile: String,
    pub status: String,
    pub thumbnail: String,
    pub photos: Vec<String>,
//    pub schedule: std::collections::HashMap<String, String>,
    pub schedule: HashMap<String, String>,

    // pub photos: Vec<String>,
    // pub schedule: std::collections::HashMap<String, String>,


}

pub fn load_girls_data(path: &Path) -> Vec<Girl> {
    let data = fs::read_to_string(path).expect("Unable to read file");
    serde_json::from_str(&data).expect("Unable to parse JSON")
}

pub fn save_girls_data(path: &Path, girls: &Vec<Girl>) {
    let data = serde_json::to_string_pretty(girls).expect("Unable to serialize data");
    fs::write(path, data).expect("Unable to write file");
}
