use chrono::Utc;
use hashbrown::HashMap;
use log::{info, warn};
use meter_core::packets::definitions::PKTIdentityGaugeChangeNotify;
use moka::sync::Cache;
use rsntp::SntpClient;
use rusqlite::Connection;
use std::cmp::{max, Ordering};
use std::default::Default;

use crate::parser::debug_print;
use tauri::{Manager, Window, Wry};
use tokio::task;

use crate::parser::entity_tracker::{Entity, EntityTracker};
use crate::parser::models::*;
use crate::parser::rdps::*;
use crate::parser::skill_tracker::SkillTracker;
use crate::parser::stats_api::{PlayerStats, StatsApi};
use crate::parser::status_tracker::StatusEffectDetails;
use crate::parser::utils::*;

const RDPS_VALID_LIMIT: i64 = 25_000;

#[derive(Debug)]
pub struct EncounterState {
    pub window: Window<Wry>,
    pub encounter: Encounter,
    pub resetting: bool,
    pub boss_dead_update: bool,
    pub saved: bool,

    pub raid_clear: bool,

    prev_stagger: i32,

    damage_log: HashMap<String, Vec<(i64, i64)>>,
    identity_log: HashMap<String, IdentityLog>,
    cast_log: HashMap<String, HashMap<u32, Vec<i32>>>,

    boss_hp_log: HashMap<String, Vec<BossHpLog>>,

    stagger_log: Vec<(i32, f32)>,
    stagger_intervals: Vec<(i32, i32)>,

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
}

