use std::{fs::File, io::Read, path::Path};

use crate::{constants::SETTINGS_FILE_NAME, Settings};

pub fn read_settings(resource_path: &Path) -> Result<Settings, Box<dyn std::error::Error>> {
    let mut path = resource_path.to_path_buf();
    path.push(SETTINGS_FILE_NAME);
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let settings = serde_json::from_str(&contents)?;
    Ok(settings)
}