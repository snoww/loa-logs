use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{fs::File, path::PathBuf};

use crate::constants::DEFAULT_SETTINGS_PATH;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub env: EnvironmentSettings,
    pub general: GeneralSettings,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for Settings {
    fn default() -> Self {
        let reader = File::open(DEFAULT_SETTINGS_PATH).expect("Missing default settings");
        let settings = serde_json::from_reader(reader).expect("Invalid default settings");
        settings
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentSettings {
    pub hearbeat_api_url: String,
    pub stats_api_url: String
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GeneralSettings {
    pub start_loa_on_start: bool,
    pub low_performance_mode: bool,
    #[serde(default = "default_true")]
    pub auto_iface: bool,
    pub port: u16,
    #[serde(default = "default_true")]
    pub always_on_top: bool,
    #[serde(default = "default_true")]
    pub boss_only_damage: bool,
    #[serde(default = "default_true")]
    pub hide_meter_on_start: bool,
    pub hide_logs_on_start: bool,
    pub mini: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

fn default_true() -> bool {
    true
}

pub struct SettingsManager(PathBuf);

impl SettingsManager {
    pub fn new(path: PathBuf) -> Result<Self> {

        if !path.exists() {
            let settings = Settings::default();
            let writer = File::create(&path)?;
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