use chrono::Utc;
use hashbrown::HashMap;
use log::{info, warn};
use meter_core::packets::common::SkillMoveOptionData;
use meter_core::packets::definitions::PKTIdentityGaugeChangeNotify;
use moka::sync::Cache;
use rsntp::SntpClient;
use rusqlite::Connection;
use std::cmp::max;
use std::default::Default;

use crate::live::entity_tracker::{Entity, EntityTracker};
use crate::live::skill_tracker::SkillTracker;
use crate::live::stats_api::{PlayerStats, StatsApi};
use crate::live::status_tracker::StatusEffectDetails;
use crate::live::utils::*;
use crate::parser::models::*;
use tauri::{AppHandle, Manager, Window, Wry};
use tokio::task;

#[derive(Debug)]
pub struct EncounterState {
    pub app: AppHandle,
    pub encounter: Encounter,
    pub resetting: bool,
    pub boss_dead_update: bool,
    pub saved: bool,

    pub raid_clear: bool,

    damage_log: HashMap<String, Vec<(i64, i64)>>,
    cast_log: HashMap<String, HashMap<u32, Vec<i32>>>,

    boss_hp_log: HashMap<String, Vec<BossHpLog>>,

    pub party_info: Vec<Vec<String>>,
    pub raid_difficulty: String,
    pub raid_difficulty_id: u32,
    pub boss_only_damage: bool,
    pub region: Option<String>,

    sntp_client: SntpClient,
    ntp_fight_start: i64,

    pub rdps_valid: bool,

    pub skill_tracker: SkillTracker,

    custom_id_map: HashMap<u32, u32>,

    pub damage_is_valid: bool,
}

impl EncounterState {
    pub fn new(window: AppHandle) -> EncounterState {
        EncounterState {
            app: window,
            encounter: Encounter::default(),
            resetting: false,
            raid_clear: false,
            boss_dead_update: false,
            saved: false,

            damage_log: HashMap::new(),
            boss_hp_log: HashMap::new(),
            cast_log: HashMap::new(),

            party_info: Vec::new(),
            raid_difficulty: "".to_string(),
            raid_difficulty_id: 0,
            boss_only_damage: false,
            region: None,

            sntp_client: SntpClient::new(),
            ntp_fight_start: 0,

            // todo
            rdps_valid: false,

            skill_tracker: SkillTracker::new(),

            custom_id_map: HashMap::new(),

            damage_is_valid: true,
        }
    }

    // keep all player entities, reset all stats
    pub fn soft_reset(&mut self, keep_bosses: bool) {
        let clone = self.encounter.clone();

        self.encounter.fight_start = 0;
        self.encounter.boss_only_damage = self.boss_only_damage;
        self.encounter.entities = HashMap::new();
        self.encounter.current_boss_name = "".to_string();
        self.encounter.encounter_damage_stats = Default::default();
        self.raid_clear = false;

        self.damage_log = HashMap::new();
        self.cast_log = HashMap::new();
        self.boss_hp_log = HashMap::new();
        self.party_info = Vec::new();

        self.ntp_fight_start = 0;

        self.rdps_valid = false;

        self.skill_tracker = SkillTracker::new();

        self.custom_id_map = HashMap::new();

        for (key, entity) in clone.entities.into_iter().filter(|(_, e)| {
            e.entity_type == EntityType::PLAYER
                || (keep_bosses && e.entity_type == EntityType::BOSS)
        }) {
            self.encounter.entities.insert(
                key,
                EncounterEntity {
                    name: entity.name,
                    id: entity.id,
                    character_id: entity.character_id,
                    npc_id: entity.npc_id,
                    class: entity.class,
                    class_id: entity.class_id,
                    entity_type: entity.entity_type,
                    gear_score: entity.gear_score,
                    max_hp: entity.max_hp,
                    current_hp: entity.current_hp,
                    is_dead: entity.is_dead,
                    ..Default::default()
                },
            );
        }
    }

    // update local player as we get more info
    pub fn update_local_player(&mut self, entity: &Entity) {
        // we replace the existing local player if it exists, since its name might have changed (from hex or "You" to character name)
        if let Some(mut local) = self.encounter.entities.remove(&self.encounter.local_player) {
            // update local player name, insert back into encounter
            self.encounter.local_player.clone_from(&entity.name);
            update_player_entity(&mut local, entity);
            self.encounter
                .entities
                .insert(self.encounter.local_player.clone(), local);
        } else {
            // cannot find old local player by name, so we look by local player's entity id
            // this can happen when the user started meter late
            let old_local = self
                .encounter
                .entities
                .iter()
                .find(|(_, e)| e.id == entity.id)
                .map(|(key, _)| key.clone());

            // if we find the old local player, we update its name and insert back into encounter
            if let Some(old_local) = old_local {
                let mut new_local = self.encounter.entities[&old_local].clone();
                update_player_entity(&mut new_local, entity);
                self.encounter.entities.remove(&old_local);
                self.encounter.local_player.clone_from(&entity.name);
                self.encounter
                    .entities
                    .insert(self.encounter.local_player.clone(), new_local);
            }
        }
    }

    pub fn on_init_env(&mut self, entity: Entity, stats_api: &StatsApi) {
        // if not already saved to db, we save again
        if !self.saved && !self.encounter.current_boss_name.is_empty() {
            self.save_to_db(stats_api, false);
        }

        // replace or insert local player
        if let Some(mut local_player) = self.encounter.entities.remove(&self.encounter.local_player)
        {
            update_player_entity(&mut local_player, &entity);
            self.encounter
                .entities
                .insert(entity.name.clone(), local_player);
        } else {
            let entity = encounter_entity_from_entity(&entity);
            self.encounter.entities.insert(entity.name.clone(), entity);
        }
        self.encounter.local_player = entity.name;

        // remove unrelated entities
        self.encounter.entities.retain(|_, e| {
            e.name == self.encounter.local_player || e.damage_stats.damage_dealt > 0
        });

        self.app
            .emit_all("zone-change", "")
            .expect("failed to emit zone-change");

        self.soft_reset(false);
    }

