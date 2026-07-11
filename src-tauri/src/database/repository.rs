use anyhow::{Ok, Result};
use chrono::Utc;
use hashbrown::HashMap;
use log::*;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{OptionalExtension, Transaction, params, params_from_iter};
use serde_json::json;
use std::cmp::{Ordering, Reverse, max};
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    constants::DB_VERSION,
    database::sql_types::{CompressedJson, JsonColumn},
    database::{models::*, queries::*, utils::*},
    models::*,
    utils::*,
};
pub struct Repository(r2d2::Pool<SqliteConnectionManager>);

impl Repository {
    pub fn new(connection: r2d2::Pool<SqliteConnectionManager>) -> Self {
        Self(connection)
    }

    pub fn optimize(&self) -> Result<()> {
        let connection = self.0.get()?;
        connection.execute_batch(OPTIMIZE_ENCOUNTER_SEARCH_FTS)?;

        Ok(())
    }

    pub fn insert_sync_logs(&self, args: InsertSyncLogsArgs) -> Result<()> {
        let InsertSyncLogsArgs {
            encounter,
            failed,
            upstream,
        } = args;

        let connection = self.0.get()?;

        let params = params![encounter, upstream, failed];
        connection.execute(INSERT_SYNC_LOGS, params)?;

        Ok(())
    }

    pub fn toggle_encounter_favorite(&self, id: i32) -> Result<()> {
        let connection = self.0.get()?;
        let mut statement = connection.prepare_cached(UPDATE_ENCOUNTER_SET_FAV_BY_ID)?;

        statement.execute(params![id])?;

        Ok(())
    }

    pub fn get_db_stats(&self, min_duration: i64) -> Result<(i32, i32)> {
        let connection = self.0.get()?;

        let encounter_count =
            connection.query_row(SELECT_ENCOUNTER_PREVIEW_COUNT, [], |row| row.get(0))?;

        let params = params![min_duration * 1000];
        let encounter_filtered_count =
            connection.query_row(SELECT_ENCOUNTER_PREVIEW_BY_GE_DURATION, params, |row| {
                row.get(0)
            })?;

        Ok((encounter_count, encounter_filtered_count))
    }

    pub fn delete_encounters(&self, ids: Vec<i32>) -> Result<()> {
        let connection = self.0.get()?;

        connection.execute(PRAGMA_FOREIGN_KEYS_ON, params![])?;

        let query = build_delete_encounters_query(ids.len());
        let mut statement = connection.prepare_cached(&query)?;

        info!("deleting encounters: {:?}", ids);

        let params = params_from_iter(ids);
        statement.execute(params)?;

        Ok(())
    }

    pub fn delete_encounter(&self, id: String) -> Result<()> {
        let connection = self.0.get()?;

        connection.execute(PRAGMA_FOREIGN_KEYS_ON, params![])?;

        let mut statement = connection.prepare_cached(DELETE_ENCOUNTER_BY_ID)?;

        info!("deleting encounter: {}", id);

        statement.execute(params![id])?;

        Ok(())
    }

    pub fn delete_encounters_below_min_duration(
        &self,
        min_duration: i64,
        keep_favorites: bool,
    ) -> Result<()> {
        let connection = self.0.get()?;
        let params = params![min_duration * 1000];

        if keep_favorites {
            connection.execute(DELETE_SHORT_NON_FAVORITE_ENCOUNTERS, params)?;
        } else {
            connection.execute(DELETE_SHORT_ENCOUNTERS, params)?;
        }

        connection.execute(VACUUM, params![])?;

        Ok(())
    }

    pub fn delete_encounters_before(&self, before: i64, keep_favorites: bool) -> Result<()> {
        let connection = self.0.get()?;
        let params = params![before];

        if keep_favorites {
            connection.execute(DELETE_OLDER_NON_FAVORITE_ENCOUNTERS, params)?;
        } else {
            connection.execute(DELETE_OLDER_ENCOUNTERS, params)?;
        }

        connection.execute(VACUUM, params![])?;

        Ok(())
    }

    pub fn get_encounter_preview(
        &self,
        args: GetEncounterPreviewArgs,
    ) -> Result<EncountersOverview> {
        let GetEncounterPreviewArgs {
            filter,
            page,
            page_size,
            search,
        } = args;

        let connection = self.0.get()?;
        let (mut params, query, count_query) = prepare_get_encounter_preview_query(search, filter);
        let count_params = params.clone();

        let mut statement = connection.prepare_cached(&query)?;

        let offset = (page - 1) * page_size;

        params.push(page_size.to_string());
        params.push(offset.to_string());

        let params = params_from_iter(params);
        let encounter_iter = statement.query_map(params, map_encounter_preview)?;

        let encounters: Vec<EncounterPreview> = encounter_iter.collect::<Result<_, _>>()?;

        let count: i32 =
            connection.query_row_and_then(&count_query, params_from_iter(count_params), |row| {
                row.get(0)
            })?;

        let value = EncountersOverview {
            encounters,
            total_encounters: count,
        };

        Ok(value)
    }

    pub fn get_local_characters(&self) -> Result<Vec<CharacterInfo>> {
        let connection = self.0.get()?;
        let mut statement = connection.prepare_cached(SELECT_LOCAL_PLAYERS)?;

        let rows = statement.query_map([], |row| {
            Result::Ok(CharacterInfo {
                name: row.get(0)?,
                class_id: row.get::<_, Option<i32>>(1)?.unwrap_or(0),
                max_gear_score: row.get::<_, Option<f32>>(2)?.unwrap_or(0.0),
                spec: row.get(3)?,
            })
        })?;

        let characters: Vec<CharacterInfo> = rows
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|c| c.name.len() >= 2 && !c.name.chars().any(|ch| ch.is_ascii_digit()))
            .collect();
        Ok(characters)
    }

    pub fn get_character_statistics(
        &self,
        criteria: CharacterStatisticsCriteria,
    ) -> Result<CharacterStatistics> {
        let connection = self.0.get()?;
        let characters = self.get_local_characters()?;
        let character = characters
            .iter()
            .find(|c| c.name == criteria.character)
            .cloned()
            .unwrap_or_else(|| CharacterInfo {
                name: criteria.character.clone(),
                ..Default::default()
            });

        let mode = criteria.mode.clone();
        let damage_type = criteria.damage_type.clone();
        let character_name = criteria.character.clone();
        let boss_to_raid = criteria.boss_to_raid.clone();
        let (params, query) = build_character_statistics_query(criteria);
        let mut rows = connection
            .prepare_cached(&query)?
            .query_map(params_from_iter(params), map_character_statistics_row)?
            .collect::<Result<Vec<_>, _>>()?;

        if mode == "support" {
            populate_support_contribution_denominators(&connection, &mut rows, &character_name);
        }

        for row in rows.iter_mut() {
            row.raid_name = boss_to_raid.get(&row.boss_name).cloned();
        }

        Ok(build_character_statistics(
            character,
            rows,
            mode.as_str(),
            damage_type.as_str(),
        ))
    }

    pub fn get_raid_progression_statistics(
        &self,
        criteria: RaidProgressionCriteria,
    ) -> Result<RaidProgressionStatistics> {
        let connection = self.0.get()?;
        let boss_to_raid = criteria.boss_to_raid.clone();
        let boss_order = criteria.boss_order.clone();
        let last_gate_bosses = criteria.last_gate_bosses.clone();
        let (params, query) = build_raid_progression_query(criteria);
        let rows = connection
            .prepare_cached(&query)?
            .query_map(params_from_iter(params), map_raid_progression_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(build_raid_progression_statistics(
            rows,
            &boss_to_raid,
            &boss_order,
            &last_gate_bosses,
        ))
    }

    pub fn get_raid_progression_range(
        &self,
        criteria: RaidProgressionRangeCriteria,
    ) -> Result<RaidProgressionRange> {
        let connection = self.0.get()?;
        let (params, query) = build_raid_progression_range_query(criteria);

        let range = connection.query_row(&query, params_from_iter(params), |row| {
            std::result::Result::Ok(RaidProgressionRange {
                first_pull: row.get("first_pull")?,
                first_clear: row.get("first_clear")?,
            })
        })?;

        Ok(range)
    }

    pub fn delete_all_uncleared_encounters(&self, keep_favorites: bool) -> Result<()> {
        let connection = self.0.get()?;

        if keep_favorites {
            connection.execute(DELETE_NOT_FAV_UNCLEARED_ENCOUNTERS, [])?;
        } else {
            connection.execute(DELETE_UNCLEARED_ENCOUNTERS, [])?;
        }

        connection.execute(VACUUM, params![])?;

        Ok(())
    }

    pub fn delete_all_encounters(&self, keep_favorites: bool) -> Result<()> {
        let connection = self.0.get()?;

        if keep_favorites {
            connection.execute(DELETE_UNFAVOURITE_ENCOUNTERS, [])?;
        } else {
            connection.execute(DELETE_ENCOUNTERS, [])?;
        }

        connection.execute(VACUUM, [])?;

        Ok(())
    }

    pub fn get_encounter(&self, id: &str) -> Result<Encounter> {
        let connection = self.0.get()?;
        let mut statement = connection.prepare_cached(SELECT_FROM_ENCOUNTER_JOIN_PREVIEW)?;

        let (mut encounter, version) = statement.query_row(params![id], map_encounter)?;

        let mut statement = connection.prepare_cached(SELECT_ENTITIES_BY_ENCOUNTER)?;

        let entities_query = statement.query_map(params![id], |row| map_entity(row, &version))?;

        let mut entities: HashMap<String, EncounterEntity> = HashMap::new();
        for entity in entities_query {
            let entity = entity?;
            entities.insert(entity.name.to_string(), entity);
        }

        let mut statement = connection.prepare_cached(SELECT_SYNC_LOGS)?;

        let sync: Option<String> = statement
            .query_row(params![id], |row| row.get(0))
            .optional()?;
        encounter.sync = sync;

        encounter.entities = entities;
        normalize_encounter_damage_totals(&mut encounter);

        Ok(encounter)
    }

    pub fn get_last_encounter_id(&self) -> Result<Option<i32>> {
        let connection = self.0.get()?;
        let mut statement = connection.prepare_cached(GET_TOP_ENCOUNTER_ID)?;

        let id: Option<i32> = statement
            .query_row(params![], |row| row.get(0))
            .optional()?;

        Ok(id)
    }

    pub fn get_encounter_count(&self) -> Result<i32> {
        let connection = self.0.get()?;
        let mut statement = connection.prepare_cached(SELECT_ENCOUNTER_PREVIEW_COUNT)?;

        let count: Result<i32, rusqlite::Error> = statement.query_row(params![], |row| row.get(0));

        let count = count.unwrap_or(0);

        Ok(count)
    }

    pub fn get_last_encounter_version(&self) -> Result<Option<String>> {
        let connection = self.0.get()?;
        let mut statement = connection.prepare_cached(SELECT_MOST_RECENT_ENCOUNTER_MISC)?;

        let misc = statement.query_row(params![], |row| row.get::<_, String>(0))?;
        let misc: EncounterMisc = serde_json::from_str(&misc)?;

        Ok(misc.version)
    }

    pub fn get_sync_candidates(&self, force_resync: bool) -> Result<Vec<i32>> {
        let connection = self.0.get()?;

        let query = build_sync_candidates_query(force_resync);
        let mut statement = connection.prepare_cached(&query)?;
        let rows = statement.query_map([], |row| row.get(0))?;

        let mut ids = Vec::new();

        for id_result in rows {
            ids.push(id_result.unwrap_or(0));
        }

        Ok(ids)
    }

    pub fn insert_data(&self, mut args: InsertEncounterArgs) -> Result<i64> {
        normalize_encounter_damage_totals(&mut args.encounter);
        if !args.rdps_valid {
            sanitize_invalid_rdps(&mut args.encounter, &mut args.contribution_splits);
        }

        let mut connection = self.0.get()?;
        let transaction = connection.transaction()?;

        let last_insert_id = self.insert_encounter(&transaction, &args)?;
        calculate_entities(&mut args)?;
        let buffs = compute_support_buffs(&args.encounter, &args.party_info);
        self.insert_entities(&transaction, &args, buffs, last_insert_id)?;
        self.insert_encounter_preview(&transaction, args, last_insert_id)?;

        transaction.commit()?;

        Ok(last_insert_id)
    }

    fn insert_encounter(
        &self,
        transaction: &rusqlite::Transaction,
        args: &InsertEncounterArgs,
    ) -> Result<i64> {
        let InsertEncounterArgs {
            encounter,
            boss_hp_log,
            raid_clear,
            party_info,
            meter_version,
            rdps_valid,
            rdps_message,
            ntp_fight_start,
            region,
            intermission_start,
            intermission_end,
            contribution_splits,
            ..
        } = args;

        let intermission_duration = match (intermission_start, intermission_end) {
            (Some(start), Some(end)) => end - start,
            _ => 0,
        };

        let duration = encounter.last_combat_packet - encounter.fight_start - intermission_duration;
        let duration_seconds = max(duration / 1000, 1);
        let mut stats = encounter.encounter_damage_stats.clone();
        stats.dps = stats.total_damage_dealt / duration_seconds;

        let misc = EncounterMisc {
            raid_clear: (*raid_clear).then_some(true),
            party_info: if party_info.is_empty() {
                None
            } else {
                Some(
                    party_info
                        .iter()
                        .enumerate()
                        .map(|(idx, party)| (idx as i32, party.clone()))
                        .collect(),
                )
            },
            region: region.clone(),
            version: Some(meter_version.clone()),
            rdps_valid: Some(*rdps_valid),
            rdps_message: if *rdps_valid {
                None
            } else {
                rdps_message
                    .clone()
                    .or_else(|| Some("invalid_stats".into()))
            },
            ntp_fight_start: Some(*ntp_fight_start),
            manual_save: Some(args.manual),
            intermission_start: *intermission_start,
            intermission_end: *intermission_end,
            contribution_splits: if contribution_splits.is_empty() {
                None
            } else {
                Some(contribution_splits.clone())
            },
            ..Default::default()
        };

        let params = params![
            encounter.last_combat_packet,
            stats.total_damage_dealt,
            stats.top_damage_dealt,
            stats.total_damage_taken,
            stats.top_damage_taken,
            stats.dps,
            compress_json(&stats.buffs)?,
            compress_json(&stats.debuffs)?,
            stats.total_shielding,
            stats.total_effective_shielding,
            compress_json(&stats.applied_shield_buffs)?,
            json!(misc),
            DB_VERSION,
            compress_json(boss_hp_log)?,
        ];

        let mut statement = transaction.prepare_cached(INSERT_ENCOUNTER)?;
        statement.execute(params)?;

        Ok(transaction.last_insert_rowid())
    }

    fn insert_entities(
        &self,
        transaction: &Transaction,
        args: &InsertEncounterArgs,
        buffs: HashMap<String, SupportBuffs>,
        encounter_id: i64,
    ) -> Result<()> {
        let InsertEncounterArgs { encounter, .. } = args;

        let mut statement = transaction.prepare_cached(INSERT_ENTITY)?;

        for (_name, entity) in encounter.entities.iter() {
            if !should_insert_entity(entity, &encounter.local_player) {
                continue;
            }

            let damage_dealt = entity.damage_stats.damage_dealt;
            let damage_without_hyper =
                (damage_dealt - entity.damage_stats.hyper_awakening_damage) as f64;
            let support_ratio = |damage: i64| {
                if damage_without_hyper > 0.0 {
                    damage as f64 / damage_without_hyper
                } else {
                    0.0
                }
            };
            let compressed_skills = compress_json(&entity.skills)?;
            let compressed_damage_stats = compress_json(&entity.damage_stats)?;

            let support_buffs = buffs.get(&entity.name);

            let params = params![
                entity.name,
                encounter_id,
                entity.npc_id,
                entity.hp_bars,
                entity.entity_type.to_string(),
                entity.class_id,
                entity.class,
                entity.gear_score,
                entity.current_hp,
                entity.max_hp,
                entity.is_dead,
                compressed_skills,
                compressed_damage_stats,
                json!(entity.skill_stats),
                entity.damage_stats.dps,
                entity.character_id,
                json!(entity.engraving_data),
                entity.loadout_hash,
                entity.combat_power,
                entity.ark_passive_active,
                entity.spec,
                json!(entity.ark_passive_data),
                support_buffs
                    .map(|b| b.buff)
                    .unwrap_or_else(|| support_ratio(entity.damage_stats.buffed_by_support)),
                support_buffs
                    .map(|b| b.brand)
                    .unwrap_or_else(|| support_ratio(entity.damage_stats.debuffed_by_support)),
                support_buffs
                    .map(|b| b.identity)
                    .unwrap_or_else(|| support_ratio(entity.damage_stats.buffed_by_identity)),
                support_buffs
                    .map(|b| b.hyper)
                    .unwrap_or_else(|| support_ratio(entity.damage_stats.buffed_by_hat)),
                entity.damage_stats.unbuffed_damage,
                entity.damage_stats.unbuffed_dps,
                entity.damage_stats.rdps_damage_received,
                entity.damage_stats.rdps_damage_received_support,
                entity.damage_stats.rdps_damage_given,
                entity.damage_stats.rdps,
                entity.damage_stats.ndps
            ];

            statement.execute(params)?;
        }

        Ok(())
    }

    fn insert_encounter_preview(
        &self,
        transaction: &Transaction,
        args: InsertEncounterArgs,
        encounter_id: i64,
    ) -> Result<()> {
        let InsertEncounterArgs {
            encounter,
            raid_clear,
            raid_difficulty,
            intermission_start,
            intermission_end,
            ..
        } = args;

        let mut players: Vec<_> = encounter
            .entities
            .values()
            .filter(|e| {
                (is_confirmed_player_entity(e, &encounter.local_player)
                    || e.name == encounter.local_player)
                    && e.damage_stats.damage_dealt > 0
            })
            .collect();

        let local_player = players.iter().find(|e| e.name == encounter.local_player);
        let local_player_dps = local_player.map(|e| e.damage_stats.dps).unwrap_or_default();
        let local_player_rdps = local_player
            .map(|e| e.damage_stats.rdps)
            .unwrap_or_default();
        let local_player_ndps = local_player
            .map(|e| e.damage_stats.ndps)
            .unwrap_or_default();

        players.sort_unstable_by_key(|e| Reverse(e.damage_stats.damage_dealt));

        let preview_players = players
            .iter()
            .map(|e| format!("{}:{}", e.class_id, e.name))
            .collect::<Vec<_>>()
            .join(",");

        let intermission_duration = match (intermission_start, intermission_end) {
            (Some(start), Some(end)) => end - start,
            _ => 0,
        };

        let params = params![
            encounter_id,
            encounter.fight_start,
            encounter.current_boss_name,
            encounter.last_combat_packet - encounter.fight_start - intermission_duration,
            preview_players,
            raid_difficulty,
            encounter.local_player,
            local_player_dps,
            raid_clear,
            encounter.boss_only_damage,
            local_player_rdps,
            local_player_ndps,
        ];

        transaction
            .prepare_cached(INSERT_ENCOUNTER_PREVIEW)?
            .execute(params)?;

        Ok(())
    }
}

