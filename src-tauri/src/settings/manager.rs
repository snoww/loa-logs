use anyhow::Result;
use std::{
    fs::File,
    path::PathBuf,
};

use crate::settings::Settings;

pub struct SettingsManager(PathBuf);

impl SettingsManager {
    pub fn new(path: PathBuf) -> Result<Self> {
        Ok(Self(path))
    }

    pub fn read(&self) -> Result<Option<Settings>> {
        if !self.0.exists() {
            return Ok(None);
        }

        let reader = File::open(&self.0)?;
        Ok(serde_json::from_reader(reader).ok())
    }

    pub fn save(&self, settings: &Settings) -> Result<()> {
        let writer = File::create(&self.0)?;
        serde_json::to_writer_pretty(writer, settings)?;

        Ok(())
    }
}