    pub fn on_phase_transition(&mut self, phase_code: i32, stats_api: &mut StatsApi) {
        self.app
            .emit_all("phase-transition", phase_code)
            .expect("failed to emit phase-transition");

        match phase_code {
            0 | 2 | 3 | 4 => {
                if !self.encounter.current_boss_name.is_empty() {
                    if phase_code == 0 {
                        stats_api.valid_zone = false;
                    }
                    self.save_to_db(stats_api, false);
                    self.saved = true;
                }
                self.resetting = true;
            }
            _ => (),
        }
    }

    // replace local player
    pub fn on_init_pc(&mut self, entity: Entity, hp: i64, max_hp: i64) {
        self.encounter.entities.remove(&self.encounter.local_player);
        self.encounter.local_player.clone_from(&entity.name);
        let mut player = encounter_entity_from_entity(&entity);
        player.current_hp = hp;
        player.max_hp = max_hp;
        self.encounter.entities.insert(player.name.clone(), player);
    }

    // add or update player to encounter
    pub fn on_new_pc(&mut self, entity: Entity, hp: i64, max_hp: i64) {
        self.encounter
            .entities
            .entry(entity.name.clone())
            .and_modify(|player| {
                player.id = entity.id;
                player.gear_score = entity.gear_level;
                player.current_hp = hp;
                player.max_hp = max_hp;
                if entity.character_id > 0 {
                    player.character_id = entity.character_id;
                }
            })
            .or_insert_with(|| {
                let mut player = encounter_entity_from_entity(&entity);
                player.current_hp = hp;
                player.max_hp = max_hp;
                player
            });
    }

    // add or update npc to encounter
    // we set current boss if npc matches criteria
    pub fn on_new_npc(&mut self, entity: Entity, hp: i64, max_hp: i64) {
        let entity_name = entity.name.clone();
        self.encounter
            .entities
            .entry(entity_name.clone())
            .and_modify(|e| {
                if entity.entity_type != EntityType::BOSS && e.entity_type != EntityType::BOSS {
                    e.npc_id = entity.npc_id;
                    e.id = entity.id;
                    e.current_hp = hp;
                    e.max_hp = max_hp;
                } else if entity.entity_type == EntityType::BOSS && e.entity_type == EntityType::NPC
                {
                    e.entity_type = EntityType::BOSS;
                    e.npc_id = entity.npc_id;
                    e.id = entity.id;
                    e.current_hp = hp;
                    e.max_hp = max_hp;
                }
            })
            .or_insert_with(|| {
                let mut npc = encounter_entity_from_entity(&entity);
                npc.current_hp = hp;
                npc.max_hp = max_hp;
                npc
            });

        if let Some(npc) = self.encounter.entities.get(&entity_name) {
            if npc.entity_type == EntityType::BOSS {
                // if current encounter has no boss, we set the boss
                // if current encounter has a boss, we check if new boss has more max hp, or if current boss is dead
                self.encounter.current_boss_name = if self
                    .encounter
                    .entities
                    .get(&self.encounter.current_boss_name)
                    .is_none_or(|boss| npc.max_hp >= boss.max_hp || boss.is_dead)
                {
                    entity_name
                } else {
                    self.encounter.current_boss_name.clone()
                };
            }
        }
    }

    pub fn on_death(&mut self, dead_entity: &Entity) {
        let entity = self
            .encounter
            .entities
            .entry(dead_entity.name.clone())
            .or_insert_with(|| encounter_entity_from_entity(dead_entity));

        if (dead_entity.entity_type != EntityType::PLAYER
            && dead_entity.entity_type != EntityType::BOSS)
            || entity.id != dead_entity.id
            || (entity.entity_type == EntityType::BOSS && entity.npc_id != dead_entity.npc_id)
        {
            return;
        }

        if entity.entity_type == EntityType::BOSS
            && dead_entity.entity_type == EntityType::BOSS
            && entity.name == self.encounter.current_boss_name
            && !entity.is_dead
        {
            self.boss_dead_update = true;
        }

        entity.current_hp = 0;
        entity.is_dead = true;
        entity.damage_stats.deaths += 1;
        entity.damage_stats.death_time = Utc::now().timestamp_millis();
        entity
            .damage_stats
            .incapacitations
            .iter_mut()
            .rev()
            .take_while(|x| x.timestamp + x.duration > entity.damage_stats.death_time)
            .for_each(|x| {
                // cap duration to death time if it exceeds it
                x.duration = x.timestamp - entity.damage_stats.death_time;
            });
    }

