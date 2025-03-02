use std::{io::Read, str::FromStr};

use flate2::bufread::GzDecoder;
use hashbrown::HashMap;
use log::info;
use rusqlite::{params, params_from_iter};
use tauri::Manager;

use crate::{constants::LOGS_WINDOW_LABEL, database::get_db_connection, ArkPassiveData, BossHpLog, DamageStats, Encounter, EncounterDamageStats, EncounterEntity, EncounterMisc, EncounterPreview, EncountersOverview, EntityType, SearchFilter, Skill, SkillStats, StaggerStats, StatusEffect};


#[tauri::command]
pub fn load_encounters_preview(
    window: tauri::Window,
    page: i32,
    page_size: i32,
    search: String,
    filter: SearchFilter,
) -> EncountersOverview {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    let conn = get_db_connection(&path).expect("could not get db connection");
    let mut params = vec![];

    let join_clause = if search.len() > 2 {
        let escaped_search = search
            .split_whitespace()
            .map(|word| format!("\"{}\"", word.replace("\"", "")))
            .collect::<Vec<_>>()
            .join(" ");
        params.push(escaped_search);
        "JOIN encounter_search(?) ON encounter_search.rowid = e.id"
    } else {
        ""
    };

    params.push((filter.min_duration * 1000).to_string());

    let boss_filter = if !filter.bosses.is_empty() {
        let mut placeholders = "?,".repeat(filter.bosses.len());
        placeholders.pop(); // remove trailing comma
        params.extend(filter.bosses);
        format!("AND e.current_boss IN ({})", placeholders)
    } else {
        "".to_string()
    };

    let raid_clear_filter = if filter.cleared {
        "AND cleared = 1"
    } else {
        ""
    };

    let favorite_filter = if filter.favorite {
        "AND favorite = 1"
    } else {
        ""
    };

    let boss_only_damage_filter = if filter.boss_only_damage {
        "AND boss_only_damage = 1"
    } else {
        ""
    };

    let difficulty_filter = if !filter.difficulty.is_empty() {
        params.push(filter.difficulty);
        "AND difficulty = ?"
    } else {
        ""
    };

    let order = if filter.order == 1 { "ASC" } else { "DESC" };
    let sort = format!("e.{}", filter.sort);

    let count_params = params.clone();

    let query = format!(
        "SELECT
    e.id,
    e.fight_start,
    e.current_boss,
    e.duration,
    e.difficulty,
    e.favorite,
    e.cleared,
    e.local_player,
    e.my_dps,
    e.players
    FROM encounter_preview e {}
    WHERE e.duration > ? {}
    {} {} {} {}
    ORDER BY {} {}
    LIMIT ?
    OFFSET ?",
        join_clause,
        boss_filter,
        raid_clear_filter,
        favorite_filter,
        difficulty_filter,
        boss_only_damage_filter,
        sort,
        order
    );

    let mut stmt = conn.prepare_cached(&query).unwrap();

    let offset = (page - 1) * page_size;

    params.push(page_size.to_string());
    params.push(offset.to_string());

    let encounter_iter = stmt
        .query_map(params_from_iter(params), |row| {
            let classes: String = row.get(9).unwrap_or_default();

            let (classes, names) = classes
                .split(',')
                .map(|s| {
                    let info: Vec<&str> = s.split(':').collect();
                    if info.len() != 2 {
                        return (101, "Unknown".to_string());
                    }
                    (info[0].parse::<i32>().unwrap_or(101), info[1].to_string())
                })
                .unzip();

            Ok(EncounterPreview {
                id: row.get(0)?,
                fight_start: row.get(1)?,
                boss_name: row.get(2)?,
                duration: row.get(3)?,
                classes,
                names,
                difficulty: row.get(4)?,
                favorite: row.get(5)?,
                cleared: row.get(6)?,
                local_player: row.get(7)?,
                my_dps: row.get(8).unwrap_or(0),
            })
        })
        .expect("could not query encounters");

    let encounters: Vec<EncounterPreview> = encounter_iter.collect::<Result<_, _>>().unwrap();

    let query = format!(
        "
        SELECT COUNT(*)
        FROM encounter_preview e {}
        WHERE duration > ? {}
        {} {} {} {}
        ",
        join_clause,
        boss_filter,
        raid_clear_filter,
        favorite_filter,
        difficulty_filter,
        boss_only_damage_filter
    );

    let count: i32 = conn
        .query_row_and_then(&query, params_from_iter(count_params), |row| row.get(0))
        .expect("could not get encounter count");

    EncountersOverview {
        encounters,
        total_encounters: count,
    }
}

