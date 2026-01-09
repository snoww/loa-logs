use anyhow::Result;
use hashbrown::HashMap;
use log::info;
use serde::{Deserialize, Serialize};
use std::{fs::File, path::PathBuf};
use uuid::Uuid;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LocalInfo {
    pub client_id: String,
    pub local_players: HashMap<u64, LocalPlayer>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LocalPlayer {
    pub name: String,
    pub count: i32,
}

pub struct LocalPlayerRepository(PathBuf);

impl LocalPlayerRepository {
    pub fn new(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            let value = Self::create()?;
            let writer = File::create(&path)?;
            serde_json::to_writer_pretty(writer, &value)?;
        }

        Ok(Self(path))
    }

    /// Reads the saved local player info from disk.
    ///
    /// This is useful in cases where the meter was opened late and
    /// needs to restore previously saved state.
    ///
    /// ### Errors
    /// Returns an error if the file cannot be opened or if deserialization fails.
    pub fn read(&self) -> Result<LocalInfo> {
        let reader = File::open(&self.0)?;
        match serde_json::from_reader(reader) {
            Ok(v) => Ok(v),
            Err(_) => {
                info!("failed to parse local info file, creating a new one.");
                Self::create()
            }
        }
    }

    pub fn write(&self, value: &LocalInfo) -> Result<()> {
        let writer = File::create(&self.0)?;
        serde_json::to_writer_pretty(writer, value)?;

        Ok(())
    }

    pub fn create() -> Result<LocalInfo> {
        Ok(LocalInfo {
            client_id: Uuid::new_v4().to_string(),
            ..Default::default()
        })
    }
}
