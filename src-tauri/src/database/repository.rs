use std::cmp::{max, Reverse};
use anyhow::{Ok, Result};
use hashbrown::HashMap;
use log::*;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, params_from_iter, OptionalExtension, Transaction};
use serde_json::json;

use crate::{constants::DB_VERSION, database::{models::*, queries::*, utils::*}, models::*};
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
            upstream
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

        let encounter_count = connection
        .query_row(SELECT_ENCOUNTER_PREVIEW_COUNT, [], |row| {
            row.get(0)
        })?;

        let params = params![min_duration * 1000];
        let encounter_filtered_count = connection
            .query_row(SELECT_ENCOUNTER_PREVIEW_BY_GE_DURATION, params, |row| row.get(0))?;

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
        keep_favorites: bool,) -> Result<()> {

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

    pub fn get_encounter_preview(&self, args: GetEncounterPreviewArgs) -> Result<EncountersOverview> {

        let GetEncounterPreviewArgs {
            filter,
            page,
            page_size,
            search
        } = args;

        let connection = self.0.get()?;
        let (mut params, query, count_query) = prepare_get_encounter_preview_query(search, filter);
        let count_params = params.clone();

        let mut statement = connection.prepare_cached(&query)?;

        let offset = (page - 1) * page_size;

        params.push(page_size.to_string());
        params.push(offset.to_string());

        let params= params_from_iter(params);    
        let encounter_iter = statement.query_map(params, map_encounter_preview)?;
        
        let encounters: Vec<EncounterPreview> = encounter_iter.collect::<Result<_, _>>()?;

        let count: i32 = connection
            .query_row_and_then(&count_query, params_from_iter(count_params), |row| row.get(0))?;

        let value = EncountersOverview {
            encounters,
            total_encounters: count,
        };

        Ok(value)
    }

    pub fn delete_all_uncleared_encounters(&self, keep_favorites: bool) -> Result<()> {
        
        let connection = self.0.get()?;

        if keep_favorites {
            connection.execute(DELETE_NOT_FAV_UNCLEARED_ENCOUNTERS,[])?;
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

        let (mut encounter, version) = statement
            .query_row(params![id], map_encounter)?;

        let mut statement = connection.prepare_cached(SELECT_ENTITIES_BY_ENCOUNTER)?;

        let entities_query = statement
            .query_map(params![id], |row| map_entity(row, &version))?;

        let mut entities: HashMap<String, EncounterEntity> = HashMap::new();
        for entity in entities_query {
            let entity = entity?;
            entities.insert(entity.name.to_string(), entity);
        }

        let mut statement = connection.prepare_cached(SELECT_SYNC_LOGS)?;

        let sync: Option<String> = statement.query_row(params![id], |row| row.get(0)).optional()?;
        encounter.sync = sync;

        encounter.entities = entities;

        Ok(encounter)
    }

    pub fn get_last_encounter_id(&self) -> Result<Option<i32>> {
        
        let connection = self.0.get()?;
        let mut statement = connection.prepare_cached(GET_TOP_ENCOUNTER_ID)?;
    
        let id: Option<i32> = statement.query_row(params![], |row| row.get(0))
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

    pub fn insert_data(&self, args: InsertEncounterArgs) -> Result<i64> {
        let mut connection = self.0.get()?;
        let transaction = connection.transaction()?;

        let last_insert_id = self.insert_encounter(&transaction, &args)?;
        self.insert_entities(&transaction, &args, last_insert_id)?;
        self.insert_encounter_preview(&transaction, &args, last_insert_id)?;

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
            rdps_message: if *rdps_valid { None } else { Some("invalid_stats".into()) },
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
        encounter_id: i64,
    ) -> Result<()> {

        let InsertEncounterArgs {
            encounter,
            cast_log,
            damage_log,
            skill_cast_log,
            player_info,
            party_info,
            ..
        } = args;

        let fight_start = encounter.fight_start;
        let fight_end = encounter.last_combat_packet;

        let buffs = compute_support_buffs(encounter, party_info);

        let mut statement = transaction.prepare_cached(INSERT_ENTITY)?;

        for (_name, entity) in encounter.entities.iter() {
            if !should_insert_entity(entity, &encounter.local_player) {
                continue;
            }

            let mut entity = entity.clone();

            update_entity_stats(&mut entity, fight_start, fight_end, damage_log);

            if let Some(info) = player_info.as_ref().and_then(|stats| stats.get(&entity.name)) {
                apply_player_info(&mut entity, info);
            }

            apply_cast_logs(&mut entity, cast_log, skill_cast_log);

            let compressed_skills = compress_json(&entity.skills)?;
            let compressed_damage_stats = compress_json(&entity.damage_stats)?;

            let support = buffs.get(&entity.name);

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
                support.map(|b| b.buff),
                support.map(|b| b.brand),
                support.map(|b| b.identity),
                support.map(|b| b.hyper),
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
        args: &InsertEncounterArgs,
        encounter_id: i64,
    ) -> Result<()> {

        let InsertEncounterArgs {
            encounter,
            raid_clear,
            raid_difficulty,
            ..
        } = args;

        let mut players: Vec<_> = encounter.entities.values()
            .filter(|e| ((e.entity_type == EntityType::Player && e.class_id != 0 && e.max_hp > 0)
                || e.name == encounter.local_player)
                && e.damage_stats.damage_dealt > 0)
            .collect();

        let local_player_dps = players.iter()
            .find(|e| e.name == encounter.local_player)
            .map(|e| e.damage_stats.dps)
            .unwrap_or_default();

        players.sort_unstable_by_key(|e| Reverse(e.damage_stats.damage_dealt));

        let preview_players = players.iter()
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

        transaction.prepare_cached(INSERT_ENCOUNTER_PREVIEW)?.execute(params)?;

        Ok(())
    }

}

#[cfg(test)]
mod tests {

    use chrono::Utc;
    use hashbrown::HashSet;
    use rand::{rngs::ThreadRng, seq::IndexedRandom, Rng};
    use crate::database::Database;

    use super::*;

    #[test]
    fn should_insert_encounter() {
        
        let version = "1.14.0";

        let database = Database::memory(version).unwrap();

        let repository = database.create_repository();

        let player11 = PlayerSpec { class_id: 102, class_name: "Berserker".to_string(), is_support: false, crit_rate: 0.25, gear_score: 1620.0, hp: 1_000_000 };
        let player12 = PlayerSpec { class_id: 502, class_name: "Sharpshooter".to_string(), is_support: false, crit_rate: 0.28, gear_score: 1600.0, hp: 1_000_000 };
        let player13 = PlayerSpec { class_id: 302, class_name: "Wardancer".to_string(), is_support: false, crit_rate: 0.30, gear_score: 1580.0, hp: 1_000_000 };
        let player14 = PlayerSpec { class_id: 204, class_name: "Bard".to_string(), is_support: true, crit_rate: 0.15, gear_score: 1500.0, hp: 1_000_000 };

        let player21 = PlayerSpec { class_id: 603, class_name: "Aeromancer".to_string(), is_support: false, crit_rate: 0.25, gear_score: 1620.0, hp: 1_000_000 };
        let player22 = PlayerSpec { class_id: 504, class_name: "Artillerist".to_string(), is_support: false, crit_rate: 0.28, gear_score: 1600.0, hp: 1_000_000 };
        let player23 = PlayerSpec { class_id: 402, class_name: "Deathblade".to_string(), is_support: false, crit_rate: 0.30, gear_score: 1580.0, hp: 1_000_000 };
        let player24 = PlayerSpec { class_id: 105, class_name: "Paladin".to_string(), is_support: true, crit_rate: 0.15, gear_score: 1500.0, hp: 1_000_000 };

        let raid_builder = RaidBuilder::new()
            .add_party((player11, player12, player13, player14))
            .add_party((player21, player22, player23, player24))
            .set_boss("Mordum, the Abyssal Punisher", 485800, 1_100_000_000_000, 15)
            .set_region("EUC")
            .set_version(version)
            .set_damage_range(1_000_000, 2_000_000)
            .set_difficulty("Hard");

        let args = raid_builder.build();
        let mut expected_encounter = args.encounter.clone();

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
        };

        let paged = repository.get_encounter_preview(GetEncounterPreviewArgs {
            page: 0,
            page_size: 10,
            search: "".to_string(),
            filter,
        }).unwrap();

        expected_encounter.last_combat_packet = actual_encounter.last_combat_packet;
        expected_encounter.fight_start = actual_encounter.fight_start;

        assert_eq!(actual_encounter.current_boss_name, expected_encounter.current_boss_name);
        assert_eq!(actual_encounter.duration, expected_encounter.duration);
        assert_eq!(actual_encounter.difficulty, expected_encounter.difficulty);
        assert_eq!(actual_encounter.cleared, expected_encounter.cleared);
        assert_eq!(actual_encounter.boss_only_damage, expected_encounter.boss_only_damage);
        
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

            assert!(actual.damage_stats.unbuffed_damage > 0);
            assert!(actual.damage_stats.unbuffed_dps > 0);
            assert!(actual.damage_stats.damage_dealt > 0);
            assert!(actual.damage_stats.hyper_awakening_damage > 0);
            assert!(actual.damage_stats.dps > 0);
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
        assert_eq!(preview.difficulty, expected_encounter.difficulty);
        assert_eq!(preview.boss_name, expected_encounter.current_boss_name);
    }


    #[derive(Clone)]
    struct PlayerSpec {
        class_id: u32,
        class_name: String,
        is_support: bool,
        crit_rate: f64,
        gear_score: f32,
        hp: i64,
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
                duration_minutes: 15,
                region: "EUC".to_string(),
                version: "0.0.1".to_string(),
                difficulty: "Hard".to_string(),
                rng: rand::rng(),
                damage_range: (500, 1500),
                damage_taken_range: (500, 1000)
            }
        }

        fn add_party(mut self, players: (PlayerSpec, PlayerSpec, PlayerSpec, PlayerSpec)) -> Self {
            self.parties.push(vec![players.0, players.1, players.2, players.3]);
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

            let (mut entities, player_names) = generate_entities_for_parties(
                &self.parties,
                &mut self.rng,
                per_player_total_damage);

            let mut boss = self.generate_boss_entity();

            boss.damage_stats.damage_dealt += self.rng.random_range(self.damage_range.0..self.damage_range.1);

            entities.insert(boss.name.clone(), boss.clone());

            let mut boss_hp_logs: HashMap<String, Vec<BossHpLog>> = HashMap::new();
            let mut boss_hp_log = Vec::with_capacity(duration_s as usize + 1);
            for t in 0..=duration_s as i32 {
                let dealt = raid_dps * t as i64;
                let hp = (self.boss_hp - dealt).max(0);
                let percent = hp as f32 / self.boss_hp as f32;
                boss_hp_log.push(BossHpLog::new(t, hp, percent));
            }
            boss_hp_logs.insert(boss.name.clone(), boss_hp_log);

            let party_vec: Vec<Vec<String>> = player_names.chunks(4)
                .map(|chunk| chunk.to_vec())
                .collect();


            let mut party_info: HashMap<i32, Vec<String>> = HashMap::new();
            for (idx, party) in player_names.chunks(4).enumerate() {
                party_info.insert(idx as i32 + 1, party.to_vec());
            }        

            let mut encounter_entities_with_stats = HashMap::new();
            for (name, mut entity) in entities.into_iter() {
                if entity.entity_type == EntityType::Player {
                    update_skill_and_damage_stats(self.damage_range, &mut self.rng, &mut entity);
                    update_damage_taken(self.damage_taken_range, &mut self.rng, &mut entity);
                    update_buffs_heals_and_absorb(&mut self.rng, &mut entity);
                }
                encounter_entities_with_stats.insert(name, entity);
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
                boss_hp_log: boss_hp_logs,
            };

            let encounter = Encounter {
                last_combat_packet,
                fight_start,
                local_player: player_names[0].clone(),
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
                damage_log: HashMap::new(),
                cast_log: HashMap::new(),
                boss_hp_log: HashMap::new(),
                raid_clear: true,
                party_info: party_vec,
                raid_difficulty: self.difficulty.clone(),
                region: Some(self.region.clone()),
                player_info: None,
                meter_version: self.version.clone(),
                ntp_fight_start: fight_start,
                rdps_valid: true,
                manual: false,
                skill_cast_log: HashMap::new(),
                skill_cooldowns: HashMap::new(),
            };

            insert_args
        }

        fn generate_boss_entity(&self) -> EncounterEntity {
            EncounterEntity {
                id: 1000,
                character_id: 0,
                npc_id: self.boss_npc_id,
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

    fn update_skill_and_damage_stats(damage_range: (i64, i64), rng: &mut ThreadRng, entity: &mut EncounterEntity) {
        entity.skill_stats = SkillStats::default();
        entity.damage_stats = DamageStats::default();

        for skill in entity.skills.values_mut() {
            skill.casts = 0;
            skill.hits = 0;
            skill.crits = 0;

            for _ in 0..100 {
                let dmg = rng.random_range(damage_range.0..=damage_range.1);
                let is_crit = rng.random_bool(entity.skill_stats.crits as f64 / 100.0);
                skill.casts += 1;
                skill.hits += 1;
                if is_crit {
                    skill.crits += 1;
                }
                entity.damage_stats.damage_dealt += dmg;
            }
        }

        entity.damage_stats.hyper_awakening_damage += rng.random_range(1_000_000_000..2_000_000_000);
        entity.skill_stats.casts = entity.skills.values().map(|s| s.casts).sum();
        entity.skill_stats.hits = entity.skills.values().map(|s| s.hits).sum();
        entity.skill_stats.crits = entity.skills.values().map(|s| s.crits).sum();
    }

    fn update_damage_taken(damage_taken: (i64, i64), rng: &mut ThreadRng, entity: &mut EncounterEntity) {
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
        entity.damage_stats.damage_absorbed_by.insert(1, absorb_value);

        let shield_value = rng.random_range(5_000..=50_000);
        entity.damage_stats.shields_given += shield_value;
        entity.damage_stats.shields_given_by.insert(1, shield_value);
        entity.damage_stats.shields_received += shield_value / 2;
        entity.damage_stats.shields_received_by.insert(1, shield_value / 2);
        entity.damage_stats.damage_absorbed_on_others += shield_value / 3;
        entity.damage_stats.damage_absorbed_on_others_by.insert(1, shield_value / 3);
    }

    fn generate_entities_for_parties(
        parties: &[Vec<PlayerSpec>],
        rng: &mut ThreadRng,
        per_player_total_damage: i64,
    ) -> (HashMap<String, EncounterEntity>, Vec<String>) {
        let mut entities = HashMap::new();
        let mut player_names = Vec::new();

        for party in parties {
            for (idx, spec) in party.iter().enumerate() {
                let name = format!("Player{}", player_names.len() + 1);
                let skills: HashMap<u32, Skill> = (1..=10)
                    .map(|skill_id| {(
                    skill_id,
                    Skill {
                            id: skill_id,
                            name: format!("Skill{}", skill_id),
                            icon: String::new(),
                            casts: 0,
                            hits: 0,
                            crits: 0,
                            ..Default::default()
                        },
                    )
                    })
                .collect();

                let mut entity = EncounterEntity {
                    id: idx as u64 + 1,
                    character_id: idx as u64 + 101,
                    npc_id: 0,
                    name: name.clone(),
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
                    engraving_data: None,
                    ark_passive_active: Some(!spec.is_support),
                    ark_passive_data: None,
                    spec: Some(spec.class_name.clone()),
                    loadout_hash: None,
                    combat_power: Some(2000.0),
                };

                for skill in entity.skills.values_mut() {
                    for _ in 0..100 {
                        if entity.damage_stats.damage_dealt >= per_player_total_damage {
                            break;
                        }

                        let damage = rng.random_range(500..=1500);
                        let is_crit = rng.random_bool(spec.crit_rate);
                        skill.casts += 1;
                        skill.hits += 1;
                        skill.rdps_received.insert(1, HashMap::from([(1, damage / 3)]));
                        
                        if is_crit {
                            skill.crits += 1;
                        }

                        entity.damage_stats.damage_dealt += damage;
                    }
                }

                entity.skill_stats.casts = entity.skills.values().map(|s| s.casts).sum();
                entity.skill_stats.hits = entity.skills.values().map(|s| s.hits).sum();
                entity.skill_stats.crits = entity.skills.values().map(|s| s.crits).sum();

                entities.insert(name.clone(), entity);
                player_names.push(name);
            }
        }

        (entities, player_names)
    }
}