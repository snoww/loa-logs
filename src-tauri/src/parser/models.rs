use std::fmt::Display;
use std::str::FromStr;

use crate::parser::entity_tracker::Entity;
use bitflags::bitflags;
use hashbrown::{HashMap, HashSet};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DefaultOnError;

pub const DB_VERSION: i32 = 4;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum EntityType {
    #[default]
    UNKNOWN,
    MONSTER,
    BOSS,
    GUARDIAN,
    PLAYER,
    NPC,
    ESTHER,
    PROJECTILE,
    SUMMON,
}

impl Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            EntityType::UNKNOWN => "UNKNOWN".to_string(),
            EntityType::MONSTER => "MONSTER".to_string(),
            EntityType::BOSS => "BOSS".to_string(),
            EntityType::GUARDIAN => "GUARDIAN".to_string(),
            EntityType::PLAYER => "PLAYER".to_string(),
            EntityType::NPC => "NPC".to_string(),
            EntityType::ESTHER => "ESTHER".to_string(),
            EntityType::PROJECTILE => "PROJECTILE".to_string(),
            EntityType::SUMMON => "SUMMON".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl FromStr for EntityType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "UNKNOWN" => Ok(EntityType::UNKNOWN),
            "MONSTER" => Ok(EntityType::MONSTER),
            "BOSS" => Ok(EntityType::BOSS),
            "GUARDIAN" => Ok(EntityType::GUARDIAN),
            "PLAYER" => Ok(EntityType::PLAYER),
            "NPC" => Ok(EntityType::NPC),
            "ESTHER" => Ok(EntityType::ESTHER),
            _ => Ok(EntityType::UNKNOWN),
        }
    }
}

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Encounter {
    pub last_combat_packet: i64,
    pub fight_start: i64,
    pub local_player: String,
    pub entities: HashMap<String, EncounterEntity>,
    pub current_boss_name: String,
    pub current_boss: Option<EncounterEntity>,
    pub encounter_damage_stats: EncounterDamageStats,
    pub duration: i64,
    pub difficulty: Option<String>,
    pub favorite: bool,
    pub cleared: bool,
    pub boss_only_damage: bool,
}

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct EncounterDamageStats {
    pub total_damage_dealt: i64,
    pub top_damage_dealt: i64,
    pub total_damage_taken: i64,
    pub top_damage_taken: i64,
    pub dps: i64,
    pub most_damage_taken_entity: MostDamageTakenEntity,
    pub buffs: HashMap<u32, StatusEffect>,
    pub debuffs: HashMap<u32, StatusEffect>,
    pub total_shielding: u64,
    pub total_effective_shielding: u64,
    pub applied_shield_buffs: HashMap<u32, StatusEffect>,
    #[serde(skip)]
    pub unknown_buffs: HashSet<u32>,
    #[serde(skip)]
    pub max_stagger: i32,
    #[serde(skip)]
    pub stagger_start: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub misc: Option<EncounterMisc>,
}

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct MostDamageTakenEntity {
    pub name: String,
    pub damage_taken: i64,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EncounterEntity {
    pub id: u64,
    pub character_id: u64,
    pub npc_id: u32,
    pub name: String,
    pub entity_type: EntityType,
    pub class_id: u32,
    pub class: String,
    pub gear_score: f32,
    pub current_hp: i64,
    pub max_hp: i64,
    pub current_shield: u64,
    pub is_dead: bool,
    pub skills: HashMap<u32, Skill>,
    pub damage_stats: DamageStats,
    pub skill_stats: SkillStats,
    pub engraving_data: Option<PlayerEngravings>,
    pub gear_hash: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerEngravings {
    pub class_engravings: Option<Vec<PlayerEngraving>>,
    pub other_engravings: Option<Vec<PlayerEngraving>>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerEngraving {
    pub name: String,
    pub id: u32,
    pub level: u8,
    pub icon: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct Skill {
    pub id: u32,
    pub name: String,
    pub icon: String,
    pub total_damage: i64,
    pub max_damage: i64,
    pub max_damage_cast: i64,
    pub buffed_by: HashMap<u32, i64>,
    pub debuffed_by: HashMap<u32, i64>,
    pub buffed_by_support: i64,
    pub buffed_by_identity: i64,
    pub debuffed_by_support: i64,
    pub casts: i64,
    pub hits: i64,
    pub crits: i64,
    pub crit_damage: i64,
    pub back_attacks: i64,
    pub front_attacks: i64,
    pub back_attack_damage: i64,
    pub front_attack_damage: i64,
    pub dps: i64,
    pub cast_log: Vec<i32>,
    pub tripod_index: Option<TripodIndex>,
    pub tripod_level: Option<TripodLevel>,
    pub gem_cooldown: Option<u8>,
    pub gem_damage: Option<u8>,
    #[serde(skip)]
    pub tripod_data: Option<Vec<TripodData>>,
    #[serde(skip)]
    pub summon_sources: Option<Vec<u32>>,
    pub rdps_damage_received: i64,
    pub rdps_damage_received_support: i64,
    pub rdps_damage_given: i64,
    pub skill_cast_log: Vec<SkillCast>,
    #[serde(skip)]
    pub last_timestamp: i64,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct TripodData {
    pub index: u8,
    pub options: Vec<SkillFeatureOption>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase", default)]
pub struct TripodLevel {
    pub first: u16,
    pub second: u16,
    pub third: u16,
}

impl PartialEq for TripodLevel {
    fn eq(&self, other: &Self) -> bool {
        self.first == other.first && self.second == other.second && self.third == other.third
    }
}

impl Eq for TripodLevel {}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase", default)]
pub struct TripodIndex {
    pub first: u8,
    pub second: u8,
    pub third: u8,
}

impl PartialEq for TripodIndex {
    fn eq(&self, other: &Self) -> bool {
        self.first == other.first && self.second == other.second && self.third == other.third
    }
}

impl Eq for TripodIndex {}

#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct DamageStats {
    pub damage_dealt: i64,
    pub damage_taken: i64,
    pub buffed_by: HashMap<u32, i64>,
    pub debuffed_by: HashMap<u32, i64>,
    pub buffed_by_support: i64,
    pub buffed_by_identity: i64,
    pub debuffed_by_support: i64,
    pub crit_damage: i64,
    pub back_attack_damage: i64,
    pub front_attack_damage: i64,
    pub shields_given: u64,
    pub shields_received: u64,
    pub damage_absorbed: u64,
    pub damage_absorbed_on_others: u64,
    pub shields_given_by: HashMap<u32, u64>,
    pub shields_received_by: HashMap<u32, u64>,
    pub damage_absorbed_by: HashMap<u32, u64>,
    pub damage_absorbed_on_others_by: HashMap<u32, u64>,
    pub deaths: i64,
    pub death_time: i64,
    pub dps: i64,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub dps_average: Vec<i64>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub dps_rolling_10s_avg: Vec<i64>,
    pub rdps_damage_received: i64,
    pub rdps_damage_received_support: i64,
    pub rdps_damage_given: i64,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillStats {
    pub casts: i64,
    pub hits: i64,
    pub crits: i64,
    pub back_attacks: i64,
    pub front_attacks: i64,
    pub counters: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity_stats: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillCast {
    pub timestamp: i64,
    pub last: i64,
    pub hits: Vec<SkillHit>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillHit {
    pub timestamp: i64,
    pub damage: i64,
    pub crit: bool,
    pub back_attack: bool,
    pub front_attack: bool,
    pub buffed_by: Vec<u32>,
    pub debuffed_by: Vec<u32>,
    pub rdps_damage_received: i64,
    pub rdps_damage_received_support: i64,
}

#[derive(Debug)]
pub struct DamageData {
    pub skill_id: u32,
    pub skill_effect_id: u32,
    pub damage: i64,
    pub modifier: i32,
    pub target_current_hp: i64,
    pub target_max_hp: i64,
    pub damage_attribute: Option<u8>,
    pub damage_type: u8,
}

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub gauge1: u32,
    pub gauge2: u32,
    pub gauge3: u32,
}

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Stagger {
    pub current: u32,
    pub max: u32,
}

pub type IdentityLog = Vec<(i64, (u32, u32, u32))>;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IdentityArcanist {
    // timestamp, (percentage, card, card)
    pub log: Vec<(i32, (f32, u32, u32))>,
    pub average: f64,
    pub card_draws: HashMap<u32, u32>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IdentityArtistBard {
    // timestamp, (percentage, bubble)
    pub log: Vec<(i32, (f32, u32))>,
    pub average: f64,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IdentityGeneric {
    // timestamp, percentage
    pub log: Vec<(i32, f32)>,
    pub average: f64,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaggerStats {
    pub average: f64,
    #[serde(default)]
    pub staggers_per_min: f64,
    pub log: Vec<(i32, f32)>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
#[serde_as]
pub struct EncounterMisc {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stagger_stats: Option<StaggerStats>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub boss_hp_log: HashMap<String, Vec<BossHpLog>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raid_clear: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub party_info: Option<HashMap<i32, Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rdps_valid: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rdps_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ntp_fight_start: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manual_save: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct BossHpLog {
    pub time: i32,
    pub hp: i64,
    #[serde(default)]
    pub p: f32,
}

impl BossHpLog {
    pub fn new(time: i32, hp: i64, p: f32) -> Self {
        Self { time, hp, p }
    }
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Npc {
    pub id: i32,
    pub name: String,
    pub grade: String,
    #[serde(rename = "type")]
    pub npc_type: String,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Esther {
    pub name: String,
    pub icon: String,
    pub skills: Vec<i32>,
    #[serde(alias = "npcs")]
    pub npc_ids: Vec<u32>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillData {
    pub id: i32,
    pub name: String,
    pub desc: String,
    #[serde(alias = "classid", alias = "classId")]
    pub class_id: u32,
    pub icon: String,
    #[serde(alias = "identitycategory", alias = "identityCategory")]
    pub identity_category: Option<String>,
    #[serde(alias = "groups")]
    pub groups: Option<Vec<i32>>,
    #[serde(alias = "summonids", alias = "summonIds")]
    pub summon_ids: Option<Vec<i32>>,
    #[serde(alias = "summonsourceskill", alias = "summonSourceSkill")]
    pub summon_source_skill: Option<Vec<u32>>,
    #[serde(alias = "sourceskill", alias = "sourceSkill")]
    pub source_skill: Option<Vec<u32>>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillEffectData {
    pub id: i32,
    pub comment: String,
    #[serde(skip)]
    pub stagger: i32,
    #[serde(rename(deserialize = "sourceskill"))]
    pub source_skill: Option<Vec<u32>>,
    #[serde(rename(deserialize = "directionalmask"))]
    pub directional_mask: i32,
    #[serde(rename(deserialize = "itemname"))]
    pub item_name: Option<String>,
    #[serde(skip, rename(deserialize = "itemdesc"))]
    pub item_desc: Option<String>,
    pub icon: Option<String>,
    #[serde(rename(deserialize = "itemcategory"))]
    pub item_category: Option<String>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillBuffData {
    pub id: i32,
    pub name: String,
    pub desc: String,
    pub icon: String,
    #[serde(rename(deserialize = "iconshowtype"))]
    pub icon_show_type: String,
    pub duration: i32,
    // buff | debuff
    pub category: String,
    #[serde(rename(deserialize = "type"))]
    pub buff_type: String,
    #[serde(rename(deserialize = "statuseffectvalues"))]
    pub status_effect_values: Option<Vec<i32>>,
    #[serde(rename(deserialize = "buffcategory"))]
    pub buff_category: String,
    pub target: String,
    #[serde(rename(deserialize = "uniquegroup"))]
    pub unique_group: u32,
    #[serde(rename(deserialize = "overlapflag"))]
    pub overlap_flag: i32,
    #[serde(skip_serializing, rename(deserialize = "passiveoption"))]
    pub passive_option: Vec<PassiveOption>,
    #[serde(rename(deserialize = "sourceskill"))]
    pub source_skill: Option<Vec<u32>>,
    #[serde(rename(deserialize = "setname"))]
    pub set_name: Option<String>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct PassiveOption {
    #[serde(rename(deserialize = "type"))]
    pub option_type: String,
    #[serde(rename(deserialize = "keystat"))]
    pub key_stat: String,
    #[serde(rename(deserialize = "keyindex"))]
    pub key_index: i32,
    pub value: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StatusEffect {
    pub target: StatusEffectTarget,
    pub category: String,
    pub buff_category: String,
    pub buff_type: u32,
    pub unique_group: u32,
    pub source: StatusEffectSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum StatusEffectTarget {
    #[default]
    OTHER,
    PARTY,
    SELF,
}

#[derive(Debug, Clone, Serialize, Default, Deserialize)]
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
        const RESOURCE = 1 << 6;
        const COOLDOWN = 1 << 7;
        const STAGGER = 1 << 8;
        const SHIELD = 1 << 9;

        const ANY = 1 << 20;
    }
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct CombatEffectData {
    pub effects: Vec<CombatEffectDetail>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct CombatEffectDetail {
    pub ratio: i32,
    pub cooldown: i32,
    pub conditions: Vec<CombatEffectCondition>,
    pub actions: Vec<CombatEffectAction>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct CombatEffectCondition {
    #[serde(rename(deserialize = "type"))]
    pub condition_type: String,
    pub actor: String,
    pub arg: i32,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct CombatEffectAction {
    #[serde(rename(deserialize = "type"))]
    pub action_type: String,
    pub actor: String,
    pub args: Vec<i32>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct SkillFeatureLevelData {
    pub tripods: HashMap<u8, Tripod>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Tripod {
    pub name: String,
    pub entries: Vec<SkillFeatureOption>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct SkillFeatureOption {
    #[serde(rename(deserialize = "type"))]
    pub effect_type: String,
    pub level: u16,
    #[serde(rename(deserialize = "paramtype"))]
    pub param_type: String,
    pub param: Vec<i32>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct ItemSet {
    #[serde(rename(deserialize = "itemids"))]
    pub item_ids: Vec<u32>,
    pub value: HashMap<u8, ItemSetDetails>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct ItemSetDetails {
    pub desc: String,
    pub options: Vec<PassiveOption>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct ItemSetShort {
    pub set_name: String,
    pub level: u8,
}

pub type ItemSetLevel = HashMap<u8, ItemSetCount>;
pub type ItemSetCount = HashMap<u8, ItemSetDetails>;

#[derive(Debug, Default, Deserialize, Clone)]
pub struct EngravingData {
    pub id: u32,
    pub name: String,
    pub icon: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EncounterPreview {
    pub id: i32,
    pub fight_start: i64,
    pub boss_name: String,
    pub duration: i32,
    pub classes: Vec<i32>,
    pub names: Vec<String>,
    pub difficulty: Option<String>,
    pub local_player: String,
    pub my_dps: i64,
    pub favorite: bool,
    pub cleared: bool,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EncountersOverview {
    pub encounters: Vec<EncounterPreview>,
    pub total_encounters: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SearchFilter {
    pub bosses: Vec<String>,
    pub classes: Vec<String>,
    pub min_duration: i32,
    pub max_duration: i32,
    pub cleared: bool,
    pub favorite: bool,
    pub difficulty: String,
    pub boss_only_damage: bool,
    pub sort: String,
    pub order: u8,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub general: GeneralSettings,
    pub shortcuts: Shortcuts,
    pub meter: MeterTabs,
    pub logs: LogTabs,
    pub buffs: BuffSettings,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GeneralSettings {
    pub low_performance_mode: bool,
    #[serde(default = "default_true")]
    pub show_names: bool,
    pub show_gear_score: bool,
    pub hide_names: bool,
    #[serde(default = "default_true")]
    pub show_esther: bool,
    #[serde(default = "default_true")]
    pub show_date: bool,
    #[serde(default = "default_true")]
    pub show_difficulty: bool,
    pub show_gate: bool,
    #[serde(default = "default_true")]
    pub split_lines: bool,
    pub underline_hovered: bool,
    pub show_details: bool,
    pub show_shields: bool,
    pub show_tanked: bool,
    pub show_bosses: bool,
    pub hide_logo: bool,
    pub accent_color: String,
    pub raw_socket: bool,
    #[serde(default = "default_true")]
    pub auto_iface: bool,
    pub if_desc: String,
    pub ip: String,
    pub port: u16,
    pub blur: bool,
    pub blur_win11: bool,
    pub transparent: bool,
    #[serde(default = "default_scale")]
    pub scale: String,
    #[serde(default = "default_scale")]
    pub log_scale: String,
    #[serde(default = "default_true")]
    pub always_on_top: bool,
    #[serde(default = "default_true")]
    pub boss_only_damage: bool,
    #[serde(default = "default_true")]
    pub keep_favorites: bool,
    pub hide_meter_on_start: bool,
    pub hide_logs_on_start: bool,
    pub constant_local_player_color: bool,
    #[serde(default = "default_true")]
    pub boss_only_damage_default_on: bool,
    pub start_on_boot: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Shortcuts {
    pub hide_meter: Shortcut,
    pub show_logs: Shortcut,
    pub show_latest_encounter: Shortcut,
    pub reset_session: Shortcut,
    pub pause_session: Shortcut,
    pub manual_save: Shortcut,
    pub disable_clickthrough: Shortcut,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Shortcut {
    pub modifier: String,
    pub key: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LogTabs {
    pub abbreviate_header: bool,
    #[serde(default = "default_true")]
    pub split_party_buffs: bool,
    pub damage: bool,
    pub dps: bool,
    pub damage_percent: bool,
    pub death_time: bool,
    pub crit_rate: bool,
    pub crit_dmg: bool,
    pub front_atk: bool,
    pub back_atk: bool,
    pub percent_buff_by_sup: bool,
    pub percent_identity_by_sup: bool,
    #[serde(default = "default_true")]
    pub positional_dmg_percent: bool,
    pub percent_brand: bool,
    pub counters: bool,
    pub min_encounter_duration: i32,
    #[serde(default = "default_true")]
    pub rdps_split_party: bool,
    #[serde(default = "default_true")]
    pub rdps_damage_given: bool,
    #[serde(default = "default_true")]
    pub rdps_damage_received: bool,
    #[serde(default = "default_true")]
    pub rdps_contribution: bool,
    #[serde(default = "default_true")]
    pub rdps_s_contribution: bool,
    #[serde(default = "default_true")]
    pub rdps_d_contribution: bool,
    #[serde(default = "default_true")]
    pub rdps_syn: bool,
    #[serde(default = "default_true")]
    pub rdps_s_syn: bool,
    #[serde(default = "default_true")]
    pub rdps_d_syn: bool,
    #[serde(default = "default_true")]
    pub ssyn: bool,
    pub breakdown: BreakdownTabs,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MeterTabs {
    pub boss_hp: bool,
    pub boss_hp_bar: bool,
    pub split_boss_hp_bar: bool,
    pub abbreviate_header: bool,
    pub show_time_until_kill: bool,
    pub show_class_colors: bool,
    #[serde(default = "default_true")]
    pub split_party_buffs: bool,
    #[serde(default = "default_true")]
    pub pin_self_party: bool,
    pub damage: bool,
    pub dps: bool,
    pub damage_percent: bool,
    pub death_time: bool,
    pub crit_rate: bool,
    pub crit_dmg: bool,
    pub front_atk: bool,
    pub back_atk: bool,
    pub percent_buff_by_sup: bool,
    pub percent_identity_by_sup: bool,
    #[serde(default = "default_true")]
    pub positional_dmg_percent: bool,
    pub percent_brand: bool,
    pub counters: bool,
    #[serde(default = "default_true")]
    pub rdps_split_party: bool,
    pub rdps_damage_given: bool,
    pub rdps_damage_received: bool,
    pub rdps_contribution: bool,
    pub rdps_s_contribution: bool,
    pub rdps_d_contribution: bool,
    #[serde(default = "default_true")]
    pub rdps_syn: bool,
    #[serde(default = "default_true")]
    pub rdps_s_syn: bool,
    #[serde(default = "default_true")]
    pub rdps_d_syn: bool,
    #[serde(default = "default_true")]
    pub ssyn: bool,
    pub breakdown: BreakdownTabs,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BreakdownTabs {
    pub damage: bool,
    pub dps: bool,
    pub damage_percent: bool,
    pub crit_rate: bool,
    pub crit_dmg: bool,
    pub front_atk: bool,
    pub back_atk: bool,
    pub percent_buff_by_sup: bool,
    pub percent_identity_by_sup: bool,
    pub percent_brand: bool,
    pub avg_damage: bool,
    pub max_damage: bool,
    pub casts: bool,
    pub cpm: bool,
    pub hits: bool,
    pub hpm: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BuffSettings {
    #[serde(default = "default_true")]
    pub default: bool,
}

#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncounterDbInfo {
    pub size: String,
    pub total_encounters: i32,
    pub total_encounters_filtered: i32,
}

#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
#[repr(i32)]
pub enum HitOption {
    NONE,
    BACK_ATTACK,
    FRONTAL_ATTACK,
    FLANK_ATTACK,
    MAX,
}

#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
#[repr(u32)]
pub enum HitFlag {
    NORMAL,
    CRITICAL,
    MISS,
    INVINCIBLE,
    DOT,
    IMMUNE,
    IMMUNE_SILENCED,
    FONT_SILENCED,
    DOT_CRITICAL,
    DODGE,
    REFLECT,
    DAMAGE_SHARE,
    DODGE_HIT,
    MAX,
}

pub struct CombatEffectConditionData<'a> {
    pub self_entity: &'a Entity,
    pub target_entity: &'a Entity,
    pub caster_entity: &'a Entity,
    pub skill: Option<&'a SkillData>,
    pub hit_option: i32,
    pub target_count: i32,
}

#[derive(Debug, Clone)]
pub struct RdpsData {
    pub multi_dmg: RdpsRates,
    pub atk_pow_sub_rate_2: RdpsSelfRates,
    pub atk_pow_sub_rate_1: RdpsRates,
    pub skill_dmg_rate: RdpsSelfRates,
    pub atk_pow_amplify: Vec<RdpsBuffData>,
    pub crit: RdpsSelfRates,
    pub crit_dmg_rate: f64,
}

impl Default for RdpsData {
    fn default() -> Self {
        Self {
            multi_dmg: RdpsRates::default(),
            atk_pow_sub_rate_2: RdpsSelfRates::default(),
            atk_pow_sub_rate_1: RdpsRates::default(),
            skill_dmg_rate: RdpsSelfRates::default(),
            atk_pow_amplify: Vec::new(),
            crit: RdpsSelfRates::default(),
            crit_dmg_rate: 2.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RdpsRates {
    pub sum_rate: f64,
    pub total_rate: f64,
    pub values: Vec<RdpsBuffData>,
}

impl Default for RdpsRates {
    fn default() -> Self {
        Self {
            sum_rate: 0.0,
            total_rate: 1.0,
            values: Vec::new(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct RdpsSelfRates {
    pub self_sum_rate: f64,
    pub sum_rate: f64,
    pub values: Vec<RdpsBuffData>,
}

#[derive(Debug, Default, Clone)]
pub struct RdpsBuffData {
    pub caster: String,
    pub rate: f64,
}

pub struct ItemSetInfo {
    pub item_ids: HashMap<u32, ItemSetShort>,
    pub set_names: HashMap<String, ItemSetLevel>,
}

lazy_static! {
    pub static ref NPC_DATA: HashMap<u32, Npc> = {
        let json_str = include_str!("../../meter-data/Npc.json");
        serde_json::from_str(json_str).unwrap()
    };
    pub static ref SKILL_DATA: HashMap<u32, SkillData> = {
        let json_str = include_str!("../../meter-data/Skill.json");
        serde_json::from_str(json_str).unwrap()
    };
    pub static ref SKILL_EFFECT_DATA: HashMap<u32, SkillEffectData> = {
        let json_str = include_str!("../../meter-data/SkillEffect.json");
        serde_json::from_str(json_str).unwrap()
    };
    pub static ref SKILL_BUFF_DATA: HashMap<u32, SkillBuffData> = {
        let json_str = include_str!("../../meter-data/SkillBuff.json");
        serde_json::from_str(json_str).unwrap()
    };
    pub static ref COMBAT_EFFECT_DATA: HashMap<i32, CombatEffectData> = {
        let json_str = include_str!("../../meter-data/CombatEffect.json");
        serde_json::from_str(json_str).unwrap()
    };
    pub static ref SKILL_FEATURE_DATA: HashMap<u32, SkillFeatureLevelData> = {
        let json_str = include_str!("../../meter-data/SkillFeature.json");
        serde_json::from_str(json_str).unwrap()
    };
    pub static ref ENGRAVING_DATA: HashMap<u32, EngravingData> = {
        let json_str = include_str!("../../meter-data/Ability.json");
        serde_json::from_str(json_str).unwrap()
    };
    pub static ref ITEM_SET_DATA: HashMap<String, HashMap<u8, ItemSet>> = {
        let json_str = include_str!("../../meter-data/ItemSet.json");
        serde_json::from_str(json_str).unwrap()
    };
    pub static ref ITEM_SET_INFO: ItemSetInfo = {
        let mut item_set_ids: HashMap<u32, ItemSetShort> = HashMap::new();
        let mut item_set_names: HashMap<String, ItemSetLevel> = HashMap::new();

        for (set_name, set_name_data) in ITEM_SET_DATA.iter() {
            let mut item_set_level: ItemSetLevel = HashMap::new();
            for (level, set_level_data) in set_name_data.iter() {
                let mut item_set_count: ItemSetCount = HashMap::new();
                for (count, set_count_data) in set_level_data.value.iter() {
                    item_set_count.insert(*count, set_count_data.clone());
                }
                item_set_level.insert(*level, item_set_count);
                for item_id in set_level_data.item_ids.iter() {
                    item_set_ids.insert(
                        *item_id,
                        ItemSetShort {
                            set_name: set_name.clone(),
                            level: *level,
                        },
                    );
                }
            }
            item_set_names.insert(set_name.clone(), item_set_level);
        }

        ItemSetInfo {
            item_ids: item_set_ids,
            set_names: item_set_names,
        }
    };
    pub static ref ESTHER_DATA: Vec<Esther> = {
        let json_str = include_str!("../../meter-data/Esther.json");
        serde_json::from_str(json_str).unwrap()
    };
    pub static ref VALID_ZONES: HashSet<u32> = {
        let valid_zones = [
            30801, 30802, 30803, 30804, 30805, 30806, 30807, 30835, 37001, 37002, 37003, 37011,
            37012, 37021, 37022, 37031, 37032, 37041, 37042, 37051, 37061, 37071, 37072, 37081,
            37091, 37092, 37093, 37094, 37101, 37102, 37111, 37112, 37121, 37122, 37123, 37124,
            308010, 308011, 308012, 308014, 308015, 308016, 308017, 308018, 308019, 308020, 308021,
            308022, 308023, 308024, 308025, 308026, 308027, 308028, 308029, 308030, 308037, 308039,
            308040, 308041, 308042, 308043, 308044, 308239, 308339, 308410, 308411, 308412, 308414,
            308415, 308416, 308417, 308418, 308419, 308420, 308421, 308422, 308423, 308424, 308425,
            308426, 308428, 308429, 308430, 308437, 309020,
        ];

        valid_zones.iter().cloned().collect()
    };
    pub static ref STAT_TYPE_MAP: HashMap<&'static str, u32> = {
        let mut map = HashMap::new();
        map.insert("none", 0);
        map.insert("hp", 1);
        map.insert("mp", 2);
        map.insert("str", 3);
        map.insert("agi", 4);
        map.insert("int", 5);
        map.insert("con", 6);
        map.insert("str_x", 7);
        map.insert("agi_x", 8);
        map.insert("int_x", 9);
        map.insert("con_x", 10);
        map.insert("criticalhit", 15);
        map.insert("specialty", 16);
        map.insert("oppression", 17);
        map.insert("rapidity", 18);
        map.insert("endurance", 19);
        map.insert("mastery", 20);
        map.insert("criticalhit_x", 21);
        map.insert("specialty_x", 22);
        map.insert("oppression_x", 23);
        map.insert("rapidity_x", 24);
        map.insert("endurance_x", 25);
        map.insert("mastery_x", 26);
        map.insert("max_hp", 27);
        map.insert("max_mp", 28);
        map.insert("max_hp_x", 29);
        map.insert("max_mp_x", 30);
        map.insert("max_hp_x_x", 31);
        map.insert("max_mp_x_x", 32);
        map.insert("normal_hp_recovery", 33);
        map.insert("combat_hp_recovery", 34);
        map.insert("normal_hp_recovery_rate", 35);
        map.insert("combat_hp_recovery_rate", 36);
        map.insert("normal_mp_recovery", 37);
        map.insert("combat_mp_recovery", 38);
        map.insert("normal_mp_recovery_rate", 39);
        map.insert("combat_mp_recovery_rate", 40);
        map.insert("self_recovery_rate", 41);
        map.insert("drain_hp_dam_rate", 42);
        map.insert("drain_mp_dam_rate", 43);
        map.insert("dam_reflection_rate", 44);
        map.insert("char_attack_dam", 47);
        map.insert("skill_effect_dam_addend", 48);
        map.insert("attack_power_rate", 49);
        map.insert("skill_damage_rate", 50);
        map.insert("attack_power_rate_x", 51);
        map.insert("skill_damage_rate_x", 52);
        map.insert("cooldown_reduction", 53);
        map.insert("paralyzation_point_rate", 54);
        map.insert("def", 55);
        map.insert("res", 56);
        map.insert("def_x", 57);
        map.insert("res_x", 58);
        map.insert("def_x_x", 59);
        map.insert("res_x_x", 60);
        map.insert("def_pen_rate", 67);
        map.insert("res_pen_rate", 68);
        map.insert("physical_inc_rate", 69);
        map.insert("magical_inc_rate", 70);
        map.insert("self_shield_rate", 71);
        map.insert("hit_rate", 72);
        map.insert("dodge_rate", 73);
        map.insert("critical_hit_rate", 74);
        map.insert("critical_res_rate", 75);
        map.insert("critical_dam_rate", 76);
        map.insert("attack_speed", 77);
        map.insert("attack_speed_rate", 78);
        map.insert("move_speed", 79);
        map.insert("move_speed_rate", 80);
        map.insert("prop_move_speed", 81);
        map.insert("prop_move_speed_rate", 82);
        map.insert("vehicle_move_speed", 83);
        map.insert("vehicle_move_speed_rate", 84);
        map.insert("ship_move_speed", 85);
        map.insert("ship_move_speed_rate", 86);
        map.insert("fire_dam_rate", 87);
        map.insert("ice_dam_rate", 88);
        map.insert("electricity_dam_rate", 89);
        map.insert("earth_dam_rate", 91);
        map.insert("dark_dam_rate", 92);
        map.insert("holy_dam_rate", 93);
        map.insert("elements_dam_rate", 94);
        map.insert("fire_res_rate", 95);
        map.insert("ice_res_rate", 96);
        map.insert("electricity_res_rate", 97);
        map.insert("earth_res_rate", 99);
        map.insert("dark_res_rate", 100);
        map.insert("holy_res_rate", 101);
        map.insert("elements_res_rate", 102);
        map.insert("self_cc_time_rate", 105);
        map.insert("enemy_cc_time_rate", 106);
        map.insert("identity_value1", 107);
        map.insert("identity_value2", 108);
        map.insert("identity_value3", 109);
        map.insert("awakening_dam_rate", 110);
        map.insert("item_drop_rate", 111);
        map.insert("gold_rate", 112);
        map.insert("exp_rate", 113);
        map.insert("attack_power_addend", 123);
        map.insert("npc_species_humanoid_dam_rate", 125);
        map.insert("npc_species_devil_dam_rate", 126);
        map.insert("npc_species_substance_dam_rate", 127);
        map.insert("npc_species_undead_dam_rate", 128);
        map.insert("npc_species_plant_dam_rate", 129);
        map.insert("npc_species_insect_dam_rate", 130);
        map.insert("npc_species_spirit_dam_rate", 131);
        map.insert("npc_species_wild_beast_dam_rate", 132);
        map.insert("npc_species_mechanic_dam_rate", 133);
        map.insert("npc_species_ancient_dam_rate", 134);
        map.insert("npc_species_god_dam_rate", 135);
        map.insert("npc_species_archfiend_dam_rate", 136);
        map.insert("vitality", 137);
        map.insert("ship_booter_speed", 138);
        map.insert("ship_wreck_speed_rate", 139);
        map.insert("island_speed_rate", 140);
        map.insert("attack_power_sub_rate_1", 141);
        map.insert("attack_power_sub_rate_2", 142);
        map.insert("physical_inc_sub_rate_1", 143);
        map.insert("physical_inc_sub_rate_2", 144);
        map.insert("magical_inc_sub_rate_1", 145);
        map.insert("magical_inc_sub_rate_2", 146);
        map.insert("skill_damage_sub_rate_1", 147);
        map.insert("skill_damage_sub_rate_2", 148);
        map.insert("resource_recovery_rate", 149);
        map.insert("weapon_dam", 151);
        map
    };
    pub static ref STAT_TYPE_MAP_TRA: HashMap<u32, &'static str> =
        STAT_TYPE_MAP.iter().map(|(k, v)| (*v, *k)).collect();
    pub static ref IDENTITY_CATEGORY: HashMap<&'static str, i32> = {
        let mut map = HashMap::new();
        map.insert("none", 0);
        map.insert("berserker_normal", 1);
        map.insert("berserker_rush", 2);
        map.insert("warlord_normal", 3);
        map.insert("warlord_shield_of_battlefield", 4);
        map.insert("destroyer_normal", 5);
        map.insert("destroyer_focus", 6);
        map.insert("destroyer_release", 7);
        map.insert("battle_master_normal", 8);
        map.insert("battle_master_bubble", 9);
        map.insert("infighter_normal", 10);
        map.insert("infighter_vigor", 11);
        map.insert("infighter_shock", 12);
        map.insert("forcemaster_normal", 13);
        map.insert("forcemaster_soul", 14);
        map.insert("lance_master_normal", 15);
        map.insert("lance_master_wild", 16);
        map.insert("lance_master_focus", 17);
        map.insert("devil_hunter_normal", 18);
        map.insert("devil_hunter_pistol", 19);
        map.insert("devil_hunter_shotgun", 20);
        map.insert("devil_hunter_rifle", 21);
        map.insert("blaster_normal", 22);
        map.insert("blaster_cannon", 23);
        map.insert("hawkeye_normal", 24);
        map.insert("hawkeye_summon", 25);
        map.insert("summoner_normal", 26);
        map.insert("summoner_ancient", 27);
        map.insert("arcana_normal", 28);
        map.insert("arcana_stack", 29);
        map.insert("arcana_ruin", 30);
        map.insert("arcana_card", 31);
        map.insert("bard_normal", 32);
        map.insert("bard_serenade", 33);
        map.insert("blade_burst", 34);
        map.insert("holyknight_normal", 35);
        map.insert("holyknight_holy", 36);
        map.insert("holyknight_retribution", 37);
        map.insert("demonic_normal", 38);
        map.insert("demonic_capture", 39);
        map.insert("demonic_demon", 40);
        map.insert("warlord_lance", 41);
        map.insert("reaper_normal", 42);
        map.insert("reaper_dagger", 43);
        map.insert("reaper_shadow", 44);
        map.insert("reaper_swoop", 45);
        map.insert("scouter_scout", 46);
        map.insert("scouter_drone", 47);
        map.insert("scouter_hyper_sync", 48);
        map.insert("scouter_fusion", 49);
        map.insert("blade_normal", 50);
        map.insert("elemental_master_normal", 51);
        map.insert("elemental_master_fire", 52);
        map.insert("elemental_master_electricity", 53);
        map.insert("elemental_master_ice", 54);
        map.insert("yinyangshi_normal", 55);
        map.insert("yinyangshi_yin", 56);
        map.insert("yinyangshi_yang", 57);
        map.insert("weather_artist_weapon", 58);
        map.insert("weather_artist_weather", 59);
        map.insert("summoner_summon", 60);
        map.insert("soul_eater_hollow", 61);
        map.insert("soul_eater_killer", 62);
        map.insert("soul_eater_death", 63);
        map
    };
    pub static ref NPC_GRADE: HashMap<&'static str, i32> = {
        let mut map = HashMap::new();
        map.insert("none", 0);
        map.insert("underling", 1);
        map.insert("normal", 2);
        map.insert("elite", 3);
        map.insert("named", 4);
        map.insert("seed", 5);
        map.insert("boss", 6);
        map.insert("raid", 7);
        map.insert("lucky", 8);
        map.insert("epic_raid", 9);
        map.insert("commander", 10);
        map
    };
}

fn default_true() -> bool {
    true
}

fn default_scale() -> String {
    "1".to_string()
}
