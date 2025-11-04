use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumString};

use crate::{models::{HitFlag, HitOption}, settings::Settings};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadResult {
    pub settings: Settings,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EncounterPreview {
    pub id: i32,
    pub fight_start: i64,
    pub boss_name: String,
    pub duration: i64,
    pub classes: Vec<i32>,
    pub names: Vec<String>,
    pub difficulty: Option<String>,
    pub local_player: String,
    pub my_dps: i64,
    pub favorite: bool,
    pub cleared: bool,
    pub spec: Option<String>,
    pub support_ap: Option<f32>,
    pub support_brand: Option<f32>,
    pub support_identity: Option<f32>,
    pub support_hyper: Option<f32>,
    pub udps: Option<i64>,
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
    pub min_duration: i32,
    pub max_duration: i32,
    pub cleared: bool,
    pub favorite: bool,
    pub difficulty: String,
    pub boss_only_damage: bool,
    pub sort: String,
    pub order: String,
    pub raids_only: bool,
}

#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncounterDbInfo {
    pub size: String,
    pub total_encounters: i32,
    pub total_encounters_filtered: i32,
}

#[derive(Debug, Clone)]
pub struct CastEvent {
    pub timestamp: i64,
    pub cooldown_duration_ms: i64,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct InspectInfo {
    pub combat_power: Option<CombatPower>,
    pub ark_passive_enabled: bool,
    pub ark_passive_data: Option<ArkPassiveData>,
    pub engravings: Option<Vec<u32>>,
    pub gems: Option<Vec<GemData>>,
    pub loadout_snapshot: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ArkPassiveData {
    pub evolution: Option<Vec<ArkPassiveNode>>,
    pub enlightenment: Option<Vec<ArkPassiveNode>>,
    pub leap: Option<Vec<ArkPassiveNode>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ArkPassiveNode {
    pub id: u32,
    pub lv: u8,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CombatPower {
    // 1 for dps, 2 for support
    pub id: u32,
    pub score: f32,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GemData {
    pub tier: u8,
    pub skill_id: u32,
    pub gem_type: u8,
    pub value: u32,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Engraving {
    pub id: u32,
    pub level: u8,
}

pub struct SupportBuffs {
    pub brand: f64,
    pub buff: f64,
    pub identity: f64,
    pub hyper: f64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, EnumString, AsRefStr)]
#[strum(serialize_all = "title_case")]
#[repr(u32)]
pub enum RaidDifficulty {
    #[default]
    Unknown = 255,
    Normal = 0,
    Hard = 1,
    Inferno = 2,
    Challenge = 3,
    Solo = 4,
    TheFirst = 5,
    Trial = 7
}

impl RaidDifficulty {
    pub fn from_raid_id(value: u32) -> Self {
        match value {
            308226 | 308227 | 308239 | 308339 => {
                Self::Trial
            }
            308428 | 308429 | 308420 | 308410 | 308411 | 308414 | 308422 | 308424
            | 308421 | 308412 | 308423 | 308426 | 308416 | 308419 | 308415 | 308437
            | 308417 | 308418 | 308425 | 308430 => {
                Self::Challenge
            }
            _ => {
                Self::Unknown
            }
        }
    }
}

impl From<u32> for RaidDifficulty {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Normal,
            1 => Self::Hard,
            2 => Self::Inferno,
            3 => Self::Challenge,
            4 => Self::Solo,
            5 => Self::TheFirst,
            _ => Self::Unknown,
        }
    }
}

pub enum TriggerSignal {
    Unknown(u32),
    Clear(u32),
    Wipe(u32)
}

impl From<u32> for TriggerSignal {
    fn from(value: u32) -> Self {
        match value {
            57 | 59 | 61 | 63 | 74 | 76 => Self::Clear(value),
            58 | 60 | 62 | 64 | 75 | 77 => Self::Wipe(value),
            value => Self::Unknown(value)
        }
    }
}