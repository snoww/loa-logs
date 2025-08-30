use std::cmp::{max, Reverse};
use anyhow::{Ok, Result};
use hashbrown::HashMap;
use log::*;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, params_from_iter, OptionalExtension};
use serde_json::json;

use crate::{constants::{WINDOW_MS, WINDOW_S}, database::{models::*, queries::*, utils::*}, live::utils::*, live::{data::GEM_SKILL_MAP, models::*}};
pub struct Repository(r2d2::Pool<SqliteConnectionManager>);

impl Repository {
    pub fn new(connection: r2d2::Pool<SqliteConnectionManager>) -> Self {
        Self(connection)
    }

    pub fn optimize(&self) -> Result<()> {
        let connection = self.0.get()?;
        connection.execute_batch(
            "
            INSERT INTO encounter_search(encounter_search)
            VALUES('optimize');
            VACUUM;
            ",
        )?;

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
        .query_row("SELECT COUNT(*) FROM encounter_preview", [], |row| {
            row.get(0)
        })?;

        let params = params![min_duration * 1000];
        let encounter_filtered_count = connection
            .query_row(SELECT_ENCOUNTER_PREVIEW_BY_GE_DURATION, params, |row| row.get(0))?;

        Ok((encounter_count, encounter_filtered_count))
    }

    pub fn delete_encounters(&self, ids: Vec<i32>) -> Result<()> {
        let connection = self.0.get()?;

        connection.execute("PRAGMA foreign_keys = ON;", params![])?;

        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let placeholders_str = placeholders.join(",");

        let sql = format!("DELETE FROM encounter WHERE id IN ({})", placeholders_str);
        let mut statement = connection.prepare_cached(&sql)?;

        info!("deleting encounters: {:?}", ids);

        let params = params_from_iter(ids);
        statement.execute(params)?;

        Ok(())
    }

    pub fn delete_encounter(&self, id: String) -> Result<()> {

        let connection = self.0.get()?;

        connection.execute("PRAGMA foreign_keys = ON;", params![])?;

        let mut statement = connection.prepare_cached(DELETE_ENCOUNTER_BY_ID)?;

        info!("deleting encounter: {}", id);

        statement.execute(params![id])?;

        Ok(())
    }

    pub fn delete_encounters_below_min_duration(
        &self,
        min_duration: i64,
        keep_favorites: bool,) -> Result<(())> {

        let connection = self.0.get()?;
        if keep_favorites {
            connection.execute(
                "DELETE FROM encounter
                WHERE id IN (
                    SELECT id
                    FROM encounter_preview
                    WHERE duration < ? AND favorite = 0
                )",
                params![min_duration * 1000],
            )?;
        } else {
            connection.execute(
                "DELETE FROM encounter
                WHERE id IN (
                    SELECT id
                    FROM encounter_preview
                    WHERE duration < ?
                )",
                params![min_duration * 1000],
            )?;
        }

        connection.execute("VACUUM", params![])?;

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

        let raids_only_filter = if filter.raids_only {
            "AND difficulty IS NOT NULL and difficulty != ''"
        } else {
            ""
        };

        let sort = format!("e.{}", filter.sort);

        let count_params = params.clone();

        let query = format!(
            "SELECT
        e.id,           -- 0
        e.fight_start,  -- 1
        e.current_boss, -- 2
        e.duration,     -- 3
        e.difficulty,   -- 4
        e.favorite,     -- 5
        e.cleared,      -- 6
        e.local_player, -- 7
        e.my_dps,       -- 8
        e.players,      -- 9
        le.spec,            -- 10
        le.support_ap,      -- 11
        le.support_brand,   -- 12
        le.support_identity,-- 13
        le.support_hyper    -- 14
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
            sort,
            filter.order
        );

        let mut statement = connection.prepare_cached(&query)?;

        let offset = (page - 1) * page_size;

        params.push(page_size.to_string());
        params.push(offset.to_string());

        let params= params_from_iter(params);    
        let encounter_iter = statement.query_map(params, map_encounter_preview)?;
        
        let encounters: Vec<EncounterPreview> = encounter_iter.collect::<Result<_, _>>().unwrap();

        let query = format!(
            "SELECT COUNT(*)
            FROM encounter_preview e {join_clause}
            WHERE duration > ? {boss_filter}
            {raid_clear_filter} {favorite_filter} {difficulty_filter} {boss_only_damage_filter}"
        );

        let count: i32 = connection
            .query_row_and_then(&query, params_from_iter(count_params), |row| row.get(0))?;

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

        connection.execute("VACUUM", params![])?;

        Ok(())
    }

