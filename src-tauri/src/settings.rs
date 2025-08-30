use anyhow::Result;
use std::{fs::File, path::PathBuf};

use crate::live::models::Settings;

pub struct SettingsManager(PathBuf);

impl SettingsManager {
    pub fn new(path: PathBuf) -> Result<Self> {

        if !path.exists() {
            let writer = File::create(&path)?;
            let settings = Settings::default();
            serde_json::to_writer_pretty(writer, &settings)?;
        }

        Ok(Self(path))
    }

    pub fn read(&self) -> Result<Settings> {
        let reader = File::open(&self.0)?;
        let settings = serde_json::from_reader(reader)?;

        Ok(settings)
    }

    pub fn save(&self, settings: &Settings) -> Result<()> {
        let writer = File::create(&self.0)?;
        serde_json::to_writer_pretty(writer, settings)?;

        Ok(())
    }
}