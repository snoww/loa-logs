use serde::{Deserialize, Serialize};

use crate::settings::Settings;
use hashbrown::HashMap;

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
    pub my_rdps: Option<i64>,
    pub my_ndps: Option<i64>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EncountersOverview {
    pub encounters: Vec<EncounterPreview>,
    pub total_encounters: i32,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct CharacterStatisticsCriteria {
    pub character: String,
    pub range: String,
    pub mode: String,
    pub damage_type: String,
    pub boss_to_raid: HashMap<String, String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub bosses: Vec<String>,
    pub excluded_bosses: Vec<String>,
    pub included_specs: Vec<String>,
    pub excluded_specs: Vec<String>,
    pub difficulty: String,
    pub min_duration: i32,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CharacterStatistics {
    pub character: CharacterInfo,
    pub summary: CharacterStatisticsSummary,
    pub trends: Vec<CharacterStatisticsTrend>,
    pub raids: Vec<RaidStatisticsRow>,
    pub recent_bests: Vec<RecentBestEncounter>,
    pub unavailable: CharacterStatisticsUnavailable,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CharacterStatisticsSummary {
    pub attempts: i32,
    pub clears: i32,
    pub wipes: i32,
    pub clear_rate: f32,
    pub best_dps: Option<i64>,
    pub best_rdps: Option<i64>,
    pub best_ndps: Option<i64>,
    pub median_dps: Option<i64>,
    pub p75_dps: Option<i64>,
    pub p75_rdps: Option<i64>,
    pub p75_ndps: Option<i64>,
    pub median_rdps: Option<i64>,
    pub median_ndps: Option<i64>,
    pub median_udps: Option<i64>,
    pub median_duration: Option<i64>,
    pub support: Option<SupportStatisticsSummary>,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SupportStatisticsSummary {
    pub logs: i32,
    pub ap: Option<f32>,
    pub brand: Option<f32>,
    pub identity: Option<f32>,
    pub hyper: Option<f32>,
    pub median_contribution: Option<f32>,
    pub best_contribution: Option<f32>,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CharacterStatisticsTrend {
    pub start_time: i64,
    pub attempts: i32,
    pub clears: i32,
    pub median_dps: Option<i64>,
    pub best_dps: Option<i64>,
    pub support: Option<SupportStatisticsSummary>,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RaidStatisticsRow {
    pub boss_name: String,
    pub difficulty: Option<String>,
    pub attempts: i32,
    pub clears: i32,
    pub clear_rate: f32,
    pub median_dps: Option<i64>,
    pub best_dps: Option<i64>,
    pub median_rdps: Option<i64>,
    pub best_rdps: Option<i64>,
    pub median_ndps: Option<i64>,
    pub best_ndps: Option<i64>,
    pub median_duration: Option<i64>,
    pub last_clear: Option<i64>,
    pub support: Option<SupportStatisticsSummary>,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecentBestEncounter {
    pub id: i32,
    pub fight_start: i64,
    pub boss_name: String,
    pub duration: i64,
    pub difficulty: Option<String>,
    pub my_dps: i64,
    pub my_rdps: Option<i64>,
    pub my_ndps: Option<i64>,
    pub support_contribution: Option<f32>,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CharacterStatisticsUnavailable {
    pub rdps_logs: i32,
    pub support_logs: i32,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct RaidProgressionCriteria {
    pub range: String,
    pub boss_to_raid: HashMap<String, String>,
    pub boss_order: HashMap<String, i32>,
    pub bosses: Vec<String>,
    pub last_gate_bosses: Vec<String>,
    pub difficulty: String,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub min_duration: i32,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct RaidProgressionRangeCriteria {
    pub bosses: Vec<String>,
    pub last_gate_bosses: Vec<String>,
    pub difficulty: String,
    pub min_duration: i32,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RaidProgressionRange {
    pub first_pull: Option<i64>,
    pub first_clear: Option<i64>,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RaidProgressionStatistics {
    pub summary: RaidProgressionSummary,
    pub gates: Vec<RaidProgressionGate>,
    pub pulls: Vec<RaidProgressionPull>,
    pub players: Vec<RaidProgressionPlayer>,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RaidProgressionSummary {
    pub attempts: i32,
    pub clears: i32,
    pub wipes: i32,
    pub clear_rate: f32,
    pub first_pull: Option<i64>,
    pub last_pull: Option<i64>,
    pub first_clear: Option<i64>,
    pub first_clear_duration: Option<i64>,
    pub total_duration: i64,
    pub average_duration: Option<i64>,
    pub average_team_dps: Option<i64>,
    pub average_damage_taken: Option<i64>,
    pub average_deaths: Option<f32>,
    pub best_progress_bars: Option<i32>,
    pub best_progress_percent: Option<f32>,
    pub best_progress_boss_name: Option<String>,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RaidProgressionGate {
    pub gate: String,
    pub attempts: i32,
    pub clears: i32,
    pub clear_rate: f32,
    pub best_progress_bars: Option<i32>,
    pub best_progress_percent: Option<f32>,
    pub best_progress_boss_name: Option<String>,
    pub median_duration: Option<i64>,
    pub fastest_clear: Option<i64>,
    pub first_clear: Option<i64>,
    pub average_team_dps: Option<i64>,
    pub average_deaths: Option<f32>,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RaidProgressionPull {
    pub id: i32,
    pub fight_start: i64,
    pub gate: String,
    pub boss_name: String,
    pub difficulty: Option<String>,
    pub duration: i64,
    pub cleared: bool,
    pub team_dps: i64,
    pub damage_taken: i64,
    pub deaths: i32,
    pub progress_bars: Option<i32>,
    pub progress_percent: Option<f32>,
    pub local_player: String,
    pub player_count: i32,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RaidProgressionPlayer {
    pub name: String,
    pub class_id: i32,
    pub class: String,
    pub spec: Option<String>,
    pub is_support: bool,
    pub pulls: i32,
    pub clears: i32,
    pub clear_rate: f32,
    pub average_dps: Option<i64>,
    pub best_dps: Option<i64>,
    pub average_rdps: Option<i64>,
    pub average_ndps: Option<i64>,
    pub average_damage_taken: Option<i64>,
    pub total_deaths: i32,
    pub deaths_per_pull: f32,
    pub average_support_ap: Option<f32>,
    pub average_support_contribution: Option<f32>,
    pub average_support_brand: Option<f32>,
    pub average_support_identity: Option<f32>,
    pub average_support_hyper: Option<f32>,
    pub last_seen: i64,
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
    pub local_player: String,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CharacterInfo {
    pub name: String,
    pub class_id: i32,
    pub max_gear_score: f32,
    pub spec: Option<String>,
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