#[derive(Clone)]
struct CharacterStatisticsRow {
    id: i32,
    fight_start: i64,
    boss_name: String,
    raid_name: Option<String>,
    duration: i64,
    difficulty: Option<String>,
    cleared: bool,
    my_dps: i64,
    my_rdps: Option<i64>,
    my_ndps: Option<i64>,
    udps: Option<i64>,
    rdps_damage_given: i64,
    support_party_damage: i64,
    support_ap: Option<f32>,
    support_brand: Option<f32>,
    support_identity: Option<f32>,
    support_hyper: Option<f32>,
}

#[derive(Clone)]
struct RaidProgressionRow {
    id: i32,
    fight_start: i64,
    boss_name: String,
    gate: String,
    duration: i64,
    difficulty: Option<String>,
    cleared: bool,
    team_dps: i64,
    damage_taken: i64,
    local_player: String,
    boss_hp_bars: Option<i32>,
    boss_hp_bars_by_name: HashMap<String, i32>,
    boss_current_hp: Option<i64>,
    boss_max_hp: Option<i64>,
    boss_hp_log: HashMap<String, Vec<BossHpLog>>,
    player_name: String,
    class_id: i32,
    class_name: String,
    spec: Option<String>,
    player_damage_dealt: i64,
    dps: i64,
    rdps: Option<i64>,
    ndps: Option<i64>,
    player_damage_taken: i64,
    death_events: Vec<i64>,
    rdps_damage_given: i64,
    party_info: Option<HashMap<i32, Vec<String>>>,
    support_ap: Option<f32>,
    support_brand: Option<f32>,
    support_identity: Option<f32>,
    support_hyper: Option<f32>,
}

#[derive(Clone)]
struct RaidProgressionPullAggregate {
    id: i32,
    fight_start: i64,
    boss_name: String,
    gate: String,
    duration: i64,
    difficulty: Option<String>,
    cleared: bool,
    team_dps: i64,
    damage_taken: i64,
    deaths: i32,
    progress_bars: Option<i32>,
    progress_percent: Option<f32>,
    progress_rank: i32,
    local_player: String,
    player_count: i32,
}

struct RaidProgressionPlayerAggregate {
    name: String,
    class_id: i32,
    class_name: String,
    spec: Option<String>,
    pulls: i32,
    clears: i32,
    dps_values: Vec<i64>,
    rdps_values: Vec<i64>,
    ndps_values: Vec<i64>,
    damage_taken_values: Vec<i64>,
    total_deaths: i32,
    support_ap_values: Vec<f32>,
    support_contribution_values: Vec<f32>,
    support_brand_values: Vec<f32>,
    support_identity_values: Vec<f32>,
    support_hyper_values: Vec<f32>,
    last_seen: i64,
}

fn build_character_statistics_query(
    criteria: CharacterStatisticsCriteria,
) -> (Vec<String>, String) {
    let mut params = vec![criteria.character];
    let min_duration = if criteria.min_duration > 0 {
        criteria.min_duration
    } else {
        10
    };
    params.push((min_duration * 1000).to_string());

    let mut filters = vec![
        "e.local_player = ?".to_string(),
        "e.duration > ?".to_string(),
        "e.difficulty IS NOT NULL AND e.difficulty != ''".to_string(),
    ];

    let (range_start, range_end) = reset_window_for_range(criteria.range.as_str());
    if let Some(start) = criteria.start_time.or(range_start) {
        filters.push("e.fight_start >= ?".to_string());
        params.push(start.to_string());
    }

    if let Some(end) = criteria.end_time.or(range_end) {
        filters.push("e.fight_start <= ?".to_string());
        params.push(end.to_string());
    }

    if !criteria.difficulty.is_empty() {
        filters.push("e.difficulty = ?".to_string());
        params.push(criteria.difficulty);
    }

    if !criteria.bosses.is_empty() {
        let placeholders = "?,".repeat(criteria.bosses.len());
        let placeholders = placeholders.trim_end_matches(',');
        filters.push(format!("e.current_boss IN ({})", placeholders));
        params.extend(criteria.bosses);
    }

    if !criteria.excluded_bosses.is_empty() {
        let placeholders = "?,".repeat(criteria.excluded_bosses.len());
        let placeholders = placeholders.trim_end_matches(',');
        filters.push(format!("e.current_boss NOT IN ({})", placeholders));
        params.extend(criteria.excluded_bosses);
    }

    if !criteria.included_specs.is_empty() {
        let placeholders = "?,".repeat(criteria.included_specs.len());
        let placeholders = placeholders.trim_end_matches(',');
        filters.push(format!("le.spec IN ({})", placeholders));
        params.extend(criteria.included_specs);
    }

    if !criteria.excluded_specs.is_empty() {
        let placeholders = "?,".repeat(criteria.excluded_specs.len());
        let placeholders = placeholders.trim_end_matches(',');
        filters.push(format!(
            "(le.spec IS NULL OR le.spec NOT IN ({}))",
            placeholders
        ));
        params.extend(criteria.excluded_specs);
    }

    let query = format!(
        "SELECT
            e.id,
            e.fight_start,
            e.current_boss,
            e.duration,
            e.difficulty,
            e.cleared,
            e.my_dps,
            e.my_rdps,
            e.my_ndps,
            le.unbuffed_dps,
            le.rdps_damage_given,
            enc.total_damage_dealt,
            le.support_ap,
            le.support_brand,
            le.support_identity,
            le.support_hyper
        FROM encounter_preview e
        LEFT JOIN encounter enc ON enc.id = e.id
        LEFT JOIN entity le ON le.encounter_id = e.id AND le.name = e.local_player
        WHERE {}
        ORDER BY e.fight_start DESC",
        filters.join(" AND ")
    );

    (params, query)
}

fn build_raid_progression_query(criteria: RaidProgressionCriteria) -> (Vec<String>, String) {
    let min_duration = if criteria.min_duration > 0 {
        criteria.min_duration
    } else {
        10
    };
    let mut params = vec![(min_duration * 1000).to_string()];
    let mut filters = vec![
        "e.duration > ?".to_string(),
        "e.difficulty IS NOT NULL AND e.difficulty != ''".to_string(),
    ];

    let (range_start, range_end) = reset_window_for_range(criteria.range.as_str());
    if let Some(start) = criteria.start_time.or(range_start) {
        filters.push("e.fight_start >= ?".to_string());
        params.push(start.to_string());
    }

    if let Some(end) = criteria.end_time.or(range_end) {
        filters.push("e.fight_start <= ?".to_string());
        params.push(end.to_string());
    }

    if !criteria.difficulty.is_empty() {
        filters.push("e.difficulty = ?".to_string());
        params.push(criteria.difficulty);
    }

    if !criteria.bosses.is_empty() {
        let placeholders = "?,".repeat(criteria.bosses.len());
        let placeholders = placeholders.trim_end_matches(',');
        filters.push(format!("e.current_boss IN ({})", placeholders));
        params.extend(criteria.bosses);
    }

    let query = format!(
        "SELECT
            e.id,
            e.fight_start,
            e.current_boss,
            e.duration,
            e.difficulty,
            e.cleared,
            enc.dps,
            enc.total_damage_taken,
            e.local_player,
            boss.hp_bars,
            (
                SELECT json_group_object(boss_entity.name, boss_entity.hp_bars)
                FROM entity boss_entity
                WHERE boss_entity.encounter_id = e.id
                    AND boss_entity.entity_type = 'BOSS'
                    AND boss_entity.hp_bars IS NOT NULL
            ) AS boss_hp_bars_by_name,
            boss.current_hp AS boss_current_hp,
            boss.max_hp AS boss_max_hp,
            enc.boss_hp_log,
            enc.misc,
            p.name AS player_name,
            p.class_id,
            p.class AS class_name,
            p.spec,
            p.dps AS player_dps,
            p.rdps,
            p.ndps,
            p.damage_stats,
            p.rdps_damage_given,
            p.support_ap,
            p.support_brand,
            p.support_identity,
            p.support_hyper
        FROM encounter_preview e
        JOIN encounter enc ON enc.id = e.id
        JOIN entity p ON p.encounter_id = e.id
            AND p.entity_type = 'PLAYER'
            AND p.class_id > 0
            AND p.dps > 0
        LEFT JOIN entity boss ON boss.encounter_id = e.id AND boss.name = e.current_boss
        WHERE {}
        ORDER BY e.fight_start ASC, p.dps DESC",
        filters.join(" AND ")
    );

    (params, query)
}

fn build_raid_progression_range_query(
    criteria: RaidProgressionRangeCriteria,
) -> (Vec<String>, String) {
    let min_duration = if criteria.min_duration > 0 {
        criteria.min_duration
    } else {
        10
    };
    let mut select_params = Vec::new();
    let mut where_params = vec![(min_duration * 1000).to_string()];
    let mut filters = vec![
        "e.duration > ?".to_string(),
        "e.difficulty IS NOT NULL AND e.difficulty != ''".to_string(),
    ];

    if !criteria.difficulty.is_empty() {
        filters.push("e.difficulty = ?".to_string());
        where_params.push(criteria.difficulty);
    }

    if !criteria.bosses.is_empty() {
        let placeholders = "?,".repeat(criteria.bosses.len());
        let placeholders = placeholders.trim_end_matches(',');
        filters.push(format!("e.current_boss IN ({})", placeholders));
        where_params.extend(criteria.bosses);
    }

    let first_clear = if criteria.last_gate_bosses.is_empty() {
        "MIN(CASE WHEN e.cleared THEN e.fight_start END) AS first_clear".to_string()
    } else {
        let placeholders = "?,".repeat(criteria.last_gate_bosses.len());
        let placeholders = placeholders.trim_end_matches(',');
        select_params.extend(criteria.last_gate_bosses);
        format!(
            "MIN(CASE WHEN e.cleared AND e.current_boss IN ({}) THEN e.fight_start END) AS first_clear",
            placeholders
        )
    };

    let query = format!(
        "SELECT
            MIN(e.fight_start) AS first_pull,
            {}
        FROM encounter_preview e
        WHERE {}",
        first_clear,
        filters.join(" AND ")
    );

    let mut params = select_params;
    params.extend(where_params);

    (params, query)
}

