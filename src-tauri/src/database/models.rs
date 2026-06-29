use hashbrown::HashMap;
use std::collections::BTreeMap;

use crate::models::*;

pub struct InsertSyncLogsArgs {
    pub encounter: i32,
    pub upstream: String,
    pub failed: bool,
}

pub struct GetEncounterPreviewArgs {
    pub page: i32,
    pub page_size: i32,
    pub search: String,
    pub filter: SearchFilter,
}

/// Sanitized cleared-encounter summary exposed by the read-only local API.
/// Intentionally contains no damage, player breakdowns, or party details.
#[derive(Debug, Clone)]
pub struct MeterClear {
    pub id: i64,
    pub boss: String,
    pub difficulty: Option<String>,
    pub fight_start_ms: i64,
    pub duration_ms: i64,
    pub local_player: Option<String>,
    pub upload_id: Option<String>,
}

/// Latest class/ilvl metadata the meter knows for a local character.
#[derive(Debug, Clone)]
pub struct MeterCharacter {
    pub name: String,
    pub class_id: i32,
    pub class: Option<String>,
    pub gear_score: f32,
}

#[derive(Debug, Clone)]
pub struct InsertEncounterArgs {
    pub encounter: Encounter,
    pub damage_log: HashMap<String, Vec<(i64, i64)>>,
    pub cast_log: HashMap<String, HashMap<u32, Vec<i32>>>,
    pub boss_hp_log: HashMap<String, Vec<BossHpLog>>,
    pub raid_clear: bool,
    pub party_info: Vec<Vec<String>>,
    pub raid_difficulty: String,
    pub region: Option<String>,
    pub player_info: Option<HashMap<String, InspectInfo>>,
    pub meter_version: String,
    pub ntp_fight_start: i64,
    pub rdps_valid: bool,
    pub rdps_message: Option<String>,
    pub manual: bool,
    pub skill_cast_log: HashMap<u64, HashMap<u32, BTreeMap<i64, SkillCast>>>,
    pub skill_cooldowns: HashMap<u32, Vec<CastEvent>>,
    pub intermission_start: Option<i64>,
    pub intermission_end: Option<i64>,
    pub contribution_splits: Vec<ContributionSplit>,
}
