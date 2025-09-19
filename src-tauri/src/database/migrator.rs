use chrono::{Local, NaiveDateTime};
use log::info;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection, OptionalExtension};
use sha256::digest;
use tar::Archive;
use std::{fs::File, io::Read, path::Path};
use anyhow::{anyhow, Result};

use crate::database::migrator;

const MIGRATIONS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS migrations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name NVARCHAR(50) NOT NULL UNIQUE,
    executed_on NVARCHAR(20) NOT NULL,
    app_version NVARCHAR(20) NOT NULL,
    checksum NVARCHAR(64) NOT NULL
)
"#;

const CONFIG_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS config (
    app_version NVARCHAR(20) NOT NULL PRIMARY KEY,
    updated_on NVARCHAR(20) NOT NULL
)
"#;

const INSERT_MIGRATION_SQL: &str = r#"
INSERT INTO migrations (
    name,
    executed_on,
    app_version,
    checksum
)
VALUES
(?1, ?2, ?3, ?4)
"#;

const INSERT_OR_UPDATE_CONFIG_SQL: &str = r#"
INSERT INTO config (
    app_version,
    updated_on
)
    VALUES (?1, ?2)
ON CONFLICT(app_version) DO UPDATE
    SET updated_on = excluded.updated_on
"#;

const TABLE_EXISTS_SQL: &str = r#"
SELECT
    EXISTS(
        SELECT 1
        FROM sqlite_master
        WHERE type='table'
            AND name = ?1)
"#;

const LAST_MIGRATION_SQL: &str = r#"
SELECT
    id,
    name
FROM migrations
ORDER BY id DESC
LIMIT 1
"#;


#[derive(Debug, Clone)]
pub enum DatabaseState {
    New,
    ExistingNoMigrations,
    ExistingWithMigrations(AppliedMigration),
}

impl DatabaseState {
    pub fn new(connection: &Connection, is_new: bool) -> Result<Self> {

        if is_new {
            return Ok(Self::New);
        }

        let migrations_exist = table_exists(&connection, "migrations")?;

        if migrations_exist {
            let migration = last_migration(&connection)?.ok_or_else(|| anyhow!("Corrupted database, migration table does not have a single record?"))?;
            return Ok(Self::ExistingWithMigrations(migration));
        }        

        Ok(Self::ExistingNoMigrations)
    }
}

#[derive(Debug, Clone)]
pub struct UnappliedMigration {
    name: String,
    query: String,
}

#[derive(Debug, Clone)]
pub struct AppliedMigration {
    id: u32,
    name: String,
}

pub struct Migrator<'a> {
    pool: r2d2::Pool<SqliteConnectionManager>,
    state: DatabaseState,
    migrations_folder: &'a Path,
    app_version: &'a str
}

impl<'a> Migrator<'a> {
    pub fn new(
        pool: r2d2::Pool<SqliteConnectionManager>,
        state: DatabaseState,
        migrations_folder: &'a Path,
        app_version: &'a str) -> Self {
        Self {
            pool,
            state,
            migrations_folder,
            app_version
        }
    }

    pub fn run(mut self) -> Result<()> {
        let connection = self.pool.get()?;
        let app_version = self.app_version;

        connection.execute(CONFIG_TABLE_SQL, [])?;
        let mut migrations: Vec<UnappliedMigration> = vec![];

        match self.state {
            DatabaseState::New => {
                info!("🆕 Detected a new database, setting up migrations");
                connection.execute(MIGRATIONS_TABLE_SQL, [])?;
                migrations = collect_migrations(self.migrations_folder, None)?;
                info!("📦 Collected {} migrations to apply", migrations.len());
            },
            DatabaseState::ExistingNoMigrations => {
                info!("⚠️ Found database without migrations, setting up migrations");
                connection.execute(MIGRATIONS_TABLE_SQL, [])?;
                migrations = collect_migrations(self.migrations_folder, None)?;
            },
            DatabaseState::ExistingWithMigrations(ref migration) => {
                info!(
                    "✅ Found database with migrations, last applied: {} (id = {})",
                    migration.name, migration.id
                );

                let last_migration = Some(migration.clone());
                migrations = collect_migrations(self.migrations_folder, last_migration)?;
            },
        }

        if migrations.is_empty() {
            info!("✨ No new migrations to apply");
        }
        else {
            info!("📦 Found {} unapplied migrations", migrations.len());
        }

        self.apply_migrations(migrations, app_version)?;

        let updated_on = Local::now().naive_local();
        self.update_config(app_version, updated_on)?;

        info!("🎉 Finished migrating database to version: {}", app_version);

        Ok(())
    }

    fn apply_migrations(&mut self, migrations: Vec<UnappliedMigration>, app_version: &str) -> Result<()> {
        
        for migration in migrations {
            let executed_on = Local::now().naive_local();
            info!("🚀 Applying migration: {}", migration.name);
            self.apply_migration(migration, executed_on, app_version)?;
        }
        
        Ok(())
    }

    fn apply_migration(&mut self, migration: UnappliedMigration, executed_on: NaiveDateTime, app_version: &str) -> Result<()> {
        let connection = self.pool.get()?;
        let query = &migration.query;
        connection.execute_batch(query)?;
        self.record_migration(&migration, executed_on, app_version, )?;

        Ok(())
    }

    fn update_config(&mut self, app_version: &str, updated_on: NaiveDateTime) -> Result<()> {
        let connection = self.pool.get()?;
        let params = params![app_version, updated_on.to_string()];
        connection.execute(INSERT_OR_UPDATE_CONFIG_SQL, params)?;

        Ok(())
    }

    fn record_migration(&mut self, migration: &UnappliedMigration, executed_on: NaiveDateTime, app_version: &str) -> Result<()> {
        let connection = self.pool.get()?;
        let checksum = digest(&migration.query);
        let params = params![migration.name, executed_on.to_string(), app_version, checksum];
        connection.execute(INSERT_MIGRATION_SQL, params)?;

        Ok(())
    }

}

fn table_exists(connection: &Connection, table_name: &str) -> Result<bool> {
    Ok(connection.query_row(TABLE_EXISTS_SQL, [table_name], exists_query_result)?)
}

fn collect_migrations(folder_path: &Path, last_migration: Option<AppliedMigration>) -> Result<Vec<UnappliedMigration>> {
    let file = File::open(folder_path).unwrap();
    let mut archive = Archive::new(file);
    let mut migrations = vec![];

    for entry in archive.entries()?{
        let mut entry = entry?;
        let path = entry.path()?;
        let file_name = path.file_name()
            .ok_or_else(|| anyhow!("Invalid migration filename: {:?}", path))?
            .to_string_lossy()
            .to_string();

        if let Some(last) = &last_migration {
            if file_name <= last.name {
                info!("⏩ Skipping migration: {}", file_name);
                continue;
            }
        }

        let mut query = String::new();
        entry.read_to_string(&mut query)?;
        
        let migration = UnappliedMigration {
            name: file_name,
            query,
        };

        migrations.push(migration);
    }


    Ok(migrations)
}

fn exists_query_result<T: rusqlite::types::FromSql>(row: &rusqlite::Row) -> rusqlite::Result<T> {
    row.get(0)
}

pub fn last_migration(connection: &Connection) -> Result<Option<AppliedMigration>> {

    let last: Option<AppliedMigration> = connection
        .query_row(LAST_MIGRATION_SQL, [], |row| {
            Ok(AppliedMigration {
                id: row.get(0)?,
                name: row.get(1)?
            })
        })
        .optional()?;

    Ok(last)
}