mod queries;
mod utils;
pub mod models;
pub mod migrator;
pub mod repository;

use anyhow::Result;
use r2d2_sqlite::SqliteConnectionManager;
use std::{fs, path::{Path, PathBuf}};

pub use migrator::*;
pub use repository::Repository;

pub struct Database(r2d2::Pool<SqliteConnectionManager>, PathBuf);

impl Database {
    pub fn new(path: PathBuf) -> Result<Self> {

        let is_new = !path.exists();
        let manager = SqliteConnectionManager::file(&path);
        let pool: r2d2::Pool<SqliteConnectionManager> = r2d2::Pool::new(manager)?;

        if is_new {
            setup(pool.get()?)?;
        }
    
        Ok(Self(pool, path))
    }

    pub fn create_repository(&self) -> Repository {
        Repository::new(self.0.clone())
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