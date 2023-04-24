use std::cmp::{max, Ordering};

use chrono::{DateTime, Utc};
use hashbrown::{HashMap, HashSet};
pub(crate) mod models;
use models::*;
mod log_lines;
use log_lines::*;
use rusqlite::{params, Connection, Transaction};
use serde_json::json;
use tauri::{Window, Wry, Manager};
use tokio::task;

const WINDOW_MS: i64 = 5_000;
const WINDOW_S: i64 = 5;

#[derive(Debug)]
pub struct Parser<'a> {
    pub window: &'a Window<Wry>,
    pub encounter: Encounter,
    pub raid_end: bool,
    saved: bool,

    prev_stagger: i32,
}

impl Parser<'_> {
    pub fn new(window: &Window<Wry>) -> Parser {
        Parser {
            window,
            encounter: Encounter::default(),
            raid_end: false,
            saved: false,

            prev_stagger: 0,
        }
    }

    pub fn parse_line(&mut self, line: String) {
        #[cfg(debug_assertions)]
        {
            println!("{}", line);
        }

        if line.is_empty() {
            return;
        }

        let line_split: Vec<&str> = line.trim().split('|').collect();
        if line_split.len() < 2 || line_split[0].is_empty() {
            return;
        }

        let log_type = match line_split[0].parse::<i32>() {
            Ok(t) => t,
            Err(_) => {
                println!("Could not parse log type");
                return;
            }
        };

        let timestamp = match line_split[1].parse::<DateTime<Utc>>() {
            Ok(t) => t.timestamp_millis(),
            Err(_) => {
                println!("Could not parse timestamp");
                return;
            }
        };

        // if there is no id associated with the log line, we can ignore it. i think.
        if line_split[2] == "0" && log_type != 2 {
            return;
        }

        // we reset our encounter only when we receive next incoming line
        if self.raid_end {
            self.raid_end = false;
            self.soft_reset();
            self.saved = false;
        }

        match log_type {
            0 => self.on_message(timestamp, &line_split),
            1 => self.on_init_env(timestamp, &line_split),
            2 => self.on_phase_transition(&line_split),
            3 => self.on_new_pc(timestamp, &line_split),
            4 => self.on_new_npc(timestamp, &line_split),
            5 => self.on_death(timestamp, &line_split),
            6 => self.on_skill_start(timestamp, &line_split),
            7 => self.on_skill_stage(&line_split),
            8 => self.on_damage(timestamp, &line_split),
            9 => self.on_heal(&line_split),
            10 => self.on_buff(&line_split),
            12 => self.on_counterattack(&line_split),
            20 => self.on_identity_gain(timestamp, &line_split),
            21 => self.on_stagger_change(timestamp, &line_split),
            _ => {}
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
        if !clone.local_player.is_empty() {
            if let Some(player) = clone.entities.get(&clone.local_player) {
                self.encounter.local_player = clone.local_player.to_string();
                self.encounter.entities.insert(
                    clone.local_player.to_string(),
                    Entity {
                        id: player.id.to_string(),
                        name: player.name.to_string(),
                        class: player.class.to_string(),
                        class_id: player.class_id,
                        entity_type: EntityType::PLAYER,
                        gear_score: player.gear_score,
                        last_update: Utc::now().timestamp_millis(),
                        ..Default::default()
                    },
                );
            }
        }
    }

    // keep all entities, reset all stats
    fn soft_reset(&mut self) {
        let clone = self.encounter.clone();
        self.reset(&clone);
        self.encounter.current_boss_name = "".to_string();
        for (key, entity) in clone.entities {
            self.encounter.entities.insert(
                key,
                Entity {
                    last_update: Utc::now().timestamp_millis(),
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

    fn on_message(&mut self, _timestamp: i64, line: &[&str]) {
        println!("Message: {:?}", line);
    }

    fn on_init_env(&mut self, timestamp: i64, line: &[&str]) {
        let init_env = LogInitEnv { player_id: line[2] };

        if init_env.player_id.is_empty() {
            return;
        }

        if let Some(player) = self
            .encounter
            .entities
            .get_mut(&self.encounter.local_player)
        {
            player.id = init_env.player_id.to_string();
            player.last_update = timestamp;
        } else {
            self.encounter.local_player = String::from("You");
            self.encounter.entities.insert(
                String::from("You"),
                Entity {
                    id: init_env.player_id.to_string(),
                    name: String::from("You"),
                    entity_type: EntityType::PLAYER,
                    last_update: timestamp,
                    ..Default::default()
                },
            );
        }

        if !self.saved && !self.encounter.current_boss_name.is_empty() {
            self.save_to_db();
        }

        self.encounter.entities.retain(|_, v| {
            v.name == self.encounter.local_player
                || (v.damage_stats.damage_dealt > 0 && v.max_hp > 0)
        });

        self.window
            .emit("zone-change", Some(self.encounter.clone()))
            .expect("failed to emit zone-change");

        self.encounter.current_boss_name = "".to_string();
        self.soft_reset();
    }

    fn on_phase_transition(&mut self, line: &[&str]) {
        let phase_transition = LogPhaseTransition {
            raid_result: line[2].parse::<i32>().unwrap_or_default()
        };

        self.window
            .emit("phase-transition", phase_transition.raid_result)
            .expect("failed to emit phase-transition");
            
        if phase_transition.raid_result == 0 || phase_transition.raid_result == 2
        {
            self.save_to_db();
            self.raid_end = true;
            self.saved = true;
        }
    }

    fn on_new_pc(&mut self, timestamp: i64, line: &[&str]) {
        let mut gear_score = line[7].parse::<f64>().unwrap_or(0.0);

        if !(0.0..=1655.0).contains(&gear_score) {
            gear_score = 0.0;
        }

        let new_pc = LogNewPc {
            id: line[2],
            name: if line[3].is_empty() {
                "Unknown Entity"
            } else {
                line[3]
            },
            class_id: line[4].parse::<i32>().unwrap_or_default(),
            class: if line[5].is_empty() {
                "Unknown Class"
            } else {
                line[5]
            },
            level: line[6].parse::<i32>().unwrap_or_default(),
            gear_score,
            current_hp: line[8].parse::<i64>().unwrap_or_default(),
            max_hp: line[9].parse::<i64>().unwrap_or_default(),
            entity_type: EntityType::PLAYER,
        };

        if !self.encounter.local_player.is_empty() {
            if let Some(player) = self
                .encounter
                .entities
                .get_mut(&self.encounter.local_player)
            {
                if new_pc.id == player.id {
                    self.encounter.local_player = new_pc.name.to_string();
                }
            }
        }

        if let Some(player) = self.encounter.entities.get_mut(new_pc.name) {
            player.id = new_pc.id.to_string();
            player.class_id = new_pc.class_id;
            player.class = new_pc.class.to_string();
            player.gear_score = new_pc.gear_score;
            player.current_hp = new_pc.current_hp;
            player.max_hp = new_pc.max_hp;
            player.last_update = timestamp;
        } else {
            self.encounter
                .entities
                .retain(|_, entity| entity.id != new_pc.id);
            self.encounter.entities.insert(
                new_pc.name.to_string(),
                Entity {
                    id: new_pc.id.to_string(),
                    name: new_pc.name.to_string(),
                    class_id: new_pc.class_id,
                    class: new_pc.class.to_string(),
                    gear_score: new_pc.gear_score,
                    current_hp: new_pc.current_hp,
                    max_hp: new_pc.max_hp,
                    entity_type: EntityType::PLAYER,
                    last_update: timestamp,
                    ..Default::default()
                },
            );
        }
    }

    fn on_new_npc(&mut self, timestamp: i64, line: &[&str]) {
        let new_npc = LogNewNpc {
            id: line[2],
            npc_id: line[3].parse::<i32>().unwrap_or_default(),
            name: if line[4].is_empty() {
                "Unknown Entity"
            } else {
                line[4]
            },
            current_hp: line[5].parse::<i64>().unwrap_or_default(),
            max_hp: line[6].parse::<i64>().unwrap_or_default(),
            entity_type: EntityType::UNKNOWN,
        };

        if let Some(npc) = self.encounter.entities.get_mut(new_npc.name) {
            npc.id = new_npc.id.to_string();
            npc.npc_id = new_npc.npc_id;
            npc.name = new_npc.name.to_string();
            npc.current_hp = new_npc.current_hp;
            npc.max_hp = new_npc.max_hp;
            npc.last_update = timestamp;
            if let Some((_, npc_info)) = NPC_DATA.get_key_value(&new_npc.npc_id) {
                if npc_info.grade == "boss"
                    || npc_info.grade == "raid"
                    || npc_info.grade == "epic_raid"
                    || npc_info.grade == "commander"
                {
                    npc.entity_type = EntityType::BOSS;
                } else {
                    npc.entity_type = EntityType::NPC;
                }
            }
        } else {
            let mut entity_type = EntityType::NPC;
            if let Some((_, npc_info)) = NPC_DATA.get_key_value(&new_npc.npc_id) {
                if npc_info.grade == "boss"
                    || npc_info.grade == "raid"
                    || npc_info.grade == "epic_raid"
                    || npc_info.grade == "commander"
                {
                    entity_type = EntityType::BOSS;
                }
            }
            self.encounter.entities.insert(
                new_npc.name.to_string(),
                Entity {
                    id: new_npc.id.to_string(),
                    npc_id: new_npc.npc_id,
                    name: new_npc.name.to_string(),
                    current_hp: new_npc.current_hp,
                    max_hp: new_npc.max_hp,
                    entity_type,
                    last_update: timestamp,
                    ..Default::default()
                },
            );
        }

        if self.encounter.current_boss_name.is_empty() {
            if let Some((_, npc)) = NPC_DATA.get_key_value(&new_npc.npc_id) {
                if npc.grade == "boss"
                    || npc.grade == "raid"
                    || npc.grade == "epic_raid"
                    || npc.grade == "commander"
                {
                    self.encounter.current_boss_name = new_npc.name.to_string();
                }
            }
        } else if !self.encounter.current_boss_name.is_empty() {
            // if for some reason current_boss_name is not in the entities list, reset it
            if let Some(boss) = self
                .encounter
                .entities
                .get(&self.encounter.current_boss_name.to_string())
            {
                if new_npc.max_hp >= boss.max_hp && boss.is_dead {
                    if let Some((_, npc)) = NPC_DATA.get_key_value(&new_npc.npc_id) {
                        if npc.grade == "boss"
                            || npc.grade == "raid"
                            || npc.grade == "epic_raid"
                            || npc.grade == "commander"
                        {
                            self.encounter.current_boss_name = new_npc.name.to_string();
                        }
                    }
                }
            } else {
                self.encounter.current_boss_name = "".to_string();
            }
        }
    }

    fn on_death(&mut self, timestamp: i64, line: &[&str]) {
        let new_death = LogDeath {
            id: line[2],
            name: if line[3].is_empty() {
                "Unknown Entity"
            } else {
                line[3]
            },
            killer_id: line[4],
            killer_name: if line[5].is_empty() {
                "Unknown Entity"
            } else {
                line[5]
            },
        };

        if let Some(entity) = self.encounter.entities.get_mut(new_death.name) {
            // the entity that died has the same name as another entity, but with different id?
            if entity.id != new_death.id {
                return;
            }
            let deaths = if entity.is_dead {
                entity.damage_stats.deaths
            } else {
                1
            };
            entity.is_dead = true;
            entity.damage_stats.deaths = deaths;
            entity.damage_stats.death_time = timestamp;
            entity.last_update = timestamp;
        } else {
            self.encounter.entities.insert(
                new_death.name.to_string(),
                Entity {
                    id: new_death.id.to_string(),
                    name: new_death.name.to_string(),
                    is_dead: true,
                    damage_stats: DamageStats {
                        deaths: 1,
                        death_time: timestamp,
                        ..Default::default()
                    },
                    last_update: timestamp,
                    ..Default::default()
                },
            );
        }
    }

    fn on_skill_start(&mut self, timestamp: i64, line: &[&str]) {
        let skill_start = LogSkillStart {
            id: line[2],
            name: if line[3].is_empty() {
                "Unknown Entity"
            } else {
                line[3]
            },
            skill_id: line[4].parse::<i32>().unwrap_or_default(),
            skill_name: if line[5].is_empty() {
                "Unknown Skill"
            } else {
                line[5]
            },
        };

        let mut entity = self
            .encounter
            .entities
            .entry(skill_start.name.to_string())
            .or_insert_with(|| {
                let (skill_name, skill_icon) = get_skill_name_and_icon(
                    skill_start.skill_id,
                    0,
                    skill_start.skill_name.to_string(),
                );
                Entity {
                    name: skill_start.name.to_string(),
                    last_update: timestamp,
                    skill_stats: SkillStats {
                        casts: 0,
                        ..Default::default()
                    },
                    skills: HashMap::from([(
                        skill_start.skill_id,
                        Skill {
                            id: skill_start.skill_id,
                            name: skill_name,
                            icon: skill_icon,
                            casts: 0,
                            ..Default::default()
                        },
                    )]),
                    ..Default::default()
                }
            });

        entity.last_update = timestamp;
        entity.is_dead = false;
        entity.skill_stats.casts += 1;

        let duration = if self.encounter.fight_start == 0 {
            0
        } else {
            ((timestamp - self.encounter.fight_start) / 1000) as i32
        };

        // if skills have different ids but the same name, we group them together
        // dunno if this is right approach xd
        if let Some(skill) = entity.skills.get_mut(&skill_start.skill_id) {
            skill.casts += 1;
            skill.cast_log.push(duration);
        } else if let Some(skill) = entity
            .skills
            .values_mut()
            .find(|s| s.name == *skill_start.skill_name)
        {
            skill.casts += 1;
            skill.cast_log.push(duration)
        } else {
            let (skill_name, skill_icon) = get_skill_name_and_icon(
                skill_start.skill_id,
                0,
                skill_start.skill_name.to_string(),
            );
            entity.skills.insert(
                skill_start.skill_id,
                Skill {
                    id: skill_start.skill_id,
                    name: skill_name,
                    icon: skill_icon,
                    casts: 1,
                    cast_log: vec![duration],
                    ..Default::default()
                },
            );
        }
    }

    fn on_skill_stage(&mut self, _line: &[&str]) {}

    fn on_damage(&mut self, timestamp: i64, line: &[&str]) {
        if line.len() < 13 {
            return;
        }
        let mut damage = LogDamage {
            source_id: line[2],
            source_name: if line[3].is_empty() {
                "Unknown Entity"
            } else {
                line[3]
            },
            skill_id: line[4].parse::<i32>().unwrap_or_default(),
            skill_name: if line[5].is_empty() {
                "Unknown Skill"
            } else {
                line[5]
            },
            skill_effect_id: line[6].parse::<i32>().unwrap_or_default(),
            skill_effect: line[7],
            target_id: line[8],
            target_name: if line[9].is_empty() {
                "Unknown Entity"
            } else {
                line[9]
            },
            damage: line[10].parse::<i64>().unwrap_or_default(),
            damage_mod: i32::from_str_radix(line[11], 16).unwrap_or_default(),
            current_hp: line[12].parse::<i64>().unwrap_or_default(),
            max_hp: line[13].parse::<i64>().unwrap_or_default(),
            effects_on_source: HashSet::new(),
            effects_on_target: HashSet::new(),
        };

        if line.len() >= 17 {
            for buff in line[14].split(',').step_by(2) {
                if !buff.is_empty() {
                    damage
                        .effects_on_target
                        .insert(buff.parse::<i32>().unwrap_or_default());
                }
            }
            for buff in line[15].split(',').step_by(2) {
                if !buff.is_empty() {
                    damage
                        .effects_on_source
                        .insert(buff.parse::<i32>().unwrap_or_default());
                }
            }
        }

        let hit_flag = match damage.damage_mod & 0xf {
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
        let hit_option = match ((damage.damage_mod >> 4) & 0x7) - 1 {
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

        let mut source_entity = self
            .encounter
            .entities
            .entry(damage.source_name.to_string())
            .or_insert_with(|| Entity {
                id: damage.source_id.to_string(),
                name: damage.source_name.to_string(),
                ..Default::default()
            })
            .to_owned();

        let mut target_entity = self
            .encounter
            .entities
            .entry(damage.target_name.to_string())
            .or_insert_with(|| Entity {
                id: damage.target_id.to_string(),
                name: damage.target_name.to_string(),
                current_hp: damage.current_hp,
                max_hp: damage.max_hp,
                ..Default::default()
            })
            .to_owned();

        source_entity.id = damage.source_id.to_string();
        target_entity.id = damage.target_id.to_string();

        if self.encounter.fight_start == 0 {
            self.encounter.fight_start = timestamp;
            self.window
                .emit("raid-start", timestamp)
                .expect("failed to emit raid-start");
        }

        target_entity.current_hp = damage.current_hp;
        target_entity.max_hp = damage.max_hp;
        target_entity.last_update = timestamp;
        source_entity.last_update = timestamp;

        if target_entity.entity_type != EntityType::PLAYER && damage.current_hp < 0 {
            damage.damage += damage.current_hp;
        }

        if damage.skill_id == 0 && damage.skill_effect_id != 0 {
            damage.skill_id = damage.skill_effect_id;
            damage.skill_name = damage.skill_effect;
        }

        let skill = source_entity.skills.contains_key(&damage.skill_id);
        let mut skill_id = damage.skill_id;
        if !skill {
            if let Some(skill) = source_entity
                .skills
                .values()
                .find(|&s| s.name == *damage.skill_name)
            {
                skill_id = skill.id;
            } else {
                let (skill_name, skill_icon) = get_skill_name_and_icon(
                    damage.skill_id,
                    damage.skill_effect_id,
                    damage.skill_name.to_string(),
                );
                source_entity.skills.insert(
                    damage.skill_id,
                    Skill {
                        id: damage.skill_id,
                        name: skill_name,
                        icon: skill_icon,
                        casts: 1,
                        ..Default::default()
                    },
                );
            }
        }

        let skill = source_entity.skills.get_mut(&skill_id).unwrap();

        if damage.skill_name == "Bleed" && hit_flag == HitFlag::DAMAGE_SHARE {
            return;
        }

        let is_crit = hit_flag == HitFlag::CRITICAL || hit_flag == HitFlag::DOT_CRITICAL;
        let is_back_atk = hit_option == HitOption::BACK_ATTACK;
        let is_front_atk = hit_option == HitOption::FRONTAL_ATTACK;

        skill.total_damage += damage.damage;
        if damage.damage > skill.max_damage {
            skill.max_damage = damage.damage;
        }

        source_entity.damage_stats.damage_dealt += damage.damage;
        target_entity.damage_stats.damage_taken += damage.damage;

        // if damage.skill_name != "Bleed" {
        source_entity.skill_stats.hits += 1;
        source_entity.skill_stats.crits += if is_crit { 1 } else { 0 };
        source_entity.skill_stats.back_attacks += if is_back_atk { 1 } else { 0 };
        source_entity.skill_stats.front_attacks += if is_front_atk { 1 } else { 0 };

        skill.hits += 1;
        skill.crits += if is_crit { 1 } else { 0 };
        skill.back_attacks += if is_back_atk { 1 } else { 0 };
        skill.front_attacks += if is_front_atk { 1 } else { 0 };
        // }

        if source_entity.entity_type == EntityType::PLAYER {
            self.encounter.encounter_damage_stats.total_damage_dealt += damage.damage;
            self.encounter.encounter_damage_stats.top_damage_dealt = max(
                self.encounter.encounter_damage_stats.top_damage_dealt,
                source_entity.damage_stats.damage_dealt,
            );

            source_entity.damage_stats.damage_log.push((timestamp, damage.damage));

            let mut is_buffed_by_support = false;
            let mut is_debuffed_by_support = false;
            for buff_id in damage.effects_on_source.iter() {
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
                        if !is_buffed_by_support && status_effect.source.skill.is_some() {
                            let skill = status_effect.source.skill.as_ref().unwrap();
                            is_buffed_by_support = (status_effect.buff_category == "classskill"
                                || status_effect.buff_category == "identity"
                                || status_effect.buff_category == "ability")
                                && status_effect.target == StatusEffectTarget::PARTY
                                && is_support_class_id(skill.class_id);
                        }
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
            }
            for buff_id in damage.effects_on_target.iter() {
                // maybe problem
                if !self
                    .encounter
                    .encounter_damage_stats
                    .unknown_buffs
                    .contains(buff_id)
                    && !self
                        .encounter
                        .encounter_damage_stats
                        .debuffs
                        .contains_key(buff_id)
                {
                    if let Some(status_effect) = get_status_effect_data(*buff_id) {
                        if !is_buffed_by_support && status_effect.source.skill.is_some() {
                            let skill = status_effect.source.skill.as_ref().unwrap();
                            is_debuffed_by_support = (status_effect.buff_category == "classskill"
                                || status_effect.buff_category == "identity"
                                || status_effect.buff_category == "ability")
                                && status_effect.target == StatusEffectTarget::PARTY
                                && is_support_class_id(skill.class_id);
                        }
                        self.encounter
                            .encounter_damage_stats
                            .debuffs
                            .insert(*buff_id, status_effect);
                    } else {
                        self.encounter
                            .encounter_damage_stats
                            .unknown_buffs
                            .insert(*buff_id);
                    }
                }
            }

            skill.buffed_by_support += if is_buffed_by_support {
                damage.damage
            } else {
                0
            };
            skill.debuffed_by_support += if is_debuffed_by_support {
                damage.damage
            } else {
                0
            };
            source_entity.damage_stats.buffed_by_support += if is_buffed_by_support {
                damage.damage
            } else {
                0
            };
            source_entity.damage_stats.debuffed_by_support += if is_debuffed_by_support {
                damage.damage
            } else {
                0
            };

            for buff_id in damage.effects_on_source.iter() {
                skill
                    .buffed_by
                    .entry(*buff_id)
                    .and_modify(|e| *e += damage.damage)
                    .or_insert(damage.damage);
                source_entity
                    .damage_stats
                    .buffed_by
                    .entry(*buff_id)
                    .and_modify(|e| *e += damage.damage)
                    .or_insert(damage.damage);
            }
            for buff_id in damage.effects_on_target.iter() {
                skill
                    .debuffed_by
                    .entry(*buff_id)
                    .and_modify(|e| *e += damage.damage)
                    .or_insert(damage.damage);
                source_entity
                    .damage_stats
                    .debuffed_by
                    .entry(*buff_id)
                    .and_modify(|e| *e += damage.damage)
                    .or_insert(damage.damage);
            }
        }

        if target_entity.entity_type == EntityType::PLAYER {
            self.encounter.encounter_damage_stats.total_damage_taken += damage.damage;
            self.encounter.encounter_damage_stats.top_damage_taken = max(
                self.encounter.encounter_damage_stats.top_damage_taken,
                target_entity.damage_stats.damage_taken,
            );
        }

        // update current_boss
        if target_entity.entity_type == EntityType::BOSS {
            self.encounter.current_boss_name = target_entity.name.to_string();
        }

        self.encounter
            .entities
            .insert(source_entity.name.to_string(), source_entity);
        self.encounter
            .entities
            .insert(target_entity.name.to_string(), target_entity);

        self.encounter.last_combat_packet = timestamp;
    }

    fn on_heal(&mut self, _line: &[&str]) {
        println!("Heal");
    }

    fn on_buff(&mut self, _line: &[&str]) {
        println!("Buff");
    }

    fn on_counterattack(&mut self, line: &[&str]) {
        let counter = LogCounterAttack {
            id: line[2],
            name: if line[3].is_empty() {
                "Unknown Entity"
            } else {
                line[3]
            },
            target_id: line[4],
            target_name: if line[5].is_empty() {
                "Unknown Entity"
            } else {
                line[5]
            },
        };

        let entity = self
            .encounter
            .entities
            .entry(counter.name.to_string())
            .or_insert_with(|| Entity {
                id: counter.id.to_string(),
                name: counter.name.to_string(),
                entity_type: EntityType::PLAYER,
                skill_stats: SkillStats {
                    counters: 1,
                    ..Default::default()
                },
                ..Default::default()
            });
        entity.skill_stats.counters += 1;
    }

    fn on_identity_gain(&mut self, timestamp: i64, line: &[&str]) {
        let identity = LogIdentityGain {
            id: line[2],
            name: if line[3].is_empty() {
                "Unknown Entity"
            } else {
                line[3]
            },
            gauge_1: line[4].parse::<i32>().unwrap_or(0),
            gauge_2: line[5].parse::<i32>().unwrap_or(0),
            gauge_3: line[6].parse::<i32>().unwrap_or(0),
        };

        if self.encounter.local_player.is_empty() || self.encounter.fight_start == 0 {
            return;
        }
        
        if let Some(entity) = self.encounter.entities.get_mut(&self.encounter.local_player) {
            entity.damage_stats.identity_log.push((timestamp, (identity.gauge_1, identity.gauge_2, identity.gauge_3)));
        }
    }

    fn on_stagger_change(&mut self, timestamp: i64, line_split: &[&str]) {
        let stagger = LogStaggerChange {
            id: line_split[2],
            current_stagger: line_split[3].parse::<i32>().unwrap_or(0),
            max_stagger: line_split[4].parse::<i32>().unwrap_or(0),
        };

        if self.encounter.current_boss_name.is_empty() || self.encounter.fight_start == 0 {
            return;
        }

        if let Some(boss) = self.encounter.entities.get_mut(&self.encounter.current_boss_name) {
            if boss.id == stagger.id {
                if stagger.current_stagger == stagger.max_stagger {
                    self.encounter.encounter_damage_stats.total_stagger += stagger.max_stagger;
                    self.prev_stagger = 0;
                } else {
                    self.prev_stagger = stagger.current_stagger;
                }

                let stagger_percent = (1.0 - (stagger.current_stagger as f32 / stagger.max_stagger as f32)) * 100.0;
                let relative_timestamp = (timestamp - self.encounter.fight_start) / 1000;
                self.encounter.encounter_damage_stats.stagger_log.push((relative_timestamp as i32, stagger_percent));

                if stagger.max_stagger > self.encounter.encounter_damage_stats.max_stagger {
                    self.encounter.encounter_damage_stats.max_stagger = stagger.max_stagger;
                }
            }
        }
    }

    fn save_to_db(&self) {
        if self.encounter.fight_start == 0
            || self.encounter.current_boss_name.is_empty()
            || !self.encounter.entities.contains_key(&self.encounter.current_boss_name)
            || !self.encounter
                .entities
                .values()
                .any(|e| e.entity_type == EntityType::PLAYER && e.skill_stats.hits > 1 && e.max_hp > 0)
        {
            return;
        }
        let mut encounter = self.encounter.clone();
        let mut path = self.window.app_handle().path_resolver().resource_dir().expect("could not get resource dir");
        path.push("encounters.db");
        let prev_stagger = self.prev_stagger;

        task::spawn(async move {
            println!("saving to db - {}", encounter.current_boss_name);

            let mut conn = Connection::open(path).expect("failed to open database");
            let tx = conn.transaction().expect("failed to create transaction");

            insert_data(&tx, &mut encounter, prev_stagger);

            tx.commit().expect("failed to commit transaction");
            println!("saved to db");
        });
    }
}

fn is_support_class_id(class_id: i32) -> bool {
    class_id == 105 || class_id == 204 || class_id == 603
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
        buff.buff_category.to_string()
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
        category: buff.category.to_string(),
        buff_category: buff_category.to_string(),
        buff_type: get_status_effect_buff_type_flags(buff),
        unique_group: buff.unique_group,
        source: StatusEffectSource {
            name: buff.name.to_string(),
            desc: buff.desc.to_string(),
            icon: buff.icon.to_string(),
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
        } else if let Some(buff_source_skill) =
            SKILL_DATA.get(&((buff_id / 100) * 10))
        {
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
                status_effect.source.name = item_name.to_string();
            }
            if let Some(item_desc) = buff_source_item.item_desc.as_ref() {
                status_effect.source.desc = item_desc.to_string();
            }
            if let Some(icon) = buff_source_item.icon.as_ref() {
                status_effect.source.icon = icon.to_string();
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
    skill_id: i32,
    skill_effect_id: i32,
    skill_name: String,
) -> (String, String) {
    if skill_id == 0 && skill_effect_id == 0 {
        ("Bleed".to_string(), "buff_168.png".to_string())
    } else if skill_id == 0 {
        if let Some(effect) = SKILL_EFFECT_DATA.get(&skill_effect_id) {
            if effect.item_name.is_some() {
                return (
                    effect.item_name.as_ref().unwrap().to_string(),
                    effect.icon.as_ref().unwrap().to_string(),
                );
            }
            if effect.source_skill.is_some() {
                if let Some(skill) = SKILL_DATA.get(&effect.source_skill.unwrap()) {
                    return (skill.name.to_string(), skill.icon.to_string());
                }
            } else if let Some(skill) =
                SKILL_DATA.get(&(skill_effect_id / 10))
            {
                return (skill.name.to_string(), skill.icon.to_string());
            }
            return (effect.comment.to_string(), "".to_string());
        } else {
            return (skill_name, "".to_string());
        }
    } else {
        let mut skill = SKILL_DATA.get(&skill_id);
        if skill.is_none() {
            skill = SKILL_DATA.get(&(skill_id - (skill_id % 10)));
            if skill.is_none() {
                return (skill_name, "".to_string());
            }
        }
        let skill = skill.unwrap();
        if skill.summon_source_skill.is_some() {
            if let Some(skill) = SKILL_DATA.get(&skill.summon_source_skill.unwrap()) {
                return (skill.name.to_string() + " (Summon)", skill.icon.to_string());
            } else {
                return (skill_name, "".to_string());
            }
        } else if skill.source_skill.is_some() {
            if let Some(skill) = SKILL_DATA.get(&skill.source_skill.unwrap()) {
                return (skill.name.to_string(), skill.icon.to_string());
            } else {
                return (skill_name, "".to_string());
            }
        } else {
            return (skill.name.to_string(), skill.icon.to_string());
        }
    }
}

fn insert_data(tx: &Transaction, encounter: &mut Encounter, prev_stagger: i32) {
    let mut encounter_stmt = tx
        .prepare(
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
    encounter.encounter_damage_stats.dps = encounter.encounter_damage_stats.total_damage_dealt / duration_seconds;

    let mut misc = EncounterMisc::default();

    if encounter.encounter_damage_stats.total_stagger > 0 && !encounter.encounter_damage_stats.stagger_log.is_empty() {
        let stagger = StaggerStats {
            average: ((encounter.encounter_damage_stats.total_stagger + prev_stagger) as f64 / duration_seconds as f64) / encounter.encounter_damage_stats.max_stagger as f64 * 100.0,
            log: encounter.encounter_damage_stats.stagger_log.clone(),
        };
    
        misc.stagger_stats = Some(stagger);
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
        .prepare(
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
        skill_stats,
        last_update
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        )
        .expect("failed to prepare entity statement");

    let fight_start = encounter.fight_start;
    let fight_end = encounter.last_combat_packet;

    for (_key, mut entity) in encounter.entities.iter_mut()
        .filter(|(_, e)| e.entity_type == EntityType::PLAYER && e.skill_stats.hits >= 1 && e.max_hp > 0) 
    {
        let intervals = generate_intervals(fight_start, fight_end);
        if !intervals.is_empty() {
            for interval in intervals {                
                let start = fight_start + interval - WINDOW_MS;
                let end = fight_start + interval + WINDOW_MS;
                
                let damage = sum_in_range(&entity.damage_stats.damage_log, start, end);
                entity.damage_stats.dps_rolling_10s_avg.push(damage / WINDOW_S);
            }
        }

        let fight_start_sec = encounter.fight_start / 1000;
        let fight_end_sec = encounter.last_combat_packet / 1000;
        entity.damage_stats.dps_average = calculate_average_dps(&entity.damage_stats.damage_log, fight_start_sec, fight_end_sec);

        entity.damage_stats.dps = entity.damage_stats.damage_dealt / duration_seconds;
        for (_, mut skill) in entity.skills.iter_mut() {
            skill.dps = skill.total_damage / duration_seconds;
        }

        if entity.name == encounter.local_player && entity.damage_stats.identity_log.len() >= 2 {
            let mut total_identity_gain = 0;
            let data = &entity.damage_stats.identity_log;
            let duration_seconds = (data[data.len() - 1].0 - data[0].0) / 1000;
            let max = match entity.class.as_str() {
                "Summoner" => 7_000.0,
                _ => 10_000.0
            };
            let stats: String = match entity.class.as_str() {
                "Arcanist" => {
                    let mut cards: HashMap::<i32, i32> = HashMap::new();
                    let mut log: Vec<(i32, (f32, i32, i32))> = Vec::new();
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
                        let percentage = if i2.0 >= max as i32 {
                            100.0
                        } else {
                            (((i2.0 as f32 / max) * 100.0) * 100.0).round() / 100.0
                        };
                        log.push((relative_time, (percentage, i2.1, i2.2)));
                    }

                    let avg_per_s = (total_identity_gain as f64 / duration_seconds as f64) / max as f64 * 100.0;
                    let identity_stats = IdentityArcanist {
                        average: avg_per_s,
                        card_draws: cards,
                        log,
                    };

                    serde_json::to_string(&identity_stats).unwrap()
                }
                "Artist" | "Bard" => {
                    let mut log: Vec<(i32, (f32, i32))> = Vec::new();

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
                        let percentage: f32 = ((((i2.0 as f32 + max * i2.1 as f32) / max) * 100.0) * 100.0).round() / 100.0;
                        log.push((relative_time, (percentage, i2.1)));
                    }

                    let avg_per_s = (total_identity_gain as f64 / duration_seconds as f64) / max as f64 * 100.0;
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
                        let percentage = (((i2.0 as f32 / max) * 100.0) * 100.0).round() / 100.0;
                        log.push((relative_time, percentage));
                    }

                    let avg_per_s = (total_identity_gain as f64 / duration_seconds as f64) / max as f64 * 100.0;
                    let identity_stats = IdentityGeneric {
                        average: avg_per_s,
                        log,
                    };
                    serde_json::to_string(&identity_stats).unwrap()
                }
            };

            entity.skill_stats.identity_stats = Some(stats);
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
                entity.last_update
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

    vec[start_idx..end_idx].iter().map(|&(_, second)| second).sum()
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