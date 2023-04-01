use std::collections::{HashMap, HashSet};

use serde::{Serialize, Deserialize};
use bitflags::bitflags;


#[derive(Debug, Default, Serialize, PartialEq, Clone)]
pub enum EntityType {
    #[default] UNKNOWN,
    MONSTER,
    BOSS,
    GUARDIAN,
    PLAYER,
    NPC
}

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Encounter {
    pub last_combat_packet: i64,
    pub fight_start: i64,
    pub local_player: String,
    pub entities: HashMap<String, Entity>,
    pub current_boss_name: String,
    pub current_boss: Option<Entity>,
    pub encounter_damage_stats: EncounterDamageStats,
    pub duration: i64,
    pub reset: bool
}

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct EncounterDamageStats {
    pub total_damage_dealt: i64,
    pub top_damage_dealt: i64,
    pub total_damage_taken: i64,
    pub top_damage_taken: i64,
    pub dps: i64,
    pub dps_intervals: HashMap<i32, i64>,
    pub most_damage_taken_entity: MostDamageTakenEntity,
    pub buffs: HashMap<i32, StatusEffect>,
    pub debuffs: HashMap<i32, StatusEffect>,
    #[serde(skip)]
    pub unknown_buffs: HashSet<i32>
}

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct MostDamageTakenEntity {
    pub name: String,
    pub damage_taken: i64,
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
    pub skills: HashMap<i32, Skill>,
    pub damage_stats: DamageStats,
    pub skill_stats: SkillStats,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub id: i32,
    pub name: String,
    pub icon: String,
    pub total_damage: i64,
    pub max_damage: i64,
    pub buffed_by: HashMap<i32, i64>,
    pub debuffed_by: HashMap<i32, i64>,
    pub buffed_by_support: i64,
    pub debuffed_by_support: i64,
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
    pub buffed_by: HashMap<i32, i64>,
    pub debuffed_by: HashMap<i32, i64>,
    pub buffed_by_support: i64,
    pub debuffed_by_support: i64,
    pub deaths: i64,
    pub death_time: i64,
    pub dps: i64,
    pub dps_intervals: HashMap<i32, i64>,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillStats {
    pub casts: i64,
    pub hits: i64,
    pub crits: i64,
    pub back_attacks: i64,
    pub front_attacks: i64,
    pub counters: i64,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Npc {
    pub id: i32,
    pub name: String,
    pub grade: String,
    #[serde(rename = "type")]
    pub npc_type: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillData {
    pub id: i32,
    pub name: String,
    pub desc: String,
    #[serde(rename = "classid")]
    pub class_id: i32,
    pub icon: String,
    #[serde(rename = "summonids")]
    pub summon_ids: Option<Vec<i32>>,
    #[serde(rename = "summonsourceskill")]
    pub summon_source_skill: Option<i32>,
    #[serde(rename = "sourceskill")]
    pub source_skill: Option<i32>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct SkillEffectData {
    pub id: i32,
    pub comment: String,
    #[serde(skip)]
    pub stagger: i32,
    #[serde(rename = "sourceskill")]
    pub source_skill: Option<i32>,
    #[serde(rename = "itemname")]
    pub item_name: Option<String>,
    #[serde(skip, rename = "itemdesc")]
    pub item_desc: Option<String>,
    pub icon: Option<String>,
    #[serde(rename = "itemcategory")]
    pub item_category: Option<String>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct SkillBuffData {
    pub id: i32,
    pub name: String,
    pub desc: String,
    pub icon: String,
    #[serde(rename = "iconshowtype")]
    pub icon_show_type: String,
    pub duration: i32,
    pub category: String,
    #[serde(rename = "type")]
    pub buff_type: String,
    #[serde(rename = "buffcategory")]
    pub buff_category: String,
    pub target: String,
    #[serde(rename = "uniquegroup")]
    pub unique_group: i32,
    #[serde(rename = "overlapflag")]
    pub overlap_flag: i32,
    #[serde(skip, rename = "passiveoption")]
    pub passive_option: HashMap<String, String>,
    #[serde(rename = "sourceskill")]
    pub source_skill: Option<i32>,
    #[serde(rename = "setname")]
    pub set_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StatusEffect {
    pub target: StatusEffectTarget,
    pub category: String,
    pub buff_category: String,
    pub buff_type: u32,
    pub unique_group: i32,
    pub source: StatusEffectSource, 
}

#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub enum StatusEffectTarget {
    #[default]
    OTHER,
    PARTY,
    SELF
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StatusEffectSource {
    pub name: String,
    pub desc: String,
    pub icon: String,
    pub skill: Option<SkillData>,
    pub set_name: Option<String>,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct StatusEffectBuffTypeFlags: u32 {
        const NONE = 0;
        const DMG = 1;
        const CRIT = 1 << 1;
        const ATKSPEED = 1 << 2;
        const MOVESPEED = 1 << 3;
        const HP = 1 << 4;
        const DEFENSE = 1 << 5;
        const RESOURCE = 1 << 6; //mana ?
        const COOLDOWN = 1 << 7;
        const STAGGER = 1 << 8;
        const SHIELD = 1 << 9; //TODO nothing is mapped there
      
        const ANY = 1 << 20; // ignore all filters if set
    }
}