impl EncounterState {
    pub fn new(window: Window<Wry>) -> EncounterState {
        EncounterState {
            window,
            encounter: Encounter::default(),
            resetting: false,
            raid_clear: false,
            boss_dead_update: false,
            saved: false,

            prev_stagger: 0,
            damage_log: HashMap::new(),
            identity_log: HashMap::new(),
            boss_hp_log: HashMap::new(),
            cast_log: HashMap::new(),
            stagger_log: Vec::new(),
            stagger_intervals: Vec::new(),

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
        self.prev_stagger = 0;
        self.raid_clear = false;

        self.damage_log = HashMap::new();
        self.identity_log = HashMap::new();
        self.cast_log = HashMap::new();
        self.boss_hp_log = HashMap::new();
        self.stagger_log = Vec::new();
        self.stagger_intervals = Vec::new();
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

    pub fn on_init_env(
        &mut self,
        entity: Entity,
        player_stats: Option<Cache<String, PlayerStats>>,
    ) {
        // if not already saved to db, we save again
        if !self.saved && !self.encounter.current_boss_name.is_empty() {
            self.save_to_db(player_stats, false);
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

        self.window
            .emit("zone-change", "")
            .expect("failed to emit zone-change");

        self.soft_reset(false);
    }

    pub fn on_phase_transition(&mut self, phase_code: i32, stats_api: &mut StatsApi) {
        self.window
            .emit("phase-transition", phase_code)
            .expect("failed to emit phase-transition");

        match phase_code {
            0 | 2 | 3 | 4 => {
                if !self.encounter.current_boss_name.is_empty() {
                    let player_stats = stats_api.get_stats(self);
                    stats_api.send_raid_info(self);
                    if phase_code == 0 {
                        stats_api.valid_zone = false;
                    }
                    self.save_to_db(player_stats, false);
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
                    .map_or(true, |boss| npc.max_hp >= boss.max_hp || boss.is_dead)
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
            let mut tripod_data: Vec<TripodData> = vec![];
            if let (Some(tripod_index), Some(tripod_level)) = (tripod_index, tripod_level) {
                let mut indexes = vec![tripod_index.first];
                if tripod_index.second != 0 {
                    indexes.push(tripod_index.second + 3);
                }
                // third row should never be set if second is not set
                if tripod_index.third != 0 {
                    indexes.push(tripod_index.third + 6);
                }
                let levels = [tripod_level.first, tripod_level.second, tripod_level.third];
                if let Some(effect) = SKILL_FEATURE_DATA.get(&skill_id) {
                    for i in 0..indexes.len() {
                        if let Some(entries) = effect.tripods.get(&indexes[i]) {
                            let mut options: Vec<SkillFeatureOption> = vec![];
                            for entry in &entries.entries {
                                if entry.level > 0 && entry.level == levels[i] {
                                    options.push(entry.clone());
                                }
                            }
                            tripod_data.push(TripodData {
                                index: indexes[i],
                                options,
                            });
                        }
                    }
                }
            }

            if !tripod_data.is_empty() {
                entity.skills.entry(skill_id).and_modify(|e| {
                    e.tripod_data = Some(tripod_data);
                });
            }
        }
        self.cast_log
            .entry(entity.name.clone())
            .or_default()
            .entry(skill_id)
            .or_default()
            .push(relative_timestamp);

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
        target_count: i32,
        entity_tracker: &EntityTracker,
        player_stats: &Option<Cache<String, PlayerStats>>,
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
            && is_battle_item(skill_effect_id, "attack")
        {
            skill_effect_id = proj_entity.skill_effect_id;
        }

        let mut source_entity = self
            .encounter
            .entities
            .entry(dmg_src_entity.name.clone())
            .or_insert_with(|| encounter_entity_from_entity(dmg_src_entity))
            .to_owned();

        let mut target_entity = self
            .encounter
            .entities
            .entry(dmg_target_entity.name.clone())
            .or_insert_with(|| {
                let mut target_entity = encounter_entity_from_entity(dmg_target_entity);
                target_entity.current_hp = damage_data.target_current_hp;
                target_entity.max_hp = damage_data.target_max_hp;
                target_entity
            })
            .to_owned();

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
            self.window
                .emit("raid-start", timestamp)
                .expect("failed to emit raid-start");
        }

        self.encounter.last_combat_packet = timestamp;

        source_entity.id = dmg_src_entity.id;

        if target_entity.id == dmg_target_entity.id {
            target_entity.current_hp = damage_data.target_current_hp;
            target_entity.max_hp = damage_data.target_max_hp;
        }

        let mut damage = damage_data.damage;
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
        if let Some(skill_data) = skill_data.as_ref() {
            skill_name = skill_data.name.clone().unwrap_or_default();
            skill_summon_sources.clone_from(&skill_data.summon_source_skill);
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
        let is_hyper_awakening = is_hyper_awakening_skill(skill.id);
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
                        source_id = Some(get_skill_id(*buff_id));
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
                if !is_buffed_by_support {
                    if let Some(buff) = self.encounter.encounter_damage_stats.buffs.get(buff_id) {
                        if let Some(skill) = buff.source.skill.as_ref() {
                            is_buffed_by_support = is_support_class_id(skill.class_id)
                                && buff.buff_type & StatusEffectBuffTypeFlags::DMG.bits() != 0
                                && buff.target == StatusEffectTarget::PARTY
                                && buff.buff_category == "classskill";
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

                if !is_buffed_by_hat && is_hat_buff(*buff_id) {
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
                            source_id = Some(get_skill_id(*debuff_id));
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
            
            for buff_id in se_on_source_ids.iter() {
                if is_hyper_awakening && !is_hat_buff(*buff_id) {
                    continue;
                }

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
            
            if is_hyper_awakening { 
                skill_hit.buffed_by = se_on_source_ids.iter().filter(|&id| is_hat_buff(*id)).cloned().collect();
            } else {
                skill_hit.buffed_by = se_on_source_ids;
                skill_hit.debuffed_by = se_on_target_ids;
            }

            // todo
            if let (true, Some(player_stats)) =
                (self.rdps_valid && damage > 0, player_stats.clone())
            {
                // rdps ported from meter-core by herysia
                // refer to here for documentation
                // https://github.com/lost-ark-dev/meter-core/blob/a93ed3dd05a251d8dee47f5e6e17f275a0bd89fb/src/logger/gameTracker.ts#L417
                if let Some(dmg_src_stats) = player_stats.get(&dmg_src_entity.name) {
                    let mut rdps_data = RdpsData::default();
                    for status_effect in se_on_source.iter() {
                        let caster_entity =
                            match entity_tracker.entities.get(&status_effect.source_id) {
                                Some(entity) => entity,
                                None => continue,
                            };
                        let caster_encounter_entity =
                            match self.encounter.entities.get(&caster_entity.name) {
                                Some(entity) => entity,
                                None => continue,
                            };
                        let caster_stats = match player_stats.get(&caster_entity.name) {
                            Some(caster) => caster,
                            None => {
                                if caster_entity.entity_type == EntityType::PLAYER
                                    && self.encounter.last_combat_packet
                                        - self.encounter.fight_start
                                        > RDPS_VALID_LIMIT
                                {
                                    warn!(
                                        "caster {:?} is not in player_stats. [{}]",
                                        caster_entity.name,
                                        self.encounter.last_combat_packet
                                            - self.encounter.fight_start
                                    );
                                    self.rdps_valid = false;
                                    if !self.rdps_valid {
                                        self.window
                                            .emit("rdps", "invalid_stats")
                                            .expect("failed to emit rdps message");
                                    }
                                }

                                PlayerStats::default()
                            }
                        };
                        let original_buff =
                            match SKILL_BUFF_DATA.get(&status_effect.status_effect_id) {
                                Some(buff) => buff,
                                None => continue,
                            };
                        let buff = get_buff_after_tripods(
                            original_buff,
                            caster_encounter_entity,
                            skill_id,
                            skill_effect_id,
                        );

                        if buff.buff_type == "skill_damage_amplify"
                            && buff.status_effect_values.is_some()
                            && caster_encounter_entity.entity_type == EntityType::PLAYER
                            && status_effect.source_id != dmg_src_entity.id
                        {
                            let status_effect_values = buff.status_effect_values.unwrap();
                            let b_skill_id =
                                status_effect_values.first().cloned().unwrap_or_default();
                            let b_skill_effect_id =
                                status_effect_values.get(4).cloned().unwrap_or_default();
                            if (b_skill_id == 0 || b_skill_id == skill_id as i32)
                                && (b_skill_effect_id == 0
                                    || b_skill_effect_id == skill_effect_id as i32)
                            {
                                if let Some(val) =
                                    status_effect_values.get(1).cloned().filter(|&v| v != 0)
                                {
                                    let rate =
                                        (val as f64 / 10000.0) * status_effect.stack_count as f64;
                                    rdps_data.multi_dmg.values.push(RdpsBuffData {
                                        caster: caster_encounter_entity.name.clone(),
                                        rate,
                                    });
                                    rdps_data.multi_dmg.sum_rate += rate;
                                    rdps_data.multi_dmg.total_rate *= 1.0 + rate;
                                }
                            }
                        } else if buff.buff_type == "attack_power_amplify"
                            && buff.status_effect_values.is_some()
                            && caster_encounter_entity.entity_type == EntityType::PLAYER
                            && status_effect.source_id != dmg_src_entity.id
                        {
                            let status_effect_values = buff.status_effect_values.unwrap();
                            if let Some(val) =
                                status_effect_values.first().cloned().filter(|&v| v != 0)
                            {
                                let mut rate =
                                    (val as f64 / 10000.0) * status_effect.stack_count as f64;
                                let caster_base_atk_power = caster_stats.stats.atk_power;
                                let target_base_atk_power = dmg_src_stats.stats.atk_power;
                                rate *= caster_base_atk_power as f64 / target_base_atk_power as f64;
                                rdps_data.atk_pow_amplify.push(RdpsBuffData {
                                    caster: caster_encounter_entity.name.clone(),
                                    rate,
                                });
                            }
                        }

                        for passive in buff.passive_options {
                            let val = passive.value as f64;
                            if passive.option_type == "stat" {
                                let rate = (val / 10000.0) * status_effect.stack_count as f64;
                                // println!("{}: {}: {}", passive.key_stat, val, status_effect.stack_count);
                                if passive.key_stat == "attack_power_sub_rate_2" && val != 0.0 {
                                    if caster_encounter_entity.entity_type == EntityType::PLAYER
                                        && status_effect.source_id != dmg_src_entity.id
                                    {
                                        rdps_data.atk_pow_sub_rate_2.values.push(RdpsBuffData {
                                            caster: caster_encounter_entity.name.clone(),
                                            rate,
                                        });
                                        rdps_data.atk_pow_sub_rate_2.sum_rate += rate;
                                    } else {
                                        rdps_data.atk_pow_sub_rate_2.self_sum_rate += rate;
                                    }
                                } else if passive.key_stat == "attack_power_sub_rate_1"
                                    && val != 0.0
                                {
                                    if caster_encounter_entity.entity_type == EntityType::PLAYER
                                        && status_effect.source_id != dmg_src_entity.id
                                    {
                                        rdps_data.atk_pow_sub_rate_1.values.push(RdpsBuffData {
                                            caster: caster_encounter_entity.name.clone(),
                                            rate,
                                        });
                                        rdps_data.atk_pow_sub_rate_1.sum_rate += rate;
                                        rdps_data.atk_pow_sub_rate_1.total_rate *= 1.0 + rate;
                                    }
                                } else if passive.key_stat == "skill_damage_rate" && val != 0.0 {
                                    if caster_encounter_entity.entity_type == EntityType::PLAYER
                                        && status_effect.source_id != dmg_src_entity.id
                                    {
                                        rdps_data.skill_dmg_rate.values.push(RdpsBuffData {
                                            caster: caster_encounter_entity.name.clone(),
                                            rate,
                                        });
                                        rdps_data.skill_dmg_rate.sum_rate += rate;
                                    } else {
                                        rdps_data.skill_dmg_rate.self_sum_rate += rate;
                                    }
                                }
                            }
                            if passive.key_stat == "critical_hit_rate" && val != 0.0 {
                                let rate = (val / 10000.0) * status_effect.stack_count as f64;
                                if caster_encounter_entity.entity_type == EntityType::PLAYER
                                    && status_effect.source_id != dmg_src_entity.id
                                {
                                    rdps_data.crit.values.push(RdpsBuffData {
                                        caster: caster_encounter_entity.name.clone(),
                                        rate,
                                    });
                                    rdps_data.crit.sum_rate += rate;
                                } else {
                                    rdps_data.crit.self_sum_rate += rate;
                                }
                            }
                            if caster_encounter_entity.entity_type == EntityType::PLAYER
                                && status_effect.source_id != dmg_src_entity.id
                            {
                                let mut rate = (val / 10000.0) * status_effect.stack_count as f64;
                                if passive.key_stat == "skill_damage_sub_rate_2" && val != 0.0 {
                                    let spec = caster_stats.stats.spec as f64;
                                    match caster_encounter_entity.class_id {
                                        105 => rate *= 1.0 + ((spec / 0.0699) * 0.63) / 10000.0,
                                        204 => rate *= 1.0 + ((spec / 0.0699) * 0.35) / 10000.0,
                                        602 => rate *= 1.0 + ((spec / 0.0699) * 0.38) / 10000.0,
                                        _ => {}
                                    }
                                    rdps_data.multi_dmg.values.push(RdpsBuffData {
                                        caster: caster_encounter_entity.name.clone(),
                                        rate,
                                    });
                                    rdps_data.multi_dmg.sum_rate += rate;
                                    rdps_data.multi_dmg.total_rate *= 1.0 + rate;
                                } else if passive.key_stat == "critical_dam_rate"
                                    && buff.buff_category.clone().unwrap_or_default() == "buff"
                                {
                                    rdps_data.crit_dmg_rate += rate;
                                }
                            } else if passive.option_type == "combat_effect" {
                                if let Some(ce) = COMBAT_EFFECT_DATA.get(&passive.key_index) {
                                    let ce_conditional_data = CombatEffectConditionData {
                                        self_entity: dmg_src_entity,
                                        target_entity: dmg_target_entity,
                                        caster_entity,
                                        skill: skill_data.as_ref(),
                                        hit_option: hit_option_raw,
                                        target_count,
                                    };
                                    let crit_multiplier = get_crit_multiplier_from_combat_effect(
                                        ce,
                                        &ce_conditional_data,
                                    );
                                    rdps_data.crit_dmg_rate +=
                                        status_effect.stack_count as f64 * crit_multiplier;
                                }
                            }
                        }
                    }

                    for status_effect in se_on_target.iter() {
                        let caster_entity =
                            match entity_tracker.entities.get(&status_effect.source_id) {
                                Some(entity) => entity,
                                None => continue,
                            };
                        let caster_encounter_entity =
                            match self.encounter.entities.get(&caster_entity.name) {
                                Some(entity) => entity,
                                None => continue,
                            };
                        let original_debuff =
                            match SKILL_BUFF_DATA.get(&status_effect.status_effect_id) {
                                Some(buff) => buff,
                                None => continue,
                            };
                        let debuff = get_buff_after_tripods(
                            original_debuff,
                            caster_encounter_entity,
                            skill_id,
                            skill_effect_id,
                        );
                        let status_effect_values = match debuff.status_effect_values {
                            Some(values) => values,
                            None => continue,
                        };
                        if debuff.buff_type == "instant_stat_amplify" {
                            if let Some(val) =
                                status_effect_values.first().cloned().filter(|&v| v != 0)
                            {
                                let rate =
                                    (val as f64 / 10000.0) * status_effect.stack_count as f64;
                                if caster_encounter_entity.entity_type == EntityType::PLAYER
                                    && status_effect.source_id != dmg_src_entity.id
                                {
                                    rdps_data.crit.values.push(RdpsBuffData {
                                        caster: caster_encounter_entity.name.clone(),
                                        rate,
                                    });
                                    rdps_data.crit.sum_rate += rate;
                                } else {
                                    rdps_data.crit.self_sum_rate += rate;
                                }
                            }
                        }
                        if caster_encounter_entity.entity_type != EntityType::PLAYER
                            || status_effect.source_id == dmg_src_entity.id
                        {
                            continue;
                        }
                        if debuff.buff_type == "instant_stat_amplify" {
                            if damage_data.damage_type == 0 {
                                if let Some(val) =
                                    status_effect_values.get(2).cloned().filter(|&v| v != 0)
                                {
                                    let rate = -(val as f64 / 10000.0)
                                        * status_effect.stack_count as f64
                                        * 0.5;
                                    rdps_data.multi_dmg.values.push(RdpsBuffData {
                                        caster: caster_encounter_entity.name.clone(),
                                        rate,
                                    });
                                    rdps_data.multi_dmg.sum_rate += rate;
                                    rdps_data.multi_dmg.total_rate *= 1.0 + rate;
                                }
                                if let Some(val) =
                                    status_effect_values.get(7).cloned().filter(|&v| v != 0)
                                {
                                    let rate =
                                        (val as f64 / 10000.0) * status_effect.stack_count as f64;
                                    rdps_data.multi_dmg.values.push(RdpsBuffData {
                                        caster: caster_encounter_entity.name.clone(),
                                        rate,
                                    });
                                    rdps_data.multi_dmg.sum_rate += rate;
                                    rdps_data.multi_dmg.total_rate *= 1.0 + rate;
                                }
                                if hit_flag == HitFlag::CRITICAL {
                                    if let Some(val) =
                                        status_effect_values.get(9).cloned().filter(|&v| v != 0)
                                    {
                                        let rate = (val as f64 / 10000.0)
                                            * status_effect.stack_count as f64;
                                        rdps_data.multi_dmg.values.push(RdpsBuffData {
                                            caster: caster_encounter_entity.name.clone(),
                                            rate,
                                        });
                                        rdps_data.multi_dmg.sum_rate += rate;
                                        rdps_data.multi_dmg.total_rate *= 1.0 + rate;
                                    }
                                }
                            } else if damage_data.damage_type == 1 {
                                if let Some(val) =
                                    status_effect_values.get(3).cloned().filter(|&v| v != 0)
                                {
                                    let rate = -(val as f64 / 10000.0)
                                        * status_effect.stack_count as f64
                                        * 0.5;
                                    rdps_data.multi_dmg.values.push(RdpsBuffData {
                                        caster: caster_encounter_entity.name.clone(),
                                        rate,
                                    });
                                    rdps_data.multi_dmg.sum_rate += rate;
                                    rdps_data.multi_dmg.total_rate *= 1.0 + rate;
                                }
                                if let Some(val) =
                                    status_effect_values.get(8).cloned().filter(|&v| v != 0)
                                {
                                    let rate =
                                        val as f64 / 10000.0 * status_effect.stack_count as f64;
                                    rdps_data.multi_dmg.values.push(RdpsBuffData {
                                        caster: caster_encounter_entity.name.clone(),
                                        rate,
                                    });
                                    rdps_data.multi_dmg.sum_rate += rate;
                                    rdps_data.multi_dmg.total_rate *= 1.0 + rate;
                                }
                                if hit_flag == HitFlag::CRITICAL {
                                    if let Some(val) =
                                        status_effect_values.get(10).cloned().filter(|&v| v != 0)
                                    {
                                        let rate =
                                            val as f64 / 10000.0 * status_effect.stack_count as f64;
                                        rdps_data.multi_dmg.values.push(RdpsBuffData {
                                            caster: caster_encounter_entity.name.clone(),
                                            rate,
                                        });
                                        rdps_data.multi_dmg.sum_rate += rate;
                                        rdps_data.multi_dmg.total_rate *= 1.0 + rate;
                                    }
                                }
                            }
                        } else if debuff.buff_type == "skill_damage_amplify" {
                            let b_skill_id =
                                status_effect_values.first().cloned().unwrap_or_default();
                            let b_skill_effect_id =
                                status_effect_values.get(4).cloned().unwrap_or_default();
                            if (b_skill_id == 0 || b_skill_id == skill_id as i32)
                                && (b_skill_effect_id == 0
                                    || b_skill_effect_id == skill_effect_id as i32)
                            {
                                if let Some(val) =
                                    status_effect_values.get(1).cloned().filter(|&v| v != 0)
                                {
                                    let rate =
                                        (val as f64 / 10000.0) * status_effect.stack_count as f64;
                                    rdps_data.multi_dmg.values.push(RdpsBuffData {
                                        caster: caster_encounter_entity.name.clone(),
                                        rate,
                                    });
                                    rdps_data.multi_dmg.sum_rate += rate;
                                    rdps_data.multi_dmg.total_rate *= 1.0 + rate;
                                }
                            }
                        }

                        if debuff.buff_type == "directional_attack_amplify" {
                            if hit_option == HitOption::FRONTAL_ATTACK {
                                if let Some(front_rate) =
                                    status_effect_values.first().cloned().filter(|&v| v != 0)
                                {
                                    let rate = (front_rate as f64 / 100.0)
                                        * status_effect.stack_count as f64;
                                    rdps_data.multi_dmg.values.push(RdpsBuffData {
                                        caster: caster_encounter_entity.name.clone(),
                                        rate,
                                    });
                                    rdps_data.multi_dmg.sum_rate += rate;
                                    rdps_data.multi_dmg.total_rate *= 1.0 + rate;
                                }
                            }
                            if hit_option == HitOption::BACK_ATTACK {
                                if let Some(back_rate) =
                                    status_effect_values.get(4).cloned().filter(|&v| v != 0)
                                {
                                    let rate = (back_rate as f64 / 100.0)
                                        * status_effect.stack_count as f64;
                                    rdps_data.multi_dmg.values.push(RdpsBuffData {
                                        caster: caster_encounter_entity.name.clone(),
                                        rate,
                                    });
                                    rdps_data.multi_dmg.sum_rate += rate;
                                    rdps_data.multi_dmg.total_rate *= 1.0 + rate;
                                }
                            }
                        }
                    }

                    if let (true, Some(skill_data)) =
                        (!rdps_data.crit.values.is_empty(), skill_data)
                    {
                        let ce_conditional_data = CombatEffectConditionData {
                            self_entity: dmg_src_entity,
                            target_entity: dmg_target_entity,
                            caster_entity: dmg_src_entity,
                            skill: Some(&skill_data),
                            hit_option: hit_option_raw,
                            target_count,
                        };
                        for option in dmg_src_entity.item_set.iter().flatten() {
                            if option.option_type == "stat"
                                && option.key_stat == "critical_dam_rate"
                            {
                                rdps_data.crit_dmg_rate += option.value as f64 / 10000.0;
                            } else if option.option_type == "combat_effect" {
                                if let Some(ce) = COMBAT_EFFECT_DATA.get(&option.key_index) {
                                    let crit_multiplier = get_crit_multiplier_from_combat_effect(
                                        ce,
                                        &ce_conditional_data,
                                    );
                                    rdps_data.crit_dmg_rate += crit_multiplier;
                                }
                            }

                            if let Some(tripod_data) = skill.tripod_data.as_ref() {
                                calculate_tripod_data(
                                    tripod_data,
                                    &mut rdps_data,
                                    skill_effect_id,
                                    &ce_conditional_data,
                                );
                            }
                        }
                    }

                    if !rdps_data.skill_dmg_rate.values.is_empty() {
                        let additional_damage = dmg_src_stats.stats.add_dmg as f64;
                        rdps_data.skill_dmg_rate.self_sum_rate += additional_damage / 10000.0;
                    }

                    let mut crit_sum_eff_gain_rate = 0.0;
                    if !rdps_data.crit.values.is_empty() {
                        let crit_stat_value = dmg_src_stats.stats.crit as f64;
                        rdps_data.crit.self_sum_rate += crit_stat_value / 0.2794 / 10000.0;
                        let capped_sum_rate = 0.0_f64
                            .max(1.0 - rdps_data.crit.self_sum_rate)
                            .min(rdps_data.crit.sum_rate);
                        crit_sum_eff_gain_rate = (capped_sum_rate * rdps_data.crit_dmg_rate
                            - capped_sum_rate)
                            / (rdps_data.crit.self_sum_rate * rdps_data.crit_dmg_rate
                                - rdps_data.crit.self_sum_rate
                                + 1.0);
                    }

                    let attack_power_amplify = if rdps_data.atk_pow_amplify.is_empty() {
                        RdpsBuffData {
                            caster: "".to_string(),
                            rate: 0.0,
                        }
                    } else {
                        rdps_data
                            .atk_pow_amplify
                            .iter()
                            .max_by(|a, b| a.rate.partial_cmp(&b.rate).unwrap_or(Ordering::Equal))
                            .unwrap()
                            .clone()
                    };

                    let total_eff_gain_rate = (1.0 + crit_sum_eff_gain_rate)
                        * (1.0
                            + rdps_data.atk_pow_sub_rate_2.sum_rate
                                / (1.0 + rdps_data.atk_pow_sub_rate_2.self_sum_rate))
                        * (1.0
                            + rdps_data.skill_dmg_rate.sum_rate
                                / (1.0 + rdps_data.skill_dmg_rate.self_sum_rate))
                        * (1.0 + attack_power_amplify.rate)
                        * rdps_data.multi_dmg.total_rate
                        * rdps_data.atk_pow_sub_rate_1.total_rate
                        - 1.0;
                    let total_sum_gain_rate = crit_sum_eff_gain_rate
                        + (rdps_data.atk_pow_sub_rate_2.sum_rate
                            / (1.0 + rdps_data.atk_pow_sub_rate_2.self_sum_rate))
                        + (rdps_data.skill_dmg_rate.sum_rate
                            / (1.0 + rdps_data.skill_dmg_rate.self_sum_rate))
                        + attack_power_amplify.rate
                        + (rdps_data.multi_dmg.total_rate - 1.0)
                        + (rdps_data.atk_pow_sub_rate_1.total_rate - 1.0);

                    let unit_rate = (total_eff_gain_rate * damage as f64)
                        / (total_sum_gain_rate * (1.0 + total_eff_gain_rate));
                    let crit_gain_unit =
                        (crit_sum_eff_gain_rate * unit_rate) / rdps_data.crit.sum_rate;
                    for crit in rdps_data.crit.values {
                        let delta = crit.rate * crit_gain_unit;
                        apply_rdps(
                            &mut source_entity,
                            self.encounter.entities.get_mut(&crit.caster),
                            skill_id,
                            delta,
                            &mut skill_hit,
                        );
                    }

                    for dmg in rdps_data.atk_pow_sub_rate_2.values {
                        let delta = (dmg.rate / (1.0 + rdps_data.atk_pow_sub_rate_2.self_sum_rate))
                            * unit_rate;
                        apply_rdps(
                            &mut source_entity,
                            self.encounter.entities.get_mut(&dmg.caster),
                            skill_id,
                            delta,
                            &mut skill_hit,
                        );
                    }

                    for dmg in rdps_data.skill_dmg_rate.values {
                        let delta =
                            (dmg.rate / (1.0 + rdps_data.skill_dmg_rate.self_sum_rate)) * unit_rate;
                        apply_rdps(
                            &mut source_entity,
                            self.encounter.entities.get_mut(&dmg.caster),
                            skill_id,
                            delta,
                            &mut skill_hit,
                        );
                    }

                    let mult_gain_unit = ((rdps_data.multi_dmg.total_rate - 1.0) * unit_rate)
                        / rdps_data.multi_dmg.sum_rate;
                    for dmg in rdps_data.multi_dmg.values {
                        let delta = dmg.rate * mult_gain_unit;
                        apply_rdps(
                            &mut source_entity,
                            self.encounter.entities.get_mut(&dmg.caster),
                            skill_id,
                            delta,
                            &mut skill_hit,
                        );
                    }

                    let atk_pow_sub_rate_1_gain_unit =
                        ((rdps_data.atk_pow_sub_rate_1.total_rate - 1.0) * unit_rate)
                            / rdps_data.atk_pow_sub_rate_1.sum_rate;
                    for dmg in rdps_data.atk_pow_sub_rate_1.values {
                        let delta = dmg.rate * atk_pow_sub_rate_1_gain_unit;
                        apply_rdps(
                            &mut source_entity,
                            self.encounter.entities.get_mut(&dmg.caster),
                            skill_id,
                            delta,
                            &mut skill_hit,
                        );
                    }

                    if attack_power_amplify.rate > 0.0 {
                        let delta = attack_power_amplify.rate * unit_rate;
                        apply_rdps(
                            &mut source_entity,
                            self.encounter
                                .entities
                                .get_mut(&attack_power_amplify.caster),
                            skill_id,
                            delta,
                            &mut skill_hit,
                        );
                    }
                } else if dmg_src_entity.entity_type == EntityType::PLAYER
                    && self.encounter.last_combat_packet - self.encounter.fight_start
                        > RDPS_VALID_LIMIT
                {
                    warn!(
                        "{:?} is not in player_stats. [{}]",
                        dmg_src_entity.name,
                        self.encounter.last_combat_packet - self.encounter.fight_start
                    );
                    self.rdps_valid = false;

                    if !self.rdps_valid {
                        self.window
                            .emit("rdps", "invalid_stats")
                            .expect("failed to emit rdps message");
                    }
                }
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

        self.encounter
            .entities
            .insert(source_entity.name.clone(), source_entity);
        self.encounter
            .entities
            .insert(target_entity.name.clone(), target_entity);
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

    pub fn on_identity_gain(&mut self, pkt: &PKTIdentityGaugeChangeNotify) {
        if self.encounter.fight_start == 0 {
            return;
        }

        if self.encounter.local_player.is_empty() {
            if let Some((_, entity)) = self
                .encounter
                .entities
                .iter()
                .find(|(_, e)| e.id == pkt.player_id)
            {
                self.encounter.local_player.clone_from(&entity.name);
            } else {
                return;
            }
        }

        if let Some(entity) = self
            .encounter
            .entities
            .get_mut(&self.encounter.local_player)
        {
            self.identity_log
                .entry(entity.name.clone())
                .or_default()
                .push((
                    Utc::now().timestamp_millis(),
                    (
                        pkt.identity_gauge1,
                        pkt.identity_gauge2,
                        pkt.identity_gauge3,
                    ),
                ));
        }
    }

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
            let mut target_entity_state = self
                .encounter
                .entities
                .entry(target_entity.name.clone())
                .or_insert_with(|| encounter_entity_from_entity(target_entity))
                .to_owned();
            let mut source_entity_state = self
                .encounter
                .entities
                .entry(source_entity.name.clone())
                .or_insert_with(|| encounter_entity_from_entity(source_entity))
                .to_owned();

            if !self
                .encounter
                .encounter_damage_stats
                .applied_shield_buffs
                .contains_key(&buff_id)
            {
                let mut source_id: Option<u32> = None;
                let original_buff_id = if let Some(deref_id) = self.custom_id_map.get(&buff_id) {
                    source_id = Some(get_skill_id(buff_id));
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

            if source_entity.id == target_entity.id {
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

                self.encounter
                    .entities
                    .insert(source_entity_state.name.clone(), source_entity_state);
            } else {
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

                self.encounter
                    .entities
                    .insert(target_entity_state.name.clone(), target_entity_state);
                self.encounter
                    .entities
                    .insert(source_entity_state.name.clone(), source_entity_state);
            }

            self.encounter.encounter_damage_stats.total_shielding += shield;
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
            let mut target_entity_state = self
                .encounter
                .entities
                .entry(target_entity.name.clone())
                .or_insert_with(|| encounter_entity_from_entity(target_entity))
                .to_owned();
            let mut source_entity_state = self
                .encounter
                .entities
                .entry(source_entity.name.clone())
                .or_insert_with(|| encounter_entity_from_entity(source_entity))
                .to_owned();

            if source_entity.id == target_entity.id {
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

                self.encounter
                    .entities
                    .insert(source_entity_state.name.clone(), source_entity_state);
            } else {
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

                self.encounter
                    .entities
                    .insert(target_entity_state.name.clone(), target_entity_state);
                self.encounter
                    .entities
                    .insert(source_entity_state.name.clone(), source_entity_state);
            }

            self.encounter
                .encounter_damage_stats
                .total_effective_shielding += shield_removed;
        }
    }

    pub fn save_to_db(&mut self, player_stats: Option<Cache<String, PlayerStats>>, manual: bool) {
        if !manual {
            if self.encounter.fight_start == 0
                || self.encounter.current_boss_name.is_empty()
                || !self
                    .encounter
                    .entities
                    .contains_key(&self.encounter.current_boss_name)
                || !self
                    .encounter
                    .entities
                    .values()
                    .any(|e| e.entity_type == EntityType::PLAYER && e.damage_stats.damage_dealt > 0)
            {
                return;
            }

            if let Some(current_boss) = self
                .encounter
                .entities
                .get(&self.encounter.current_boss_name)
            {
                if current_boss.current_hp == current_boss.max_hp {
                    return;
                }
            }
        }

        let encounter = self.encounter.clone();
        let mut path = self
            .window
            .app_handle()
            .path_resolver()
            .resource_dir()
            .expect("could not get resource dir");
        path.push("encounters.db");
        let prev_stagger = self.prev_stagger;

        let damage_log = self.damage_log.clone();
        let identity_log = self.identity_log.clone();
        let cast_log = self.cast_log.clone();
        let boss_hp_log = self.boss_hp_log.clone();
        let stagger_log = self.stagger_log.clone();
        let stagger_intervals = self.stagger_intervals.clone();
        let raid_clear = self.raid_clear;
        let party_info = self.party_info.clone();
        let raid_difficulty = self.raid_difficulty.clone();
        let region = self.region.clone();
        let meter_version = self.window.app_handle().package_info().version.to_string();

        let ntp_fight_start = self.ntp_fight_start;

        let rdps_valid = self.rdps_valid;

        let skill_cast_log = self.skill_tracker.get_cast_log();

        // debug_print(format_args!("skill cast log:\n{}", serde_json::to_string(&skill_cast_log).unwrap()));

        debug_print(format_args!("rdps_data valid: [{}]", rdps_valid));
        info!(
            "saving to db - cleared: [{}], difficulty: [{}] {}",
            raid_clear, self.raid_difficulty, encounter.current_boss_name
        );

        let window = self.window.clone();
        task::spawn(async move {
            let mut conn = Connection::open(path).expect("failed to open database");
            let tx = conn.transaction().expect("failed to create transaction");

            let encounter_id = insert_data(
                &tx,
                encounter,
                prev_stagger,
                damage_log,
                identity_log,
                cast_log,
                boss_hp_log,
                stagger_log,
                stagger_intervals,
                raid_clear,
                party_info,
                raid_difficulty,
                region,
                player_stats,
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
                    .emit("clear-encounter", encounter_id)
                    .expect("failed to emit clear-encounter");
            }
        });
    }
}
