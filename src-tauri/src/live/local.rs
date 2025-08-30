use anyhow::Result;
use uuid::Uuid;
use std::{fs::File, path::PathBuf};

use crate::live::models::LocalInfo;

pub struct LocalPlayerRepository(PathBuf);

impl LocalPlayerRepository {
    pub fn new(path: PathBuf) -> Result<Self> {        

        if !path.exists() {
            let client_id = Uuid::new_v4().to_string();
            let mut value = LocalInfo::default();
            value.client_id = client_id;

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
        let value = serde_json::from_reader(reader)?;

        Ok(value)
    }

    pub fn write(&self, value: &LocalInfo) -> Result<()> {

        let writer = File::create(&self.0)?;
        serde_json::to_writer_pretty(writer, value)?;

        Ok(())
    }
}