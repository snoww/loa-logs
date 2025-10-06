#![allow(dead_code)]

use std::path::PathBuf;

use anyhow::Result;

use crate::constants::*;

#[derive(Debug, Clone)]
pub struct AppContext {
    pub version: String,
    pub app_path: PathBuf,
    pub current_dir: PathBuf,
    pub settings_path: PathBuf,
    pub database_path: PathBuf,
    pub migrations_path: PathBuf,
    pub local_player_path: PathBuf,
    pub region_file_path: PathBuf,
}

impl AppContext {
    pub fn new(version: String) -> Result<Self> {
        let app_path = std::env::current_exe()?;
        let current_dir = app_path.parent().unwrap().to_path_buf();
        let settings_path = current_dir.join(SETTINGS_PATH);
        let database_path = current_dir.join(DATABASE_PATH);
        let migrations_path = current_dir.join(MIGRATIONS_PATH);
        let local_player_path = current_dir.join(LOCAL_PLAYERS_PATH);
        let region_file_path = current_dir.join(REGION_PATH);

        Ok(Self {
            version,
            app_path,
            current_dir,
            settings_path,
            database_path,
            migrations_path,
            local_player_path,
            region_file_path,
        })
    }
}
