use std::path::Path;

use log::info;
use rusqlite::{Connection, Transaction};

use crate::constants::{DATABASE_FILE_NAME, DB_VERSION};


pub fn get_db_connection(resource_path: &Path) -> Result<Connection, rusqlite::Error> {
    let path = resource_path.join(DATABASE_FILE_NAME);
    if !path.exists() {
        setup_db(resource_path)?;
    }
    Connection::open(path)
}

pub fn setup_db(resource_path: &Path) -> Result<(), rusqlite::Error> {
    info!("setting up database");
    let mut conn = Connection::open(resource_path.join(DATABASE_FILE_NAME))?;
    let tx = conn.transaction()?;

    // FIXME: replace me with idempotent migrations

    let mut stmt = tx.prepare("SELECT 1 FROM sqlite_master WHERE type=? AND name=?")?;
    if !stmt.exists(["table", "encounter"])? {
        info!("creating tables");
        migration_legacy_encounter(&tx)?;
        migration_legacy_entity(&tx)?;
    }

    // NOTE: for databases, where the bad migration code already ran
    migration_legacy_entity(&tx)?;

    if !stmt.exists(["table", "encounter_preview"])? {
        info!("optimizing searches");
        migration_legacy_encounter(&tx)?;
        migration_legacy_entity(&tx)?;
        migration_full_text_search(&tx)?;
    }

    if !stmt.exists(["table", "sync_logs"])? {
        info!("adding sync table");
        migration_sync(&tx)?;
    }

    migration_specs(&tx)?;

    stmt.finalize()?;
    info!("finished setting up database");
    tx.commit()
}

fn migration_legacy_encounter(tx: &Transaction) -> Result<(), rusqlite::Error> {
    tx.execute_batch(&format!(
        "
    CREATE TABLE IF NOT EXISTS encounter (
        id INTEGER PRIMARY KEY,
        last_combat_packet INTEGER,
        fight_start INTEGER,
        local_player TEXT,
        current_boss TEXT,
        duration INTEGER,
        total_damage_dealt INTEGER,
        top_damage_dealt INTEGER,
        total_damage_taken INTEGER,
        top_damage_taken INTEGER,
        dps INTEGER,
        buffs TEXT,
        debuffs TEXT,
        total_shielding INTEGER DEFAULT 0,
        total_effective_shielding INTEGER DEFAULT 0,
        applied_shield_buffs TEXT,
        misc TEXT,
        difficulty TEXT,
        favorite BOOLEAN NOT NULL DEFAULT 0,
        cleared BOOLEAN,
        version INTEGER NOT NULL DEFAULT {},
        boss_only_damage BOOLEAN NOT NULL DEFAULT 0
    );
    CREATE INDEX IF NOT EXISTS encounter_fight_start_index
    ON encounter (fight_start desc);
    CREATE INDEX IF NOT EXISTS encounter_current_boss_index
    ON encounter (current_boss);
    ",
        DB_VERSION
    ))?;

    let mut stmt = tx.prepare("SELECT 1 FROM pragma_table_info(?) WHERE name=?")?;
    if !stmt.exists(["encounter", "misc"])? {
        tx.execute("ALTER TABLE encounter ADD COLUMN misc TEXT", [])?;
    }
    if !stmt.exists(["encounter", "difficulty"])? {
        tx.execute("ALTER TABLE encounter ADD COLUMN difficulty TEXT", [])?;
    }
    if !stmt.exists(["encounter", "favorite"])? {
        tx.execute_batch(&format!(
            "
            ALTER TABLE encounter ADD COLUMN favorite BOOLEAN DEFAULT 0;
            ALTER TABLE encounter ADD COLUMN version INTEGER DEFAULT {};
            ALTER TABLE encounter ADD COLUMN cleared BOOLEAN;
            ",
            DB_VERSION,
        ))?;
    }
    if !stmt.exists(["encounter", "boss_only_damage"])? {
        tx.execute(
            "ALTER TABLE encounter ADD COLUMN boss_only_damage BOOLEAN NOT NULL DEFAULT 0",
            [],
        )?;
    }
    if !stmt.exists(["encounter", "total_shielding"])? {
        tx.execute_batch(
            "
                ALTER TABLE encounter ADD COLUMN total_shielding INTEGER DEFAULT 0;
                ALTER TABLE encounter ADD COLUMN total_effective_shielding INTEGER DEFAULT 0;
                ALTER TABLE encounter ADD COLUMN applied_shield_buffs TEXT;
                ",
        )?;
    }
    tx.execute("UPDATE encounter SET cleared = coalesce(json_extract(misc, '$.raidClear'), 0) WHERE cleared IS NULL;", [])?;
    stmt.finalize()
}

