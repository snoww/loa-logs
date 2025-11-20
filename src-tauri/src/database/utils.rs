use anyhow::Result;
use flate2::Compression;
use flate2::write::GzEncoder;
use hashbrown::HashMap;
use semver::Version;
use serde::Serialize;
use std::cmp::{Ordering, max};
use std::collections::BTreeMap;
use std::io::Write;
use std::str::FromStr;

use crate::constants::{WINDOW_MS, WINDOW_S};
use crate::data::{ENGRAVING_DATA, GEM_SKILL_MAP};
use crate::database::sql_types::{CompressedJson, JsonColumn};
use crate::models::*;
use crate::utils::*;

pub const VERSION_1_13_5: Version = Version::new(1, 13, 5);

pub fn build_delete_encounters_query(ids_len: usize) -> String {
    let placeholders = std::iter::repeat_n("?", ids_len)
        .collect::<Vec<_>>()
        .join(",");
    format!("DELETE FROM encounter WHERE id IN ({})", placeholders)
}

pub fn build_sync_candidates_query(force_resync: bool) -> String {
    let upstream_condition = if force_resync { "= '0'" } else { "IS NULL" };
    format!(
        "
        SELECT id
        FROM encounter_preview
        LEFT JOIN sync_logs ON encounter_id = id
        WHERE cleared = true AND boss_only_damage = 1 AND upstream_id {}
        ORDER BY fight_start;
        ",
        upstream_condition
    )
}

