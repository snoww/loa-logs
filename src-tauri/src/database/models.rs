use std::collections::BTreeMap;
use hashbrown::HashMap;

use crate::models::*;

pub struct InsertSyncLogsArgs {
    pub encounter: i32,
    pub upstream: String,
    pub failed: bool
}

pub struct GetEncounterPreviewArgs {
    pub page: i32,
    pub page_size: i32,
    pub search: String,
    pub filter: SearchFilter,
}

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
    pub manual: bool,
    pub skill_cast_log: HashMap<u64, HashMap<u32, BTreeMap<i64, SkillCast>>>,
    pub skill_cooldowns: HashMap<u32, Vec<CastEvent>>,
}