fn reset_window_for_range(range: &str) -> (Option<i64>, Option<i64>) {
    const WEEK_MS: i64 = 7 * 24 * 60 * 60 * 1000;
    const RESET_ANCHOR_MS: i64 = 6 * 24 * 60 * 60 * 1000 + 10 * 60 * 60 * 1000;

    let now = Utc::now().timestamp_millis();
    let current_reset = now - (now - RESET_ANCHOR_MS).rem_euclid(WEEK_MS);

    match range {
        "all" => (None, None),
        "previous_week" => (Some(current_reset - WEEK_MS), Some(current_reset - 1)),
        "last4_weeks" => (Some(current_reset - 3 * WEEK_MS), None),
        "last8_weeks" => (Some(current_reset - 7 * WEEK_MS), None),
        "" | "current_week" => (Some(current_reset), None),
        _ => (Some(current_reset), None),
    }
}

fn map_character_statistics_row(row: &rusqlite::Row) -> rusqlite::Result<CharacterStatisticsRow> {
    std::result::Result::Ok(CharacterStatisticsRow {
        id: row.get("id")?,
        fight_start: row.get("fight_start")?,
        boss_name: row.get("current_boss")?,
        raid_name: None,
        duration: row.get("duration")?,
        difficulty: row.get("difficulty")?,
        cleared: row.get("cleared")?,
        my_dps: row.get("my_dps").unwrap_or_default(),
        my_rdps: row.get("my_rdps").unwrap_or_default(),
        my_ndps: row.get("my_ndps").unwrap_or_default(),
        udps: row.get("unbuffed_dps").unwrap_or_default(),
        rdps_damage_given: row.get("rdps_damage_given").unwrap_or_default(),
        support_party_damage: row.get("total_damage_dealt").unwrap_or_default(),
        support_ap: row.get("support_ap").unwrap_or_default(),
        support_brand: row.get("support_brand").unwrap_or_default(),
        support_identity: row.get("support_identity").unwrap_or_default(),
        support_hyper: row.get("support_hyper").unwrap_or_default(),
    })
}

fn map_raid_progression_row(row: &rusqlite::Row) -> rusqlite::Result<RaidProgressionRow> {
    let misc_str: String = row.get("misc").unwrap_or_default();
    let misc = serde_json::from_str::<EncounterMisc>(misc_str.as_str())
        .ok()
        .unwrap_or_default();
    let version = misc
        .version
        .as_ref()
        .and_then(|version| semver::Version::parse(version).ok())
        .unwrap_or_else(|| semver::Version::new(0, 0, 0));

    let damage_stats = if version >= VERSION_1_13_5 {
        let CompressedJson(damage_stats): CompressedJson<DamageStats> = row.get("damage_stats")?;
        damage_stats
    } else {
        let JsonColumn(damage_stats): JsonColumn<DamageStats> = row.get("damage_stats")?;
        damage_stats
    };

    let CompressedJson(mut boss_hp_log): CompressedJson<HashMap<String, Vec<BossHpLog>>> =
        row.get("boss_hp_log")?;
    if boss_hp_log.is_empty() {
        boss_hp_log = misc.boss_hp_log.unwrap_or_default();
    }

    let boss_name: String = row.get("current_boss")?;
    let boss_hp_bars_by_name = row
        .get::<_, Option<String>>("boss_hp_bars_by_name")
        .ok()
        .flatten()
        .and_then(|value| serde_json::from_str::<HashMap<String, i32>>(&value).ok())
        .unwrap_or_default();

    std::result::Result::Ok(RaidProgressionRow {
        id: row.get("id")?,
        fight_start: row.get("fight_start")?,
        boss_name: boss_name.clone(),
        gate: boss_name,
        duration: row.get("duration")?,
        difficulty: row.get("difficulty")?,
        cleared: row.get("cleared")?,
        team_dps: row.get("dps").unwrap_or_default(),
        damage_taken: row.get("total_damage_taken").unwrap_or_default(),
        local_player: row.get("local_player").unwrap_or_default(),
        boss_hp_bars: row.get("hp_bars").unwrap_or_default(),
        boss_hp_bars_by_name,
        boss_current_hp: row.get("boss_current_hp").unwrap_or_default(),
        boss_max_hp: row.get("boss_max_hp").unwrap_or_default(),
        boss_hp_log,
        player_name: row.get("player_name")?,
        class_id: row.get("class_id").unwrap_or_default(),
        class_name: row.get("class_name").unwrap_or_default(),
        spec: row.get("spec").unwrap_or_default(),
        player_damage_dealt: damage_stats.damage_dealt,
        dps: row.get("player_dps").unwrap_or_default(),
        rdps: row.get("rdps").unwrap_or_default(),
        ndps: row.get("ndps").unwrap_or_default(),
        player_damage_taken: damage_stats.damage_taken,
        death_events: death_events_from_stats(&damage_stats),
        rdps_damage_given: row.get("rdps_damage_given").unwrap_or_default(),
        party_info: misc.party_info,
        support_ap: row.get("support_ap").unwrap_or_default(),
        support_brand: row.get("support_brand").unwrap_or_default(),
        support_identity: row.get("support_identity").unwrap_or_default(),
        support_hyper: row.get("support_hyper").unwrap_or_default(),
    })
}