#[tauri::command(async)]
pub fn load_encounter(window: tauri::Window, id: String) -> Encounter {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    let conn = get_db_connection(&path).expect("could not get db connection");
    let mut encounter_stmt = conn
        .prepare_cached(
            "
    SELECT last_combat_packet,
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
       boss_hp_log,
       stagger_log
    FROM encounter JOIN encounter_preview USING (id)
    WHERE id = ?
    ",
        )
        .unwrap();

    let mut compressed = false;
    let mut encounter = encounter_stmt
        .query_row(params![id], |row| {
            let misc_str: String = row.get(12).unwrap_or_default();
            let misc = serde_json::from_str::<EncounterMisc>(misc_str.as_str())
                .map(Some)
                .unwrap_or_default();

            let mut boss_hp_log: HashMap<String, Vec<BossHpLog>> = HashMap::new();
            let mut stagger_stats: Option<StaggerStats> = None;

            if let Some(misc) = misc.as_ref() {
                let version = misc
                    .version
                    .clone()
                    .unwrap_or_default()
                    .split('.')
                    .map(|x| x.parse::<i32>().unwrap_or_default())
                    .collect::<Vec<_>>();

                if version[0] > 1
                    || (version[0] == 1 && version[1] >= 14)
                    || (version[0] == 1 && version[1] == 13 && version[2] >= 5)
                {
                    compressed = true;
                }

                if !compressed {
                    boss_hp_log = misc.boss_hp_log.clone().unwrap_or_default();
                    stagger_stats.clone_from(&misc.stagger_stats);
                }
            }

            let buffs: HashMap<u32, StatusEffect>;
            let debuffs: HashMap<u32, StatusEffect>;
            let applied_shield_buffs: HashMap<u32, StatusEffect>;
            if compressed {
                let raw_bytes: Vec<u8> = row.get(10).unwrap_or_default();
                let mut decompress = GzDecoder::new(raw_bytes.as_slice());
                let mut buff_string = String::new();
                decompress
                    .read_to_string(&mut buff_string)
                    .expect("could not decompress buffs");
                buffs = serde_json::from_str::<HashMap<u32, StatusEffect>>(buff_string.as_str())
                    .unwrap_or_default();

                let raw_bytes: Vec<u8> = row.get(11).unwrap_or_default();
                let mut decompress = GzDecoder::new(raw_bytes.as_slice());
                let mut debuff_string = String::new();
                decompress
                    .read_to_string(&mut debuff_string)
                    .expect("could not decompress debuffs");
                debuffs =
                    serde_json::from_str::<HashMap<u32, StatusEffect>>(debuff_string.as_str())
                        .unwrap_or_default();

                let raw_bytes: Vec<u8> = row.get(19).unwrap_or_default();
                let mut decompress = GzDecoder::new(raw_bytes.as_slice());
                let mut applied_shield_buff_string = String::new();
                decompress
                    .read_to_string(&mut applied_shield_buff_string)
                    .expect("could not decompress applied_shield_buffs");
                applied_shield_buffs = serde_json::from_str::<HashMap<u32, StatusEffect>>(
                    applied_shield_buff_string.as_str(),
                )
                .unwrap_or_default();

                let raw_bytes: Vec<u8> = row.get(20).unwrap_or_default();
                let mut decompress = GzDecoder::new(raw_bytes.as_slice());
                let mut boss_string = String::new();
                decompress
                    .read_to_string(&mut boss_string)
                    .expect("could not decompress boss_hp_log");
                boss_hp_log =
                    serde_json::from_str::<HashMap<String, Vec<BossHpLog>>>(boss_string.as_str())
                        .unwrap_or_default();

                let stagger_str: String = row.get(21).unwrap_or_default();
                stagger_stats = serde_json::from_str::<Option<StaggerStats>>(stagger_str.as_str())
                    .unwrap_or_default();
            } else {
                let buff_str: String = row.get(10).unwrap_or_default();
                buffs = serde_json::from_str::<HashMap<u32, StatusEffect>>(buff_str.as_str())
                    .unwrap_or_default();
                let debuff_str: String = row.get(11).unwrap_or_default();
                debuffs = serde_json::from_str::<HashMap<u32, StatusEffect>>(debuff_str.as_str())
                    .unwrap_or_default();
                let applied_shield_buff_str: String = row.get(19).unwrap_or_default();
                applied_shield_buffs = serde_json::from_str::<HashMap<u32, StatusEffect>>(
                    applied_shield_buff_str.as_str(),
                )
                .unwrap_or_default();
            }

            let total_shielding = row.get(17).unwrap_or_default();
            let total_effective_shielding = row.get(18).unwrap_or_default();

            Ok(Encounter {
                last_combat_packet: row.get(0)?,
                fight_start: row.get(1)?,
                local_player: row.get(2).unwrap_or("You".to_string()),
                current_boss_name: row.get(3)?,
                duration: row.get(4)?,
                encounter_damage_stats: EncounterDamageStats {
                    total_damage_dealt: row.get(5)?,
                    top_damage_dealt: row.get(6)?,
                    total_damage_taken: row.get(7)?,
                    top_damage_taken: row.get(8)?,
                    dps: row.get(9)?,
                    buffs,
                    debuffs,
                    misc,
                    total_shielding,
                    total_effective_shielding,
                    applied_shield_buffs,
                    boss_hp_log,
                    stagger_stats,
                    ..Default::default()
                },
                difficulty: row.get(13)?,
                favorite: row.get(14)?,
                cleared: row.get(15)?,
                boss_only_damage: row.get(16)?,
                ..Default::default()
            })
        })
        .unwrap_or_else(|_| Encounter::default());

    let mut entity_stmt = conn
        .prepare_cached(
            "
    SELECT name,
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
        ark_passive_data
    FROM entity
    WHERE encounter_id = ?;
    ",
        )
        .unwrap();

    let entity_iter = entity_stmt
        .query_map(params![id], |row| {
            let skills: HashMap<u32, Skill>;
            let damage_stats: DamageStats;

            if compressed {
                let raw_bytes: Vec<u8> = row.get(7).unwrap_or_default();
                let mut decompress = GzDecoder::new(raw_bytes.as_slice());
                let mut skill_string = String::new();
                decompress
                    .read_to_string(&mut skill_string)
                    .expect("could not decompress skills");
                skills = serde_json::from_str::<HashMap<u32, Skill>>(skill_string.as_str())
                    .unwrap_or_default();

                let raw_bytes: Vec<u8> = row.get(8).unwrap_or_default();
                let mut decompress = GzDecoder::new(raw_bytes.as_slice());
                let mut damage_stats_string = String::new();
                decompress
                    .read_to_string(&mut damage_stats_string)
                    .expect("could not decompress damage stats");
                damage_stats = serde_json::from_str::<DamageStats>(damage_stats_string.as_str())
                    .unwrap_or_default();
            } else {
                let skill_str: String = row.get(7).unwrap_or_default();
                skills = serde_json::from_str::<HashMap<u32, Skill>>(skill_str.as_str())
                    .unwrap_or_default();

                let damage_stats_str: String = row.get(8).unwrap_or_default();
                damage_stats = serde_json::from_str::<DamageStats>(damage_stats_str.as_str())
                    .unwrap_or_default();
            }

            let skill_stats_str: String = row.get(9).unwrap_or_default();
            let skill_stats =
                serde_json::from_str::<SkillStats>(skill_stats_str.as_str()).unwrap_or_default();

            let entity_type: String = row.get(11).unwrap_or_default();

            let engravings_str: String = row.get(14).unwrap_or_default();
            let engravings = serde_json::from_str::<Option<Vec<String>>>(engravings_str.as_str())
                .unwrap_or_default();

            let spec: Option<String> = row.get(15).unwrap_or_default();
            let ark_passive_active: Option<bool> = row.get(16).unwrap_or_default();

            let ark_passive_data_str: String = row.get(17).unwrap_or_default();
            let ark_passive_data =
                serde_json::from_str::<Option<ArkPassiveData>>(ark_passive_data_str.as_str())
                    .unwrap_or_default();

            Ok(EncounterEntity {
                name: row.get(0)?,
                class_id: row.get(1)?,
                class: row.get(2)?,
                gear_score: row.get(3)?,
                current_hp: row.get(4)?,
                max_hp: row.get(5)?,
                is_dead: row.get(6)?,
                skills,
                damage_stats,
                skill_stats,
                entity_type: EntityType::from_str(entity_type.as_str())
                    .unwrap_or(EntityType::UNKNOWN),
                npc_id: row.get(12)?,
                character_id: row.get(13).unwrap_or_default(),
                engraving_data: engravings,
                spec,
                ark_passive_active,
                ark_passive_data,
                ..Default::default()
            })
        })
        .unwrap();

    let mut entities: HashMap<String, EncounterEntity> = HashMap::new();
    for entity in entity_iter.flatten() {
        entities.insert(entity.name.to_string(), entity);
    }

    let mut sync_stmt = conn
        .prepare_cached(
            "
    SELECT upstream_id
    FROM sync_logs
    WHERE encounter_id = ? AND failed = false;
            ",
        )
        .unwrap();

    let sync: Result<String, rusqlite::Error> = sync_stmt.query_row(params![id], |row| row.get(0));
    encounter.sync = sync.ok();

    encounter.entities = entities;

    encounter
}


