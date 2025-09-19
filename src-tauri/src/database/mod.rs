mod queries;
pub mod utils;
mod sql_types;
pub mod models;
pub mod migrator;
pub mod repository;

use anyhow::Result;
use chrono::Local;
use log::info;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::{fs, path::{Path, PathBuf}};

pub use migrator::*;
pub use repository::Repository;

pub struct Database(r2d2::Pool<SqliteConnectionManager>, PathBuf);

impl Database {
    pub fn memory(
        path: PathBuf,
        migrations_folder: &Path,
        app_version: &str) -> Result<Self> {

        let is_new = true;
        let manager = SqliteConnectionManager::memory();

        let pool: r2d2::Pool<SqliteConnectionManager> = r2d2::Pool::new(manager)?;
        let state = DatabaseState::new(&*pool.get()?, is_new)?;

        let migrator = Migrator::new(
            pool.clone(),
            state,
            migrations_folder,
            app_version);
        migrator.run()?;
    
        Ok(Self(pool, path))
    }

    pub fn new(
        path: PathBuf,
        migrations_folder: &Path,
        app_version: &str) -> Result<Self> {

        let is_new = !path.exists();

        if !is_new {
            let backup_path = make_backup(&path)?;
            info!("🗂️ Created database backup at {}", backup_path.display());
        }

        let manager = SqliteConnectionManager::file(&path);

        let pool: r2d2::Pool<SqliteConnectionManager> = r2d2::Pool::new(manager)?;
        let state = DatabaseState::new(&*pool.get()?, is_new)?;

        let migrator = Migrator::new(
            pool.clone(),
            state,
            migrations_folder,
            app_version);
        migrator.run()?;
    
        Ok(Self(pool, path))
    }

    pub fn create_repository(&self) -> Repository {
        Repository::new(Pool::clone(&self.0))
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

/// Create a backup of the given SQLite database, keeping only the latest `max_backups`.
pub fn make_backup(path: &Path) -> Result<PathBuf> {
    let backups_dir = get_backups_dir(path)?;
    let backup_path = create_backup(path, &backups_dir)?;
    prune_old_backups(&backups_dir, 3)?;
    Ok(backup_path)
}

/// Returns the backups directory path, creating it if necessary.
fn get_backups_dir(path: &Path) -> Result<PathBuf> {
    let backups_dir = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("backups");

    if backups_dir.exists() {
        return Ok(backups_dir);
    }

    fs::create_dir_all(&backups_dir)?;
    Ok(backups_dir)
}

/// Creates a timestamped backup of the database file in the given directory.
fn create_backup(path: &Path, backups_dir: &Path) -> Result<PathBuf> {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = backups_dir.join(format!("backup_{}.sqlite", timestamp));
    fs::copy(path, &backup_path)?;
    Ok(backup_path)
}

/// Keeps only the `max_backups` most recent files, removing older ones.
fn prune_old_backups(backups_dir: &Path, max_backups: usize) -> Result<()> {
    let mut backups: Vec<_> = fs::read_dir(backups_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().map(|ext| ext == "sqlite").unwrap_or(false))
        .collect();

    // Sort oldest first
    backups.sort_by_key(|entry| {
        entry.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });

    while backups.len() > max_backups {
        if let Some(oldest) = backups.first() {
            let _ = fs::remove_file(oldest.path());
            backups.remove(0);
        }
    }

    Ok(())
}