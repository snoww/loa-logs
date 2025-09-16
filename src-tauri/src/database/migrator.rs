use chrono::{Local, NaiveDateTime};
use log::info;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection, Transaction};
use std::{fs, path::{Path, PathBuf}};
use anyhow::Result;

const MIGRATIONS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS migrations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path NVARCHAR(50) NOT NULL UNIQUE,
    executed_on NVARCHAR(20) NOT NULL,
    app_version NVARCHAR(20) NOT NULL
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
    path,
    executed_on,
    app_version
) VALUES (?1, ?2, ?3)
"#;

const INSERT_OR_UPDATE_CONFIG_SQL: &str = r#"
INSERT INTO config (
    app_version,
    updated_on
) VALUES (?1, ?2)
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

const MIGRATION_EXISTS_SQL: &str = r#"
SELECT
    EXISTS(
        SELECT 1
        FROM migrations
        WHERE path = ?1)
"#;

#[derive(Debug, Clone)]
struct Migration {
    order: i32,
    path: String,
}

pub struct Migrator<'a> {
    connection: PooledConnection<SqliteConnectionManager>,
    is_new: bool,
    migrations_folder: &'a Path,
    app_version: &'a str
}

impl<'a> Migrator<'a> {
    pub fn new(
        connection: PooledConnection<SqliteConnectionManager>,
        is_new: bool,
        migrations_folder: &'a Path,
        app_version: &'a str) -> Self {
        Self {
            connection,
            is_new,
            migrations_folder,
            app_version
        }
    }

    pub fn run(self) -> Result<()> {
        let mut connection = self.connection;
        let app_version = self.app_version;

        connection.execute(CONFIG_TABLE_SQL, [])?;

        let migrations_exist = table_exists(&connection, "migrations")?;
        let mut migrations = collect_migrations(self.migrations_folder)?;

        if !migrations_exist {
            connection.execute(MIGRATIONS_TABLE_SQL, [])?;
            info!("Created migrations table");

            if !self.is_new {
                let path = "1_init.sql";
                let executed_on = Local::now().naive_local();
                record_migration(&connection, path, executed_on, app_version)?;
                info!("Marked \"{}\" as applied ( old db )", path);

                migrations = migrations.into_iter().skip(1).collect();
            }
        }

        info!("Running migrations");
        
        for migration in migrations {
            if migration_applied(&connection, &migration)? {
                info!("Skipping migration \"{}\"", migration.path);
                continue;
            }

            let executed_on = Local::now().naive_local();
            apply_migration(&mut connection, &migration, executed_on, app_version)?;
            info!("Applied migration \"{}\"", migration.path);
        }

        let updated_on = Local::now().naive_local();
        update_config(&connection, app_version, updated_on)?;

        Ok(())
    }
}

fn update_config(connection: &Connection, app_version: &str, updated_on: NaiveDateTime) -> Result<()> {
    
    let params = params![app_version, updated_on.to_string()];
    connection.execute(INSERT_OR_UPDATE_CONFIG_SQL, params)?;

    info!("Updated config table with app_version \"{}\"", app_version);
    Ok(())
}

fn table_exists(connection: &Connection, table_name: &str) -> Result<bool> {
    Ok(connection.query_row(TABLE_EXISTS_SQL, [table_name], exists_query_result)?)
}

fn record_migration(connection: &Connection, path: &str, executed_on: NaiveDateTime, app_version: &str) -> Result<()> {
    let params = params![path, executed_on.to_string(), app_version];
    connection.execute(INSERT_MIGRATION_SQL, params)?;

    Ok(())
}

fn migration_applied(connection: &Connection, migration: &Migration) -> Result<bool> {
    let params = [migration.path.as_str()];
    Ok(connection.query_row(MIGRATION_EXISTS_SQL, params,exists_query_result)?)
}

fn apply_migration(connection: &mut Connection, migration: &Migration, executed_on: NaiveDateTime, app_version: &str) -> Result<()> {
    let sql = fs::read_to_string(&migration.path)?;

    connection.execute_batch(&sql)?;
    record_migration(&connection, &migration.path, executed_on, app_version)?;

    Ok(())
}

fn collect_migrations(folder: &Path) -> Result<Vec<Migration>, std::io::Error> {
    let mut migrations: Vec<_> = fs::read_dir(folder)?
        .filter_map(|entry| parse_migration(entry.ok()?))
        .collect();

    migrations.sort_by_key(|m| m.order);
    Ok(migrations)
}

fn parse_migration(entry: fs::DirEntry) -> Option<Migration> {
    let path: PathBuf = entry.path();

    if path.extension()?.to_str()? != "sql" {
        return None;
    }

    let filename = path.file_stem()?.to_str()?.to_string();
    let mut parts = filename.splitn(2, '_');
    let order = parts.next()?.parse::<i32>().ok()?;

    Some(Migration {
        order,
        path: path.to_string_lossy().to_string(),
    })
}

fn exists_query_result<T: rusqlite::types::FromSql>(row: &rusqlite::Row) -> rusqlite::Result<T> {
    row.get(0)
}