#[tauri::command]
pub fn get_sync_candidates(window: tauri::Window, force_resync: bool) -> Vec<i32> {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    let conn = get_db_connection(&path).expect("could not get db connection");
    let query = if force_resync { "= '0'" } else { "IS NULL" };
    let mut stmt = conn
        .prepare_cached(&format!(
            "
    SELECT id
    FROM encounter_preview
    LEFT JOIN sync_logs ON encounter_id = id
    WHERE cleared = true AND boss_only_damage = 1 AND upstream_id {}
    ORDER BY fight_start;
            ",
            query
        ))
        .unwrap();
    let rows = stmt.query_map([], |row| row.get(0)).unwrap();

    let mut ids = Vec::new();
    for id_result in rows {
        ids.push(id_result.unwrap_or(0));
    }
    ids
}

#[tauri::command]
pub fn get_encounter_count(window: tauri::Window) -> i32 {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    let conn = get_db_connection(&path).expect("could not get db connection");
    let mut stmt = conn
        .prepare_cached("SELECT COUNT(*) FROM encounter_preview")
        .unwrap();

    let count: Result<i32, rusqlite::Error> = stmt.query_row(params![], |row| row.get(0));

    count.unwrap_or(0)
}

#[tauri::command]
pub fn open_most_recent_encounter(window: tauri::Window) {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    let conn = get_db_connection(&path).expect("could not get db connection");
    let mut stmt = conn
        .prepare_cached(
            "
    SELECT id
    FROM encounter_preview
    ORDER BY fight_start DESC
    LIMIT 1;
    ",
        )
        .unwrap();

    let id_result: Result<i32, rusqlite::Error> = stmt.query_row(params![], |row| row.get(0));

    if let Some(logs) = window.app_handle().get_window(LOGS_WINDOW_LABEL) {
        match id_result {
            Ok(id) => {
                logs.emit("show-latest-encounter", id.to_string()).unwrap();
            }
            Err(_) => {
                logs.emit("redirect-url", "logs").unwrap();
            }
        }
    }
}

