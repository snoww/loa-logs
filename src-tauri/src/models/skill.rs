use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

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
    pub buffed_by_hat: i64,
    pub debuffed_by_support: i64,
    pub casts: i64,
    pub hits: i64,
    pub crits: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjusted_crit: Option<f64>,
    pub crit_damage: i64,
    pub back_attacks: i64,
    pub front_attacks: i64,
    pub back_attack_damage: i64,
    pub front_attack_damage: i64,
    pub dps: i64,
    pub cast_log: Vec<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tripod_index: Option<TripodIndex>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tripod_level: Option<TripodLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gem_cooldown: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gem_tier: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gem_damage: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gem_tier_dmg: Option<u8>,
    pub rdps_damage_received: i64,
    pub rdps_damage_received_support: i64,
    pub rdps_damage_given: i64,
    pub skill_cast_log: Vec<SkillCast>,

    #[serde(default)]
    pub is_hyper_awakening: bool,
    // for skills that cannot crit or be buffed
    // like hyper awakening, paradise orb, transcendence, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub special: Option<bool>,

    #[serde(skip)]
    pub last_timestamp: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_available: Option<i64>, // total time skill was available to cast
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

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct TripodData {
    pub index: u8,
    pub options: Vec<SkillFeatureOption>,
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
