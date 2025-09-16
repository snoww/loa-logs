pub const SELECT_FROM_ENCOUNTER_JOIN_PREVIEW: &str = r"
SELECT
    last_combat_packet,
    fight_start,
    local_player,
    current_boss,
    duration,
    total_damage_dealt,
    top_damage_dealt,
    total_damage_taken,
    top_damage_taken,
    dps,
    buffs,
    debuffs,
    misc,
    difficulty,
    favorite,
    cleared,
    boss_only_damage,
    total_shielding,
    total_effective_shielding,
    applied_shield_buffs,
    boss_hp_log
FROM encounter
JOIN encounter_preview
    USING (id)
WHERE id = ?
";

pub const SELECT_ENTITIES_BY_ENCOUNTER: &str = r"
SELECT
    name,
    class_id,
    class,
    gear_score,
    current_hp,
    max_hp,
    is_dead,
    skills,
    damage_stats,
    skill_stats,
    last_update,
    entity_type,
    npc_id,
    character_id,
    engravings,
    spec,
    ark_passive_active,
    ark_passive_data,
    loadout_hash,
    combat_power
FROM entity
WHERE encounter_id = ?;
";

pub const INSERT_ENCOUNTER: &str = r"
INSERT INTO encounter
(
    last_combat_packet,
    total_damage_dealt,
    top_damage_dealt,
    total_damage_taken,
    top_damage_taken,
    dps,
    buffs,
    debuffs,
    total_shielding,
    total_effective_shielding,
    applied_shield_buffs,
    misc,
    version,
    boss_hp_log
)
VALUES
(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)";

pub const INSERT_ENTITY: &str = r"
INSERT INTO entity (
    name,
    encounter_id,
    npc_id,
    entity_type,
    class_id,
    class,
    gear_score,
    current_hp,
    max_hp,
    is_dead,
    skills,
    damage_stats,
    skill_stats,
    dps,
    character_id,
    engravings,
    loadout_hash,
    combat_power,
    ark_passive_active,
    spec,
    ark_passive_data,
    support_ap,
    support_brand,
    support_identity,
    support_hyper
)
VALUES
(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25)";

pub const INSERT_ENCOUNTER_PREVIEW: &str = r"
INSERT INTO encounter_preview
(
    id,
    fight_start,
    current_boss,
    duration,
    players,
    difficulty,
    local_player,
    my_dps,
    cleared,
    boss_only_damage
    )
VALUES
(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)";

pub const DELETE_NOT_FAV_UNCLEARED_ENCOUNTERS: &str = r"
DELETE
FROM encounter
WHERE id IN (
    SELECT id
    FROM encounter_preview
    WHERE cleared = 0 AND favorite = 0
)";

pub const DELETE_UNCLEARED_ENCOUNTERS: &str = r"
DELETE
FROM encounter
WHERE id IN (
    SELECT id
    FROM encounter_preview
    WHERE cleared = 0
)";

pub const DELETE_UNFAVOURITE_ENCOUNTERS: &str = r"
DELETE
FROM encounter
WHERE id IN (
    SELECT id
    FROM encounter_preview
    WHERE favorite = 0
)";

pub const DELETE_ENCOUNTERS: &str = r"DELETE FROM encounter";

pub const DELETE_ENCOUNTER_BY_ID: &str = r"
DELETE FROM encounter
WHERE id = ?;
";

pub const GET_TOP_ENCOUNTER_ID: &str ="
SELECT id
FROM encounter_preview
ORDER BY fight_start DESC
LIMIT 1;
";

pub const SELECT_ENCOUNTER_PREVIEW_COUNT: &str = "SELECT COUNT(*) FROM encounter_preview";
pub const SELECT_ENCOUNTER_PREVIEW_BY_GE_DURATION: &str = "SELECT COUNT(*) FROM encounter_preview WHERE duration >= ?";

pub const SELECT_SYNC_LOGS: &str = r"
SELECT upstream_id
FROM sync_logs
WHERE encounter_id = ? AND failed = false;";

pub const INSERT_SYNC_LOGS: &str = r"
INSERT OR REPLACE INTO sync_logs
(encounter_id, upstream_id, failed)
VALUES
(?, ?, ?);
";

pub const UPDATE_ENCOUNTER_SET_FAV_BY_ID: &str =  "
UPDATE encounter_preview
SET favorite = NOT favorite
WHERE id = ?;
";

pub const DELETE_SHORT_NON_FAVORITE_ENCOUNTERS: &str = r"
DELETE FROM encounter
WHERE id IN (
    SELECT id
    FROM encounter_preview
    WHERE duration < ? AND favorite = 0
);
";

pub const DELETE_SHORT_ENCOUNTERS: &str = r"
DELETE FROM encounter
WHERE id IN (
    SELECT id
    FROM encounter_preview
    WHERE duration < ?
);
";

/// Enables foreign key enforcement for the current SQLite connection.
///
/// By default, SQLite does not enforce foreign key constraints. Running this
/// PRAGMA ensures that inserts, updates, and deletes respect foreign keys
/// defined in the schema. Must be executed for each new connection.
pub const PRAGMA_FOREIGN_KEYS_ON: &str = "PRAGMA foreign_keys = ON;";

/// SQL statement to rebuild and defragment the SQLite database file.
///
/// Running `VACUUM;` performs the following:
/// 1. Reclaims unused space left by deleted or updated rows.
/// 2. Reduces database file size.
/// 3. Reorders and optimizes the database pages for faster I/O.
///
/// Use this after performing large numbers of inserts, updates, or deletes
/// to maintain optimal database performance and file size.
///
/// Note: `VACUUM` can be expensive for large databases because it rewrites the entire file.
///
/// See [SQLite VACUUM](https://www.sqlite.org/lang_vacuum.html)
pub const VACUUM: &str = "VACUUM";

/// This query performs two operations:
/// 1. `INSERT INTO encounter_search(encounter_search) VALUES('optimize');`
///    triggers FTS5 to rebuild and optimize its internal index.
/// 2. `VACUUM;` defragments the database file and reduces its size.
///
/// Use this after performing many updates or inserts to `encounter_preview`
/// to maintain fast full-text search performance.
pub const OPTIMIZE_ENCOUNTER_SEARCH_FTS: &str = "
    INSERT INTO encounter_search
    (encounter_search)
    VALUES
    ('optimize');
    
    VACUUM;
";