fn migration_legacy_entity(tx: &Transaction) -> Result<(), rusqlite::Error> {
    tx.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS entity (
            name TEXT,
            character_id INTEGER,
            encounter_id INTEGER NOT NULL,
            npc_id INTEGER,
            entity_type TEXT,
            class_id INTEGER,
            class TEXT,
            gear_score REAL,
            current_hp INTEGER,
            max_hp INTEGER,
            is_dead INTEGER,
            skills TEXT,
            damage_stats TEXT,
            dps INTEGER,
            skill_stats TEXT,
            last_update INTEGER,
            engravings TEXT,
            PRIMARY KEY (name, encounter_id),
            FOREIGN KEY (encounter_id) REFERENCES encounter (id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS entity_encounter_id_index
        ON entity (encounter_id desc);
        CREATE INDEX IF NOT EXISTS entity_name_index
        ON entity (name);
        CREATE INDEX IF NOT EXISTS entity_class_index
        ON entity (class);
        ",
    )?;

    let mut stmt = tx.prepare("SELECT 1 FROM pragma_table_info(?) WHERE name=?")?;
    if !stmt.exists(["entity", "dps"])? {
        tx.execute("ALTER TABLE entity ADD COLUMN dps INTEGER", [])?;
    }
    if !stmt.exists(["entity", "character_id"])? {
        tx.execute("ALTER TABLE entity ADD COLUMN character_id INTEGER", [])?;
    }
    if !stmt.exists(["entity", "engravings"])? {
        tx.execute("ALTER TABLE entity ADD COLUMN engravings TEXT", [])?;
    }
    if !stmt.exists(["entity", "gear_hash"])? {
        tx.execute("ALTER TABLE entity ADD COLUMN gear_hash TEXT", [])?;
    }
    tx.execute("UPDATE entity SET dps = coalesce(json_extract(damage_stats, '$.dps'), 0) WHERE dps IS NULL;", [])?;
    stmt.finalize()
}

fn migration_full_text_search(tx: &Transaction) -> Result<(), rusqlite::Error> {
    tx.execute_batch(
        "
        CREATE TABLE encounter_preview (
            id INTEGER PRIMARY KEY,
            fight_start INTEGER,
            current_boss TEXT,
            duration INTEGER,
            players TEXT,
            difficulty TEXT,
            local_player TEXT,
            my_dps INTEGER,
            favorite BOOLEAN NOT NULL DEFAULT 0,
            cleared BOOLEAN,
            boss_only_damage BOOLEAN NOT NULL DEFAULT 0,
            FOREIGN KEY (id) REFERENCES encounter(id) ON DELETE CASCADE
        );

        INSERT INTO encounter_preview SELECT
            id, fight_start, current_boss, duration, 
            (
                SELECT GROUP_CONCAT(class_id || ':' || name ORDER BY dps DESC)
                FROM entity
                WHERE encounter_id = encounter.id AND entity_type = 'PLAYER'
            ) AS players,
            difficulty, local_player,
            (
                SELECT dps
                FROM entity
                WHERE encounter_id = encounter.id AND name = encounter.local_player
            ) AS my_dps,
            favorite, cleared, boss_only_damage
        FROM encounter;

        DROP INDEX IF EXISTS encounter_fight_start_index;
        DROP INDEX IF EXISTS encounter_current_boss_index;
        DROP INDEX IF EXISTS encounter_favorite_index;
        DROP INDEX IF EXISTS entity_name_index;
        DROP INDEX IF EXISTS entity_class_index;

        ALTER TABLE encounter DROP COLUMN fight_start;
        ALTER TABLE encounter DROP COLUMN current_boss;
        ALTER TABLE encounter DROP COLUMN duration;
        ALTER TABLE encounter DROP COLUMN difficulty;
        ALTER TABLE encounter DROP COLUMN local_player;
        ALTER TABLE encounter DROP COLUMN favorite;
        ALTER TABLE encounter DROP COLUMN cleared;
        ALTER TABLE encounter DROP COLUMN boss_only_damage;

        ALTER TABLE encounter ADD COLUMN boss_hp_log BLOB;
        ALTER TABLE encounter ADD COLUMN stagger_log TEXT;

        CREATE INDEX encounter_preview_favorite_index ON encounter_preview(favorite);
        CREATE INDEX encounter_preview_fight_start_index ON encounter_preview(fight_start);
        CREATE INDEX encounter_preview_my_dps_index ON encounter_preview(my_dps);
        CREATE INDEX encounter_preview_duration_index ON encounter_preview(duration);

        CREATE VIRTUAL TABLE encounter_search USING fts5(
            current_boss, players, columnsize=0, detail=full,
            tokenize='trigram remove_diacritics 1',
            content=encounter_preview, content_rowid=id
        );
        INSERT INTO encounter_search(encounter_search) VALUES('rebuild');
        CREATE TRIGGER encounter_preview_ai AFTER INSERT ON encounter_preview BEGIN
            INSERT INTO encounter_search(rowid, current_boss, players)
            VALUES (new.id, new.current_boss, new.players);
        END;
        CREATE TRIGGER encounter_preview_ad AFTER DELETE ON encounter_preview BEGIN
            INSERT INTO encounter_search(encounter_search, rowid, current_boss, players)
            VALUES('delete', old.id, old.current_boss, old.players);
        END;
        CREATE TRIGGER encounter_preview_au AFTER UPDATE OF current_boss, players ON encounter_preview BEGIN
            INSERT INTO encounter_search(encounter_search, rowid, current_boss, players)
            VALUES('delete', old.id, old.current_boss, old.players);
            INSERT INTO encounter_search(rowid, current_boss, players)
            VALUES (new.id, new.current_boss, new.players);
        END;
        ",
    )
}

fn migration_sync(tx: &Transaction) -> Result<(), rusqlite::Error> {
    tx.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS sync_logs (
        encounter_id INTEGER PRIMARY KEY,
        upstream_id TEXT,
        failed BOOLEAN NOT NULL DEFAULT 0,
        FOREIGN KEY (encounter_id) REFERENCES encounter (id) ON DELETE CASCADE
    );",
    )
}

fn migration_specs(tx: &Transaction) -> Result<(), rusqlite::Error> {
    let mut stmt = tx.prepare("SELECT 1 FROM pragma_table_info(?) WHERE name=?")?;
    if !stmt.exists(["entity", "spec"])? {
        info!("adding spec info columns");
        tx.execute_batch(
            "
                ALTER TABLE entity ADD COLUMN spec TEXT;
                ALTER TABLE entity ADD COLUMN ark_passive_active BOOLEAN;
                ALTER TABLE entity ADD COLUMN ark_passive_data TEXT;
                ",
        )?;
    }

    stmt.finalize()
}