pub fn prepare_get_encounter_preview_query(
    search: String,
    filter: SearchFilter,
) -> (Vec<String>, String, String) {
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

    let boss_filter = if filter.bosses.is_empty() {
        if let Some(raid_type) = filter.raid_type {
            let bosses = raid_type.get_bosses();
            let mut placeholders = "?,".repeat(bosses.len());
            placeholders.pop();
            params.extend(bosses.iter().map(|s| s.to_string()));
            format!("AND e.current_boss IN ({})", placeholders)
        } else {
            "".to_string()
        }
    } else {  
        let mut placeholders = "?,".repeat(filter.bosses.len());
        placeholders.pop(); // remove trailing comma
        params.extend(filter.bosses);
        format!("AND e.current_boss IN ({})", placeholders)
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

    let raids_only_filter = if filter.raids_only {
        "AND difficulty IS NOT NULL and difficulty != ''"
    } else {
        ""
    };

    let query = format!(
        "SELECT
    e.id,               -- 0
    e.fight_start,      -- 1
    e.current_boss,     -- 2
    e.duration,         -- 3
    e.difficulty,       -- 4
    e.favorite,         -- 5
    e.cleared,          -- 6
    e.local_player,     -- 7
    e.my_dps,           -- 8
    e.players,          -- 9
    le.spec,            -- 10
    le.support_ap,      -- 11
    le.support_brand,   -- 12
    le.support_identity,-- 13
    le.support_hyper,   -- 14
    le.unbuffed_dps     -- 15
    FROM encounter_preview e
    LEFT JOIN entity le ON le.encounter_id = e.id AND le.name = e.local_player
    {}
    WHERE e.duration > ? {}
    {} {} {} {} {}
    ORDER BY {} {}
    LIMIT ?
    OFFSET ?",
        join_clause,
        boss_filter,
        raid_clear_filter,
        favorite_filter,
        difficulty_filter,
        raids_only_filter,
        boss_only_damage_filter,
        filter.sort,
        filter.order
    );

    let count_query = format!(
        "SELECT COUNT(*)
        FROM encounter_preview e {join_clause}
        WHERE duration > ? {boss_filter}
        {raid_clear_filter} {raids_only_filter} {favorite_filter} {difficulty_filter} {boss_only_damage_filter}"
    );

    (params, query, count_query)
}

pub fn map_encounter(row: &rusqlite::Row) -> rusqlite::Result<(Encounter, Version)> {
    let misc_str: String = row.get("misc").unwrap_or_default();
    let misc = serde_json::from_str::<EncounterMisc>(misc_str.as_str())
        .map(Some)
        .unwrap_or_default();

    let version: Version = misc
        .as_ref()
        .and_then(|m| m.version.as_ref())
        .and_then(|v| Version::parse(v).ok())
        .unwrap_or_else(|| Version::new(0, 0, 0));

    let encounter_damage_stats = if version >= VERSION_1_13_5 {
        stats_for_1_13_5_and_up(row, misc.clone())?
    } else {
        stats_for_older_versions(row, misc.clone())?
    };

    let encounter = Encounter {
        last_combat_packet: row.get("last_combat_packet")?,
        fight_start: row.get("fight_start")?,
        local_player: row.get("local_player").unwrap_or("You".to_string()),
        current_boss_name: row.get("current_boss")?,
        duration: row.get("duration")?,
        encounter_damage_stats,
        difficulty: row.get("difficulty")?,
        favorite: row.get("favorite")?,
        cleared: row.get("cleared")?,
        boss_only_damage: row.get("boss_only_damage")?,
        ..Default::default()
    };

    Ok((encounter, version))
}

fn stats_for_1_13_5_and_up(
    row: &rusqlite::Row,
    misc: Option<EncounterMisc>,
) -> rusqlite::Result<EncounterDamageStats> {
    let CompressedJson(boss_hp_log): CompressedJson<HashMap<String, Vec<BossHpLog>>> =
        row.get("boss_hp_log")?;
    let CompressedJson(buffs): CompressedJson<HashMap<u32, StatusEffect>> = row.get("buffs")?;
    let CompressedJson(debuffs): CompressedJson<HashMap<u32, StatusEffect>> = row.get("debuffs")?;
    let CompressedJson(applied_shield_buffs): CompressedJson<HashMap<u32, StatusEffect>> =
        row.get("applied_shield_buffs")?;

    let total_shielding = row.get("total_shielding").unwrap_or_default();
    let total_effective_shielding = row.get("total_effective_shielding").unwrap_or_default();

    Ok(EncounterDamageStats {
        total_damage_dealt: row.get("total_damage_dealt")?,
        top_damage_dealt: row.get("top_damage_dealt")?,
        total_damage_taken: row.get("total_damage_taken")?,
        top_damage_taken: row.get("top_damage_taken")?,
        dps: row.get("dps")?,
        buffs,
        debuffs,
        misc,
        total_shielding,
        total_effective_shielding,
        applied_shield_buffs,
        boss_hp_log,
        ..Default::default()
    })
}

fn stats_for_older_versions(
    row: &rusqlite::Row,
    misc: Option<EncounterMisc>,
) -> rusqlite::Result<EncounterDamageStats> {
    let boss_hp_log: HashMap<String, Vec<BossHpLog>> = misc
        .as_ref()
        .and_then(|pr| pr.boss_hp_log.clone())
        .unwrap_or_default();

    let JsonColumn(buffs): JsonColumn<HashMap<u32, StatusEffect>> = row.get("buffs")?;
    let JsonColumn(debuffs): JsonColumn<HashMap<u32, StatusEffect>> = row.get("debuffs")?;
    let JsonColumn(applied_shield_buffs): JsonColumn<HashMap<u32, StatusEffect>> =
        row.get("applied_shield_buffs")?;

    let total_shielding = row.get("total_shielding").unwrap_or_default();
    let total_effective_shielding = row.get("total_effective_shielding").unwrap_or_default();

    Ok(EncounterDamageStats {
        total_damage_dealt: row.get("total_damage_dealt")?,
        top_damage_dealt: row.get("top_damage_dealt")?,
        total_damage_taken: row.get("total_damage_taken")?,
        top_damage_taken: row.get("top_damage_taken")?,
        dps: row.get("dps")?,
        buffs,
        debuffs,
        misc,
        total_shielding,
        total_effective_shielding,
        applied_shield_buffs,
        boss_hp_log,
        ..Default::default()
    })
}

pub fn map_encounter_preview(row: &rusqlite::Row) -> rusqlite::Result<EncounterPreview> {
    let classes_str: String = row.get("players").unwrap_or_default();
    let (classes, names) = parse_class_names(classes_str);

    Ok(EncounterPreview {
        id: row.get("id")?,
        fight_start: row.get("fight_start")?,
        boss_name: row.get("current_boss")?,
        duration: row.get("duration")?,
        classes,
        names,
        difficulty: row.get("difficulty")?,
        favorite: row.get("favorite")?,
        cleared: row.get("cleared")?,
        local_player: row.get("local_player")?,
        my_dps: row.get("my_dps").unwrap_or(0),
        spec: row.get("spec").unwrap_or_default(),
        support_ap: row.get("support_ap").unwrap_or_default(),
        support_brand: row.get("support_brand").unwrap_or_default(),
        support_identity: row.get("support_identity").unwrap_or_default(),
        support_hyper: row.get("support_hyper").unwrap_or_default(),
        udps: row.get("unbuffed_dps").unwrap_or_default(),
    })
}

pub fn map_entity(row: &rusqlite::Row, version: &Version) -> rusqlite::Result<EncounterEntity> {
    let (skills, damage_stats) = if version >= &VERSION_1_13_5 {
        let CompressedJson(skills): CompressedJson<HashMap<u32, Skill>> = row.get("skills")?;
        let CompressedJson(damage_stats): CompressedJson<DamageStats> = row.get("damage_stats")?;

        (skills, damage_stats)
    } else {
        let JsonColumn(skills): JsonColumn<HashMap<u32, Skill>> = row.get("skills")?;
        let JsonColumn(damage_stats): JsonColumn<DamageStats> = row.get("damage_stats")?;

        (skills, damage_stats)
    };

    let JsonColumn(skill_stats): JsonColumn<SkillStats> = row.get("skill_stats")?;
    let entity_type: String = row.get("entity_type").unwrap_or_default();
    let engraving_data: Option<Vec<String>> = row
        .get::<_, JsonColumn<Option<Vec<String>>>>("engravings")
        .ok()
        .and_then(|JsonColumn(inner)| inner);
    let spec: Option<String> = row.get("spec").unwrap_or_default();
    let ark_passive_active: Option<bool> = row.get("ark_passive_active").unwrap_or_default();
    let JsonColumn(ark_passive_data): JsonColumn<Option<ArkPassiveData>> =
        row.get("ark_passive_data")?;

    let entity = EncounterEntity {
        name: row.get("name")?,
        class_id: row.get("class_id")?,
        class: row.get("class")?,
        gear_score: row.get("gear_score")?,
        current_hp: row.get("current_hp")?,
        max_hp: row.get("max_hp")?,
        is_dead: row.get("is_dead")?,
        skills,
        damage_stats,
        skill_stats,
        entity_type: EntityType::from_str(entity_type.as_str()).unwrap_or_default(),
        npc_id: row.get("npc_id")?,
        character_id: row.get("character_id").unwrap_or_default(),
        engraving_data,
        spec,
        ark_passive_active,
        ark_passive_data,
        loadout_hash: row.get("loadout_hash").unwrap_or_default(),
        combat_power: row.get("combat_power").unwrap_or_default(),
        ..Default::default()
    };

    Ok(entity)
}

fn parse_class_names(input: String) -> (Vec<i32>, Vec<String>) {
    input
        .split(',')
        .map(|s| {
            let parts: Vec<&str> = s.split(':').collect();
            if parts.len() != 2 {
                (101, "Unknown".to_string())
            } else {
                (parts[0].parse::<i32>().unwrap_or(101), parts[1].to_string())
            }
        })
        .unzip()
}

pub fn get_total_available_time(
    skill_cooldown: &Vec<CastEvent>,
    encounter_start: i64,
    encounter_end: i64,
) -> i64 {
    let mut total_available_time = 0;
    let mut current_available_from = encounter_start;

    for event in skill_cooldown {
        if event.timestamp > current_available_from {
            total_available_time += event.timestamp - current_available_from;
        }

        let cooldown_end = event.timestamp + event.cooldown_duration_ms;
        current_available_from = current_available_from.max(cooldown_end);
    }

    if encounter_end > current_available_from {
        total_available_time += encounter_end - current_available_from;
    }

    total_available_time
}

pub fn should_insert_entity(entity: &EncounterEntity, local_player: &str) -> bool {
    ((entity.entity_type == EntityType::Player && entity.class_id > 0)
        || entity.name == local_player
        || entity.entity_type == EntityType::Esther
        || (entity.entity_type == EntityType::Boss && entity.max_hp > 0))
        && entity.damage_stats.damage_dealt > 0
}

pub fn update_entity_stats(
    entity: &mut EncounterEntity,
    fight_start: i64,
    fight_end: i64,
    damage_log: &HashMap<String, Vec<(i64, i64)>>,
) {
    if entity.entity_type != EntityType::Player {
        return;
    }

    let duration = fight_end - fight_start;
    let duration_seconds = max(duration / 1000, 1);
    let intervals = generate_intervals(fight_start, fight_end);

    if let Some(player_log) = damage_log.get(&entity.name) {
        for interval in intervals {
            let start = fight_start + interval - WINDOW_MS;
            let end = fight_start + interval + WINDOW_MS;
            let damage = sum_in_range(player_log, start, end);
            entity
                .damage_stats
                .dps_rolling_10s_avg
                .push(damage / (WINDOW_S * 2));
        }

        let fight_start_sec = fight_start / 1000;
        let fight_end_sec = fight_end / 1000;
        entity.damage_stats.dps_average =
            calculate_average_dps(player_log, fight_start_sec, fight_end_sec);
    }

    let mut buffed_damage = 0;
    for skill in entity.skills.values() {
        for (rdps_type, entry) in skill.rdps_received.iter() {
            if matches!(*rdps_type, 1 | 3 | 5) {
                buffed_damage += entry.values().sum::<i64>();
            }
        }
    }

    let unbuffed_damage = entity.damage_stats.damage_dealt - buffed_damage;
    let unbuffed_dps = unbuffed_damage / duration_seconds;

    entity.damage_stats.dps = entity.damage_stats.damage_dealt / duration_seconds;
    entity.damage_stats.unbuffed_damage = unbuffed_damage;
    entity.damage_stats.unbuffed_dps = unbuffed_dps;

    for (_, skill) in entity.skills.iter_mut() {
        skill.dps = skill.total_damage / duration_seconds;
    }
}

pub fn apply_player_info(entity: &mut EncounterEntity, info: &InspectInfo) {
    for gem in info.gems.iter().flatten() {
        let skill_ids = if matches!(gem.gem_type, 34 | 35 | 65 | 63 | 61) {
            GEM_SKILL_MAP
                .get(&gem.skill_id)
                .cloned()
                .unwrap_or_default()
        } else {
            vec![gem.skill_id]
        };

        for skill_id in skill_ids {
            if let Some(skill) = entity.skills.get_mut(&skill_id) {
                match gem.gem_type {
                    27 | 35 => {
                        skill.gem_cooldown = Some(cooldown_gem_value_to_level(gem.value, gem.tier));
                        skill.gem_tier = Some(gem.tier);
                    }
                    64 | 65 => {
                        skill.gem_damage = Some(support_damage_gem_value_to_level(gem.value));
                        skill.gem_tier_dmg = Some(gem.tier);
                    }
                    _ => {
                        skill.gem_damage = Some(damage_gem_value_to_level(gem.value, gem.tier));
                        skill.gem_tier_dmg = Some(gem.tier);
                    }
                }
            }
        }
    }

    entity.ark_passive_active = Some(info.ark_passive_enabled);
    entity.engraving_data = get_engravings(&info.engravings);
    entity.ark_passive_data = info.ark_passive_data.clone();
    entity.loadout_hash = info.loadout_snapshot.clone();
    entity.combat_power = info.combat_power.as_ref().map(|c| c.score);

    // Set spec for special cases
    if entity.class_id == 104
        && let Some(engr) = &entity.engraving_data
        && engr
            .iter()
            .any(|e| e == "Awakening" || e == "Drops of Ether")
    {
        entity.spec = Some("Princess".to_string());
    }

    // Fallback spec detection
    if entity.spec.as_deref() == Some("Unknown")
        && let Some(tree) = info.ark_passive_data.as_ref()
        && let Some(enlightenment) = tree.enlightenment.as_ref()
    {
        for node in enlightenment.iter() {
            let spec = get_spec_from_ark_passive(node);
            if spec != "Unknown" {
                entity.spec = Some(spec);
                break;
            }
        }
    }
}

pub fn apply_cast_logs(
    entity: &mut EncounterEntity,
    cast_log: &HashMap<String, HashMap<u32, Vec<i32>>>,
    skill_cast_log: &HashMap<u64, HashMap<u32, BTreeMap<i64, SkillCast>>>,
) {
    if let Some(player_log) = cast_log.get(&entity.name) {
        for (skill_id, log) in player_log {
            entity.skills.entry(*skill_id).and_modify(|skill| {
                skill.cast_log.clone_from(log);
            });
        }
    }

    if let Some(entity_log) = skill_cast_log.get(&entity.id) {
        for (skill_id, log) in entity_log {
            entity.skills.entry(*skill_id).and_modify(|skill| {
                let average_cast = skill.total_damage as f64 / skill.casts as f64;
                let filter = average_cast * 0.05;
                let mut adj_hits = 0;
                let mut adj_crits = 0;
                for cast in log.values() {
                    for hit in cast.hits.iter() {
                        if hit.damage as f64 > filter {
                            adj_hits += 1;
                            if hit.crit {
                                adj_crits += 1;
                            }
                        }
                    }
                }
                if adj_hits > 0 {
                    skill.adjusted_crit = Some(adj_crits as f64 / adj_hits as f64);
                }

                skill.max_damage_cast = log
                    .values()
                    .map(|cast| cast.hits.iter().map(|hit| hit.damage).sum::<i64>())
                    .max()
                    .unwrap_or_default();

                skill.skill_cast_log = log.values().cloned().collect();
            });
        }
    }
}

pub fn calculate_average_dps(data: &[(i64, i64)], start_time: i64, end_time: i64) -> Vec<i64> {
    let step = 5;
    let mut results = vec![0; ((end_time - start_time) / step + 1) as usize];
    let mut current_sum = 0;
    let mut data_iter = data.iter();
    let mut current_data = data_iter.next();

    for t in (start_time..=end_time).step_by(step as usize) {
        while let Some((timestamp, value)) = current_data {
            if *timestamp / 1000 <= t {
                current_sum += value;
                current_data = data_iter.next();
            } else {
                break;
            }
        }

        results[((t - start_time) / step) as usize] = current_sum / (t - start_time + 1);
    }

    results
}

pub fn get_damage_without_hyper_or_special(e: &EncounterEntity) -> i64 {
    let hyper = e.damage_stats.hyper_awakening_damage;
    let special = e
        .skills
        .values()
        .filter(|s| s.special.unwrap_or(false))
        .map(|s| s.total_damage)
        .sum::<i64>();
    e.damage_stats.damage_dealt - hyper - special
}

pub fn generate_intervals(start: i64, end: i64) -> Vec<i64> {
    if start >= end {
        return Vec::new();
    }

    (0..end - start).step_by(1_000).collect()
}

pub fn sum_in_range(vec: &[(i64, i64)], start: i64, end: i64) -> i64 {
    let start_idx = binary_search_left(vec, start);
    let end_idx = binary_search_left(vec, end + 1);

    vec[start_idx..end_idx]
        .iter()
        .map(|&(_, second)| second)
        .sum()
}

pub fn binary_search_left(vec: &[(i64, i64)], target: i64) -> usize {
    let mut left = 0;
    let mut right = vec.len();

    while left < right {
        let mid = left + (right - left) / 2;
        match vec[mid].0.cmp(&target) {
            Ordering::Less => left = mid + 1,
            _ => right = mid,
        }
    }

    left
}

pub fn get_engravings(engraving_ids: &Option<Vec<u32>>) -> Option<Vec<String>> {
    let ids = match engraving_ids {
        Some(engravings) => engravings,
        None => return None,
    };
    let mut engravings: Vec<String> = Vec::new();

    for engraving_id in ids.iter() {
        if let Some(engraving_data) = ENGRAVING_DATA.get(engraving_id) {
            engravings.push(engraving_data.name.clone().unwrap_or("Unknown".to_string()));
        }
    }

    engravings.sort_unstable();
    Some(engravings)
}

pub fn compress_json<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    let bytes = serde_json::to_vec(value)?;
    encoder.write_all(&bytes)?;
    let data = encoder.finish()?;
    Ok(data)
}