fn death_events_from_stats(stats: &DamageStats) -> Vec<i64> {
    let mut death_events = stats
        .death_info
        .as_ref()
        .map(|deaths| {
            deaths
                .iter()
                .filter_map(|death| positive(Some(death.death_time)))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if death_events.is_empty() && stats.deaths > 0 {
        for _ in 0..stats.deaths {
            death_events.push(stats.death_time);
        }
    }

    death_events.sort_unstable();
    death_events
}

fn build_raid_progression_statistics(
    rows: Vec<RaidProgressionRow>,
    boss_to_raid: &HashMap<String, String>,
    boss_order: &HashMap<String, i32>,
    last_gate_bosses: &[String],
) -> RaidProgressionStatistics {
    let mut pulls_by_id: BTreeMap<i32, Vec<RaidProgressionRow>> = BTreeMap::new();
    let mut players_by_name: BTreeMap<String, RaidProgressionPlayerAggregate> = BTreeMap::new();

    for mut row in rows {
        row.gate = boss_to_raid
            .get(&row.boss_name)
            .cloned()
            .unwrap_or_else(|| row.boss_name.clone());
        pulls_by_id.entry(row.id).or_default().push(row);
    }

    let mut pulls = Vec::new();
    for rows in pulls_by_id.values() {
        let death_counts = progression_death_counts(rows);

        if let Some(row) = rows.first() {
            pulls.push(build_progression_pull(row, rows, boss_order, &death_counts));
        }

        for row in rows {
            let deaths = death_counts
                .get(&row.player_name)
                .copied()
                .unwrap_or_default();
            let support_contribution = progression_support_contribution(row, rows);
            players_by_name
                .entry(row.player_name.clone())
                .and_modify(|player| {
                    update_progression_player(player, row, deaths, support_contribution)
                })
                .or_insert_with(|| new_progression_player(row, deaths, support_contribution));
        }
    }

    let summary = build_progression_summary(&pulls, last_gate_bosses);
    let gates = build_progression_gates(&pulls);
    let mut players = players_by_name
        .into_values()
        .map(build_progression_player)
        .collect::<Vec<_>>();

    players.sort_by_key(|player| {
        (
            Reverse(player.pulls),
            !player.is_support,
            player.name.to_lowercase(),
        )
    });

    RaidProgressionStatistics {
        summary,
        gates,
        pulls: pulls
            .into_iter()
            .rev()
            .map(build_progression_pull_row)
            .collect(),
        players,
    }
}

fn build_progression_pull(
    row: &RaidProgressionRow,
    rows: &[RaidProgressionRow],
    boss_order: &HashMap<String, i32>,
    death_counts: &BTreeMap<String, i32>,
) -> RaidProgressionPullAggregate {
    let progress = if row.cleared {
        Some(ProgressionBossProgress {
            boss_name: row.boss_name.clone(),
            bars: Some(0),
            percent: 0.0,
            rank: boss_order.get(&row.boss_name).copied().unwrap_or_default(),
            last_seen: i32::MAX,
        })
    } else {
        progression_boss_progress(row, boss_order)
    };
    let boss_name = progress
        .as_ref()
        .map(|progress| progress.boss_name.clone())
        .unwrap_or_else(|| row.boss_name.clone());

    RaidProgressionPullAggregate {
        id: row.id,
        fight_start: row.fight_start,
        boss_name,
        gate: row.gate.clone(),
        duration: row.duration,
        difficulty: row.difficulty.clone(),
        cleared: row.cleared,
        team_dps: row.team_dps,
        damage_taken: row.damage_taken,
        deaths: death_counts.values().sum(),
        progress_bars: progress.as_ref().and_then(|progress| progress.bars),
        progress_percent: progress.as_ref().map(|progress| progress.percent),
        progress_rank: progress
            .as_ref()
            .map(|progress| progress.rank)
            .unwrap_or_else(|| boss_order.get(&row.boss_name).copied().unwrap_or_default()),
        local_player: row.local_player.clone(),
        player_count: rows.len() as i32,
    }
}

#[derive(Clone)]
struct ProgressionBossProgress {
    boss_name: String,
    bars: Option<i32>,
    percent: f32,
    rank: i32,
    last_seen: i32,
}

fn progression_boss_progress(
    row: &RaidProgressionRow,
    boss_order: &HashMap<String, i32>,
) -> Option<ProgressionBossProgress> {
    // Multi-boss gates can save a dead sub-boss as current_boss. Prefer the latest
    // non-zero HP-log candidate so best pull reflects the boss the group actually reached.
    let candidates = row
        .boss_hp_log
        .iter()
        .filter_map(|(boss_name, log)| {
            if !boss_order.is_empty() && !boss_order.contains_key(boss_name) {
                return None;
            }

            let last = log.last()?;
            let percent = if boss_name == &row.boss_name {
                current_boss_percent(row).unwrap_or_else(|| normalized_boss_hp_percent(last.p))
            } else {
                normalized_boss_hp_percent(last.p)
            };
            Some(ProgressionBossProgress {
                boss_name: boss_name.clone(),
                bars: row
                    .boss_hp_bars_by_name
                    .get(boss_name)
                    .copied()
                    .or_else(|| {
                        (boss_name == &row.boss_name)
                            .then_some(row.boss_hp_bars)
                            .flatten()
                    })
                    .and_then(|bars| progress_bars(percent, bars)),
                percent,
                rank: boss_order.get(boss_name).copied().unwrap_or_default(),
                last_seen: last.time,
            })
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        let percent = current_boss_percent(row)?;
        return Some(ProgressionBossProgress {
            boss_name: row.boss_name.clone(),
            bars: row
                .boss_hp_bars
                .and_then(|bars| progress_bars(percent, bars)),
            percent,
            rank: boss_order.get(&row.boss_name).copied().unwrap_or_default(),
            last_seen: i32::MAX,
        });
    }

    candidates
        .iter()
        .filter(|candidate| candidate.percent > 0.0)
        .max_by(|a, b| compare_progress_candidate(a, b))
        .cloned()
        .or_else(|| candidates.into_iter().max_by(compare_progress_candidate))
}

fn current_boss_percent(row: &RaidProgressionRow) -> Option<f32> {
    match (row.boss_current_hp, row.boss_max_hp) {
        (Some(current_hp), Some(max_hp)) if max_hp > 0 => {
            let current_hp = current_hp.clamp(0, max_hp);
            Some(((current_hp as f32 / max_hp as f32) * 100.0).clamp(0.0, 100.0))
        }
        _ => None,
    }
}

fn compare_progress_candidate(
    a: &ProgressionBossProgress,
    b: &ProgressionBossProgress,
) -> Ordering {
    // Within one pull, later boss HP logs represent deeper gate progression; rank and HP
    // only break ties when logs were updated at the same second.
    a.last_seen.cmp(&b.last_seen).then_with(|| {
        a.rank
            .cmp(&b.rank)
            .then_with(|| b.percent.total_cmp(&a.percent))
    })
}

fn normalized_boss_hp_percent(percent: f32) -> f32 {
    let percent = if percent <= 1.0 {
        percent * 100.0
    } else {
        percent
    };
    percent.clamp(0.0, 100.0)
}

fn progress_bars(percent: f32, boss_hp_bars: i32) -> Option<i32> {
    if boss_hp_bars <= 0 {
        return None;
    }

    Some(if percent <= 0.0 {
        0
    } else {
        ((boss_hp_bars as f32 * percent) / 100.0).ceil() as i32
    })
}

fn build_progression_pull_row(pull: RaidProgressionPullAggregate) -> RaidProgressionPull {
    RaidProgressionPull {
        id: pull.id,
        fight_start: pull.fight_start,
        gate: pull.gate,
        boss_name: pull.boss_name,
        difficulty: pull.difficulty,
        duration: pull.duration,
        cleared: pull.cleared,
        team_dps: pull.team_dps,
        damage_taken: pull.damage_taken,
        deaths: pull.deaths,
        progress_bars: pull.progress_bars,
        progress_percent: pull.progress_percent,
        local_player: pull.local_player,
        player_count: pull.player_count,
    }
}

fn build_progression_summary(
    pulls: &[RaidProgressionPullAggregate],
    last_gate_bosses: &[String],
) -> RaidProgressionSummary {
    let attempts = pulls.len() as i32;
    let clears = pulls
        .iter()
        .filter(|pull| is_raid_clear(pull, last_gate_bosses))
        .count() as i32;
    let wipes = attempts - clears;
    let first_clear_pull = pulls
        .iter()
        .filter(|pull| is_raid_clear(pull, last_gate_bosses))
        .min_by_key(|pull| pull.fight_start);
    let best_progress_pull = best_progress_pull(pulls.iter());

    RaidProgressionSummary {
        attempts,
        clears,
        wipes,
        clear_rate: percent(clears, attempts),
        first_pull: pulls.iter().map(|pull| pull.fight_start).min(),
        last_pull: pulls.iter().map(|pull| pull.fight_start).max(),
        first_clear: first_clear_pull.map(|pull| pull.fight_start),
        first_clear_duration: first_clear_pull.map(|pull| pull.duration),
        total_duration: pulls.iter().map(|pull| pull.duration).sum(),
        average_duration: average_i64(pulls.iter().map(|pull| pull.duration)),
        average_team_dps: average_i64(pulls.iter().map(|pull| pull.team_dps)),
        average_damage_taken: average_i64(pulls.iter().map(|pull| pull.damage_taken)),
        average_deaths: average_i32(pulls.iter().map(|pull| pull.deaths)),
        best_progress_bars: best_progress_pull.and_then(|pull| pull.progress_bars),
        best_progress_percent: best_progress_pull.and_then(|pull| pull.progress_percent),
        best_progress_boss_name: best_progress_pull.map(|pull| pull.boss_name.clone()),
    }
}

fn build_progression_gates(pulls: &[RaidProgressionPullAggregate]) -> Vec<RaidProgressionGate> {
    let mut grouped: BTreeMap<String, Vec<&RaidProgressionPullAggregate>> = BTreeMap::new();
    for pull in pulls {
        grouped.entry(pull.gate.clone()).or_default().push(pull);
    }

    grouped
        .into_iter()
        .map(|(gate, rows)| {
            let attempts = rows.len() as i32;
            let clears = rows.iter().filter(|pull| pull.cleared).count() as i32;
            let clear_rows = rows
                .iter()
                .copied()
                .filter(|pull| pull.cleared)
                .collect::<Vec<_>>();
            let best_progress_pull = best_progress_pull(rows.iter().copied());

            RaidProgressionGate {
                gate,
                attempts,
                clears,
                clear_rate: percent(clears, attempts),
                best_progress_bars: best_progress_pull.and_then(|pull| pull.progress_bars),
                best_progress_percent: best_progress_pull.and_then(|pull| pull.progress_percent),
                best_progress_boss_name: best_progress_pull.map(|pull| pull.boss_name.clone()),
                median_duration: median_i64(clear_rows.iter().map(|pull| pull.duration)),
                fastest_clear: clear_rows.iter().map(|pull| pull.duration).min(),
                first_clear: clear_rows.iter().map(|pull| pull.fight_start).min(),
                average_team_dps: average_i64(rows.iter().map(|pull| pull.team_dps)),
                average_deaths: average_i32(rows.iter().map(|pull| pull.deaths)),
            }
        })
        .collect()
}

fn is_raid_clear(pull: &RaidProgressionPullAggregate, last_gate_bosses: &[String]) -> bool {
    if !pull.cleared {
        return false;
    }

    last_gate_bosses.is_empty() || last_gate_bosses.iter().any(|boss| boss == &pull.boss_name)
}

fn best_progress_pull<'a>(
    pulls: impl Iterator<Item = &'a RaidProgressionPullAggregate>,
) -> Option<&'a RaidProgressionPullAggregate> {
    pulls.max_by(|a, b| compare_progress(a, b))
}

fn compare_progress(
    a: &RaidProgressionPullAggregate,
    b: &RaidProgressionPullAggregate,
) -> Ordering {
    // Across pulls, later bosses beat lower HP on earlier bosses. For the same boss,
    // lower remaining HP is better progression.
    a.progress_rank.cmp(&b.progress_rank).then_with(|| {
        let a_percent = a.progress_percent.unwrap_or(100.0);
        let b_percent = b.progress_percent.unwrap_or(100.0);
        b_percent.total_cmp(&a_percent)
    })
}

#[derive(Clone)]
struct ProgressionDeathEvent {
    player_name: String,
    elapsed: i64,
}

fn progression_death_counts(rows: &[RaidProgressionRow]) -> BTreeMap<String, i32> {
    const WIPE_DEATH_CLUSTER_MS: i64 = 5_000;

    let mut events = rows
        .iter()
        .flat_map(|row| {
            row.death_events.iter().filter_map(|death_time| {
                normalize_death_elapsed(row, *death_time).map(|elapsed| ProgressionDeathEvent {
                    player_name: row.player_name.clone(),
                    elapsed,
                })
            })
        })
        .collect::<Vec<_>>();

    events.sort_by(|a, b| {
        a.elapsed
            .cmp(&b.elapsed)
            .then_with(|| a.player_name.cmp(&b.player_name))
    });

    let Some(last_event) = events.last() else {
        return BTreeMap::new();
    };

    // A wipe usually records everyone as dead near the reset. Count deaths before
    // that final reset cluster, plus the first death in the cluster as the wipe cause.
    let cleared = rows.first().is_some_and(|row| row.cleared);
    let first_wipe_event_index = if cleared {
        events.len()
    } else {
        let wipe_cluster_start = (last_event.elapsed - WIPE_DEATH_CLUSTER_MS).max(0);
        events
            .iter()
            .position(|event| event.elapsed >= wipe_cluster_start)
            .unwrap_or(events.len())
    };
    let wipe_cluster_is_reset =
        !cleared && final_death_cluster_is_reset(rows, &events, first_wipe_event_index);

    let mut counts = BTreeMap::new();
    for (index, event) in events.iter().enumerate() {
        if index < first_wipe_event_index
            || !wipe_cluster_is_reset
            || index == first_wipe_event_index
        {
            *counts.entry(event.player_name.clone()).or_insert(0) += 1;
        }
    }

    counts
}

fn final_death_cluster_is_reset(
    rows: &[RaidProgressionRow],
    events: &[ProgressionDeathEvent],
    cluster_start_index: usize,
) -> bool {
    if cluster_start_index >= events.len() {
        return false;
    }

    let active_players = rows
        .iter()
        .map(|row| row.player_name.as_str())
        .collect::<BTreeSet<_>>();
    let dead_before_cluster = events[..cluster_start_index]
        .iter()
        .map(|event| event.player_name.as_str())
        .collect::<BTreeSet<_>>();
    let cluster_players = events[cluster_start_index..]
        .iter()
        .map(|event| event.player_name.as_str())
        .collect::<BTreeSet<_>>();
    let alive_before_cluster = active_players
        .len()
        .saturating_sub(dead_before_cluster.len())
        .max(1);

    // If the final cluster covers everyone who was still alive, treat it as the
    // encounter reset instead of independent player deaths.
    cluster_players.len() >= alive_before_cluster
}

fn normalize_death_elapsed(row: &RaidProgressionRow, death_time: i64) -> Option<i64> {
    if death_time <= 0 {
        return None;
    }

    Some(if death_time >= row.fight_start {
        (death_time - row.fight_start).max(0)
    } else {
        death_time
    })
}

fn new_progression_player(
    row: &RaidProgressionRow,
    deaths: i32,
    support_contribution: Option<f32>,
) -> RaidProgressionPlayerAggregate {
    let mut player = RaidProgressionPlayerAggregate {
        name: row.player_name.clone(),
        class_id: row.class_id,
        class_name: row.class_name.clone(),
        spec: row.spec.clone(),
        pulls: 0,
        clears: 0,
        dps_values: Vec::new(),
        rdps_values: Vec::new(),
        ndps_values: Vec::new(),
        damage_taken_values: Vec::new(),
        total_deaths: 0,
        support_ap_values: Vec::new(),
        support_contribution_values: Vec::new(),
        support_brand_values: Vec::new(),
        support_identity_values: Vec::new(),
        support_hyper_values: Vec::new(),
        last_seen: 0,
    };
    update_progression_player(&mut player, row, deaths, support_contribution);
    player
}

fn update_progression_player(
    player: &mut RaidProgressionPlayerAggregate,
    row: &RaidProgressionRow,
    deaths: i32,
    support_contribution: Option<f32>,
) {
    player.pulls += 1;
    if row.cleared {
        player.clears += 1;
    }
    if row.dps > 0 {
        player.dps_values.push(row.dps);
    }
    if let Some(rdps) = positive(row.rdps) {
        player.rdps_values.push(rdps);
    }
    if let Some(ndps) = positive(row.ndps) {
        player.ndps_values.push(ndps);
    }
    if row.player_damage_taken > 0 {
        player.damage_taken_values.push(row.player_damage_taken);
    }
    if let Some(support_ap) = positive_f32(row.support_ap) {
        player.support_ap_values.push(support_ap);
    }
    if let Some(support_contribution) = positive_f32(support_contribution) {
        player
            .support_contribution_values
            .push(support_contribution);
    }
    if let Some(support_brand) = positive_f32(row.support_brand) {
        player.support_brand_values.push(support_brand);
    }
    if let Some(support_identity) = positive_f32(row.support_identity) {
        player.support_identity_values.push(support_identity);
    }
    if let Some(support_hyper) = positive_f32(row.support_hyper) {
        player.support_hyper_values.push(support_hyper);
    }
    player.total_deaths += deaths;
    player.last_seen = player.last_seen.max(row.fight_start);
    if player.spec.is_none() {
        player.spec = row.spec.clone();
    }
}

fn build_progression_player(player: RaidProgressionPlayerAggregate) -> RaidProgressionPlayer {
    let is_support = player.spec.as_deref().is_some_and(is_support_spec);
    let best_dps = player.dps_values.iter().copied().max();

    RaidProgressionPlayer {
        name: player.name,
        class_id: player.class_id,
        class: player.class_name,
        spec: player.spec,
        is_support,
        pulls: player.pulls,
        clears: player.clears,
        clear_rate: percent(player.clears, player.pulls),
        average_dps: average_i64(player.dps_values.into_iter()),
        best_dps,
        average_rdps: average_i64(player.rdps_values.into_iter()),
        average_ndps: average_i64(player.ndps_values.into_iter()),
        average_damage_taken: average_i64(player.damage_taken_values.into_iter()),
        total_deaths: player.total_deaths,
        deaths_per_pull: if player.pulls == 0 {
            0.0
        } else {
            player.total_deaths as f32 / player.pulls as f32
        },
        average_support_ap: average_f32(player.support_ap_values.into_iter()),
        average_support_contribution: average_f32(player.support_contribution_values.into_iter()),
        average_support_brand: average_f32(player.support_brand_values.into_iter()),
        average_support_identity: average_f32(player.support_identity_values.into_iter()),
        average_support_hyper: average_f32(player.support_hyper_values.into_iter()),
        last_seen: player.last_seen,
    }
}

fn progression_support_contribution(
    row: &RaidProgressionRow,
    rows: &[RaidProgressionRow],
) -> Option<f32> {
    if row.rdps_damage_given <= 0 {
        return None;
    }

    let support_party_damage = progression_support_party_damage(row, rows);
    if support_party_damage <= 0 {
        return None;
    }

    Some(row.rdps_damage_given as f32 / support_party_damage as f32)
}

fn progression_support_party_damage(row: &RaidProgressionRow, rows: &[RaidProgressionRow]) -> i64 {
    let party = row.party_info.as_ref().and_then(|parties| {
        parties
            .values()
            .find(|party| party.iter().any(|name| name == &row.player_name))
    });

    rows.iter()
        .filter(|player| {
            player.player_damage_dealt > 0
                && !player.spec.as_deref().is_some_and(is_support_spec)
                && party.is_none_or(|party| party.iter().any(|name| name == &player.player_name))
        })
        .map(|player| player.player_damage_dealt)
        .sum()
}

fn populate_support_contribution_denominators(
    connection: &rusqlite::Connection,
    rows: &mut [CharacterStatisticsRow],
    support_name: &str,
) {
    for row in rows.iter_mut().filter(|row| row.rdps_damage_given > 0) {
        match support_contribution_denominator(connection, row.id, support_name) {
            std::result::Result::Ok(damage) if damage > 0 => row.support_party_damage = damage,
            std::result::Result::Ok(_) => {}
            Err(err) => warn!(
                "failed to calculate support party damage for encounter {}: {:?}",
                row.id, err
            ),
        }
    }
}

fn support_contribution_denominator(
    connection: &rusqlite::Connection,
    encounter_id: i32,
    support_name: &str,
) -> Result<i64> {
    let misc = connection
        .query_row(
            "SELECT misc FROM encounter WHERE id = ?",
            params![encounter_id],
            |row| row.get::<_, String>("misc"),
        )
        .optional()?
        .and_then(|misc| serde_json::from_str::<EncounterMisc>(misc.as_str()).ok())
        .unwrap_or_default();
    let version = misc
        .version
        .as_ref()
        .and_then(|version| semver::Version::parse(version).ok())
        .unwrap_or_else(|| semver::Version::new(0, 0, 0));

    let entities = connection
        .prepare_cached(SELECT_ENTITIES_BY_ENCOUNTER)?
        .query_map(params![encounter_id], |row| map_entity(row, &version))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(support_contribution_denominator_from_entities(
        &entities,
        misc.party_info.as_ref(),
        support_name,
    ))
}

fn support_contribution_denominator_from_entities(
    entities: &[EncounterEntity],
    party_info: Option<&HashMap<i32, Vec<String>>>,
    support_name: &str,
) -> i64 {
    let party = party_info.and_then(|parties| {
        parties
            .values()
            .find(|party| party.iter().any(|name| name == support_name))
    });

    entities
        .iter()
        .filter(|entity| {
            entity.entity_type == EntityType::Player
                && entity.class_id != 0
                && entity.damage_stats.damage_dealt > 0
                && !is_support(entity)
                && party.is_none_or(|party| party.iter().any(|name| name == &entity.name))
        })
        .map(|entity| entity.damage_stats.damage_dealt)
        .sum()
}

fn build_character_statistics(
    character: CharacterInfo,
    rows: Vec<CharacterStatisticsRow>,
    mode: &str,
    damage_type: &str,
) -> CharacterStatistics {
    let metric_damage_type = if mode == "support" {
        "dps"
    } else {
        damage_type
    };
    let attempts = rows.len() as i32;
    let performance_rows: Vec<_> = rows
        .iter()
        .filter(|row| {
            row.cleared
                && if mode == "support" {
                    row.my_dps > 0
                } else {
                    damage_value(row, metric_damage_type).is_some()
                }
        })
        .cloned()
        .collect();
    let clears = rows.iter().filter(|row| row.cleared).count() as i32;
    let wipes = attempts - clears;
    let support = support_summary(&performance_rows);

    let summary = CharacterStatisticsSummary {
        attempts,
        clears,
        wipes,
        clear_rate: percent(clears, attempts),
        best_dps: performance_rows
            .iter()
            .filter_map(|row| damage_value(row, "dps"))
            .max(),
        best_rdps: performance_rows
            .iter()
            .filter_map(|row| damage_value(row, "rdps"))
            .max(),
        best_ndps: performance_rows
            .iter()
            .filter_map(|row| damage_value(row, "ndps"))
            .max(),
        median_dps: median_i64(
            performance_rows
                .iter()
                .filter_map(|row| damage_value(row, "dps")),
        ),
        p75_dps: percentile_i64(
            performance_rows
                .iter()
                .filter_map(|row| damage_value(row, "dps")),
            0.75,
        ),
        p75_rdps: percentile_i64(
            performance_rows
                .iter()
                .filter_map(|row| damage_value(row, "rdps")),
            0.75,
        ),
        p75_ndps: percentile_i64(
            performance_rows
                .iter()
                .filter_map(|row| damage_value(row, "ndps")),
            0.75,
        ),
        median_rdps: median_i64(
            performance_rows
                .iter()
                .filter_map(|row| positive(row.my_rdps)),
        ),
        median_ndps: median_i64(
            performance_rows
                .iter()
                .filter_map(|row| positive(row.my_ndps)),
        ),
        median_udps: median_i64(performance_rows.iter().filter_map(|row| positive(row.udps))),
        median_duration: median_i64(performance_rows.iter().map(|row| row.duration)),
        support: support.clone(),
    };

    let recent_bests = performance_rows
        .clone()
        .into_iter()
        .map(|row| {
            let support_contribution = support_contribution(&row);
            RecentBestEncounter {
                id: row.id,
                fight_start: row.fight_start,
                boss_name: row.boss_name,
                duration: row.duration,
                difficulty: row.difficulty,
                my_dps: row.my_dps,
                my_rdps: row.my_rdps,
                my_ndps: row.my_ndps,
                support_contribution,
            }
        })
        .collect();

    let trends = build_trends(&rows);
    let raids = build_raid_rows(&rows);
    let unavailable = CharacterStatisticsUnavailable {
        rdps_logs: performance_rows
            .iter()
            .filter(|row| positive(row.my_rdps).is_some())
            .count() as i32,
        support_logs: support.as_ref().map(|s| s.logs).unwrap_or_default(),
    };

    CharacterStatistics {
        character,
        summary,
        trends,
        raids,
        recent_bests,
        unavailable,
    }
}

fn build_trends(rows: &[CharacterStatisticsRow]) -> Vec<CharacterStatisticsTrend> {
    const WEEK_MS: i64 = 7 * 24 * 60 * 60 * 1000;
    let mut buckets: BTreeMap<i64, Vec<CharacterStatisticsRow>> = BTreeMap::new();
    for row in rows {
        let bucket = row.fight_start - row.fight_start.rem_euclid(WEEK_MS);
        buckets.entry(bucket).or_default().push(row.clone());
    }

    buckets
        .into_iter()
        .map(|(start_time, bucket_rows)| {
            let attempts = bucket_rows.len() as i32;
            let cleared_rows: Vec<_> = bucket_rows
                .iter()
                .filter(|row| row.cleared && row.my_dps > 0)
                .cloned()
                .collect();
            let clears = bucket_rows.iter().filter(|row| row.cleared).count() as i32;
            CharacterStatisticsTrend {
                start_time,
                attempts,
                clears,
                median_dps: median_i64(cleared_rows.iter().map(|row| row.my_dps)),
                best_dps: cleared_rows.iter().map(|row| row.my_dps).max(),
                support: support_summary(&cleared_rows),
            }
        })
        .collect()
}

fn build_raid_rows(rows: &[CharacterStatisticsRow]) -> Vec<RaidStatisticsRow> {
    let mut grouped: BTreeMap<(String, Option<String>), Vec<CharacterStatisticsRow>> =
        BTreeMap::new();
    for row in rows {
        grouped
            .entry((
                row.raid_name
                    .clone()
                    .unwrap_or_else(|| row.boss_name.clone()),
                row.difficulty.clone(),
            ))
            .or_default()
            .push(row.clone());
    }

    let mut raids: Vec<_> = grouped
        .into_iter()
        .map(|((boss_name, difficulty), rows)| {
            let attempts = rows.len() as i32;
            let cleared_rows: Vec<_> = rows
                .iter()
                .filter(|row| row.cleared && row.my_dps > 0)
                .cloned()
                .collect();
            let clears = rows.iter().filter(|row| row.cleared).count() as i32;
            RaidStatisticsRow {
                boss_name,
                difficulty,
                attempts,
                clears,
                clear_rate: percent(clears, attempts),
                median_dps: median_i64(
                    cleared_rows
                        .iter()
                        .filter_map(|row| damage_value(row, "dps")),
                ),
                best_dps: cleared_rows
                    .iter()
                    .filter_map(|row| damage_value(row, "dps"))
                    .max(),
                median_rdps: median_i64(
                    cleared_rows
                        .iter()
                        .filter_map(|row| damage_value(row, "rdps")),
                ),
                best_rdps: cleared_rows
                    .iter()
                    .filter_map(|row| damage_value(row, "rdps"))
                    .max(),
                median_ndps: median_i64(
                    cleared_rows
                        .iter()
                        .filter_map(|row| damage_value(row, "ndps")),
                ),
                best_ndps: cleared_rows
                    .iter()
                    .filter_map(|row| damage_value(row, "ndps"))
                    .max(),
                median_duration: median_i64(cleared_rows.iter().map(|row| row.duration)),
                last_clear: rows
                    .iter()
                    .filter(|row| row.cleared)
                    .map(|row| row.fight_start)
                    .max(),
                support: support_summary(&cleared_rows),
            }
        })
        .collect();

    raids.sort_by_key(|row| {
        (
            Reverse(row.last_clear.unwrap_or_default()),
            row.boss_name.clone(),
        )
    });
    raids
}

fn damage_value(row: &CharacterStatisticsRow, damage_type: &str) -> Option<i64> {
    match damage_type {
        "rdps" => positive(row.my_rdps),
        "ndps" => positive(row.my_ndps),
        _ => positive(Some(row.my_dps)),
    }
}

fn support_summary(rows: &[CharacterStatisticsRow]) -> Option<SupportStatisticsSummary> {
    let support_rows: Vec<_> = rows
        .iter()
        .filter(|row| {
            positive_f32(row.support_ap).is_some()
                || positive_f32(row.support_brand).is_some()
                || positive_f32(row.support_identity).is_some()
                || positive_f32(row.support_hyper).is_some()
        })
        .collect();

    if support_rows.is_empty() {
        return None;
    }

    Some(SupportStatisticsSummary {
        logs: support_rows.len() as i32,
        ap: average_f32(
            support_rows
                .iter()
                .filter_map(|row| positive_f32(row.support_ap)),
        ),
        brand: average_f32(
            support_rows
                .iter()
                .filter_map(|row| positive_f32(row.support_brand)),
        ),
        identity: average_f32(
            support_rows
                .iter()
                .filter_map(|row| positive_f32(row.support_identity)),
        ),
        hyper: average_f32(
            support_rows
                .iter()
                .filter_map(|row| positive_f32(row.support_hyper)),
        ),
        median_contribution: median_f32(
            support_rows
                .iter()
                .filter_map(|row| support_contribution(row)),
        ),
        best_contribution: support_rows
            .iter()
            .filter_map(|row| support_contribution(row))
            .reduce(f32::max),
    })
}

fn support_contribution(row: &CharacterStatisticsRow) -> Option<f32> {
    if row.rdps_damage_given <= 0 || row.support_party_damage <= 0 {
        return None;
    }

    Some(row.rdps_damage_given as f32 / row.support_party_damage as f32)
}

fn percent(numerator: i32, denominator: i32) -> f32 {
    if denominator == 0 {
        return 0.0;
    }

    numerator as f32 / denominator as f32 * 100.0
}

fn positive(value: Option<i64>) -> Option<i64> {
    value.filter(|v| *v > 0)
}

fn positive_f32(value: Option<f32>) -> Option<f32> {
    value.filter(|v| *v > 0.0)
}

fn average_f32(values: impl Iterator<Item = f32>) -> Option<f32> {
    let mut count = 0;
    let mut total = 0.0;
    for value in values {
        count += 1;
        total += value;
    }

    if count > 0 {
        Some(total / count as f32)
    } else {
        None
    }
}

fn average_i64(values: impl Iterator<Item = i64>) -> Option<i64> {
    let mut count = 0;
    let mut total = 0;
    for value in values {
        if value <= 0 {
            continue;
        }
        count += 1;
        total += value;
    }

    if count > 0 { Some(total / count) } else { None }
}

fn average_i32(values: impl Iterator<Item = i32>) -> Option<f32> {
    let mut count = 0;
    let mut total = 0;
    for value in values {
        count += 1;
        total += value;
    }

    if count > 0 {
        Some(total as f32 / count as f32)
    } else {
        None
    }
}

fn median_f32(values: impl Iterator<Item = f32>) -> Option<f32> {
    let mut values: Vec<f32> = values.collect();
    if values.is_empty() {
        return None;
    }

    values.sort_by(|a, b| a.total_cmp(b));
    let middle = values.len() / 2;
    if values.len().is_multiple_of(2) {
        Some((values[middle - 1] + values[middle]) / 2.0)
    } else {
        values.get(middle).copied()
    }
}

fn median_i64(values: impl Iterator<Item = i64>) -> Option<i64> {
    let mut values: Vec<i64> = values.collect();
    if values.is_empty() {
        return None;
    }

    values.sort_unstable();
    let middle = values.len() / 2;
    if values.len().is_multiple_of(2) {
        Some((values[middle - 1] + values[middle]) / 2)
    } else {
        values.get(middle).copied()
    }
}

fn percentile_i64(values: impl Iterator<Item = i64>, percentile: f32) -> Option<i64> {
    let mut values: Vec<i64> = values.collect();
    if values.is_empty() {
        return None;
    }

    values.sort_unstable();
    let index = ((values.len() - 1) as f32 * percentile).round() as usize;
    values.get(index).copied()
}

pub fn calculate_entities(args: &mut InsertEncounterArgs) -> Result<()> {
    let InsertEncounterArgs {
        encounter,
        cast_log,
        damage_log,
        rdps_valid,
        skill_cast_log,
        player_info,
        skill_cooldowns,
        intermission_start,
        intermission_end,
        ..
    } = args;

    let fight_start = encounter.fight_start;
    let fight_end = encounter.last_combat_packet;
    let local_player_str = encounter.local_player.as_str();

    let (intermission_duration, intermission_range_seconds) =
        match (&intermission_start, &intermission_end) {
            (Some(start), Some(end)) if end > start => {
                (*end - *start, Some((*start / 1000, *end / 1000)))
            }
            _ => (0, None),
        };

    for (name, entity) in encounter.entities.iter_mut() {
        if !should_insert_entity(entity, &encounter.local_player) {
            continue;
        }

        update_entity_stats(
            entity,
            fight_start,
            fight_end,
            intermission_duration,
            intermission_range_seconds,
            damage_log,
            *rdps_valid,
        );

        if let Some(info) = player_info
            .as_ref()
            .and_then(|stats| stats.get(&entity.name))
        {
            // if fight didnt request in-game inspect, apply api inspect
            if entity.combat_power.is_none() || !*rdps_valid {
                apply_player_info(entity, info, false);
            } else {
                entity.loadout_hash = info.loadout_snapshot.clone();
                apply_gems_to_skills(entity, info);
            }
        }

        apply_cast_logs(entity, cast_log, skill_cast_log);

        if name == local_player_str {
            for (skill_id, events) in skill_cooldowns.iter() {
                if let Some(skill) = entity.skills.get_mut(skill_id) {
                    skill.time_available = Some(
                        get_total_available_time(events, fight_start, fight_end)
                            - intermission_duration,
                    );
                }
            }
        }

        if entity.spec.as_deref().is_none_or(|spec| spec == "Unknown") {
            let spec = get_player_spec(entity, &encounter.encounter_damage_stats.buffs, false);
            entity.spec = Some(spec);
        }
    }

    Ok(())
}

pub fn compute_support_buffs(
    encounter: &Encounter,
    party_info: &[Vec<String>],
) -> HashMap<String, SupportBuffs> {
    let mut buffs = HashMap::new();

    for party in party_info.iter() {
        let party_members: Vec<_> = encounter
            .entities
            .iter()
            .filter(|(name, _)| party.contains(name))
            .map(|(_, entity)| entity)
            .collect();

        let party_without_support: Vec<_> = party_members
            .iter()
            .filter(|entity| !is_support(entity))
            .collect();

        if party_members.len() - party_without_support.len() != 1 {
            continue;
        }

        let party_damage_total: i64 = party_without_support
            .iter()
            .map(|e| get_damage_without_hyper_or_special(e))
            .sum();

        if party_damage_total <= 0 {
            continue;
        }

        let mut average_brand = 0.0;
        let mut average_buff = 0.0;
        let mut average_identity = 0.0;
        let mut average_hyper = 0.0;

        for player in party_without_support {
            let damage_dealt = get_damage_without_hyper_or_special(player) as f64;
            if damage_dealt <= 0.0 {
                continue;
            }
            let party_damage_percent = damage_dealt / party_damage_total as f64;

            average_brand += (player.damage_stats.debuffed_by_support as f64 / damage_dealt)
                * party_damage_percent;
            average_buff += (player.damage_stats.buffed_by_support as f64 / damage_dealt)
                * party_damage_percent;
            average_identity += (player.damage_stats.buffed_by_identity as f64 / damage_dealt)
                * party_damage_percent;
            average_hyper +=
                (player.damage_stats.buffed_by_hat as f64 / damage_dealt) * party_damage_percent;
        }

        if let Some(support) = party_members.iter().find(|e| is_support(e)) {
            buffs.insert(
                support.name.clone(),
                SupportBuffs {
                    brand: average_brand,
                    buff: average_buff,
                    identity: average_identity,
                    hyper: average_hyper,
                },
            );
        }
    }

    buffs
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{data::AssetPreloader, database::Database};
    use chrono::Utc;
    use hashbrown::HashSet;
    use rand::{Rng, rngs::ThreadRng, seq::IndexedRandom};

    use super::*;

    #[test]
    fn deletes_encounters_before_cutoff_and_can_keep_favorites() {
        let database = Database::memory("1.14.0").unwrap();

        {
            let connection = database.get_connection();
            for (id, fight_start, favorite) in [
                (1, 1_000, false),
                (2, 2_000, false),
                (3, 1_500, true),
                (4, 4_000, false),
            ] {
                connection
                    .execute(
                        "INSERT INTO encounter (id, version) VALUES (?, ?)",
                        params![id, 1],
                    )
                    .unwrap();
                connection
                    .execute(
                        "INSERT INTO encounter_preview (id, fight_start, favorite) VALUES (?, ?, ?)",
                        params![id, fight_start, favorite],
                    )
                    .unwrap();
            }
        }

        let repository = database.create_repository();
        repository.delete_encounters_before(2_000, true).unwrap();

        {
            let connection = database.get_connection();
            let remaining = connection
                .prepare("SELECT id FROM encounter ORDER BY id")
                .unwrap()
                .query_map([], |row| row.get::<_, i32>(0))
                .unwrap()
                .collect::<rusqlite::Result<Vec<_>>>()
                .unwrap();
            assert_eq!(remaining, vec![2, 3, 4]);
        }

        repository.delete_encounters_before(3_500, false).unwrap();

        let connection = database.get_connection();
        let remaining = connection
            .prepare("SELECT id FROM encounter ORDER BY id")
            .unwrap()
            .query_map([], |row| row.get::<_, i32>(0))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap();
        assert_eq!(remaining, vec![4]);
    }

    #[test]
    fn statistics_counts_wipes_without_skewing_performance() {
        let character = CharacterInfo {
            name: "Localplayer".to_string(),
            class_id: 102,
            max_gear_score: 1700.0,
            spec: Some("Mayhem".to_string()),
        };
        let rows = vec![
            CharacterStatisticsRow {
                id: 1,
                fight_start: 1_000,
                boss_name: "Boss".to_string(),
                raid_name: None,
                duration: 300_000,
                difficulty: Some("Hard".to_string()),
                cleared: true,
                my_dps: 100,
                my_rdps: Some(110),
                my_ndps: Some(90),
                udps: Some(80),
                rdps_damage_given: 0,
                support_party_damage: 1_000,
                support_ap: None,
                support_brand: None,
                support_identity: None,
                support_hyper: None,
            },
            CharacterStatisticsRow {
                id: 2,
                fight_start: 2_000,
                boss_name: "Boss".to_string(),
                raid_name: None,
                duration: 240_000,
                difficulty: Some("Hard".to_string()),
                cleared: true,
                my_dps: 300,
                my_rdps: Some(330),
                my_ndps: Some(270),
                udps: Some(240),
                rdps_damage_given: 0,
                support_party_damage: 1_000,
                support_ap: None,
                support_brand: None,
                support_identity: None,
                support_hyper: None,
            },
            CharacterStatisticsRow {
                id: 3,
                fight_start: 3_000,
                boss_name: "Boss".to_string(),
                raid_name: None,
                duration: 30_000,
                difficulty: Some("Hard".to_string()),
                cleared: false,
                my_dps: 10,
                my_rdps: Some(0),
                my_ndps: Some(0),
                udps: Some(0),
                rdps_damage_given: 0,
                support_party_damage: 1_000,
                support_ap: None,
                support_brand: None,
                support_identity: None,
                support_hyper: None,
            },
        ];

        let statistics = build_character_statistics(character, rows, "damage", "dps");

        assert_eq!(statistics.summary.attempts, 3);
        assert_eq!(statistics.summary.clears, 2);
        assert_eq!(statistics.summary.wipes, 1);
        assert!((statistics.summary.clear_rate - 66.666).abs() < 0.1);
        assert_eq!(statistics.summary.median_dps, Some(200));
        assert_eq!(statistics.summary.best_dps, Some(300));
        assert_eq!(statistics.summary.median_duration, Some(270_000));
        assert_eq!(statistics.recent_bests.len(), 2);
        assert!(statistics.recent_bests.iter().all(|row| row.id != 3));
    }

    #[test]
    fn support_contribution_uses_support_party_damage() {
        let support = stats_test_entity("Support", 204, Some("Blessed Aura"), 10, true);
        let dps_a = stats_test_entity("DpsA", 102, Some("Mayhem"), 400, false);
        let dps_b = stats_test_entity("DpsB", 103, Some("Esoteric Flurry"), 600, false);
        let other_party_dps =
            stats_test_entity("OtherDps", 104, Some("Barrage Enhancement"), 1_000, false);
        let entities = vec![support, dps_a, dps_b, other_party_dps];
        let party_info = HashMap::from([
            (
                0,
                vec![
                    "Support".to_string(),
                    "DpsA".to_string(),
                    "DpsB".to_string(),
                ],
            ),
            (1, vec!["OtherDps".to_string()]),
        ]);
        let party_damage =
            support_contribution_denominator_from_entities(&entities, Some(&party_info), "Support");
        let row = CharacterStatisticsRow {
            id: 1,
            fight_start: 1_000,
            boss_name: "Boss".to_string(),
            raid_name: None,
            duration: 300_000,
            difficulty: Some("Hard".to_string()),
            cleared: true,
            my_dps: 10,
            my_rdps: Some(0),
            my_ndps: Some(0),
            udps: Some(0),
            rdps_damage_given: 100,
            support_party_damage: party_damage,
            support_ap: Some(0.9),
            support_brand: None,
            support_identity: None,
            support_hyper: None,
        };
        let statistics = build_character_statistics(
            CharacterInfo {
                name: "Support".to_string(),
                class_id: 204,
                max_gear_score: 1700.0,
                spec: Some("Blessed Aura".to_string()),
            },
            vec![row],
            "support",
            "dps",
        );

        assert_eq!(party_damage, 1_000);
        assert_eq!(
            statistics
                .summary
                .support
                .and_then(|support| support.median_contribution),
            Some(0.1)
        );
    }

    #[test]
    fn progression_support_contribution_uses_support_party_damage() {
        let party_info = HashMap::from([
            (
                0,
                vec![
                    "Support".to_string(),
                    "DpsA".to_string(),
                    "DpsB".to_string(),
                ],
            ),
            (1, vec!["OtherDps".to_string()]),
        ]);
        let mut support = progression_row("Support", vec![], true);
        support.class_id = 204;
        support.spec = Some("Blessed Aura".to_string());
        support.player_damage_dealt = 10;
        support.rdps_damage_given = 100;
        support.party_info = Some(party_info.clone());

        let mut dps_a = progression_row("DpsA", vec![], true);
        dps_a.spec = Some("Mayhem".to_string());
        dps_a.player_damage_dealt = 400;
        dps_a.party_info = Some(party_info.clone());

        let mut dps_b = progression_row("DpsB", vec![], true);
        dps_b.spec = Some("Esoteric Flurry".to_string());
        dps_b.player_damage_dealt = 600;
        dps_b.party_info = Some(party_info.clone());

        let mut other_party_dps = progression_row("OtherDps", vec![], true);
        other_party_dps.spec = Some("Barrage Enhancement".to_string());
        other_party_dps.player_damage_dealt = 1_000;
        other_party_dps.party_info = Some(party_info);

        let statistics = build_raid_progression_statistics(
            vec![support, dps_a, dps_b, other_party_dps],
            &HashMap::new(),
            &HashMap::new(),
            &[],
        );
        let support = statistics
            .players
            .iter()
            .find(|player| player.name == "Support")
            .unwrap();

        assert!(support.is_support);
        assert_eq!(support.average_support_contribution, Some(0.1));
    }

    #[test]
    fn progression_deaths_count_until_final_wipe_reset() {
        let rows = vec![
            progression_row("EarlyDeath", vec![50_000], false),
            progression_row("SecondDeath", vec![120_000], false),
            progression_row("WipeCause", vec![295_000], false),
            progression_row("WipeReset", vec![296_000], false),
        ];

        let deaths = progression_death_counts(&rows);

        assert_eq!(deaths.get("EarlyDeath"), Some(&1));
        assert_eq!(deaths.get("SecondDeath"), Some(&1));
        assert_eq!(deaths.get("WipeCause"), Some(&1));
        assert_eq!(deaths.get("WipeReset"), None);
        assert_eq!(deaths.values().sum::<i32>(), 3);
    }

    #[test]
    fn progression_deaths_keep_late_deaths_when_they_are_not_a_full_reset() {
        let rows = vec![
            progression_row("LateDeathA", vec![295_000], false),
            progression_row("LateDeathB", vec![296_000], false),
            progression_row("AliveA", vec![], false),
            progression_row("AliveB", vec![], false),
        ];

        let deaths = progression_death_counts(&rows);

        assert_eq!(deaths.get("LateDeathA"), Some(&1));
        assert_eq!(deaths.get("LateDeathB"), Some(&1));
        assert_eq!(deaths.values().sum::<i32>(), 2);
    }

    #[test]
    fn progression_pull_uses_later_boss_hp_log_when_current_boss_is_dead() {
        let mut row = progression_row("Player", vec![], false);
        row.boss_name = "First Boss".to_string();
        row.boss_current_hp = Some(0);
        row.boss_max_hp = Some(1_000);
        row.boss_hp_bars = Some(100);
        row.boss_hp_bars_by_name
            .insert("First Boss".to_string(), 100);
        row.boss_hp_bars_by_name
            .insert("Second Boss".to_string(), 80);
        row.boss_hp_log
            .insert("First Boss".to_string(), vec![BossHpLog::new(120, 0, 0.0)]);
        row.boss_hp_log.insert(
            "Second Boss".to_string(),
            vec![BossHpLog::new(180, 400, 0.4)],
        );
        let mut boss_order = HashMap::new();
        boss_order.insert("First Boss".to_string(), 0);
        boss_order.insert("Second Boss".to_string(), 1);
        let rows = vec![row.clone()];

        let pull = build_progression_pull(&row, &rows, &boss_order, &BTreeMap::new());

        assert_eq!(pull.boss_name, "Second Boss");
        assert_eq!(pull.progress_bars, Some(32));
        assert_eq!(pull.progress_percent, Some(40.0));
    }

    #[test]
    fn raid_rows_group_bosses_by_raid_name() {
        let rows = vec![
            CharacterStatisticsRow {
                id: 1,
                fight_start: 1_000,
                boss_name: "Dark Mountain Predator".to_string(),
                raid_name: Some("Valtan G1".to_string()),
                duration: 120_000,
                difficulty: Some("Hard".to_string()),
                cleared: false,
                my_dps: 100,
                my_rdps: Some(110),
                my_ndps: Some(90),
                udps: Some(80),
                rdps_damage_given: 0,
                support_party_damage: 1_000,
                support_ap: None,
                support_brand: None,
                support_identity: None,
                support_hyper: None,
            },
            CharacterStatisticsRow {
                id: 2,
                fight_start: 2_000,
                boss_name: "Leader Lugaru".to_string(),
                raid_name: Some("Valtan G1".to_string()),
                duration: 180_000,
                difficulty: Some("Hard".to_string()),
                cleared: true,
                my_dps: 200,
                my_rdps: Some(220),
                my_ndps: Some(180),
                udps: Some(160),
                rdps_damage_given: 0,
                support_party_damage: 1_000,
                support_ap: None,
                support_brand: None,
                support_identity: None,
                support_hyper: None,
            },
        ];
        let raids = build_raid_rows(&rows);

        assert_eq!(raids.len(), 1);
        assert_eq!(raids[0].boss_name, "Valtan G1");
        assert_eq!(raids[0].attempts, 2);
        assert_eq!(raids[0].clears, 1);
        assert_eq!(raids[0].median_dps, Some(200));
        assert_eq!(raids[0].median_rdps, Some(220));
        assert_eq!(raids[0].median_ndps, Some(180));
    }

    fn stats_test_entity(
        name: &str,
        class_id: u32,
        spec: Option<&str>,
        damage_dealt: i64,
        support_contribution: bool,
    ) -> EncounterEntity {
        EncounterEntity {
            name: name.to_string(),
            entity_type: EntityType::Player,
            class_id,
            spec: spec.map(|spec| spec.to_string()),
            damage_stats: DamageStats {
                damage_dealt,
                rdps_damage_given: if support_contribution { 100 } else { 0 },
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn progression_row(name: &str, death_events: Vec<i64>, cleared: bool) -> RaidProgressionRow {
        RaidProgressionRow {
            id: 1,
            fight_start: 1_000,
            boss_name: "Boss".to_string(),
            gate: "Gate".to_string(),
            duration: 300_000,
            difficulty: Some("Hard".to_string()),
            cleared,
            team_dps: 1,
            damage_taken: 0,
            local_player: "Local".to_string(),
            boss_hp_bars: None,
            boss_hp_bars_by_name: HashMap::new(),
            boss_current_hp: None,
            boss_max_hp: None,
            boss_hp_log: HashMap::new(),
            player_name: name.to_string(),
            class_id: 102,
            class_name: "Class".to_string(),
            spec: Some("Spec".to_string()),
            player_damage_dealt: 1,
            dps: 1,
            rdps: None,
            ndps: None,
            player_damage_taken: 0,
            death_events,
            rdps_damage_given: 0,
            party_info: None,
            support_ap: None,
            support_brand: None,
            support_identity: None,
            support_hyper: None,
        }
    }

    #[test]
    fn should_insert_encounter() {
        let version = "1.14.0";
        let current_dir = std::env::current_dir().unwrap();
        AssetPreloader::new(&current_dir).unwrap();
        let database = Database::memory(version).unwrap();

        let repository = database.create_repository();
        let args = build_args(version);

        let expected_encounter = {
            let mut cloned = args.clone();
            calculate_entities(&mut cloned).unwrap();
            let mut encounter = cloned.encounter;
            normalize_encounter_damage_totals(&mut encounter);
            encounter
        };

        let id = repository.insert_data(args).unwrap();
        let id_str = id.to_string();

        let actual_encounter = repository.get_encounter(&id_str).unwrap();
        let filter = SearchFilter {
            bosses: vec![],
            min_duration: 10,
            max_duration: 10000000,
            cleared: true,
            favorite: false,
            difficulty: "Hard".to_string(),
            boss_only_damage: false,
            sort: "id".to_string(),
            order: "desc".to_string(),
            raids_only: true,
            local_player: "".to_string(),
        };

        let paged = repository
            .get_encounter_preview(GetEncounterPreviewArgs {
                page: 0,
                page_size: 10,
                search: "".to_string(),
                filter,
            })
            .unwrap();

        assert_eq!(
            actual_encounter.current_boss_name,
            expected_encounter.current_boss_name
        );
        assert_eq!(actual_encounter.duration, expected_encounter.duration);
        assert_eq!(actual_encounter.difficulty, expected_encounter.difficulty);
        assert_eq!(actual_encounter.cleared, expected_encounter.cleared);
        assert_eq!(
            actual_encounter.boss_only_damage,
            expected_encounter.boss_only_damage
        );

        assert!(actual_encounter.encounter_damage_stats.dps > 0);
        assert_eq!(
            actual_encounter.encounter_damage_stats.dps,
            expected_encounter.encounter_damage_stats.dps
        );
        assert!(actual_encounter.encounter_damage_stats.top_damage_dealt > 0);
        assert_eq!(
            actual_encounter.encounter_damage_stats.top_damage_dealt,
            expected_encounter.encounter_damage_stats.top_damage_dealt
        );
        assert_eq!(
            actual_encounter.encounter_damage_stats.top_damage_taken,
            expected_encounter.encounter_damage_stats.top_damage_taken
        );
        assert!(actual_encounter.encounter_damage_stats.total_damage_dealt > 0);
        assert_eq!(
            actual_encounter.encounter_damage_stats.total_damage_dealt,
            expected_encounter.encounter_damage_stats.total_damage_dealt
        );
        assert_eq!(
            actual_encounter.encounter_damage_stats.total_damage_taken,
            expected_encounter.encounter_damage_stats.total_damage_taken
        );
        assert_eq!(
            actual_encounter
                .encounter_damage_stats
                .total_effective_shielding,
            expected_encounter
                .encounter_damage_stats
                .total_effective_shielding
        );
        assert!(actual_encounter.encounter_damage_stats.boss_hp_log.len() > 0);

        let actual_misc = actual_encounter.encounter_damage_stats.misc.unwrap();
        assert!(actual_misc.raid_clear.filter(|pr| *pr).is_some());
        assert!(actual_misc.region.filter(|pr| !pr.is_empty()).is_some());
        assert!(actual_misc.version.filter(|pr| !pr.is_empty()).is_some());
        assert!(actual_misc.party_info.filter(|pr| !pr.is_empty()).is_some());

        let mut actual: Vec<_> = actual_encounter.entities.values().collect();
        actual.sort_by_key(|e| &e.name);

        let mut expected: Vec<_> = expected_encounter.entities.values().collect();
        expected.sort_by_key(|e| &e.name);

        for (actual, expected) in actual.iter().zip(expected.iter()) {
            assert_eq!(actual.name, expected.name);

            if actual.entity_type == EntityType::Boss {
                assert!(actual.damage_stats.damage_dealt > 0);
                assert!(actual.damage_stats.damage_taken > 0);
                continue;
            }

            assert_eq!(actual.gear_score, expected.gear_score);
            assert!(actual.spec.is_some());
            assert_eq!(actual.spec, expected.spec);
            assert!(actual.skill_stats.casts > 0);
            assert!(actual.skill_stats.crits > 0);
            assert!(actual.skill_stats.hits > 0);

            for (_, skill) in actual.skills.iter() {
                assert!(skill.dps > 0);
                assert!(skill.hits > 0);
                assert!(skill.crits > 0);
                assert!(skill.total_damage > 0);
                assert!(skill.cast_log.len() > 0);
            }

            assert!(actual.combat_power.filter(|pr| *pr > 1500.0).is_some());
            assert!(actual.loadout_hash.is_some());
            assert!(
                actual
                    .engraving_data
                    .as_ref()
                    .filter(|pr| pr.len() > 1)
                    .is_some()
            );
            assert!(actual.ark_passive_active.unwrap());
            assert!(
                actual
                    .ark_passive_data
                    .as_ref()
                    .filter(|pr| pr.enlightenment.is_some()
                        && pr.leap.is_some()
                        && pr.evolution.is_some())
                    .is_some()
            );

            assert!(actual.damage_stats.unbuffed_damage > 0);
            assert!(actual.damage_stats.unbuffed_dps > 0);
            assert!(actual.damage_stats.damage_dealt > 0);
            assert!(actual.damage_stats.hyper_awakening_damage > 0);
            assert!(actual.damage_stats.dps > 0);
            assert!(actual.damage_stats.dps_average.len() > 0);
            assert!(actual.damage_stats.dps_rolling_10s_avg.len() > 0);
            assert!(actual.damage_stats.damage_taken > 0);
            assert!(actual.damage_stats.shields_given > 0);
            assert!(!actual.damage_stats.shields_given_by.is_empty());
            assert!(actual.damage_stats.shields_received > 0);
            assert!(!actual.damage_stats.shields_received_by.is_empty());
            assert!(actual.damage_stats.damage_absorbed > 0);
            assert!(!actual.damage_stats.damage_absorbed_by.is_empty());
            assert!(actual.damage_stats.damage_absorbed_on_others > 0);
            assert!(!actual.damage_stats.damage_absorbed_on_others_by.is_empty());
        }

        let preview = paged.encounters.first().unwrap();

        assert_eq!(preview.duration, expected_encounter.duration);
        assert_eq!(preview.classes.len(), 8);
        assert_eq!(preview.names.len(), 8);
        assert_eq!(preview.fight_start, expected_encounter.fight_start);
        assert_eq!(preview.difficulty, expected_encounter.difficulty);
        assert_eq!(preview.boss_name, expected_encounter.current_boss_name);
        assert_eq!(preview.cleared, true);
        assert_eq!(preview.cleared, expected_encounter.cleared);

        assert!(preview.support_ap.is_some());
        assert!(preview.support_brand.is_some());
        assert!(preview.support_identity.is_some());
        assert!(preview.support_hyper.is_some());

        {
            let local_player_name = &expected_encounter.local_player;
            let local_player = expected_encounter.entities.get(local_player_name).unwrap();
            assert!(preview.my_dps > 0);
            assert_eq!(preview.my_dps, local_player.damage_stats.dps);
        }
    }

    fn build_args(version: &str) -> InsertEncounterArgs {
        let player11 = PlayerSpec {
            class_id: 102,
            class_name: "Berserker".to_string(),
            specialisation: "Mayhem",
            crit_rate: 0.25,
            gear_score: 1620.0,
            hp: 1_000_000,
            info: InspectInfo {
                combat_power: Some(CombatPower {
                    id: 1,
                    score: 1800.0,
                }),
                ark_passive_enabled: true,
                ark_passive_data: Some(ArkPassiveData {
                    evolution: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    enlightenment: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    leap: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    ark_grid_order: None,
                }),
                engravings: Some(vec![1118, 1299]),
                gems: Some(vec![
                    GemData {
                        tier: 2,
                        skill_id: 16640,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 16120,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 16080,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 16300,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 16050,
                        gem_type: 63,
                        value: 4400,
                    },
                ]),
                loadout_snapshot: Some(String::from("")),
            },
        };
        let player12 = PlayerSpec {
            class_id: 502,
            class_name: "Sharpshooter".to_string(),
            specialisation: "Loyal Companion",
            crit_rate: 0.28,
            gear_score: 1600.0,
            hp: 1_000_000,
            info: InspectInfo {
                combat_power: Some(CombatPower {
                    id: 1,
                    score: 1800.0,
                }),
                ark_passive_enabled: true,
                ark_passive_data: Some(ArkPassiveData {
                    evolution: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    enlightenment: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    leap: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    ark_grid_order: None,
                }),
                engravings: Some(vec![1118, 1299]),
                gems: Some(vec![
                    GemData {
                        tier: 2,
                        skill_id: 50010,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 28220,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 28090,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 28250,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 28070,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 28110,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 28130,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 28150,
                        gem_type: 63,
                        value: 4400,
                    },
                ]),
                loadout_snapshot: Some(String::from("")),
            },
        };
        let player13 = PlayerSpec {
            class_id: 302,
            class_name: "Wardancer".to_string(),
            specialisation: "Esoteric Skill Enhancement",
            crit_rate: 0.30,
            gear_score: 1580.0,
            hp: 1_000_000,
            info: InspectInfo {
                combat_power: Some(CombatPower {
                    id: 1,
                    score: 1800.0,
                }),
                ark_passive_enabled: true,
                ark_passive_data: Some(ArkPassiveData {
                    evolution: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    enlightenment: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    leap: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    ark_grid_order: None,
                }),
                engravings: Some(vec![1118, 1299]),
                gems: Some(vec![
                    GemData {
                        tier: 2,
                        skill_id: 22340,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 22080,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 22120,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 22310,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 22270,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 22240,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 22210,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 22160,
                        gem_type: 63,
                        value: 4400,
                    },
                ]),
                loadout_snapshot: Some(String::from("")),
            },
        };
        let player14 = PlayerSpec {
            class_id: 204,
            class_name: "Bard".to_string(),
            specialisation: "Desperate Salvation",
            crit_rate: 0.15,
            gear_score: 1500.0,
            hp: 1_000_000,
            info: InspectInfo {
                combat_power: Some(CombatPower {
                    id: 2,
                    score: 1800.0,
                }),
                ark_passive_enabled: true,
                ark_passive_data: Some(ArkPassiveData {
                    evolution: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    enlightenment: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    leap: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    ark_grid_order: None,
                }),
                engravings: Some(vec![1255, 1251, 1134, 1167, 77300001]),
                gems: Some(vec![
                    GemData {
                        tier: 2,
                        skill_id: 21170,
                        gem_type: 35,
                        value: 2400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 21080,
                        gem_type: 35,
                        value: 2400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 21250,
                        gem_type: 35,
                        value: 2400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 21290,
                        gem_type: 35,
                        value: 2400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 21160,
                        gem_type: 35,
                        value: 2400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 21160,
                        gem_type: 64,
                        value: 1000,
                    },
                ]),
                loadout_snapshot: Some(String::from("")),
            },
        };

        let player21 = PlayerSpec {
            class_id: 603,
            class_name: "Aeromancer".to_string(),
            specialisation: "Drizzle",
            crit_rate: 0.25,
            gear_score: 1620.0,
            hp: 0,
            info: InspectInfo {
                combat_power: Some(CombatPower {
                    id: 1,
                    score: 1800.0,
                }),
                ark_passive_enabled: true,
                ark_passive_data: Some(ArkPassiveData {
                    evolution: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    enlightenment: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    leap: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    ark_grid_order: None,
                }),
                engravings: Some(vec![1118, 1299]),
                gems: Some(vec![
                    GemData {
                        tier: 2,
                        skill_id: 32010,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 32150,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 32160,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 32170,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 32190,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 32210,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 32220,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 32230,
                        gem_type: 63,
                        value: 4400,
                    },
                ]),
                loadout_snapshot: Some(String::from("")),
            },
        };
        let player22 = PlayerSpec {
            class_id: 504,
            class_name: "Artillerist".to_string(),
            specialisation: "Barrage Enhancement",
            crit_rate: 0.28,
            gear_score: 1600.0,
            hp: 1_000_000,
            info: InspectInfo {
                combat_power: Some(CombatPower {
                    id: 1,
                    score: 1800.0,
                }),
                ark_passive_enabled: true,
                ark_passive_data: Some(ArkPassiveData {
                    evolution: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    enlightenment: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    leap: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    ark_grid_order: None,
                }),
                engravings: Some(vec![1118, 1299]),
                gems: Some(vec![
                    GemData {
                        tier: 2,
                        skill_id: 30260,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 30270,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 30290,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 30340,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 30380,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 30310,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 30320,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 30392,
                        gem_type: 63,
                        value: 4400,
                    },
                ]),
                loadout_snapshot: Some(String::from("")),
            },
        };
        let player23 = PlayerSpec {
            class_id: 402,
            class_name: "Deathblade".to_string(),
            specialisation: "Remaining Energy",
            crit_rate: 0.30,
            gear_score: 1580.0,
            hp: 1_000_000,
            info: InspectInfo {
                combat_power: Some(CombatPower {
                    id: 1,
                    score: 1800.0,
                }),
                ark_passive_enabled: true,
                ark_passive_data: Some(ArkPassiveData {
                    evolution: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    enlightenment: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    leap: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    ark_grid_order: None,
                }),
                engravings: Some(vec![1118, 1299]),
                gems: Some(vec![
                    GemData {
                        tier: 2,
                        skill_id: 25010,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 25180,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 25160,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 25110,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 25120,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 25030,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 25040,
                        gem_type: 63,
                        value: 4400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 25050,
                        gem_type: 63,
                        value: 4400,
                    },
                ]),
                loadout_snapshot: Some(String::from("")),
            },
        };
        let player24 = PlayerSpec {
            class_id: 105,
            class_name: "Paladin".to_string(),
            specialisation: "Blessed Aura",
            crit_rate: 0.15,
            gear_score: 1500.0,
            hp: 1_000_000,
            info: InspectInfo {
                combat_power: Some(CombatPower {
                    id: 2,
                    score: 1800.0,
                }),
                ark_passive_enabled: true,
                ark_passive_data: Some(ArkPassiveData {
                    evolution: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    enlightenment: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    leap: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                    ark_grid_order: None,
                }),
                engravings: Some(vec![1255, 1251, 1134, 1167, 77300001]),
                gems: Some(vec![
                    GemData {
                        tier: 2,
                        skill_id: 36080,
                        gem_type: 35,
                        value: 2400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 36120,
                        gem_type: 35,
                        value: 2400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 36220,
                        gem_type: 35,
                        value: 2400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 36170,
                        gem_type: 35,
                        value: 2400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 36200,
                        gem_type: 35,
                        value: 2400,
                    },
                    GemData {
                        tier: 2,
                        skill_id: 36200,
                        gem_type: 64,
                        value: 1000,
                    },
                ]),
                loadout_snapshot: Some(String::from("")),
            },
        };

        let raid_builder = RaidBuilder::new()
            .add_party((player11, player12, player13, player14))
            .add_party((player21, player22, player23, player24))
            .set_boss(
                "Mordum, the Abyssal Punisher",
                485800,
                1_100_000_000_000,
                15,
            )
            .set_region("EUC")
            .set_version(version)
            .set_damage_range(1_000_000, 2_000_000)
            .set_difficulty("Hard")
            .set_cleared(true);

        let args = raid_builder.build();

        args
    }

    #[derive(Clone)]
    struct PlayerSpec {
        class_id: u32,
        class_name: String,
        specialisation: &'static str,
        crit_rate: f64,
        gear_score: f32,
        hp: i64,
        info: InspectInfo,
    }

    struct RaidBuilder {
        parties: Vec<Vec<PlayerSpec>>,
        boss_name: String,
        boss_npc_id: u32,
        boss_hp: i64,
        duration_minutes: i64,
        region: String,
        version: String,
        difficulty: String,
        cleared: bool,
        rng: ThreadRng,
        damage_range: (i64, i64),
        damage_taken_range: (i64, i64),
    }

    impl RaidBuilder {
        fn new() -> Self {
            Self {
                parties: Vec::new(),
                boss_name: String::new(),
                boss_npc_id: 0,
                boss_hp: 0,
                cleared: false,
                duration_minutes: 15,
                region: "EUC".to_string(),
                version: "0.0.1".to_string(),
                difficulty: "Hard".to_string(),
                rng: rand::rng(),
                damage_range: (500, 1500),
                damage_taken_range: (500, 1000),
            }
        }

        fn add_party(mut self, players: (PlayerSpec, PlayerSpec, PlayerSpec, PlayerSpec)) -> Self {
            self.parties
                .push(vec![players.0, players.1, players.2, players.3]);
            self
        }

        fn set_boss(mut self, name: &str, npc_id: u32, hp: i64, duration_minutes: i64) -> Self {
            self.boss_name = name.to_string();
            self.boss_npc_id = npc_id;
            self.boss_hp = hp;
            self.duration_minutes = duration_minutes;
            self
        }

        fn set_region(mut self, region: &str) -> Self {
            self.region = region.to_string();
            self
        }

        fn set_version(mut self, version: &str) -> Self {
            self.version = version.to_string();
            self
        }

        fn set_cleared(mut self, cleared: bool) -> Self {
            self.cleared = cleared;
            self
        }

        fn set_difficulty(mut self, difficulty: &str) -> Self {
            self.difficulty = difficulty.to_string();
            self
        }

        fn set_damage_range(mut self, min: i64, max: i64) -> Self {
            assert!(min > 0 && max >= min, "Invalid damage range");
            self.damage_range = (min, max);
            self
        }

        fn build(mut self) -> InsertEncounterArgs {
            let fight_start = Utc::now().timestamp_millis();
            let last_combat_packet = fight_start + self.duration_minutes * 60 * 1000;
            let duration_ms = last_combat_packet - fight_start;
            let duration_s = duration_ms / 1000;

            let total_players = self.parties.iter().map(|p| p.len()).sum::<usize>();
            let raid_dps: i64 = self.boss_hp / duration_s;
            let per_player_total_damage: i64 = raid_dps / total_players as i64 * duration_s;

            let (entities_with_spec, player_names) = generate_entities_for_parties(&self.parties);

            let local_player = player_names[0].clone();
            let mut boss = self.generate_boss_entity();

            boss.damage_stats.damage_dealt += self
                .rng
                .random_range(self.damage_range.0..self.damage_range.1);

            let mut boss_hp_logs: HashMap<String, Vec<BossHpLog>> = HashMap::new();
            let mut boss_hp_log = Vec::with_capacity(duration_s as usize + 1);
            for t in 0..=duration_s as i32 {
                let dealt = raid_dps * t as i64;
                let hp = (self.boss_hp - dealt).max(0);
                let percent = hp as f32 / self.boss_hp as f32;
                boss_hp_log.push(BossHpLog::new(t, hp, percent));
            }
            boss_hp_logs.insert(boss.name.clone(), boss_hp_log);

            let party_vec: Vec<Vec<String>> =
                player_names.chunks(4).map(|chunk| chunk.to_vec()).collect();

            let mut party_info: HashMap<i32, Vec<String>> = HashMap::new();
            for (idx, party) in player_names.chunks(4).enumerate() {
                party_info.insert(idx as i32 + 1, party.to_vec());
            }

            let mut encounter_entities_with_stats = HashMap::new();
            let mut damage_log: HashMap<String, Vec<(i64, i64)>> = HashMap::new();
            let mut cast_log: HashMap<String, HashMap<u32, Vec<i32>>> = HashMap::new();
            let mut skill_cast_log: HashMap<u64, HashMap<u32, BTreeMap<i64, SkillCast>>> =
                HashMap::new();
            let mut player_info: HashMap<String, InspectInfo> = HashMap::new();
            for (name, (spec, mut entity)) in entities_with_spec.into_iter() {
                if entity.entity_type == EntityType::Player {
                    update_skill_and_damage_stats(
                        duration_s,
                        &spec,
                        &mut damage_log,
                        &mut cast_log,
                        &mut skill_cast_log,
                        self.damage_range,
                        &mut self.rng,
                        &mut entity,
                    );
                    player_info.insert(name.clone(), spec.info);
                    update_damage_taken(self.damage_taken_range, &mut self.rng, &mut entity);
                    update_buffs_heals_and_absorb(&mut self.rng, &mut entity);
                }
                encounter_entities_with_stats.insert(name, entity);
            }

            encounter_entities_with_stats.insert(boss.name.clone(), boss.clone());

            let local_player_entity = encounter_entities_with_stats.get(&local_player).unwrap();
            let mut skill_cooldowns = HashMap::new();

            for (id, _) in local_player_entity.skills.iter() {
                skill_cooldowns.insert(
                    *id,
                    vec![CastEvent {
                        cooldown_duration_ms: self.rng.random_range(1000..5000),
                        timestamp: self.rng.random_range(1000..5000),
                    }],
                );
            }

            let misc = EncounterMisc {
                boss_hp_log: None,
                raid_clear: Some(true),
                party_info: Some(party_info.clone()),
                region: Some(self.region.clone()),
                version: Some(self.version.clone()),
                rdps_valid: None,
                rdps_message: None,
                ntp_fight_start: Some(fight_start),
                manual_save: None,
                intermission_start: None,
                intermission_end: None,
                contribution_splits: None,
            };

            let encounter_damage_stats = EncounterDamageStats {
                total_damage_dealt: self.boss_hp,
                top_damage_dealt: per_player_total_damage,
                total_damage_taken: 0,
                top_damage_taken: 0,
                dps: raid_dps,
                buffs: HashMap::new(),
                debuffs: HashMap::new(),
                total_shielding: 0,
                total_effective_shielding: 0,
                applied_shield_buffs: HashMap::new(),
                unknown_buffs: HashSet::new(),
                misc: Some(misc.clone()),
                boss_hp_log: boss_hp_logs.clone(),
            };

            let encounter = Encounter {
                last_combat_packet,
                fight_start,
                local_player,
                entities: encounter_entities_with_stats.clone(),
                current_boss_name: boss.name.clone(),
                current_boss: Some(boss),
                encounter_damage_stats,
                duration: duration_ms,
                difficulty: Some(self.difficulty.clone()),
                favorite: false,
                cleared: true,
                boss_only_damage: false,
                sync: None,
                region: Some(self.region.clone()),
            };

            let insert_args = InsertEncounterArgs {
                encounter,
                damage_log,
                cast_log,
                boss_hp_log: boss_hp_logs,
                raid_clear: true,
                party_info: party_vec,
                raid_difficulty: self.difficulty.clone(),
                region: Some(self.region.clone()),
                player_info: Some(player_info),
                meter_version: self.version.clone(),
                ntp_fight_start: fight_start,
                rdps_valid: true,
                rdps_message: None,
                manual: false,
                skill_cast_log,
                skill_cooldowns,
                intermission_start: None,
                intermission_end: None,
                contribution_splits: vec![],
            };

            insert_args
        }

        fn generate_boss_entity(&self) -> EncounterEntity {
            EncounterEntity {
                id: 1000,
                character_id: 0,
                npc_id: self.boss_npc_id,
                hp_bars: Some(100),
                name: self.boss_name.clone(),
                entity_type: EntityType::Boss,
                class_id: 0,
                class: String::new(),
                gear_score: 0.0,
                current_hp: self.boss_hp,
                max_hp: self.boss_hp,
                current_shield: 0,
                is_dead: false,
                skills: HashMap::new(),
                damage_stats: DamageStats {
                    damage_taken: self.boss_hp,
                    ..Default::default()
                },
                skill_stats: SkillStats::default(),
                engraving_data: None,
                ark_passive_active: None,
                ark_passive_data: None,
                spec: None,
                loadout_hash: None,
                combat_power: None,
            }
        }
    }

    fn update_skill_and_damage_stats(
        duration_seconds: i64,
        spec: &PlayerSpec,
        entities_damage_log: &mut HashMap<String, Vec<(i64, i64)>>,
        cast_log: &mut HashMap<String, HashMap<u32, Vec<i32>>>,
        skill_cast_log: &mut HashMap<u64, HashMap<u32, BTreeMap<i64, SkillCast>>>,
        damage_range: (i64, i64),
        rng: &mut ThreadRng,
        entity: &mut EncounterEntity,
    ) {
        entity.skill_stats = SkillStats::default();
        entity.damage_stats = DamageStats::default();

        let damage_log = entities_damage_log.entry(entity.name.clone()).or_default();

        for skill in entity.skills.values_mut() {
            skill.casts = 0;
            skill.hits = 0;
            skill.crits = 0;

            let mut per_skill_map: BTreeMap<i64, SkillCast> = BTreeMap::new();
            let mut per_skill_vec: Vec<i32> = Vec::new();

            for it in 0..100 {
                let dmg = rng.random_range(damage_range.0..=damage_range.1);
                let is_crit = rng.random_bool(spec.crit_rate);
                let timestamp = it * 1000;
                skill.casts += 1;
                entity.skill_stats.casts += 1;
                skill.hits += 1;
                entity.skill_stats.hits += 1;
                skill.total_damage += dmg;

                let mut skill_hit = SkillHit {
                    damage: dmg,
                    timestamp,
                    ..Default::default()
                };

                if is_crit {
                    entity.skill_stats.crits += 1;
                    skill_hit.crit = true;
                    skill.crits += 1;
                }

                entity.damage_stats.damage_dealt += dmg;

                damage_log.push((timestamp, dmg));

                let skill_cast = SkillCast {
                    hits: vec![skill_hit],
                    last: timestamp,
                    timestamp,
                    ..Default::default()
                };

                per_skill_vec.push(timestamp as i32);
                per_skill_map.insert(timestamp, skill_cast);
            }

            cast_log
                .entry(entity.name.clone())
                .or_default()
                .insert(skill.id, per_skill_vec);

            skill_cast_log
                .entry(entity.id)
                .or_default()
                .insert(skill.id, per_skill_map);
        }

        entity.damage_stats.hyper_awakening_damage +=
            rng.random_range(1_000_000_000..2_000_000_000);
    }

    fn update_damage_taken(
        damage_taken: (i64, i64),
        rng: &mut ThreadRng,
        entity: &mut EncounterEntity,
    ) {
        entity.damage_stats.damage_taken += rng.random_range(damage_taken.0..damage_taken.1)
    }

    fn update_buffs_heals_and_absorb(rng: &mut ThreadRng, entity: &mut EncounterEntity) {
        let support_buff_ids = [101u32, 102u32, 103u32];
        let pick_count = rng.random_range(1..=3);

        for buff_id in support_buff_ids.choose_multiple(rng, pick_count) {
            let value = rng.random_range(1_000..=2_000);
            entity.damage_stats.buffed_by_support += value;
            entity.damage_stats.debuffed_by.insert(*buff_id, value / 2);
        }

        entity.damage_stats.buffed_by_identity += rng.random_range(0..=5);
        entity.damage_stats.buffed_by_hat += rng.random_range(0..=5);
        entity.damage_stats.debuffed_by_support += rng.random_range(0..=3);

        let absorb_value = rng.random_range(10_000..=100_000);
        entity.damage_stats.damage_absorbed += absorb_value;
        entity
            .damage_stats
            .damage_absorbed_by
            .insert(1, absorb_value);

        let shield_value = rng.random_range(5_000..=50_000);
        entity.damage_stats.shields_given += shield_value;
        entity.damage_stats.shields_given_by.insert(1, shield_value);
        entity.damage_stats.shields_received += shield_value / 2;
        entity
            .damage_stats
            .shields_received_by
            .insert(1, shield_value / 2);
        entity.damage_stats.damage_absorbed_on_others += shield_value / 3;
        entity
            .damage_stats
            .damage_absorbed_on_others_by
            .insert(1, shield_value / 3);
    }

    fn get_skills_by_spec(specialisation: &str) -> HashMap<u32, Skill> {
        let mut skills = HashMap::new();

        match specialisation {
            "Mayhem" => {
                for id in [16010, 16640, 16120, 16080, 16300, 16050, 16220, 16030] {
                    skills.insert(
                        id,
                        Skill {
                            id,
                            name: format!("Skill {}", id),
                            ..Default::default()
                        },
                    );
                }
            }
            "Loyal Companion" => {
                for id in [50010, 28220, 28090, 28250, 28070, 28110, 28130, 28150] {
                    skills.insert(
                        id,
                        Skill {
                            id,
                            name: format!("Skill {}", id),
                            ..Default::default()
                        },
                    );
                }
            }
            "Esoteric Skill Enhancement" => {
                for id in [22340, 22080, 22120, 22160, 22210, 22240, 22270, 22310] {
                    skills.insert(
                        id,
                        Skill {
                            id,
                            name: format!("Skill {}", id),
                            ..Default::default()
                        },
                    );
                }
            }
            "Desperate Salvation" => {
                for id in [21290, 21170, 21080, 21160, 21250, 21040, 21020, 21210] {
                    skills.insert(
                        id,
                        Skill {
                            id,
                            name: format!("Skill {}", id),
                            ..Default::default()
                        },
                    );
                }
            }
            "Drizzle" => {
                for id in [32010, 32150, 32160, 32170, 32190, 32210, 32220, 32230] {
                    skills.insert(
                        id,
                        Skill {
                            id,
                            name: format!("Skill {}", id),
                            ..Default::default()
                        },
                    );
                }
            }
            "Barrage Enhancement" => {
                for id in [30260, 30270, 30290, 30340, 30392, 30320, 30310, 30380] {
                    skills.insert(
                        id,
                        Skill {
                            id,
                            name: format!("Skill {}", id),
                            ..Default::default()
                        },
                    );
                }
            }
            "Remaining Energy" => {
                for id in [25010, 25180, 25160, 25110, 25120, 25030, 25040, 25050] {
                    skills.insert(
                        id,
                        Skill {
                            id,
                            name: format!("Skill {}", id),
                            ..Default::default()
                        },
                    );
                }
            }
            "Blessed Aura" => {
                for id in [36080, 36120, 36200, 36170, 36800, 36040, 36020, 36220] {
                    skills.insert(
                        id,
                        Skill {
                            id,
                            name: format!("Skill {}", id),
                            ..Default::default()
                        },
                    );
                }
            }
            _ => {}
        }

        skills
    }

    fn build_entity_from_spec(name: &str, spec: &PlayerSpec, idx: usize) -> EncounterEntity {
        let skills = get_skills_by_spec(&spec.specialisation);

        let entity = EncounterEntity {
            id: idx as u64 + 1,
            character_id: idx as u64 + 101,
            npc_id: 0,
            name: name.to_string(),
            entity_type: EntityType::Player,
            class_id: spec.class_id,
            class: spec.class_name.clone(),
            gear_score: spec.gear_score,
            current_hp: spec.hp,
            max_hp: spec.hp,
            current_shield: 0,
            is_dead: false,
            skills,
            damage_stats: DamageStats::default(),
            skill_stats: SkillStats::default(),
            ..Default::default()
        };

        entity
    }

    fn generate_entities_for_parties(
        parties: &[Vec<PlayerSpec>],
    ) -> (HashMap<String, (PlayerSpec, EncounterEntity)>, Vec<String>) {
        let mut entities: HashMap<String, (PlayerSpec, EncounterEntity)> = HashMap::new();
        let mut player_names = Vec::new();

        for party in parties {
            for (idx, spec) in party.iter().enumerate() {
                let name = format!("Player{}", player_names.len() + 1);
                let entity = build_entity_from_spec(&name, spec, idx);

                entities.insert(name.clone(), (spec.clone(), entity));
                player_names.push(name);
            }
        }

        (entities, player_names)
    }
}
