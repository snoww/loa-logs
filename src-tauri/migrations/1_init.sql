CREATE TABLE encounter (
        id INTEGER PRIMARY KEY,
        last_combat_packet INTEGER,
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
        version INTEGER NOT NULL DEFAULT 5, boss_hp_log BLOB, stagger_log TEXT);
        
CREATE TABLE entity (
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
            engravings TEXT, gear_hash TEXT, spec TEXT, ark_passive_active BOOLEAN, ark_passive_data TEXT, loadout_hash TEXT, combat_power REAL, support_ap REAL, support_brand REAL, support_identity REAL, support_hyper REAL,
            PRIMARY KEY (name, encounter_id),
            FOREIGN KEY (encounter_id) REFERENCES encounter (id) ON DELETE CASCADE
        );
CREATE INDEX entity_encounter_id_index
        ON entity (encounter_id desc);
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

CREATE INDEX encounter_preview_favorite_index ON encounter_preview(favorite);

CREATE INDEX encounter_preview_fight_start_index ON encounter_preview(fight_start);

CREATE INDEX encounter_preview_my_dps_index ON encounter_preview(my_dps);

CREATE INDEX encounter_preview_duration_index ON encounter_preview(duration);

CREATE VIRTUAL TABLE encounter_search USING fts5(
        current_boss,
        players,
        columnsize = 0,
        detail = full,
        tokenize = 'trigram remove_diacritics 1',
        content = encounter_preview,
        content_rowid = id
)
/* encounter_search(current_boss,players) */;

CREATE TABLE IF NOT EXISTS 'encounter_search_data'(id INTEGER PRIMARY KEY, block BLOB);

CREATE TABLE IF NOT EXISTS 'encounter_search_idx'(segid, term, pgno, PRIMARY KEY(segid, term)) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS 'encounter_search_config'(k PRIMARY KEY, v) WITHOUT ROWID;

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

CREATE TABLE sync_logs (
        encounter_id INTEGER PRIMARY KEY,
        upstream_id TEXT,
        failed BOOLEAN NOT NULL DEFAULT 0,
        FOREIGN KEY (encounter_id) REFERENCES encounter (id) ON DELETE CASCADE
    );

CREATE INDEX entity_name_index
        ON entity (name);

CREATE INDEX entity_class_index
        ON entity (class);
