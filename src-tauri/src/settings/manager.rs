use anyhow::Result;
use std::{fs::File, path::{Path, PathBuf}};

use crate::settings::Settings;

pub struct SettingsManager(PathBuf);

impl SettingsManager {
    pub fn new(version: String, path: PathBuf) -> Result<Self> {

        let default_settings = Settings::default();

        if !path.exists() {
            let writer = File::create(&path)?;
            serde_json::to_writer_pretty(writer, &default_settings)?;
        }
        else {
            let _ = migrate(version, &path, default_settings)?;
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

pub fn migrate(version: String, path: &Path, default_settings: Settings) -> Result<Settings> {
    let mut settings: Settings = {
        let reader = File::open(&path)?;
        serde_json::from_reader(reader)?
    };

    if settings.version.is_none() {
        settings.version = Some(version);
        settings.env = default_settings.env;

        let writer = File::create(&path)?;
        serde_json::to_writer_pretty(writer, &settings)?;
    }

    // if settings.version.as_ref().is_some_and(|pr| pr.as_str() == "1.31.6") {

    // }

    Ok(settings)
}