#[tauri::command]
pub fn toggle_encounter_favorite(window: tauri::Window, id: i32) {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");

    let conn = get_db_connection(&path).expect("could not get db connection");
    let mut stmt = conn
        .prepare_cached(
            "
    UPDATE encounter_preview
    SET favorite = NOT favorite
    WHERE id = ?;
    ",
        )
        .unwrap();

    stmt.execute(params![id]).unwrap();
}

#[tauri::command]
pub fn delete_encounter(window: tauri::Window, id: String) {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    let conn = get_db_connection(&path).expect("could not get db connection");
    conn.execute("PRAGMA foreign_keys = ON;", params![])
        .unwrap();
    let mut stmt = conn
        .prepare_cached(
            "
        DELETE FROM encounter
        WHERE id = ?;
    ",
        )
        .unwrap();

    info!("deleting encounter: {}", id);

    stmt.execute(params![id]).unwrap();
}

#[tauri::command]
pub fn delete_encounters(window: tauri::Window, ids: Vec<i32>) {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    let conn = get_db_connection(&path).expect("could not get db connection");
    conn.execute("PRAGMA foreign_keys = ON;", params![])
        .unwrap();

    let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
    let placeholders_str = placeholders.join(",");

    let sql = format!("DELETE FROM encounter WHERE id IN ({})", placeholders_str);
    let mut stmt = conn.prepare_cached(&sql).unwrap();

    info!("deleting encounters: {:?}", ids);

    stmt.execute(params_from_iter(ids)).unwrap();
}

