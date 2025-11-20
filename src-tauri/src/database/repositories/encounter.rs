use anyhow::{Ok, Result};
use hashbrown::HashMap;
use log::*;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, params_from_iter, OptionalExtension, Transaction};
use serde_json::json;
use std::cmp::{max, Reverse};

use crate::{
    constants::DB_VERSION,
    database::{models::*, queries::*, utils::*},
    models::*,
    utils::*,
};
pub struct EncounterRepository(r2d2::Pool<SqliteConnectionManager>);

impl EncounterRepository {
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
            ntp_fight_start,
            region,
            ..
        } = args;

        let duration = encounter.last_combat_packet - encounter.fight_start;
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
                Some("invalid_stats".into())
            },
            ntp_fight_start: Some(*ntp_fight_start),
            manual_save: Some(args.manual),
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
            let compressed_skills = compress_json(&entity.skills)?;
            let compressed_damage_stats = compress_json(&entity.damage_stats)?;

            let support_buffs = buffs.get(&entity.name);

            let params = params![
                entity.name,
                encounter_id,
                entity.npc_id,
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
                    .unwrap_or(entity.damage_stats.buffed_by_support as f64 / damage_without_hyper),
                support_buffs.map(|b| b.brand).unwrap_or(
                    entity.damage_stats.debuffed_by_support as f64 / damage_without_hyper
                ),
                support_buffs.map(|b| b.identity).unwrap_or(
                    entity.damage_stats.buffed_by_identity as f64 / damage_without_hyper
                ),
                support_buffs
                    .map(|b| b.hyper)
                    .unwrap_or(entity.damage_stats.buffed_by_hat as f64 / damage_without_hyper),
                entity.damage_stats.unbuffed_damage,
                entity.damage_stats.unbuffed_dps
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
            ..
        } = args;

        let mut players: Vec<_> = encounter
            .entities
            .values()
            .filter(|e| {
                ((e.entity_type == EntityType::Player && e.class_id != 0)
                    || e.name == encounter.local_player)
                    && e.damage_stats.damage_dealt > 0
            })
            .collect();

        let local_player_dps = players
            .iter()
            .find(|e| e.name == encounter.local_player)
            .map(|e| e.damage_stats.dps)
            .unwrap_or_default();

        players.sort_unstable_by_key(|e| Reverse(e.damage_stats.damage_dealt));

        let preview_players = players
            .iter()
            .map(|e| format!("{}:{}", e.class_id, e.name))
            .collect::<Vec<_>>()
            .join(",");

        let params = params![
            encounter_id,
            encounter.fight_start,
            encounter.current_boss_name,
            encounter.last_combat_packet - encounter.fight_start,
            preview_players,
            raid_difficulty,
            encounter.local_player,
            local_player_dps,
            raid_clear,
            encounter.boss_only_damage,
        ];

        transaction
            .prepare_cached(INSERT_ENCOUNTER_PREVIEW)?
            .execute(params)?;

        Ok(())
    }
}

pub fn calculate_entities(args: &mut InsertEncounterArgs) -> Result<()> {
    let InsertEncounterArgs {
        encounter,
        cast_log,
        damage_log,
        skill_cast_log,
        player_info,
        skill_cooldowns,
        ..
    } = args;

    let fight_start = encounter.fight_start;
    let fight_end = encounter.last_combat_packet;
    let local_player_str = encounter.local_player.as_str();

    for (name, entity) in encounter.entities.iter_mut() {
        if !should_insert_entity(entity, &encounter.local_player) {
            continue;
        }

        update_entity_stats(entity, fight_start, fight_end, damage_log);

        if let Some(info) = player_info
            .as_ref()
            .and_then(|stats| stats.get(&entity.name))
        {
            apply_player_info(entity, info);
        }

        apply_cast_logs(entity, cast_log, skill_cast_log);

        if name == local_player_str {
            for (skill_id, events) in skill_cooldowns.iter() {
                if let Some(skill) = entity.skills.get_mut(skill_id) {
                    skill.time_available =
                        Some(get_total_available_time(events, fight_start, fight_end));
                }
            }
        }

        let spec = get_player_spec(entity, &encounter.encounter_damage_stats.buffs, false);
        entity.spec = Some(spec.clone());
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
    use rand::{rngs::ThreadRng, seq::IndexedRandom, Rng};
    use crate::database::repositories::tests::build_args;
    use super::*;

    #[test]
    fn should_insert_encounter() {
        let version = "1.14.0";
        let current_dir = std::env::current_dir().unwrap();
        AssetPreloader::new(&current_dir).unwrap();
        let database = Database::memory(version).unwrap();

        let repository = database.create_encounter_repository();
        let args = build_args(version);

        let expected_encounter = {
            let mut cloned = args.clone();
            calculate_entities(&mut cloned).unwrap();
            let encounter = cloned.encounter;
            encounter
        };

        let id = repository.insert_data(args).unwrap();
        let id_str = id.to_string();

        let actual_encounter = repository.get_encounter(&id_str).unwrap();
        let filter = SearchFilter {
            raid_type: None,
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
}