    pub fn on_skill_start(
        &mut self,
        source_entity: &Entity,
        skill_id: u32,
        tripod_index: Option<TripodIndex>,
        tripod_level: Option<TripodLevel>,
        timestamp: i64,
    ) -> (u32, Option<Vec<u32>>) {
        // do not track skills if encounter not started
        if self.encounter.fight_start == 0 {
            return (0, None);
        }
        let skill_name = get_skill_name(&skill_id);
        let mut tripod_change = false;
        let entity = self
            .encounter
            .entities
            .entry(source_entity.name.clone())
            .or_insert_with(|| {
                let (skill_name, skill_icon, summons) = get_skill_name_and_icon(
                    &skill_id,
                    &0,
                    skill_name.clone(),
                    &self.skill_tracker,
                    source_entity.id,
                );
                let mut entity = encounter_entity_from_entity(source_entity);
                entity.skill_stats = SkillStats {
                    casts: 0,
                    ..Default::default()
                };
                entity.skills = HashMap::from([(
                    skill_id,
                    Skill {
                        id: skill_id,
                        name: {
                            if skill_name.is_empty() {
                                skill_id.to_string()
                            } else {
                                skill_name
                            }
                        },
                        icon: skill_icon,
                        tripod_index,
                        tripod_level,
                        summon_sources: summons,
                        casts: 0,
                        ..Default::default()
                    },
                )]);
                tripod_change = true;
                entity
            });

        if entity.class_id == 0
            && source_entity.entity_type == EntityType::PLAYER
            && source_entity.class_id > 0
        {
            entity.class_id = source_entity.class_id;
            entity.class = get_class_from_id(&source_entity.class_id);
        }

        entity.is_dead = false;
        entity.skill_stats.casts += 1;

        let relative_timestamp = if self.encounter.fight_start == 0 {
            0
        } else {
            (timestamp - self.encounter.fight_start) as i32
        };

        // if skills have different ids but the same name, we group them together
        // dunno if this is right approach xd
        let mut skill_id = skill_id;
        let mut skill_summon_sources: Option<Vec<u32>> = None;
        if let Some(skill) = entity.skills.get_mut(&skill_id) {
            skill.casts += 1;
            tripod_change = check_tripod_index_change(skill.tripod_index, tripod_index)
                || check_tripod_level_change(skill.tripod_level, tripod_level);
            skill.tripod_index = tripod_index;
            skill.tripod_level = tripod_level;
            skill_summon_sources.clone_from(&skill.summon_sources);
        } else if let Some(skill) = entity
            .skills
            .values_mut()
            .find(|s| s.name == skill_name.clone())
        {
            skill.casts += 1;
            skill_id = skill.id;
            tripod_change = check_tripod_index_change(skill.tripod_index, tripod_index)
                || check_tripod_level_change(skill.tripod_level, tripod_level);
            skill.tripod_index = tripod_index;
            skill.tripod_level = tripod_level;
            skill_summon_sources.clone_from(&skill.summon_sources);
        } else {
            let (skill_name, skill_icon, summons) = get_skill_name_and_icon(
                &skill_id,
                &0,
                skill_name.clone(),
                &self.skill_tracker,
                source_entity.id,
            );
            skill_summon_sources.clone_from(&summons);
            entity.skills.insert(
                skill_id,
                Skill {
                    id: skill_id,
                    name: {
                        if skill_name.is_empty() {
                            skill_id.to_string()
                        } else {
                            skill_name
                        }
                    },
                    icon: skill_icon,
                    tripod_index,
                    tripod_level,
                    summon_sources: summons,
                    casts: 1,
                    ..Default::default()
                },
            );
            tripod_change = true;
        }
        if tripod_change {
            if let (Some(tripod_index), Some(_tripod_level)) = (tripod_index, tripod_level) {
                let mut indexes = vec![tripod_index.first];
                if tripod_index.second != 0 {
                    indexes.push(tripod_index.second + 3);
                }
                // third row should never be set if second is not set
                if tripod_index.third != 0 {
                    indexes.push(tripod_index.third + 6);
                }
                // let levels = [tripod_level.first, tripod_level.second, tripod_level.third];
                // if let Some(effect) = SKILL_FEATURE_DATA.get(&skill_id) {
                //     for i in 0..indexes.len() {
                //         if let Some(entries) = effect.tripods.get(&indexes[i]) {
                //             let mut options: Vec<SkillFeatureOption> = vec![];
                //             for entry in &entries.entries {
                //                 if entry.level > 0 && entry.level == levels[i] {
                //                     options.push(entry.clone());
                //                 }
                //             }
                //             tripod_data.push(TripodData {
                //                 index: indexes[i],
                //                 options,
                //             });
                //         }
                //     }
                // }
            }

            // if !tripod_data.is_empty() {
            //     entity.skills.entry(skill_id).and_modify(|e| {
            //         e.tripod_data = Some(tripod_data);
            //     });
            // }
        }
        self.cast_log
            .entry(entity.name.clone())
            .or_default()
            .entry(skill_id)
            .or_default()
            .push(relative_timestamp);

        // if this is a getup skill and we have an ongoing abnormal move incapacitation, this will end it
        if let Some(skill_data) = SKILL_DATA.get(&skill_id) {
            if skill_data.skill_type == "getup" {
                for ongoing_event in entity
                    .damage_stats
                    .incapacitations
                    .iter_mut()
                    .rev()
                    .take_while(|x| x.timestamp + x.duration > timestamp)
                    .filter(|x| x.event_type == IncapacitationEventType::FALL_DOWN)
                {
                    info!(
                        "Shortening down duration from {} to {} because of getup skill",
                        ongoing_event.duration,
                        timestamp - ongoing_event.timestamp
                    );
                    ongoing_event.duration = timestamp - ongoing_event.timestamp;
                }
            }
        }

        (skill_id, skill_summon_sources)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn on_damage(
        &mut self,
        dmg_src_entity: &Entity,
        proj_entity: &Entity,
        dmg_target_entity: &Entity,
        damage_data: DamageData,
        se_on_source: Vec<StatusEffectDetails>,
        se_on_target: Vec<StatusEffectDetails>,
        _target_count: i32,
        _entity_tracker: &EntityTracker,
        _player_stats: &Option<Cache<String, PlayerStats>>,
        timestamp: i64,
    ) {
        let hit_flag = match damage_data.modifier & 0xf {
            0 => HitFlag::NORMAL,
            1 => HitFlag::CRITICAL,
            2 => HitFlag::MISS,
            3 => HitFlag::INVINCIBLE,
            4 => HitFlag::DOT,
            5 => HitFlag::IMMUNE,
            6 => HitFlag::IMMUNE_SILENCED,
            7 => HitFlag::FONT_SILENCED,
            8 => HitFlag::DOT_CRITICAL,
            9 => HitFlag::DODGE,
            10 => HitFlag::REFLECT,
            11 => HitFlag::DAMAGE_SHARE,
            12 => HitFlag::DODGE_HIT,
            13 => HitFlag::MAX,
            _ => {
                return;
            }
        };
        let hit_option_raw = ((damage_data.modifier >> 4) & 0x7) - 1;
        let hit_option = match hit_option_raw {
            -1 => HitOption::NONE,
            0 => HitOption::BACK_ATTACK,
            1 => HitOption::FRONTAL_ATTACK,
            2 => HitOption::FLANK_ATTACK,
            3 => HitOption::MAX,
            _ => {
                return;
            }
        };

        if hit_flag == HitFlag::INVINCIBLE {
            return;
        }
        if hit_flag == HitFlag::DAMAGE_SHARE
            && damage_data.skill_id == 0
            && damage_data.skill_effect_id == 0
        {
            return;
        }

        let mut skill_effect_id = damage_data.skill_effect_id;
        if proj_entity.entity_type == EntityType::PROJECTILE
            && is_battle_item(&proj_entity.skill_effect_id, "attack")
        {
            skill_effect_id = proj_entity.skill_effect_id;
        }

        // ensure source entity exists in encounter
        self.encounter
            .entities
            .entry(dmg_src_entity.name.clone())
            .or_insert_with(|| encounter_entity_from_entity(dmg_src_entity));

        // ensure target entity exists in encounter
        self.encounter
            .entities
            .entry(dmg_target_entity.name.clone())
            .or_insert_with(|| {
                let mut target_entity = encounter_entity_from_entity(dmg_target_entity);
                target_entity.current_hp = damage_data.target_current_hp;
                target_entity.max_hp = damage_data.target_max_hp;
                target_entity
            });

        if dmg_src_entity.name == dmg_target_entity.name {
            info!("ignoring self damage from {}", dmg_src_entity.name);
            return;
        }

        let [Some(source_entity), Some(target_entity)] = self
            .encounter
            .entities
            .get_many_mut([&dmg_src_entity.name, &dmg_target_entity.name])
        else {
            warn!(
                "{}, {} not found in encounter entities",
                dmg_src_entity.name, dmg_target_entity.name
            );
            return;
        };

        // if boss only damage is enabled
        // check if target is boss and not player
        // check if target is player and source is boss
        if self.boss_only_damage
            && ((target_entity.entity_type != EntityType::BOSS
                && target_entity.entity_type != EntityType::PLAYER)
                || (target_entity.entity_type == EntityType::PLAYER
                    && source_entity.entity_type != EntityType::BOSS))
        {
            return;
        }

        if self.encounter.fight_start == 0 {
            self.encounter.fight_start = timestamp;
            self.skill_tracker.fight_start = timestamp;
            if source_entity.entity_type == EntityType::PLAYER && damage_data.skill_id > 0 {
                self.skill_tracker.new_cast(
                    source_entity.id,
                    damage_data.skill_id,
                    None,
                    timestamp,
                );
            }

            if let Ok(result) = self.sntp_client.synchronize("time.cloudflare.com") {
                let dt = result.datetime().into_chrono_datetime().unwrap_or_default();
                self.ntp_fight_start = dt.timestamp_millis();
                // debug_print(format_args!("fight start local: {}, ntp: {}", Utc::now().to_rfc3339(), dt.to_rfc3339()));
            };

            self.encounter.boss_only_damage = self.boss_only_damage;
            self.app
                .emit_all("raid-start", timestamp)
                .expect("failed to emit raid-start");
        }

        self.encounter.last_combat_packet = timestamp;

        source_entity.id = dmg_src_entity.id;

        if target_entity.id == dmg_target_entity.id {
            target_entity.current_hp = damage_data.target_current_hp;
            target_entity.max_hp = damage_data.target_max_hp;
        }

        let mut damage = damage_data.damage + damage_data.shield_damage.unwrap_or(0);
        if target_entity.entity_type != EntityType::PLAYER && damage_data.target_current_hp < 0 {
            damage += damage_data.target_current_hp;
        }

        let mut skill_id = if damage_data.skill_id != 0 {
            damage_data.skill_id
        } else {
            skill_effect_id
        };

        let skill_data = get_skill(&skill_id);
        let mut skill_name = "".to_string();
        let mut skill_summon_sources: Option<Vec<u32>> = None;
        let mut is_hyper_awakening = false;
        if let Some(skill_data) = skill_data.as_ref() {
            skill_name = skill_data.name.clone().unwrap_or_default();
            skill_summon_sources.clone_from(&skill_data.summon_source_skills);
            is_hyper_awakening = skill_data.is_hyper_awakening;
        }

        if skill_name.is_empty() {
            (skill_name, _, skill_summon_sources) = get_skill_name_and_icon(
                &skill_id,
                &skill_effect_id,
                skill_id.to_string(),
                &self.skill_tracker,
                source_entity.id,
            );
        }
        let relative_timestamp = (timestamp - self.encounter.fight_start) as i32;

        if !source_entity.skills.contains_key(&skill_id) {
            if let Some(skill) = source_entity
                .skills
                .values()
                .find(|&s| s.name == *skill_name)
            {
                skill_id = skill.id;
            } else {
                let (skill_name, skill_icon, _) = get_skill_name_and_icon(
                    &skill_id,
                    &skill_effect_id,
                    skill_name.clone(),
                    &self.skill_tracker,
                    source_entity.id,
                );
                source_entity.skills.insert(
                    skill_id,
                    Skill {
                        id: skill_id,
                        name: {
                            if skill_name.is_empty() {
                                skill_id.to_string()
                            } else {
                                skill_name
                            }
                        },
                        icon: skill_icon,
                        summon_sources: skill_summon_sources.clone(),
                        casts: 1,
                        ..Default::default()
                    },
                );
            }
        }

        let skill = source_entity.skills.get_mut(&skill_id).unwrap();

        let mut skill_hit = SkillHit {
            damage,
            timestamp: relative_timestamp as i64,
            ..Default::default()
        };

        skill.total_damage += damage;
        if damage > skill.max_damage {
            skill.max_damage = damage;
        }
        skill.last_timestamp = timestamp;

        source_entity.damage_stats.damage_dealt += damage;

        if is_hyper_awakening {
            source_entity.damage_stats.hyper_awakening_damage += damage;
        }

        target_entity.damage_stats.damage_taken += damage;

        source_entity.skill_stats.hits += 1;
        skill.hits += 1;

        if hit_flag == HitFlag::CRITICAL || hit_flag == HitFlag::DOT_CRITICAL {
            source_entity.skill_stats.crits += 1;
            source_entity.damage_stats.crit_damage += damage;
            skill.crits += 1;
            skill.crit_damage += damage;
            skill_hit.crit = true;
        }
        if hit_option == HitOption::BACK_ATTACK {
            source_entity.skill_stats.back_attacks += 1;
            source_entity.damage_stats.back_attack_damage += damage;
            skill.back_attacks += 1;
            skill.back_attack_damage += damage;
            skill_hit.back_attack = true;
        }
        if hit_option == HitOption::FRONTAL_ATTACK {
            source_entity.skill_stats.front_attacks += 1;
            source_entity.damage_stats.front_attack_damage += damage;
            skill.front_attacks += 1;
            skill.front_attack_damage += damage;
            skill_hit.front_attack = true;
        }

        if source_entity.entity_type == EntityType::PLAYER {
            self.encounter.encounter_damage_stats.total_damage_dealt += damage;
            self.encounter.encounter_damage_stats.top_damage_dealt = max(
                self.encounter.encounter_damage_stats.top_damage_dealt,
                source_entity.damage_stats.damage_dealt,
            );

            self.damage_log
                .entry(source_entity.name.clone())
                .or_default()
                .push((timestamp, damage));

            let mut is_buffed_by_support = false;
            let mut is_buffed_by_identity = false;
            let mut is_debuffed_by_support = false;
            let mut is_buffed_by_hat = false;
            let se_on_source_ids = se_on_source
                .iter()
                .map(|se| map_status_effect(se, &mut self.custom_id_map))
                .collect::<Vec<_>>();
            for buff_id in se_on_source_ids.iter() {
                if !self
                    .encounter
                    .encounter_damage_stats
                    .unknown_buffs
                    .contains(buff_id)
                    && !self
                        .encounter
                        .encounter_damage_stats
                        .buffs
                        .contains_key(buff_id)
                {
                    let mut source_id: Option<u32> = None;
                    let original_buff_id = if let Some(deref_id) = self.custom_id_map.get(buff_id) {
                        source_id = Some(get_skill_id(*buff_id, *deref_id));
                        *deref_id
                    } else {
                        *buff_id
                    };

                    if let Some(status_effect) = get_status_effect_data(original_buff_id, source_id)
                    {
                        self.encounter
                            .encounter_damage_stats
                            .buffs
                            .insert(*buff_id, status_effect);
                    } else {
                        self.encounter
                            .encounter_damage_stats
                            .unknown_buffs
                            .insert(*buff_id);
                    }
                }
                if !is_buffed_by_support && !is_hat_buff(buff_id) {
                    if let Some(buff) = self.encounter.encounter_damage_stats.buffs.get(buff_id) {
                        if let Some(skill) = buff.source.skill.as_ref() {
                            is_buffed_by_support = is_support_class_id(skill.class_id)
                                && buff.buff_type & StatusEffectBuffTypeFlags::DMG.bits() != 0
                                && buff.target == StatusEffectTarget::PARTY
                                && (buff.buff_category == "classskill"
                                    || buff.buff_category == "arkpassive");
                        }
                    }
                }
                if !is_buffed_by_identity {
                    if let Some(buff) = self.encounter.encounter_damage_stats.buffs.get(buff_id) {
                        if let Some(skill) = buff.source.skill.as_ref() {
                            is_buffed_by_identity = is_support_class_id(skill.class_id)
                                && buff.buff_type & StatusEffectBuffTypeFlags::DMG.bits() != 0
                                && buff.target == StatusEffectTarget::PARTY
                                && buff.buff_category == "identity";
                        }
                    }
                }

                if !is_buffed_by_hat && is_hat_buff(buff_id) {
                    is_buffed_by_hat = true;
                }
            }
            let se_on_target_ids = se_on_target
                .iter()
                .map(|se| map_status_effect(se, &mut self.custom_id_map))
                .collect::<Vec<_>>();
            for debuff_id in se_on_target_ids.iter() {
                if !self
                    .encounter
                    .encounter_damage_stats
                    .unknown_buffs
                    .contains(debuff_id)
                    && !self
                        .encounter
                        .encounter_damage_stats
                        .debuffs
                        .contains_key(debuff_id)
                {
                    let mut source_id: Option<u32> = None;
                    let original_debuff_id =
                        if let Some(deref_id) = self.custom_id_map.get(debuff_id) {
                            source_id = Some(get_skill_id(*debuff_id, *deref_id));
                            *deref_id
                        } else {
                            *debuff_id
                        };

                    if let Some(status_effect) =
                        get_status_effect_data(original_debuff_id, source_id)
                    {
                        self.encounter
                            .encounter_damage_stats
                            .debuffs
                            .insert(*debuff_id, status_effect);
                    } else {
                        self.encounter
                            .encounter_damage_stats
                            .unknown_buffs
                            .insert(*debuff_id);
                    }
                }
                if !is_debuffed_by_support {
                    if let Some(debuff) =
                        self.encounter.encounter_damage_stats.debuffs.get(debuff_id)
                    {
                        if let Some(skill) = debuff.source.skill.as_ref() {
                            is_debuffed_by_support = is_support_class_id(skill.class_id)
                                && debuff.buff_type & StatusEffectBuffTypeFlags::DMG.bits() != 0
                                && debuff.target == StatusEffectTarget::PARTY;
                        }
                    }
                }
            }

            if is_buffed_by_support && !is_hyper_awakening {
                skill.buffed_by_support += damage;
                source_entity.damage_stats.buffed_by_support += damage;
            }
            if is_buffed_by_identity && !is_hyper_awakening {
                skill.buffed_by_identity += damage;
                source_entity.damage_stats.buffed_by_identity += damage;
            }
            if is_debuffed_by_support && !is_hyper_awakening {
                skill.debuffed_by_support += damage;
                source_entity.damage_stats.debuffed_by_support += damage;
            }
            if is_buffed_by_hat {
                skill.buffed_by_hat += damage;
                source_entity.damage_stats.buffed_by_hat += damage;
            }

            let stabilized_status_active =
                (source_entity.current_hp as f64 / source_entity.max_hp as f64) > 0.65;
            let mut filtered_se_on_source_ids: Vec<u32> = vec![];

            for buff_id in se_on_source_ids.iter() {
                if is_hyper_awakening && !is_hat_buff(buff_id) {
                    continue;
                }

                if let Some(buff) = self.encounter.encounter_damage_stats.buffs.get(buff_id) {
                    if !stabilized_status_active && buff.source.name.contains("Stabilized Status") {
                        continue;
                    }
                }

                filtered_se_on_source_ids.push(*buff_id);

                skill
                    .buffed_by
                    .entry(*buff_id)
                    .and_modify(|e| *e += damage)
                    .or_insert(damage);
                source_entity
                    .damage_stats
                    .buffed_by
                    .entry(*buff_id)
                    .and_modify(|e| *e += damage)
                    .or_insert(damage);
            }
            for debuff_id in se_on_target_ids.iter() {
                if is_hyper_awakening {
                    break;
                }

                skill
                    .debuffed_by
                    .entry(*debuff_id)
                    .and_modify(|e| *e += damage)
                    .or_insert(damage);
                source_entity
                    .damage_stats
                    .debuffed_by
                    .entry(*debuff_id)
                    .and_modify(|e| *e += damage)
                    .or_insert(damage);
            }

            skill_hit.buffed_by = filtered_se_on_source_ids;
            if !is_hyper_awakening {
                skill_hit.debuffed_by = se_on_target_ids;
            }
        }

        if target_entity.entity_type == EntityType::PLAYER {
            self.encounter.encounter_damage_stats.total_damage_taken += damage;
            self.encounter.encounter_damage_stats.top_damage_taken = max(
                self.encounter.encounter_damage_stats.top_damage_taken,
                target_entity.damage_stats.damage_taken,
            );
        }
        // update current_boss
        else if target_entity.entity_type == EntityType::BOSS {
            self.encounter
                .current_boss_name
                .clone_from(&target_entity.name);
            target_entity.id = dmg_target_entity.id;
            target_entity.npc_id = dmg_target_entity.npc_id;

            let log = self
                .boss_hp_log
                .entry(target_entity.name.clone())
                .or_default();

            let current_hp = if target_entity.current_hp >= 0 {
                target_entity.current_hp + target_entity.current_shield as i64
            } else {
                0
            };
            let hp_percent = if target_entity.max_hp != 0 {
                current_hp as f32 / target_entity.max_hp as f32
            } else {
                0.0
            };

            let relative_timestamp_s = relative_timestamp / 1000;

            if log.is_empty() || log.last().unwrap().time != relative_timestamp_s {
                log.push(BossHpLog::new(relative_timestamp_s, current_hp, hp_percent));
            } else {
                let last = log.last_mut().unwrap();
                last.hp = current_hp;
                last.p = hp_percent;
            }
        }

        if skill_id > 0 {
            self.skill_tracker.on_hit(
                source_entity.id,
                proj_entity.id,
                skill_id,
                skill_hit,
                skill_summon_sources,
            );
        }
    }

    pub fn on_counterattack(&mut self, source_entity: &Entity) {
        let entity = self
            .encounter
            .entities
            .entry(source_entity.name.clone())
            .or_insert_with(|| {
                let mut entity = encounter_entity_from_entity(source_entity);
                entity.skill_stats = SkillStats {
                    counters: 0,
                    ..Default::default()
                };
                entity
            });
        entity.skill_stats.counters += 1;
    }

    pub fn on_abnormal_move(
        &mut self,
        victim_entity: &Entity,
        movement: &SkillMoveOptionData,
        timestamp: i64,
    ) {
        if victim_entity.entity_type != EntityType::PLAYER {
            // we don't care about npc knockups
            return;
        }

        // only count movement events that would result in a knockup
        let Some(down_time) = movement.down_time else {
            return;
        };

        // todo: unclear if this is fully correct. It's hard to debug this, but it seems roughly accurate
        // if this is not accurate, we should probably factor out the stand_up_time and instead add in the
        // animation duration of the standup action for each class (seems to be 0.9s)
        let total_incapacitated_time = down_time
            + movement.move_time.unwrap_or_default()
            + movement.stand_up_time.unwrap_or_default();
        let incapacitated_time_ms = (total_incapacitated_time * 1000.0) as i64;

        let victim_entity_state = self
            .encounter
            .entities
            .entry(victim_entity.name.clone())
            .or_insert_with(|| encounter_entity_from_entity(victim_entity));

        // see if we have a previous incapacitation event that is still in effect (i.e. the player was knocked up again before
        // they could stand up), in which case we should shorten the previous event duration to the current timestamp
        let prev_incapacitation = victim_entity_state
            .damage_stats
            .incapacitations
            .iter_mut()
            .rev()
            .take_while(|e| e.timestamp + e.duration > timestamp) // stop as soon as we only hit expired events
            .find(|x| x.event_type == IncapacitationEventType::FALL_DOWN); // find an unexpired one that was caused by an abnormal move
        if let Some(prev_incapacitation) = prev_incapacitation {
            info!(
                "Shortening down duration from {} to {} because of new abnormal move",
                prev_incapacitation.duration,
                timestamp - prev_incapacitation.timestamp
            );
            prev_incapacitation.duration = timestamp - prev_incapacitation.timestamp;
        }

        let new_event = IncapacitatedEvent {
            timestamp,
            duration: incapacitated_time_ms,
            event_type: IncapacitationEventType::FALL_DOWN,
        };
        victim_entity_state
            .damage_stats
            .incapacitations
            .push(new_event);
        info!(
            "Player {} will be incapacitated for {}ms",
            victim_entity_state.name, incapacitated_time_ms
        );
    }

    pub fn on_cc_applied(&mut self, victim_entity: &Entity, status_effect: &StatusEffectDetails) {
        let victim_entity_state = self
            .encounter
            .entities
            .entry(victim_entity.name.clone())
            .or_insert_with(|| encounter_entity_from_entity(victim_entity));

        // expiration delay is zero or negative for infinite effects. Instead of applying them now,
        // only apply them after they've been removed (this avoids an issue where if we miss the removal
        // we end up applying a very long incapacitation)
        if status_effect_is_infinite(status_effect) {
            return;
        }

        let duration_ms = status_effect.expiration_delay * 1000.0;
        let new_event = IncapacitatedEvent {
            timestamp: status_effect.timestamp.timestamp_millis(),
            duration: duration_ms as i64,
            event_type: IncapacitationEventType::CROWD_CONTROL,
        };
        info!(
            "Player {} will be status-effect incapacitated for {}ms by buff {}",
            victim_entity_state.name, duration_ms, status_effect.status_effect_id
        );
        victim_entity_state
            .damage_stats
            .incapacitations
            .push(new_event);
    }

    pub fn on_cc_removed(
        &mut self,
        victim_entity: &Entity,
        status_effect: &StatusEffectDetails,
        timestamp: i64,
    ) {
        let victim_entity_state = self
            .encounter
            .entities
            .entry(victim_entity.name.clone())
            .or_insert_with(|| encounter_entity_from_entity(victim_entity));

        if status_effect_is_infinite(status_effect) {
            // this status effect was infinite, meaning we didn't apply it on_cc_applied
            // apply it now retroactively, then sort the events to ensure that our sorted
            // invariant does not get violated
            let duration_ms = timestamp - status_effect.timestamp.timestamp_millis();
            let new_event = IncapacitatedEvent {
                timestamp: status_effect.timestamp.timestamp_millis(),
                duration: duration_ms,
                event_type: IncapacitationEventType::CROWD_CONTROL,
            };
            info!(
                "Player {} was incapacitated by an infinite status effect buff for {}ms",
                victim_entity_state.name, duration_ms
            );
            victim_entity_state
                .damage_stats
                .incapacitations
                .push(new_event);
            victim_entity_state
                .damage_stats
                .incapacitations
                .sort_by_key(|x| x.timestamp);
            return;
        }

        // we use the application timestamp as the key. Attempt to find all buff instances that started
        // at this time and cap their duration to the current timestamp
        for event in victim_entity_state
            .damage_stats
            .incapacitations
            .iter_mut()
            .rev()
            .take_while(|e| e.timestamp + e.duration > timestamp)
        {
            if event.event_type == IncapacitationEventType::CROWD_CONTROL
                && event.timestamp == status_effect.timestamp.timestamp_millis()
            {
                info!(
                    "Removing status-effect {} incapacitation for player {} (shortened {}ms to {}ms)",
                    status_effect.status_effect_id,
                    victim_entity_state.name,
                    event.duration,
                    timestamp - event.timestamp
                );
                event.duration = timestamp - event.timestamp;
            }
        }
    }

    // pub fn on_identity_gain(&mut self, pkt: &PKTIdentityGaugeChangeNotify) {
    //     if self.encounter.fight_start == 0 {
    //         return;
    //     }
    //
    //     if self.encounter.local_player.is_empty() {
    //         if let Some((_, entity)) = self
    //             .encounter
    //             .entities
    //             .iter()
    //             .find(|(_, e)| e.id == pkt.player_id)
    //         {
    //             self.encounter.local_player.clone_from(&entity.name);
    //         } else {
    //             return;
    //         }
    //     }
    //
    //     if let Some(entity) = self
    //         .encounter
    //         .entities
    //         .get_mut(&self.encounter.local_player)
    //     {
    //         self.identity_log
    //             .entry(entity.name.clone())
    //             .or_default()
    //             .push((
    //                 Utc::now().timestamp_millis(),
    //                 (
    //                     pkt.identity_gauge1,
    //                     pkt.identity_gauge2,
    //                     pkt.identity_gauge3,
    //                 ),
    //             ));
    //     }
    // }

    // pub fn on_stagger_change(&mut self, pkt: &PKTParalyzationStateNotify) {
    //     if self.encounter.current_boss_name.is_empty() || self.encounter.fight_start == 0 {
    //         return;
    //     }

    //     if let Some(boss) = self
    //         .encounter
    //         .entities
    //         .get_mut(&self.encounter.current_boss_name)
    //     {
    //         let timestamp = Utc::now().timestamp_millis();
    //         let current_stagger = pkt.paralyzation_point as i32;
    //         let max_stagger = pkt.paralyzation_max_point as i32;
    //         if boss.id == pkt.object_id {
    //             if current_stagger == max_stagger {
    //                 let staggered_in =
    //                     (timestamp - self.encounter.encounter_damage_stats.stagger_start) / 1000;
    //                 self.stagger_intervals
    //                     .push((staggered_in as i32, max_stagger))
    //             } else if current_stagger != 0 && self.prev_stagger == 0 {
    //                 self.encounter.encounter_damage_stats.stagger_start = timestamp;
    //             }

    //             self.prev_stagger = current_stagger;

    //             let relative_timestamp_s = ((timestamp - self.encounter.fight_start) / 1000) as i32;
    //             let stagger_percent = (1.0 - (current_stagger as f32 / max_stagger as f32)) * 100.0;
    //             if let Some(last) = self.stagger_log.last_mut() {
    //                 if last.0 == relative_timestamp_s {
    //                     last.1 = stagger_percent;
    //                 } else {
    //                     self.stagger_log
    //                         .push((relative_timestamp_s, stagger_percent));
    //                 }
    //             } else {
    //                 self.stagger_log
    //                     .push((relative_timestamp_s, stagger_percent));
    //             }

    //             if max_stagger > self.encounter.encounter_damage_stats.max_stagger {
    //                 self.encounter.encounter_damage_stats.max_stagger = max_stagger;
    //             }
    //         }
    //     }
    // }

    pub fn on_boss_shield(&mut self, target_entity: &Entity, shield: u64) {
        if target_entity.entity_type == EntityType::BOSS
            && target_entity.name == self.encounter.current_boss_name
        {
            self.encounter
                .entities
                .entry(target_entity.name.clone())
                .and_modify(|e| {
                    e.current_shield = shield;
                });
        }
    }

    pub fn on_shield_applied(
        &mut self,
        source_entity: &Entity,
        target_entity: &Entity,
        buff_id: u32,
        shield: u64,
    ) {
        if source_entity.entity_type == EntityType::PLAYER
            && target_entity.entity_type == EntityType::PLAYER
        {
            if !self
                .encounter
                .encounter_damage_stats
                .applied_shield_buffs
                .contains_key(&buff_id)
            {
                let mut source_id: Option<u32> = None;
                let original_buff_id = if let Some(deref_id) = self.custom_id_map.get(&buff_id) {
                    source_id = Some(get_skill_id(buff_id, *deref_id));
                    *deref_id
                } else {
                    buff_id
                };

                if let Some(status_effect) = get_status_effect_data(original_buff_id, source_id) {
                    self.encounter
                        .encounter_damage_stats
                        .applied_shield_buffs
                        .insert(buff_id, status_effect);
                }
            }

            self.encounter.encounter_damage_stats.total_shielding += shield;

            let source_entity_state = self
                .encounter
                .entities
                .entry(source_entity.name.clone())
                .or_insert_with(|| encounter_entity_from_entity(source_entity));

            // shields on self
            if source_entity.id == target_entity.id || source_entity.name == target_entity.name {
                source_entity_state.damage_stats.shields_received += shield;
                source_entity_state.damage_stats.shields_given += shield;
                source_entity_state
                    .damage_stats
                    .shields_given_by
                    .entry(buff_id)
                    .and_modify(|e| *e += shield)
                    .or_insert(shield);
                source_entity_state
                    .damage_stats
                    .shields_received_by
                    .entry(buff_id)
                    .and_modify(|e| *e += shield)
                    .or_insert(shield);

                return;
            }

            // shields on others
            self.encounter
                .entities
                .entry(target_entity.name.clone())
                .or_insert_with(|| encounter_entity_from_entity(target_entity));

            let [Some(source_entity_state), Some(target_entity_state)] = self
                .encounter
                .entities
                .get_many_mut([&source_entity.name, &target_entity.name])
            else {
                warn!(
                    "{}, {} not found in encounter entities",
                    source_entity.name, target_entity.name
                );
                return;
            };

            target_entity_state.damage_stats.shields_received += shield;
            source_entity_state.damage_stats.shields_given += shield;
            source_entity_state
                .damage_stats
                .shields_given_by
                .entry(buff_id)
                .and_modify(|e| *e += shield)
                .or_insert(shield);
            target_entity_state
                .damage_stats
                .shields_received_by
                .entry(buff_id)
                .and_modify(|e| *e += shield)
                .or_insert(shield);
        }
    }

    pub fn on_shield_used(
        &mut self,
        source_entity: &Entity,
        target_entity: &Entity,
        buff_id: u32,
        shield_removed: u64,
    ) {
        if source_entity.entity_type == EntityType::PLAYER
            && target_entity.entity_type == EntityType::PLAYER
        {
            self.encounter
                .encounter_damage_stats
                .total_effective_shielding += shield_removed;

            let source_entity_state = self
                .encounter
                .entities
                .entry(source_entity.name.clone())
                .or_insert_with(|| encounter_entity_from_entity(source_entity));

            // shields on self
            if source_entity.id == target_entity.id || source_entity.name == target_entity.name {
                source_entity_state.damage_stats.damage_absorbed += shield_removed;
                source_entity_state.damage_stats.damage_absorbed_on_others += shield_removed;
                source_entity_state
                    .damage_stats
                    .damage_absorbed_by
                    .entry(buff_id)
                    .and_modify(|e| *e += shield_removed)
                    .or_insert(shield_removed);
                source_entity_state
                    .damage_stats
                    .damage_absorbed_on_others_by
                    .entry(buff_id)
                    .and_modify(|e| *e += shield_removed)
                    .or_insert(shield_removed);

                return;
            }

            // shields on others
            self.encounter
                .entities
                .entry(target_entity.name.clone())
                .or_insert_with(|| encounter_entity_from_entity(target_entity));

            let [Some(source_entity_state), Some(target_entity_state)] = self
                .encounter
                .entities
                .get_many_mut([&source_entity.name, &target_entity.name])
            else {
                warn!(
                    "{}, {} not found in encounter entities",
                    source_entity.name, target_entity.name
                );
                return;
            };

            target_entity_state.damage_stats.damage_absorbed += shield_removed;
            source_entity_state.damage_stats.damage_absorbed_on_others += shield_removed;
            target_entity_state
                .damage_stats
                .damage_absorbed_by
                .entry(buff_id)
                .and_modify(|e| *e += shield_removed)
                .or_insert(shield_removed);
            source_entity_state
                .damage_stats
                .damage_absorbed_on_others_by
                .entry(buff_id)
                .and_modify(|e| *e += shield_removed)
                .or_insert(shield_removed);
        }
    }

    pub fn save_to_db(&mut self, stats_api: &StatsApi, manual: bool) {
        if !manual
            && (self.encounter.fight_start == 0
                || self.encounter.current_boss_name.is_empty()
                || !self
                    .encounter
                    .entities
                    .contains_key(&self.encounter.current_boss_name)
                || !self.encounter.entities.values().any(|e| {
                    e.entity_type == EntityType::PLAYER && e.damage_stats.damage_dealt > 0
                }))
        {
            info!("not saving to db, no players with damage dealt");
            return;
        }

        if !self.damage_is_valid {
            warn!("damage decryption is invalid, not saving to db");
        }

        let mut encounter = self.encounter.clone();
        let mut path = self
            .app
            .app_handle()
            .path_resolver()
            .resource_dir()
            .expect("could not get resource dir");
        path.push("encounters.db");

        let damage_log = self.damage_log.clone();
        let cast_log = self.cast_log.clone();
        let boss_hp_log = self.boss_hp_log.clone();
        let raid_clear = self.raid_clear;
        let party_info = self.party_info.clone();
        let raid_difficulty = self.raid_difficulty.clone();
        let region = self.region.clone();
        let meter_version = self.app.app_handle().package_info().version.to_string();

        let ntp_fight_start = self.ntp_fight_start;

        let rdps_valid = self.rdps_valid;

        let skill_cast_log = self.skill_tracker.get_cast_log();

        let stats_api = stats_api.clone();

        // debug_print(format_args!("skill cast log:\n{}", serde_json::to_string(&skill_cast_log).unwrap()));

        // debug_print(format_args!("rdps_data valid: [{}]", rdps_valid));
        info!(
            "saving to db - cleared: [{}], difficulty: [{}] {}",
            raid_clear, self.raid_difficulty, encounter.current_boss_name
        );

        encounter.current_boss_name = update_current_boss_name(&encounter.current_boss_name);

        let window = self.app.clone();
        task::spawn(async move {
            let player_infos =
                if !raid_difficulty.is_empty() && !encounter.current_boss_name.is_empty() {
                    info!("fetching player info");
                    stats_api.get_character_info(&encounter).await
                } else {
                    None
                };

            let mut conn = Connection::open(path).expect("failed to open database");
            let tx = conn.transaction().expect("failed to create transaction");

            let encounter_id = insert_data(
                &tx,
                encounter,
                damage_log,
                cast_log,
                boss_hp_log,
                raid_clear,
                party_info,
                raid_difficulty,
                region,
                player_infos,
                meter_version,
                ntp_fight_start,
                rdps_valid,
                manual,
                skill_cast_log,
            );

            tx.commit().expect("failed to commit transaction");
            info!("saved to db");

            if raid_clear {
                window
                    .emit_all("clear-encounter", encounter_id)
                    .expect("failed to emit clear-encounter");
            }
        });
    }
}

fn status_effect_is_infinite(status_effect: &StatusEffectDetails) -> bool {
    // infinite if duration is (sub-)zero or longer than an hour
    status_effect.expiration_delay <= 0.0 || status_effect.expiration_delay > 3600.0
}
