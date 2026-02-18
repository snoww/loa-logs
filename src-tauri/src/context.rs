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

        // on Windows: store data in the same place as the exe
        #[cfg(target_os = "windows")]
        let assets_path = current_dir.clone();

        // on Linux: store data in ~/.local/share/xyz.snow.loa-logs or the current executable dir as fallback
        #[cfg(target_os = "linux")]
        let assets_path = dirs::data_dir().map_or_else(|| current_dir.clone(), |x| x.join("xyz.snow.loa-logs"));

        std::fs::create_dir_all(&assets_path)?;

        let settings_path = assets_path.join(SETTINGS_PATH);
        let database_path = assets_path.join(DATABASE_PATH);
        let migrations_path = assets_path.join(MIGRATIONS_PATH);
        let local_player_path = assets_path.join(LOCAL_PLAYERS_PATH);
        let region_file_path = assets_path.join(REGION_PATH);

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
