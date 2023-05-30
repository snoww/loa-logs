use std::cmp::{max, Ordering};

use crate::parser::entity_tracker::Entity;
use crate::parser::models::*;
use chrono::Utc;
use hashbrown::HashMap;
use meter_core::packets::definitions::{PKTIdentityGaugeChangeNotify, PKTParalyzationStateNotify};
use rusqlite::{params, Connection, Transaction};
use serde_json::json;
use tauri::{Manager, Window, Wry};
use tokio::task;

const WINDOW_MS: i64 = 5_000;
const WINDOW_S: i64 = 5;

#[derive(Debug)]
pub struct EncounterState {
    pub window: Window<Wry>,
    pub encounter: Encounter,
    pub raid_end: bool,
    pub saved: bool,

    prev_stagger: i32,

    damage_log: HashMap<String, Vec<(i64, i64)>>,
    identity_log: HashMap<String, Vec<(i64, (u32, u32, u32))>>,
    cast_log: HashMap<String, HashMap<i32, Vec<i32>>>,

    stagger_log: Vec<(i32, f32)>,
    stagger_intervals: Vec<(i32, i32)>,
}

impl EncounterState {
    pub fn new(window: Window<Wry>) -> EncounterState {
        EncounterState {
            window,
            encounter: Encounter::default(),
            raid_end: false,
            saved: false,

            prev_stagger: 0,
            damage_log: HashMap::new(),
            identity_log: HashMap::new(),
            cast_log: HashMap::new(),
            stagger_log: Vec::new(),
            stagger_intervals: Vec::new(),
        }
    }

    // reset everything except local player
    fn reset(&mut self, clone: &Encounter) {
        self.encounter.fight_start = 0;
        self.encounter.entities = HashMap::new();
        self.encounter.current_boss_name = "".to_string();
        self.encounter.encounter_damage_stats = Default::default();
        self.encounter.reset = false;
        self.prev_stagger = 0;

        self.damage_log = HashMap::new();
        self.identity_log = HashMap::new();
        self.cast_log = HashMap::new();
        self.stagger_log = Vec::new();
        self.stagger_intervals = Vec::new();

        if !clone.local_player.is_empty() {
            if let Some(player) = clone.entities.get(&clone.local_player) {
                self.encounter.local_player = clone.local_player.clone();
                self.encounter.entities.insert(
                    player.name.clone(),
                    EncounterEntity {
                        id: player.id,
                        name: player.name.clone(),
                        class: player.class.clone(),
                        class_id: player.class_id,
                        current_hp: player.current_hp,
                        max_hp: player.max_hp,
                        entity_type: EntityType::PLAYER,
                        gear_score: player.gear_score,
                        ..Default::default()
                    },
                );
            }
        }
    }