    pub fn delete_all_encounters(&self, keep_favorites: bool) -> Result<()> {

        let connection = self.0.get()?;

        if keep_favorites {
            connection.execute(DELETE_UNFAVOURITE_ENCOUNTERS, [])?;
        } else {
            connection.execute(DELETE_ENCOUNTERS, [])?;
        }
        
        connection.execute("VACUUM", [])?;

        Ok(())
    }

    pub fn get_encounter(&self, id: String) -> Result<Encounter> {

        let connection = self.0.get()?;
        let mut statement = connection.prepare_cached(SELECT_FROM_ENCOUNTER_JOIN_PREVIEW)?;

        let (mut encounter, is_compressed) = statement
            .query_row(params![id], map_encounter)
            .unwrap_or_else(|_| (Encounter::default(), false));

        let mut statement = connection.prepare_cached(SELECT_ENTITIES_BY_ENCOUNTER)?;

        let entity_iter = statement
            .query_map(params![id], |row| map_entity(row, is_compressed))?;

        let mut entities: HashMap<String, EncounterEntity> = HashMap::new();
        for entity in entity_iter.flatten() {
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

        let query = if force_resync { "= '0'" } else { "IS NULL" };
        let query = &format!("
            SELECT id
            FROM encounter_preview
            LEFT JOIN sync_logs ON encounter_id = id
            WHERE cleared = true AND boss_only_damage = 1 AND upstream_id {}
            ORDER BY fight_start;
            ",
            query
        );
        let mut statement = connection.prepare_cached(query)?;
        let rows = statement.query_map([], |row| row.get(0))?;

        let mut ids = Vec::new();
        
        for id_result in rows {
            ids.push(id_result.unwrap_or(0));
        }

        Ok(ids)
    }

    pub fn insert_data(&self, args: InsertEncounterArgs) -> Result<i64> {

        let InsertEncounterArgs {
            mut encounter,
            boss_hp_log,
            cast_log,
            damage_log,
            skill_cast_log,
            skill_cooldowns,
            manual,
            meter_version,
            ntp_fight_start,
            party_info,
            player_info,
            raid_clear,
            raid_difficulty,
            rdps_valid,
            region 
        } = args;

        let mut connection = self.0.get()?;
        let transaction = connection.transaction()?;
        let mut statement = transaction.prepare_cached(INSERT_ENCOUNTER)?;

        encounter.duration = encounter.last_combat_packet - encounter.fight_start;
        let duration_seconds = max(encounter.duration / 1000, 1);
        encounter.encounter_damage_stats.dps =
            encounter.encounter_damage_stats.total_damage_dealt / duration_seconds;

        let misc: EncounterMisc = EncounterMisc {
            raid_clear: if raid_clear { Some(true) } else { None },
            party_info: if party_info.is_empty() {
                None
            } else {
                Some(
                    party_info
                        .iter()
                        .enumerate()
                        .map(|(index, party)| (index as i32, party.clone()))
                        .collect(),
                )
            },
            region,
            version: Some(meter_version),
            rdps_valid: Some(rdps_valid),
            rdps_message: if rdps_valid {
                None
            } else {
                Some("invalid_stats".to_string())
            },
            ntp_fight_start: Some(ntp_fight_start),
            manual_save: Some(manual),
            ..Default::default()
        };

        let compressed_boss_hp = compress_json(&boss_hp_log);
        let compressed_buffs = compress_json(&encounter.encounter_damage_stats.buffs);
        let compressed_debuffs = compress_json(&encounter.encounter_damage_stats.debuffs);
        let compressed_shields = compress_json(&encounter.encounter_damage_stats.applied_shield_buffs);

        let params = params![
            encounter.last_combat_packet,
            encounter.encounter_damage_stats.total_damage_dealt,
            encounter.encounter_damage_stats.top_damage_dealt,
            encounter.encounter_damage_stats.total_damage_taken,
            encounter.encounter_damage_stats.top_damage_taken,
            encounter.encounter_damage_stats.dps,
            compressed_buffs,
            compressed_debuffs,
            encounter.encounter_damage_stats.total_shielding,
            encounter.encounter_damage_stats.total_effective_shielding,
            compressed_shields,
            json!(misc),
            DB_VERSION,
            compressed_boss_hp,
        ];
        statement.execute(params)?;

        let last_insert_id = transaction.last_insert_rowid();

        let mut statement = transaction.prepare_cached(INSERT_ENTITY)?;

        let fight_start = encounter.fight_start;
        let fight_end = encounter.last_combat_packet;

        // get average support buffs for supports
        let mut buffs = HashMap::new();
        for party in party_info.iter() {
            let party_members: Vec<_> = encounter
                .entities
                .iter()
                .filter(|(name, _)| party.contains(name))
                .map(|(name, entity)| entity)
                .collect();

            // specs are not determined for dps classes, but should be set for supports
            let party_without_support: Vec<_> = party_members
                .iter()
                .filter(|entity| !is_support(entity))
                .collect();

            if party_members.len() - party_without_support.len() == 1 {
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

                    let brand_ratio = player.damage_stats.debuffed_by_support as f64 / damage_dealt;
                    let buff_ratio = player.damage_stats.buffed_by_support as f64 / damage_dealt;
                    let identity_ratio = player.damage_stats.buffed_by_identity as f64 / damage_dealt;

                    average_brand += brand_ratio * party_damage_percent;
                    average_buff += buff_ratio * party_damage_percent;
                    average_identity += identity_ratio * party_damage_percent;
                    average_hyper += (player.damage_stats.buffed_by_hat as f64
                        / player.damage_stats.damage_dealt as f64)
                        * party_damage_percent;
                }

                if let Some(support) = party_members.iter().find(|entity| is_support(entity)) {
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
        }

        for (_key, entity) in encounter.entities.iter_mut().filter(|(_, e)| {
            ((e.entity_type == EntityType::PLAYER && e.class_id > 0)
                || e.name == encounter.local_player
                || e.entity_type == EntityType::ESTHER
                || (e.entity_type == EntityType::BOSS && e.max_hp > 0))
                && e.damage_stats.damage_dealt > 0
        }) {
            if entity.entity_type == EntityType::PLAYER {
                let intervals = generate_intervals(fight_start, fight_end);
                if let Some(damage_log) = damage_log.get(&entity.name) {
                    if !intervals.is_empty() {
                        for interval in intervals {
                            let start = fight_start + interval - WINDOW_MS;
                            let end = fight_start + interval + WINDOW_MS;

                            let damage = sum_in_range(damage_log, start, end);
                            entity
                                .damage_stats
                                .dps_rolling_10s_avg
                                .push(damage / (WINDOW_S * 2));
                        }
                    }
                    let fight_start_sec = encounter.fight_start / 1000;
                    let fight_end_sec = encounter.last_combat_packet / 1000;
                    entity.damage_stats.dps_average =
                        calculate_average_dps(damage_log, fight_start_sec, fight_end_sec);
                }

                let spec = get_player_spec(entity, &encounter.encounter_damage_stats.buffs, false);
                entity.spec = Some(spec.clone());

                if let Some(info) = player_info
                    .as_ref()
                    .and_then(|stats| stats.get(&entity.name))
                {
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
                                        // cooldown gems
                                        skill.gem_cooldown =
                                            Some(cooldown_gem_value_to_level(gem.value, gem.tier));
                                        skill.gem_tier = Some(gem.tier);
                                    }
                                    64 | 65 => {
                                        // support effect damage gems
                                        skill.gem_damage =
                                            Some(support_damage_gem_value_to_level(gem.value));
                                        skill.gem_tier_dmg = Some(gem.tier);
                                    }
                                    _ => {
                                        // damage gems
                                        skill.gem_damage =
                                            Some(damage_gem_value_to_level(gem.value, gem.tier));
                                        skill.gem_tier_dmg = Some(gem.tier);
                                    }
                                }
                            }
                        }
                    }

                    entity.ark_passive_active = Some(info.ark_passive_enabled);

                    let engravings = get_engravings(&info.engravings);
                    if entity.class_id == 104
                        && engravings.as_ref().is_some_and(|engravings| {
                            engravings
                                .iter()
                                .any(|e| e == "Awakening" || e == "Drops of Ether")
                        })
                    {
                        entity.spec = Some("Princess".to_string());
                    } else if spec == "Unknown" {
                        // not reliable enough to be used on its own
                        if let Some(tree) = info.ark_passive_data.as_ref() {
                            if let Some(enlightenment) = tree.enlightenment.as_ref() {
                                for node in enlightenment.iter() {
                                    let spec = get_spec_from_ark_passive(node);
                                    if spec != "Unknown" {
                                        entity.spec = Some(spec);
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if entity.combat_power.is_none() {
                        entity.combat_power = info.combat_power.as_ref().map(|c| c.score);
                    }

                    entity.engraving_data = engravings;
                    entity.ark_passive_data = info.ark_passive_data.clone();
                    entity.loadout_hash = info.loadout_snapshot.clone();
                }
            }

            if entity.name == encounter.local_player {
                for (skill_id, events) in skill_cooldowns.iter() {
                    if let Some(skill) = entity.skills.get_mut(skill_id) {
                        skill.time_available =
                            Some(get_total_available_time(events, fight_start, fight_end));
                    }
                }
            }

            entity.damage_stats.dps = entity.damage_stats.damage_dealt / duration_seconds;

            for (_, skill) in entity.skills.iter_mut() {
                skill.dps = skill.total_damage / duration_seconds;
            }

            for (_, cast_log) in cast_log.iter().filter(|&(s, _)| *s == entity.name) {
                for (skill, log) in cast_log {
                    entity.skills.entry(*skill).and_modify(|e| {
                        e.cast_log.clone_from(log);
                    });
                }
            }

            for (_, skill_cast_log) in skill_cast_log.iter().filter(|&(s, _)| *s == entity.id) {
                for (skill, log) in skill_cast_log {
                    entity.skills.entry(*skill).and_modify(|e| {
                        let average_cast = e.total_damage as f64 / e.casts as f64;
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
                            e.adjusted_crit = Some(adj_crits as f64 / adj_hits as f64);
                        }

                        e.max_damage_cast = log
                            .values()
                            .map(|cast| cast.hits.iter().map(|hit| hit.damage).sum::<i64>())
                            .max()
                            .unwrap_or_default();
                        e.skill_cast_log = log.values().cloned().collect();
                    });
                }
            }

            let compressed_skills = compress_json(&entity.skills);
            let compressed_damage_stats = compress_json(&entity.damage_stats);

            let damage_dealt = entity.damage_stats.damage_dealt;
            let damage_without_hyper =
                (damage_dealt - entity.damage_stats.hyper_awakening_damage) as f64;
            let support_buffs = buffs.get(&entity.name);

            let params = params![
                entity.name,
                last_insert_id,
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
            ];


            statement.execute(params)?;
        }

        let mut players = encounter
            .entities
            .values()
            .filter(|e| {
                ((e.entity_type == EntityType::PLAYER && e.class_id != 0 && e.max_hp > 0)
                    || e.name == encounter.local_player)
                    && e.damage_stats.damage_dealt > 0
            })
            .collect::<Vec<_>>();

        let local_player_dps = players
            .iter()
            .find(|e| e.name == encounter.local_player)
            .map(|e| e.damage_stats.dps)
            .unwrap_or_default();

        players.sort_unstable_by_key(|e| Reverse(e.damage_stats.damage_dealt));

        let preview_players = players
            .into_iter()
            .map(|e| format!("{}:{}", e.class_id, e.name))
            .collect::<Vec<_>>()
            .join(",");

        let params = params![
            last_insert_id,
            encounter.fight_start,
            encounter.current_boss_name,
            encounter.duration,
            preview_players,
            raid_difficulty,
            encounter.local_player,
            local_player_dps,
            raid_clear,
            encounter.boss_only_damage
        ];
        
        let mut statement = transaction.prepare_cached(INSERT_ENCOUNTER_PREVIEW)?;
        statement.execute(params)?;

        Ok(last_insert_id)
    }

}