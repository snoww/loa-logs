pub mod migrator;
pub mod models;
mod queries;
pub mod repositories;
mod sql_types;
pub mod utils;

use anyhow::Result;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use std::{fs, path::PathBuf};

pub use migrator::*;
pub use repositories::*;

pub struct Database(r2d2::Pool<SqliteConnectionManager>, PathBuf);

impl Database {
    #[cfg(test)]
    pub fn memory(app_version: &str) -> Result<Self> {
        let manager = SqliteConnectionManager::memory();

        let pool: r2d2::Pool<SqliteConnectionManager> = r2d2::Pool::new(manager)?;

        let migrator = Migrator::new(pool.clone(), app_version);
        migrator.run()?;

        Ok(Self(pool, PathBuf::new()))
    }

    pub fn new(path: PathBuf, app_version: &str) -> Result<Self> {
        let manager = SqliteConnectionManager::file(&path);

        let pool: r2d2::Pool<SqliteConnectionManager> = r2d2::Pool::new(manager)?;

        let migrator = Migrator::new(pool.clone(), app_version);
        migrator.run()?;

        Ok(Self(pool, path))
    }

    pub fn create_encounter_repository(&self) -> EncounterRepository {
        EncounterRepository::new(Pool::clone(&self.0))
    }

    pub fn create_stats_repository(&self) -> StatsRepository {
        StatsRepository::new(Pool::clone(&self.0))
    }

    pub fn get_connection(&self) -> PooledConnection<SqliteConnectionManager> {
        self.0.get().expect("could not get db connection")
    }

    pub fn get_metadata(&self) -> Result<String> {
        let metadata = fs::metadata(&self.1)?;

        let size_in_bytes = metadata.len();
        let size_in_kb = size_in_bytes as f64 / 1024.0;
        let size_in_mb = size_in_kb / 1024.0;
        let size_in_gb = size_in_mb / 1024.0;

        let size_str = if size_in_gb >= 1.0 {
            format!("{:.2} GB", size_in_gb)
        } else if size_in_mb >= 1.0 {
            format!("{:.2} MB", size_in_mb)
        } else {
            format!("{:.2} KB", size_in_kb)
        };

        Ok(size_str)
    }
}