#[tauri::command]
pub fn sync(window: tauri::Window, encounter: i32, upstream: String, failed: bool) {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    let conn = get_db_connection(&path).expect("could not get db connection");

    conn.execute(
        "
        INSERT OR REPLACE INTO sync_logs (encounter_id, upstream_id, failed)
        VALUES(?, ?, ?);
        ",
        params![encounter, upstream, failed],
    )
    .unwrap();
}

#[tauri::command]
pub fn delete_all_uncleared_encounters(window: tauri::Window, keep_favorites: bool) {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    let conn = get_db_connection(&path).expect("could not get db connection");
    if keep_favorites {
        conn.execute(
            "DELETE FROM encounter
            WHERE id IN (
                SELECT id
                FROM encounter_preview
                WHERE cleared = 0 AND favorite = 0
            )",
            [],
        )
        .unwrap();
    } else {
        conn.execute(
            "DELETE FROM encounter
            WHERE id IN (
                SELECT id
                FROM encounter_preview
                WHERE cleared = 0
            )",
            [],
        )
        .unwrap();
    }
    conn.execute("VACUUM", params![]).unwrap();
}

#[tauri::command]
pub fn delete_all_encounters(window: tauri::Window, keep_favorites: bool) {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    let conn = get_db_connection(&path).expect("could not get db connection");

    if keep_favorites {
        conn.execute(
            "DELETE FROM encounter
            WHERE id IN (
                SELECT id
                FROM encounter_preview
                WHERE favorite = 0
            )",
            [],
        )
        .unwrap();
    } else {
        conn.execute("DELETE FROM encounter", []).unwrap();
    }
    conn.execute("VACUUM", []).unwrap();
}

#[tauri::command]
pub fn delete_encounters_below_min_duration(
    window: tauri::Window,
    min_duration: i64,
    keep_favorites: bool,
) {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    let conn = get_db_connection(&path).expect("could not get db connection");
    if keep_favorites {
        conn.execute(
            "DELETE FROM encounter
            WHERE id IN (
                SELECT id
                FROM encounter_preview
                WHERE duration < ? AND favorite = 0
            )",
            params![min_duration * 1000],
        )
        .unwrap();
    } else {
        conn.execute(
            "DELETE FROM encounter
            WHERE id IN (
                SELECT id
                FROM encounter_preview
                WHERE duration < ?
            )",
            params![min_duration * 1000],
        )
        .unwrap();
    }
    conn.execute("VACUUM", params![]).unwrap();
}

