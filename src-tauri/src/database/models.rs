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

pub struct EncounterColumns;

impl EncounterColumns {
    pub const LAST_COMBAT_PACKET: usize = 0;
    pub const FIGHT_START: usize = 1;
    pub const LOCAL_PLAYER: usize = 2;
    pub const CURRENT_BOSS_NAME: usize = 3;
    pub const DURATION: usize = 4;
    pub const TOTAL_DAMAGE_DEALT: usize = 5;
    pub const TOP_DAMAGE_DEALT: usize = 6;
    pub const TOTAL_DAMAGE_TAKEN: usize = 7;
    pub const TOP_DAMAGE_TAKEN: usize = 8;
    pub const DPS: usize = 9;
    pub const BUFFS: usize = 10;
    pub const DEBUFFS: usize = 11;
    pub const MISC: usize = 12;
    pub const DIFFICULTY: usize = 13;
    pub const FAVORITE: usize = 14;
    pub const CLEARED: usize = 15;
    pub const BOSS_ONLY_DAMAGE: usize = 16;
    pub const TOTAL_SHIELDING: usize = 17;
    pub const TOTAL_EFFECTIVE_SHIELDING: usize = 18;
    pub const APPLIED_SHIELD_BUFFS: usize = 19;
    pub const BOSS_HP_LOG: usize = 20;
}

pub struct EncounterPreviewColumns;

impl EncounterPreviewColumns {
    pub const ID: usize = 0;
    pub const FIGHT_START: usize = 1;
    pub const BOSS_NAME: usize = 2;
    pub const DURATION: usize = 3;
    pub const DIFFICULTY: usize = 4;
    pub const FAVORITE: usize = 5;
    pub const CLEARED: usize = 6;
    pub const LOCAL_PLAYER: usize = 7;
    pub const MY_DPS: usize = 8;
    pub const CLASS_NAMES: usize = 9;
    pub const SPEC: usize = 10;
    pub const SUPPORT_AP: usize = 11;
    pub const SUPPORT_BRAND: usize = 12;
    pub const SUPPORT_IDENTITY: usize = 13;
    pub const SUPPORT_HYPER: usize = 14;
}

pub struct EncounterEntityColumns;

impl EncounterEntityColumns {
    pub const NAME: usize = 0;
    pub const CLASS_ID: usize = 1;
    pub const CLASS: usize = 2;
    pub const GEAR_SCORE: usize = 3;
    pub const CURRENT_HP: usize = 4;
    pub const MAX_HP: usize = 5;
    pub const IS_DEAD: usize = 6;
    pub const SKILLS: usize = 7;
    pub const DAMAGE_STATS: usize = 8;
    pub const SKILL_STATS: usize = 9;
    pub const ENTITY_TYPE: usize = 11;
    pub const NPC_ID: usize = 12;
    pub const CHARACTER_ID: usize = 13;
    pub const ENGRAVINGS: usize = 14;
    pub const SPEC: usize = 15;
    pub const ARK_PASSIVE_ACTIVE: usize = 16;
    pub const ARK_PASSIVE_DATA: usize = 17;
    pub const LOADOUT_HASH: usize = 18;
    pub const COMBAT_POWER: usize = 19;
    pub const CURRENT_SHIELD: usize = 20;
}