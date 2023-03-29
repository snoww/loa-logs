use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, PartialEq, Clone)]
pub enum EntityType {
    #[default] UNKNOWN,
    MONSTER,
    BOSS,
    GUARDIAN,
    PLAYER,
    NPC
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Encounter {
    pub last_combat_packet: i64,
    pub fight_start: i64,
    pub local_player: String,
    pub entities: HashMap<String, Entity>,
    pub current_boss: Option<Entity>,
    pub encounter_damage_stats: EncounterDamageStats,
    pub duration: i64,
    pub reset: bool
}

impl Encounter {
    pub fn new() -> Encounter {
        Encounter {
            last_combat_packet: 0,
            fight_start: 0,
            local_player: String::new(),
            entities: HashMap::new(),
            current_boss: None,
            encounter_damage_stats: EncounterDamageStats::new(),
            duration: 0,
            reset: false
        }
    }    
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EncounterDamageStats {
    pub total_damage_dealt: i64,
    pub top_damage_dealt: i64,
    pub total_damage_taken: i64,
    pub top_damage_taken: i64,
    pub dps: i64,
    pub dps_intervals: HashMap<i32, i64>,
    pub most_damage_taken_entity: MostDamageTakenEntity,
}

impl EncounterDamageStats {
    pub fn new() -> EncounterDamageStats {
        EncounterDamageStats {
            total_damage_dealt: 0,
            top_damage_dealt: 0,
            total_damage_taken: 0,
            top_damage_taken: 0,
            dps: 0,
            dps_intervals: HashMap::new(),
            most_damage_taken_entity: MostDamageTakenEntity::new()
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MostDamageTakenEntity {
    pub name: String,
    pub damage_taken: i64,
}

impl MostDamageTakenEntity {
    pub fn new() -> MostDamageTakenEntity {
        MostDamageTakenEntity {
            name: String::new(),
            damage_taken: 0,
        }
    }
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Entity {
    pub last_update: i64,
    pub id: String,
    pub npc_id: i32,
    pub name: String,
    pub entity_type: EntityType,
    pub class_id: i32,
    pub class: String,
    pub gear_score: f64,
    pub current_hp: i64,
    pub max_hp: i64,
    pub is_dead: bool,
    pub skills: HashMap<String, Skill>,
    pub damage_stats: DamageStats,
    pub skill_stats: SkillStats,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub id: i32,
    pub name: String,
    pub total_damage: i64,
    pub max_damage: i64,
    pub casts: i64,
    pub hits: i64,
    pub crits: i64,
    pub back_attacks: i64,
    pub front_attacks: i64,
    pub dps: i64,
    pub dps_intervals: HashMap<i32, i64>,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DamageStats {
    pub damage_dealt: i64,
    pub damage_taken: i64,
    pub deaths: i64,
    pub death_time: i64,
    pub dps: i64,
    pub dps_intervals: HashMap<i32, i64>,
}

impl DamageStats {
    pub fn new() -> DamageStats {
        DamageStats {
            damage_dealt: 0,
            damage_taken: 0,
            deaths: 0,
            death_time: 0,
            dps: 0,
            dps_intervals: HashMap::new(),
        }
    }
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillStats {
    pub casts: i64,
    pub hits: i64,
    pub crits: i64,
    pub back_attacks: i64,
    pub front_attacks: i64,
    pub counters: i64
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Npc {
    pub id: i32,
    pub name: String,
    pub grade: String,
    #[serde(rename = "type")]
    pub npc_type: String,
}