    // keep all entities, reset all stats
    pub fn soft_reset(&mut self) {
        let clone = self.encounter.clone();
        self.reset(&clone);
        self.encounter.current_boss_name = "".to_string();
        for (key, entity) in clone.entities.into_iter().filter(|(_, e)| e.entity_type == EntityType::PLAYER) {
            self.encounter.entities.insert(
                key,
                EncounterEntity {
                    name: entity.name,
                    id: entity.id,
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

    pub fn update_local_player(&mut self, entity: &Entity) {
        if self.encounter.local_player == entity.name {
            return;
        }

        if let Some(mut local) = self.encounter.entities.remove(&self.encounter.local_player) {
            self.encounter.local_player = entity.name.to_string();
            local.name = entity.name.to_string();
            self.encounter.entities.insert(self.encounter.local_player.clone(), local);
        } else {
            let old_local = self.encounter.entities.iter()
                .find(|(_, e)| e.id == entity.id)
                .map(|(key, _)| key.clone());
    
            if let Some(old_local) = old_local {
                let mut new_local = self.encounter.entities[&old_local].clone();
                new_local.name = entity.name.to_string();
                self.encounter.entities.remove(&old_local);
                self.encounter.local_player = entity.name.to_string();
                self.encounter.entities.insert(self.encounter.local_player.clone(), new_local);
            }
        }
    }

    pub fn on_init_env(&mut self, entity: Entity) {
        if let Some(local_player) = self
            .encounter
            .entities
            .get_mut(&self.encounter.local_player)
        {
            local_player.id = entity.id;
            local_player.name = entity.name.clone();
            local_player.class_id = entity.class_id;
            local_player.class = get_class_from_id(&entity.class_id);
        } else {
            let entity = encounter_entity_from_entity(&entity);
            self.encounter.local_player = entity.name.clone();
            self.encounter.entities.insert(entity.name.clone(), entity);
        }

        if !self.saved && !self.encounter.current_boss_name.is_empty() {
            self.save_to_db();
        }

        self.encounter.entities.retain(|_, e| {
            e.name == self.encounter.local_player
                || e.damage_stats.damage_dealt > 0
        });

        self.window
            .emit("zone-change", Some(self.encounter.clone()))
            .expect("failed to emit zone-change");

        self.encounter.current_boss_name = "".to_string();
        self.soft_reset();
    }

    pub fn on_phase_transition(&mut self, phase_code: i32) {
        self.window
            .emit("phase-transition", phase_code)
            .expect("failed to emit phase-transition");

        if phase_code == 0 || phase_code == 2 {
            if !self.encounter.current_boss_name.is_empty() {
                self.save_to_db();
                self.saved = true;
            }
            self.raid_end = true;
        }
    }

    pub fn on_init_pc(&mut self, entity: Entity, hp: i64, max_hp: i64) {
        self.encounter.entities.remove(&self.encounter.local_player);
        self.encounter.local_player = entity.name.clone();
        let mut player = encounter_entity_from_entity(&entity);
        player.current_hp = hp;
        player.max_hp = max_hp;
        self.encounter
            .entities
            .insert(player.name.clone(), player);
    }

    pub fn on_new_pc(&mut self, entity: Entity, hp: i64, max_hp: i64) {
        if let Some(player) = self.encounter.entities.get_mut(&entity.name) {
            player.id = entity.id;
            player.gear_score = entity.gear_level;
            player.current_hp = hp;
            player.max_hp = max_hp;
        } else {
            let mut player = encounter_entity_from_entity(&entity);
            player.current_hp = hp;
            player.max_hp = max_hp;
            self.encounter
                .entities
                .insert(player.name.clone(), player);
        }
    }

    pub fn on_new_npc(&mut self, entity: Entity, hp: i64, max_hp: i64) {
        let entity_name = entity.name.clone();
        self.encounter
            .entities
            .entry(entity_name.clone())
            .and_modify(|e| {
                if (e.npc_id as i32 - entity.npc_id as i32).abs() < 50 || entity.entity_type != EntityType::BOSS {
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
                // get the npc that we just added
                if self.encounter.current_boss_name.is_empty() {
                    self.encounter.current_boss_name = entity_name;
                }
                // get the current boss
                else if let Some(boss) = self
                    .encounter
                    .entities
                    .get(&self.encounter.current_boss_name)
                {
                    // check if the new npc has more hp than the current boss or if the current boss is dead
                    if npc.max_hp >= boss.max_hp || boss.is_dead {
                        self.encounter.current_boss_name = entity_name;
                    }
                } else {
                    self.encounter.current_boss_name = entity_name;
                }
            }
        }
    }

    pub fn on_death(&mut self, dead_entity: &Entity) {
        if let Some(entity) = self.encounter.entities.get_mut(&dead_entity.name) {
            if entity.id != dead_entity.id {
                return;
            }
            let deaths = if entity.is_dead {
                entity.damage_stats.deaths + 1
            } else {
                1
            };
            entity.is_dead = true;
            entity.damage_stats.deaths = deaths;
            entity.damage_stats.death_time = Utc::now().timestamp_millis();
        } else {
            let mut entity = encounter_entity_from_entity(dead_entity);
            entity.is_dead = true;
            entity.damage_stats = DamageStats {
                deaths: 1,
                death_time: Utc::now().timestamp_millis(),
                ..Default::default()
            };
            self.encounter
                .entities
                .insert(dead_entity.name.clone(), entity);
        }
    }

    pub fn on_skill_start(&mut self, source_entity: Entity, skill_id: i32, timestamp: i64) {
        let skill_name = get_skill_name(&skill_id);
        let mut entity = self
            .encounter
            .entities
            .entry(source_entity.name.clone())
            .or_insert_with(|| {
                let (skill_name, skill_icon) =
                    get_skill_name_and_icon(&skill_id,& 0, skill_name.clone());
                let mut entity = encounter_entity_from_entity(&source_entity);
                entity.skill_stats = SkillStats {
                    casts: 0,
                    ..Default::default()
                };
                entity.skills = HashMap::from([(
                    skill_id,
                    Skill {
                        id: skill_id,
                        name: skill_name,
                        icon: skill_icon,
                        casts: 0,
                        ..Default::default()
                    },
                )]);
                entity
            });

        if entity.entity_type == EntityType::PLAYER && entity.class_id == 0 {
            entity.class_id = source_entity.class_id;
            entity.class = get_class_from_id(&source_entity.class_id);
            entity.entity_type = source_entity.entity_type;
        }

        entity.is_dead = false;
        entity.skill_stats.casts += 1;

        let duration = if self.encounter.fight_start == 0 {
            0
        } else {
            ((timestamp - self.encounter.fight_start) / 1000) as i32
        };

        // if skills have different ids but the same name, we group them together
        // dunno if this is right approach xd
        if let Some(skill) = entity.skills.get_mut(&skill_id) {
            skill.casts += 1;
            self.cast_log
                .entry(entity.name.clone())
                .or_default()
                .entry(skill_id)
                .or_default()
                .push(duration);
        } else if let Some(skill) = entity
            .skills
            .values_mut()
            .find(|s| s.name == skill_name.clone())
        {
            skill.casts += 1;
            self.cast_log
                .entry(entity.name.clone())
                .or_default()
                .entry(skill_id)
                .or_default()
                .push(duration);
        } else {
            let (skill_name, skill_icon) = get_skill_name_and_icon(&skill_id, &0, skill_name.clone());
            self.cast_log
                .entry(entity.name.clone())
                .or_default()
                .entry(skill_id)
                .or_default()
                .push(duration);
            entity.skills.insert(
                skill_id,
                Skill {
                    id: skill_id,
                    name: skill_name,
                    icon: skill_icon,
                    casts: 1,
                    ..Default::default()
                },
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn on_damage(
        &mut self,
        dmg_src_entity: &Entity,
        proj_entity: &Entity,
        dmg_target_entity: &Entity,
        damage: i64,
        skill_id: i32,
        skill_effect_id: i32,
        modifier: i32,
        target_current_hp: i64,
        target_max_hp: i64,
        se_on_source: Vec<(u32, u64)>,
        se_on_target: Vec<(u32, u64)>,
    ) {
        let hit_flag = match modifier & 0xf {
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
        let hit_option = match ((modifier >> 4) & 0x7) - 1 {
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
        if hit_flag == HitFlag::DAMAGE_SHARE && skill_id == 0 && skill_effect_id == 0 {
            return;
        }

        let mut skill_effect_id = skill_effect_id;
        if is_battle_item(skill_effect_id, "attack")
            && proj_entity.entity_type == EntityType::PROJECTILE
        {
            skill_effect_id = proj_entity.skill_effect_id as i32;
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
                target_entity.current_hp = target_current_hp;
                target_entity.max_hp = target_max_hp;
                target_entity
            })
            .to_owned();

        source_entity.id = dmg_src_entity.id;

        let timestamp = Utc::now().timestamp_millis();

        if self.encounter.fight_start == 0 {
            self.encounter.fight_start = timestamp;
            self.window
                .emit("raid-start", timestamp)
                .expect("failed to emit raid-start");
        }

        // test
        if target_entity.id == dmg_target_entity.id {
            target_entity.current_hp = target_current_hp;
            target_entity.max_hp = target_max_hp;
        }

        let mut damage = damage;
        if target_entity.entity_type != EntityType::PLAYER && target_current_hp < 0 {
            damage += target_current_hp;
        }

        let mut skill_id = if skill_id != 0 {
            skill_id
        } else {
            skill_effect_id
        };

        let skill_name = get_skill_name(&skill_id);
        if !source_entity.skills.contains_key(&skill_id) {
            if let Some(skill) = source_entity
                .skills
                .values()
                .find(|&s| s.name == *skill_name)
            {
                skill_id = skill.id;
            } else {
                let (skill_name, skill_icon) =
                    get_skill_name_and_icon(&skill_id, &skill_effect_id, skill_name.clone());
                let duration = if self.encounter.fight_start == 0 {
                    0
                } else {
                    ((timestamp - self.encounter.fight_start) / 1000) as i32
                };
                self.cast_log
                    .entry(source_entity.name.clone())
                    .or_default()
                    .entry(skill_id)
                    .or_default()
                    .push(duration);
                source_entity.skills.insert(
                    skill_id,
                    Skill {
                        id: skill_id,
                        name: skill_name,
                        icon: skill_icon,
                        casts: 1,
                        ..Default::default()
                    },
                );
            }
        }

        let skill = source_entity.skills.get_mut(&skill_id).unwrap();

        skill.total_damage += damage;
        if damage > skill.max_damage {
            skill.max_damage = damage;
        }

        source_entity.damage_stats.damage_dealt += damage;
        target_entity.damage_stats.damage_taken += damage;

        source_entity.skill_stats.hits += 1;
        skill.hits += 1;

        if hit_flag == HitFlag::CRITICAL || hit_flag == HitFlag::DOT_CRITICAL {
            source_entity.skill_stats.crits += 1;
            skill.crits += 1;
        }
        if hit_option == HitOption::BACK_ATTACK {
            source_entity.skill_stats.back_attacks += 1;
            skill.back_attacks += 1;
        }
        if hit_option == HitOption::FRONTAL_ATTACK {
            source_entity.skill_stats.front_attacks += 1;
            skill.front_attacks += 1;
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
            let mut is_debuffed_by_support = false;
            let se_on_source = se_on_source
                .iter()
                .map(|(se, _)| (*se) as i32)
                .collect::<Vec<_>>();
            for buff_id in se_on_source.iter() {
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
                    if let Some(status_effect) = get_status_effect_data(*buff_id) {
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
                            is_buffed_by_support = (buff.buff_category == "classskill"
                                || buff.buff_category == "identity"
                                || buff.buff_category == "ability")
                                && buff.target == StatusEffectTarget::PARTY
                                && is_support_class_id(skill.class_id);
                        }
                    }
                }
            }
            let se_on_target = se_on_target
                .iter()
                .map(|(se, _)| (*se) as i32)
                .collect::<Vec<_>>();
            for debuff_id in se_on_target.iter() {
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
                    if let Some(status_effect) = get_status_effect_data(*debuff_id) {
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
                            is_debuffed_by_support = (debuff.buff_category == "classskill"
                                || debuff.buff_category == "identity"
                                || debuff.buff_category == "ability")
                                && debuff.target == StatusEffectTarget::PARTY
                                && is_support_class_id(skill.class_id);
                        }
                    }
                }
            }

            if is_buffed_by_support {
                skill.buffed_by_support += damage;
                source_entity.damage_stats.buffed_by_support += damage;
            }
            if is_debuffed_by_support {
                skill.debuffed_by_support += damage;
                source_entity.damage_stats.debuffed_by_support += damage;
            }

            for buff_id in se_on_source.into_iter() {
                skill
                    .buffed_by
                    .entry(buff_id)
                    .and_modify(|e| *e += damage)
                    .or_insert(damage);
                source_entity
                    .damage_stats
                    .buffed_by
                    .entry(buff_id)
                    .and_modify(|e| *e += damage)
                    .or_insert(damage);
            }
            for debuff_id in se_on_target.into_iter() {
                skill
                    .debuffed_by
                    .entry(debuff_id)
                    .and_modify(|e| *e += damage)
                    .or_insert(damage);
                source_entity
                    .damage_stats
                    .debuffed_by
                    .entry(debuff_id)
                    .and_modify(|e| *e += damage)
                    .or_insert(damage);
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
            self.encounter.current_boss_name = target_entity.name.clone();
        }

        self.encounter
            .entities
            .insert(source_entity.name.clone(), source_entity);
        self.encounter
            .entities
            .insert(target_entity.name.clone(), target_entity);

        self.encounter.last_combat_packet = timestamp;
    }
    //
    // fn on_heal(&mut self, _line: &[&str]) {
    //     println!("Heal");
    // }
    //
    // fn on_buff(&mut self, _line: &[&str]) {
    //     println!("Buff");
    // }
    //
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

    pub fn on_identity_gain(&mut self, pkt: PKTIdentityGaugeChangeNotify) {
        if self.encounter.local_player.is_empty() || self.encounter.fight_start == 0 {
            return;
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

    pub fn on_stagger_change(&mut self, pkt: PKTParalyzationStateNotify) {
        if self.encounter.current_boss_name.is_empty() || self.encounter.fight_start == 0 {
            return;
        }

        if let Some(boss) = self
            .encounter
            .entities
            .get_mut(&self.encounter.current_boss_name)
        {
            let timestamp = Utc::now().timestamp_millis();
            let current_stagger = pkt.paralyzation_point as i32;
            let max_stagger = pkt.paralyzation_max_point as i32;
            if boss.id == pkt.object_id {
                if current_stagger == max_stagger {
                    let staggered_in =
                        (timestamp - self.encounter.encounter_damage_stats.stagger_start) / 1000;
                    self.stagger_intervals
                        .push((staggered_in as i32, max_stagger))
                } else if current_stagger != 0 && self.prev_stagger == 0 {
                    self.encounter.encounter_damage_stats.stagger_start = timestamp;
                }

                self.prev_stagger = current_stagger;

                let relative_timestamp = (timestamp - self.encounter.fight_start) / 1000;
                let stagger_percent = (1.0 - (current_stagger as f32 / max_stagger as f32)) * 100.0;
                self.stagger_log
                    .push((relative_timestamp as i32, stagger_percent));

                if max_stagger > self.encounter.encounter_damage_stats.max_stagger {
                    self.encounter.encounter_damage_stats.max_stagger = max_stagger;
                }
            }
        }
    }

    fn save_to_db(&self) {
        if self.encounter.fight_start == 0
            || self.encounter.current_boss_name.is_empty()
            || !self
                .encounter
                .entities
                .contains_key(&self.encounter.current_boss_name)
            || !self.encounter.entities.values().any(|e| {
                e.entity_type == EntityType::PLAYER && e.damage_stats.damage_dealt > 0
            })
        {
            return;
        }
        let mut encounter = self.encounter.clone();
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
        let stagger_log = self.stagger_log.clone();
        let stagger_intervals = self.stagger_intervals.clone();

        task::spawn(async move {
            println!("saving to db - {}", encounter.current_boss_name);

            let mut conn = Connection::open(path).expect("failed to open database");
            let tx = conn.transaction().expect("failed to create transaction");

            insert_data(
                &tx,
                &mut encounter,
                prev_stagger,
                damage_log,
                identity_log,
                cast_log,
                stagger_log,
                stagger_intervals,
            );

            tx.commit().expect("failed to commit transaction");
            println!("saved to db");
        });
    }
}

fn encounter_entity_from_entity(entity: &Entity) -> EncounterEntity {
    EncounterEntity {
        id: entity.id,
        name: entity.name.clone(),
        entity_type: entity.entity_type,
        npc_id: entity.npc_id,
        class_id: entity.class_id,
        class: get_class_from_id(&entity.class_id),
        gear_score: entity.gear_level,
        ..Default::default()
    }
}

fn is_support_class_id(class_id: u32) -> bool {
    class_id == 105 || class_id == 204 || class_id == 602
}

fn is_battle_item(skill_effect_id: i32, _item_type: &str) -> bool {
    if let Some(item) = SKILL_EFFECT_DATA.get(&skill_effect_id) {
        if let Some(category) = item.item_category.as_ref() {
            return category == "useup_battle_item_common_attack";
        }
    }
    false
}

fn get_status_effect_data(buff_id: i32) -> Option<StatusEffect> {
    let buff = SKILL_BUFF_DATA.get(&buff_id);
    if buff.is_none() || buff.unwrap().icon_show_type == "none" {
        return None;
    }

    let buff = buff.unwrap();
    let buff_category = if buff.buff_category == "ability"
        && [501, 502, 503, 504, 505].contains(&buff.unique_group)
    {
        "dropsofether".to_string()
    } else {
        buff.buff_category.clone()
    };
    let mut status_effect = StatusEffect {
        target: {
            if buff.target == "none" {
                StatusEffectTarget::OTHER
            } else if buff.target == "self" {
                StatusEffectTarget::SELF
            } else {
                StatusEffectTarget::PARTY
            }
        },
        category: buff.category.clone(),
        buff_category: buff_category.clone(),
        buff_type: get_status_effect_buff_type_flags(buff),
        unique_group: buff.unique_group,
        source: StatusEffectSource {
            name: buff.name.clone(),
            desc: buff.desc.clone(),
            icon: buff.icon.clone(),
            ..Default::default()
        },
    };

    if buff_category == "classskill"
        || buff_category == "identity"
        || (buff_category == "ability" && buff.unique_group != 0)
    {
        if buff.source_skill.is_some() {
            let buff_source_skill = SKILL_DATA.get(&buff.source_skill.unwrap());
            if buff_source_skill.is_some() {
                status_effect.source.skill = buff_source_skill.cloned();
            }
        } else if let Some(buff_source_skill) = SKILL_DATA.get(&(buff_id / 10)) {
            status_effect.source.skill = Some(buff_source_skill.clone());
        } else if let Some(buff_source_skill) = SKILL_DATA.get(&((buff_id / 100) * 10)) {
            status_effect.source.skill = Some(buff_source_skill.clone());
        } else {
            let skill_id = buff.unique_group / 10;
            let buff_source_skill = SKILL_DATA.get(&skill_id);
            status_effect.source.skill = buff_source_skill.cloned();
        }
    } else if buff_category == "set" && buff.set_name.is_some() {
        status_effect.source.set_name = buff.set_name.clone();
    } else if buff_category == "battleitem" {
        if let Some(buff_source_item) = SKILL_EFFECT_DATA.get(&buff_id) {
            if let Some(item_name) = buff_source_item.item_name.as_ref() {
                status_effect.source.name = item_name.clone();
            }
            if let Some(item_desc) = buff_source_item.item_desc.as_ref() {
                status_effect.source.desc = item_desc.clone();
            }
            if let Some(icon) = buff_source_item.icon.as_ref() {
                status_effect.source.icon = icon.clone();
            }
        }
    }

    Some(status_effect)
}

fn get_status_effect_buff_type_flags(buff: &SkillBuffData) -> u32 {
    let dmg_buffs = [
        "weaken_defense",
        "weaken_resistance",
        "skill_damage_amplify",
        "beattacked_damage_amplify",
        "skill_damage_amplify_attack",
        "directional_attack_amplify",
        "instant_stat_amplify",
        "attack_power_amplify",
        "instant_stat_amplify_by_contents",
    ];

    let mut buff_type = StatusEffectBuffTypeFlags::NONE;
    if dmg_buffs.contains(&buff.buff_type.as_str()) {
        buff_type |= StatusEffectBuffTypeFlags::DMG;
    } else if ["move_speed_down", "all_speed_down"].contains(&buff.buff_type.as_str()) {
        buff_type |= StatusEffectBuffTypeFlags::MOVESPEED;
    } else if buff.buff_type == "reset_cooldown" {
        buff_type |= StatusEffectBuffTypeFlags::COOLDOWN;
    } else if ["change_ai_point", "ai_point_amplify"].contains(&buff.buff_type.as_str()) {
        buff_type |= StatusEffectBuffTypeFlags::STAGGER;
    } else if buff.buff_type == "increase_identity_gauge" {
        buff_type |= StatusEffectBuffTypeFlags::RESOURCE;
    }

    for option in buff.passive_option.iter() {
        let key_stat_str = option.key_stat.as_str();
        let option_type = option.option_type.as_str();
        if option_type == "stat" {
            let stat = STAT_TYPE_MAP.get(key_stat_str);
            if stat.is_none() {
                continue;
            }
            let stat = stat.unwrap().to_owned();
            if ["mastery", "mastery_x", "paralyzation_point_rate"].contains(&key_stat_str) {
                buff_type |= StatusEffectBuffTypeFlags::STAGGER;
            } else if ["rapidity", "rapidity_x", "cooldown_reduction"].contains(&key_stat_str) {
                buff_type |= StatusEffectBuffTypeFlags::COOLDOWN;
            } else if [
                "max_mp",
                "max_mp_x",
                "max_mp_x_x",
                "normal_mp_recovery",
                "combat_mp_recovery",
                "normal_mp_recovery_rate",
                "combat_mp_recovery_rate",
                "resource_recovery_rate",
            ]
            .contains(&key_stat_str)
            {
                buff_type |= StatusEffectBuffTypeFlags::RESOURCE;
            } else if [
                "con",
                "con_x",
                "max_hp",
                "max_hp_x",
                "max_hp_x_x",
                "normal_hp_recovery",
                "combat_hp_recovery",
                "normal_hp_recovery_rate",
                "combat_hp_recovery_rate",
                "self_recovery_rate",
                "drain_hp_dam_rate",
                "vitality",
            ]
            .contains(&key_stat_str)
            {
                buff_type |= StatusEffectBuffTypeFlags::HP;
            } else if STAT_TYPE_MAP["def"] <= stat && stat <= STAT_TYPE_MAP["magical_inc_rate"]
                || ["endurance", "endurance_x"].contains(&option.key_stat.as_str())
            {
                if buff.category == "buff" && option.value >= 0
                    || buff.category == "debuff" && option.value <= 0
                {
                    buff_type |= StatusEffectBuffTypeFlags::DMG;
                } else {
                    buff_type |= StatusEffectBuffTypeFlags::DEFENSE;
                }
            } else if STAT_TYPE_MAP["move_speed"] <= stat
                && stat <= STAT_TYPE_MAP["vehicle_move_speed_rate"]
            {
                buff_type |= StatusEffectBuffTypeFlags::MOVESPEED;
            }
            if [
                "attack_speed",
                "attack_speed_rate",
                "rapidity",
                "rapidity_x",
            ]
            .contains(&key_stat_str)
            {
                buff_type |= StatusEffectBuffTypeFlags::ATKSPEED;
            } else if ["critical_hit_rate", "criticalhit", "criticalhit_x"].contains(&key_stat_str)
            {
                buff_type |= StatusEffectBuffTypeFlags::CRIT;
            } else if STAT_TYPE_MAP["attack_power_sub_rate_1"] <= stat
                && stat <= STAT_TYPE_MAP["skill_damage_sub_rate_2"]
                || STAT_TYPE_MAP["fire_dam_rate"] <= stat
                    && stat <= STAT_TYPE_MAP["elements_dam_rate"]
                || [
                    "str",
                    "agi",
                    "int",
                    "str_x",
                    "agi_x",
                    "int_x",
                    "char_attack_dam",
                    "attack_power_rate",
                    "skill_damage_rate",
                    "attack_power_rate_x",
                    "skill_damage_rate_x",
                    "hit_rate",
                    "dodge_rate",
                    "critical_dam_rate",
                    "awakening_dam_rate",
                    "attack_power_addend",
                    "weapon_dam",
                ]
                .contains(&key_stat_str)
            {
                if buff.category == "buff" && option.value >= 0
                    || buff.category == "debuff" && option.value <= 0
                {
                    buff_type |= StatusEffectBuffTypeFlags::DMG;
                } else {
                    buff_type |= StatusEffectBuffTypeFlags::DEFENSE;
                }
            }
        } else if option_type == "skill_critical_ratio" {
            buff_type |= StatusEffectBuffTypeFlags::CRIT;
        } else if [
            "skill_damage",
            "class_option",
            "skill_group_damage",
            "skill_critical_damage",
            "skill_penetration",
        ]
        .contains(&option_type)
        {
            if buff.category == "buff" && option.value >= 0
                || buff.category == "debuff" && option.value <= 0
            {
                buff_type |= StatusEffectBuffTypeFlags::DMG;
            } else {
                buff_type |= StatusEffectBuffTypeFlags::DEFENSE;
            }
        } else if ["skill_cooldown_reduction", "skill_group_cooldown_reduction"]
            .contains(&option_type)
        {
            buff_type |= StatusEffectBuffTypeFlags::COOLDOWN;
        } else if ["skill_mana_reduction", "mana_reduction"].contains(&option_type) {
            buff_type |= StatusEffectBuffTypeFlags::RESOURCE;
        } else if option_type == "combat_effect" {
            if let Some(combat_effect) = COMBAT_EFFECT_DATA.get(&option.key_index) {
                for action in combat_effect.actions.iter() {
                    if [
                        "modify_damage",
                        "modify_final_damage",
                        "modify_critical_multiplier",
                        "modify_penetration",
                        "modify_penetration_when_critical",
                        "modify_penetration_addend",
                        "modify_penetration_addend_when_critical",
                        "modify_damage_shield_multiplier",
                    ]
                    .contains(&action.action_type.as_str())
                    {
                        buff_type |= StatusEffectBuffTypeFlags::DMG;
                    } else if action.action_type == "modify_critical_ratio" {
                        buff_type |= StatusEffectBuffTypeFlags::CRIT;
                    }
                }
            }
        }
    }

    buff_type.bits()
}

fn get_skill_name_and_icon(
    skill_id: &i32,
    skill_effect_id: &i32,
    skill_name: String,
) -> (String, String) {
    if *skill_id == 0 && *skill_effect_id == 0 {
        ("Bleed".to_string(), "buff_168.png".to_string())
    } else if *skill_effect_id != 0 {
        return if let Some(effect) = SKILL_EFFECT_DATA.get(skill_effect_id) {
            if let Some(item_name) = effect.item_name.as_ref() {
                return (
                    item_name.clone(),
                    effect
                        .icon
                        .as_ref()
                        .unwrap_or(&String::from(""))
                        .clone(),
                );
            }
            if let Some(source_skill) = effect.source_skill {
                if let Some(skill) = SKILL_DATA.get(&source_skill) {
                    return (skill.name.clone(), skill.icon.clone());
                }
            } else if let Some(skill) = SKILL_DATA.get(&(skill_effect_id / 10)) {
                return (skill.name.clone(), skill.icon.clone());
            }
            (effect.comment.clone(), "".to_string())
        } else {
            (skill_name, "".to_string())
        };
    } else {
        return if let Some(skill) = SKILL_DATA.get(skill_id) {
            if let Some(summon_source_skill) = skill.summon_source_skill {
                if let Some(skill) = SKILL_DATA.get(&summon_source_skill) {
                    (skill.name.clone() + " (Summon)", skill.icon.clone())
                } else {
                    (skill_name, "".to_string())
                }
            } else if let Some(source_skill) = skill.source_skill {
                if let Some(skill) = SKILL_DATA.get(&source_skill) {
                    (skill.name.clone(), skill.icon.clone())
                } else {
                    (skill_name, "".to_string())
                }
            } else {
                (skill.name.clone(), skill.icon.clone())
            }
        } else if let Some(skill) = SKILL_DATA.get(&(skill_id - (skill_id % 10))) {
            (skill.name.clone(), skill.icon.clone())
        } else {
            (skill_name, "".to_string())
        };
    }
}

fn get_skill_name(skill_id: &i32) -> String {
    if let Some(skill) = SKILL_DATA.get(skill_id) {
        skill.name.clone()
    } else {
        "".to_string()
    }
}

#[allow(clippy::too_many_arguments)]
fn insert_data(
    tx: &Transaction,
    encounter: &mut Encounter,
    prev_stagger: i32,
    damage_log: HashMap<String, Vec<(i64, i64)>>,
    identity_log: HashMap<String, Vec<(i64, (u32, u32, u32))>>,
    cast_log: HashMap<String, HashMap<i32, Vec<i32>>>,
    stagger_log: Vec<(i32, f32)>,
    mut stagger_intervals: Vec<(i32, i32)>,
) {
    let mut encounter_stmt = tx
        .prepare_cached(
            "
    INSERT INTO encounter (
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
        misc
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        )
        .expect("failed to prepare encounter statement");

    encounter.duration = encounter.last_combat_packet - encounter.fight_start;
    let duration_seconds = encounter.duration / 1000;
    encounter.encounter_damage_stats.dps =
        encounter.encounter_damage_stats.total_damage_dealt / duration_seconds;

    let mut misc: Option<EncounterMisc> = None;

    if !stagger_log.is_empty() {
        if prev_stagger > 0 && prev_stagger != encounter.encounter_damage_stats.max_stagger {
            // never finished staggering the boss, calculate average from whatever stagger has been done
            let stagger_start_s = ((encounter.encounter_damage_stats.stagger_start
                - encounter.fight_start)
                / 1000) as i32;
            let stagger_duration = stagger_log.last().unwrap().0 - stagger_start_s;
            if stagger_duration > 0 {
                stagger_intervals.push((stagger_duration, prev_stagger));
            }
        }

        let (total_stagger_time, total_stagger_dealt) = stagger_intervals.iter().fold(
            (0, 0),
            |(total_time, total_stagger), (time, stagger)| {
                (total_time + time, total_stagger + stagger)
            },
        );

        if total_stagger_time > 0 {
            let stagger = StaggerStats {
                average: (total_stagger_dealt as f64 / total_stagger_time as f64)
                    / encounter.encounter_damage_stats.max_stagger as f64
                    * 100.0,
                staggers_per_min: (total_stagger_dealt as f64 / (total_stagger_time as f64 / 60.0))
                    / encounter.encounter_damage_stats.max_stagger as f64,
                log: stagger_log,
            };
            misc = Some(EncounterMisc {
                stagger_stats: Some(stagger),
            });
        }
    }

    // let boss_name = encounter.entities
    //     .iter()
    //     .filter(|&(_, e)| e.entity_type != EntityType::PLAYER)
    //     .max_by(|&(_, e1), &(_, e2)| e1.damage_stats.damage_taken.cmp(&e2.damage_stats.damage_taken))
    //     .unwrap();

    encounter_stmt
        .execute(params![
            encounter.last_combat_packet,
            encounter.fight_start,
            encounter.local_player,
            encounter.current_boss_name,
            encounter.duration,
            encounter.encounter_damage_stats.total_damage_dealt,
            encounter.encounter_damage_stats.top_damage_dealt,
            encounter.encounter_damage_stats.total_damage_taken,
            encounter.encounter_damage_stats.top_damage_taken,
            encounter.encounter_damage_stats.dps,
            json!(encounter.encounter_damage_stats.buffs),
            json!(encounter.encounter_damage_stats.debuffs),
            json!(misc)
        ])
        .expect("failed to insert encounter");

    let last_insert_id = tx.last_insert_rowid();

    let mut entity_stmt = tx
        .prepare_cached(
            "
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
        skill_stats
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        )
        .expect("failed to prepare entity statement");

    let fight_start = encounter.fight_start;
    let fight_end = encounter.last_combat_packet;

    for (_key, mut entity) in encounter.entities.iter_mut().filter(|(_, e)| {
        (e.entity_type == EntityType::PLAYER || e.entity_type == EntityType::ESTHER)
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
                            .push(damage / WINDOW_S);
                    }
                }
                let fight_start_sec = encounter.fight_start / 1000;
                let fight_end_sec = encounter.last_combat_packet / 1000;
                entity.damage_stats.dps_average =
                    calculate_average_dps(damage_log, fight_start_sec, fight_end_sec);

                for (_, mut skill) in entity.skills.iter_mut() {
                    skill.dps = skill.total_damage / duration_seconds;
                }
            }

            for (_, cast_log) in cast_log.iter().filter(|&(s, _)| *s == entity.name) {
                for (skill, log) in cast_log {
                    entity.skills.entry(*skill).and_modify(|e| {
                        e.cast_log = log.to_owned();
                    });
                }
            }
        }
        entity.damage_stats.dps = entity.damage_stats.damage_dealt / duration_seconds;

        if let Some(identity_log) = identity_log.get(&entity.name) {
            if entity.name == encounter.local_player && identity_log.len() >= 2 {
                let mut total_identity_gain = 0;
                let data = identity_log;
                let duration_seconds = (data[data.len() - 1].0 - data[0].0) / 1000;
                let max = match entity.class.as_str() {
                    "Summoner" => 7_000.0,
                    _ => 10_000.0,
                };
                let stats: String = match entity.class.as_str() {
                    "Arcanist" => {
                        let mut cards: HashMap<u32, u32> = HashMap::new();
                        let mut log: Vec<(i32, (f32, u32, u32))> = Vec::new();
                        for i in 1..data.len() {
                            let (t1, i1) = data[i - 1];
                            let (t2, i2) = data[i];

                            // don't count clown cards draws as card draws
                            if i2.1 != 0 && i2.1 != i1.1 && i1.1 != 19284 {
                                cards.entry(i2.1).and_modify(|e| *e += 1).or_insert(1);
                            }
                            if i2.2 != 0 && i2.2 != i1.2 && i1.2 != 19284 {
                                cards.entry(i2.2).and_modify(|e| *e += 1).or_insert(1);
                            }

                            if t2 > t1 && i2.0 > i1.0 {
                                total_identity_gain += i2.0 - i1.0;
                            }

                            let relative_time = ((t2 - fight_start) as f32 / 1000.0) as i32;
                            // calculate percentage, round to 2 decimal places
                            let percentage = if i2.0 >= max as u32 {
                                100.0
                            } else {
                                (((i2.0 as f32 / max) * 100.0) * 100.0).round() / 100.0
                            };
                            log.push((relative_time, (percentage, i2.1, i2.2)));
                        }

                        let avg_per_s = (total_identity_gain as f64 / duration_seconds as f64)
                            / max as f64
                            * 100.0;
                        let identity_stats = IdentityArcanist {
                            average: avg_per_s,
                            card_draws: cards,
                            log,
                        };

                        serde_json::to_string(&identity_stats).unwrap()
                    }
                    "Artist" | "Bard" => {
                        let mut log: Vec<(i32, (f32, u32))> = Vec::new();

                        for i in 1..data.len() {
                            let (t1, i1) = data[i - 1];
                            let (t2, i2) = data[i];

                            if t2 <= t1 {
                                continue;
                            }

                            if i2.0 > i1.0 {
                                total_identity_gain += i2.0 - i1.0;
                            }

                            let relative_time = ((t2 - fight_start) as f32 / 1000.0) as i32;
                            // since bard and artist have 3 bubbles, i.1 is the number of bubbles
                            // we scale percentage to 3 bubbles
                            // current bubble + max * number of bubbles
                            let percentage: f32 =
                                ((((i2.0 as f32 + max * i2.1 as f32) / max) * 100.0) * 100.0)
                                    .round()
                                    / 100.0;
                            log.push((relative_time, (percentage, i2.1)));
                        }

                        let avg_per_s = (total_identity_gain as f64 / duration_seconds as f64)
                            / max as f64
                            * 100.0;
                        let identity_stats = IdentityArtistBard {
                            average: avg_per_s,
                            log,
                        };
                        serde_json::to_string(&identity_stats).unwrap()
                    }
                    _ => {
                        let mut log: Vec<(i32, f32)> = Vec::new();
                        for i in 1..data.len() {
                            let (t1, i1) = data[i - 1];
                            let (t2, i2) = data[i];

                            if t2 <= t1 {
                                continue;
                            }

                            if i2.0 > i1.0 {
                                total_identity_gain += i2.0 - i1.0;
                            }

                            let relative_time = ((t2 - fight_start) as f32 / 1000.0) as i32;
                            let percentage =
                                (((i2.0 as f32 / max) * 100.0) * 100.0).round() / 100.0;
                            log.push((relative_time, percentage));
                        }

                        let avg_per_s = (total_identity_gain as f64 / duration_seconds as f64)
                            / max as f64
                            * 100.0;
                        let identity_stats = IdentityGeneric {
                            average: avg_per_s,
                            log,
                        };
                        serde_json::to_string(&identity_stats).unwrap()
                    }
                };

                entity.skill_stats.identity_stats = Some(stats);
            }
        }

        entity_stmt
            .execute(params![
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
                json!(entity.skills),
                json!(entity.damage_stats),
                json!(entity.skill_stats),
            ])
            .expect("failed to insert entity");
    }
    // if let Some(boss) = encounter.entities.get(&encounter.current_boss_name.to_string()) {
    //     entity_stmt.execute(params![
    //         boss.name,
    //         last_insert_id,
    //         boss.npc_id,
    //         boss.entity_type.to_string(),
    //         boss.class_id,
    //         boss.class,
    //         boss.gear_score,
    //         boss.current_hp,
    //         boss.max_hp,
    //         boss.is_dead,
    //         json!(boss.skills),
    //         json!(boss.damage_stats),
    //         json!(boss.skill_stats)
    //     ]).expect("failed to insert entity");
    // }
}

fn generate_intervals(start: i64, end: i64) -> Vec<i64> {
    if start >= end {
        return Vec::new();
    }

    (0..end - start).step_by(1_000).collect()
}

fn sum_in_range(vec: &Vec<(i64, i64)>, start: i64, end: i64) -> i64 {
    let start_idx = binary_search_left(vec, start);
    let end_idx = binary_search_left(vec, end + 1);

    vec[start_idx..end_idx]
        .iter()
        .map(|&(_, second)| second)
        .sum()
}

fn binary_search_left(vec: &Vec<(i64, i64)>, target: i64) -> usize {
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

fn calculate_average_dps(data: &[(i64, i64)], start_time: i64, end_time: i64) -> Vec<i64> {
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

fn get_class_from_id(class_id: &u32) -> String {
    let class = match class_id {
        0 => "",
        101 => "Warrior (Male)",
        102 => "Berserker",
        103 => "Destroyer",
        104 => "Gunlancer",
        105 => "Paladin",
        111 => "Female Warrior",
        112 => "Slayer",
        201 => "Mage",
        202 => "Arcanist",
        203 => "Summoner",
        204 => "Bard",
        205 => "Sorceress",
        301 => "Martial Artist (Female)",
        302 => "Wardancer",
        303 => "Scrapper",
        304 => "Soulfist",
        305 => "Glaivier",
        311 => "Martial Artist (Male)",
        312 => "Striker",
        401 => "Assassin",
        402 => "Deathblade",
        403 => "Shadowhunter",
        404 => "Reaper",
        501 => "Gunner (Male)",
        502 => "Sharpshooter",
        503 => "Deadeye",
        504 => "Artillerist",
        505 => "Machinist",
        511 => "Gunner (Female)",
        512 => "Gunslinger",
        601 => "Specialist",
        602 => "Artist",
        603 => "Aeromancer",
        604 => "Alchemist",
        _ => "Unknown",
    };

    class.to_string()
}
