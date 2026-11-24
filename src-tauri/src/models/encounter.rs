use crate::models::ArkPassiveData;
use crate::models::Skill;
use hashbrown::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
use serde_with::serde_as;

use crate::models::EntityType;
use crate::models::StatusEffect;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_support_buffs: Option<crate::models::SupportBuffs>,
}

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct EncounterDamageStats {
    pub total_damage_dealt: i64,
    pub top_damage_dealt: i64,
    pub total_damage_taken: i64,
    pub top_damage_taken: i64,
    pub dps: i64,
    pub buffs: HashMap<u32, StatusEffect>,
    pub debuffs: HashMap<u32, StatusEffect>,
    pub total_shielding: u64,
    pub total_effective_shielding: u64,
    pub applied_shield_buffs: HashMap<u32, StatusEffect>,
    #[serde(skip)]
    pub unknown_buffs: HashSet<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub misc: Option<EncounterMisc>,
    pub boss_hp_log: HashMap<String, Vec<BossHpLog>>,
}

#[serde_as]
#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EncounterEntity {
    pub id: u64,
    #[serde_as(as = "DisplayFromStr")]
    pub character_id: u64,
    pub npc_id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hp_bars: Option<u32>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engraving_data: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ark_passive_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ark_passive_data: Option<ArkPassiveData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loadout_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub combat_power: Option<f32>,
}

#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct DamageStats {
    #[serde(skip)]
    pub(crate) grab_state: super::grab::GrabState,
    pub npc_window_incomplete_hits: i64,
    pub npc_window_tracked_hits: i64,
    pub rdps_damage_received_npc: i64,
    pub rdps_damage_received_atropine: i64,
    pub damage_dealt: i64,
    pub hyper_awakening_damage: i64,
    pub damage_taken: i64,
    pub buffed_by: HashMap<u32, i64>,
    pub debuffed_by: HashMap<u32, i64>,
    pub buffed_by_support: i64,
    pub buffed_by_identity: i64,
    pub debuffed_by_support: i64,
    pub buffed_by_hat: i64,
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
    #[serde(default)]
    pub death_info: Option<Vec<DeathInfo>>, // only used to track kazeros gate 2 deaths, where there are multiple respawns
    #[serde(skip)]
    pub boss_hp_at_death: Option<i64>,
    pub dps: i64,
    #[serde(default)]
    pub dps_average: Vec<i64>,
    #[serde(default)]
    pub ndps_average: Vec<i64>,
    #[serde(default)]
    pub dps_rolling_10s_avg: Vec<i64>,
    #[serde(default)]
    pub rdps_damage_received: i64,
    #[serde(default)]
    pub rdps_damage_received_support: i64,
    #[serde(default)]
    pub rdps_damage_given: i64,
    #[serde(default)]
    pub incapacitations: Vec<IncapacitatedEvent>,
    #[serde(default)]
    pub stagger: i64,
    #[serde(skip)]
    pub buffed_damage: i64, // legacy uDPS-only buffed damage from PKTCombatAnalyzerNotify
    #[serde(default)]
    pub unbuffed_damage: i64, // legacy uDPS-only unbuffed damage derived from analyzer packets
    #[serde(default)]
    pub unbuffed_dps: i64, // legacy uDPS-only unbuffed dps derived from analyzer packets
    #[serde(default)]
    pub rdps: i64,
    #[serde(default)]
    pub ndps: i64,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct DeathInfo {
    pub death_time: i64,
    pub dead_for: Option<i64>,
}

impl DamageStats {
    pub fn start_grab(&mut self, source_id: u64, timestamp: i64) {
        self.advance_grab_movements(timestamp);
        self.grab_state.pending_detach = false;
        self.clip_abnormal_moves(timestamp);
        if let Some(event) = self
            .incapacitations
            .iter_mut()
            .find(|event| event.grab_source_id.is_some())
        {
            if event.grab_source_id == Some(source_id) {
                return;
            }
            event.duration = (timestamp - event.timestamp).max(0);
            event.grab_source_id = None;
        }
        self.incapacitations.push(IncapacitatedEvent {
            event_type: IncapacitationEventType::GRAB,
            timestamp,
            duration: 0,
            grab_source_id: Some(source_id),
        });
    }

    pub fn end_grab(&mut self, timestamp: i64) {
        self.grab_state.pending_detach = false;
        if let Some(event) = self
            .incapacitations
            .iter_mut()
            .find(|event| event.grab_source_id.is_some())
        {
            event.duration = (timestamp - event.timestamp).max(0);
            event.grab_source_id = None;
        }
    }

    pub fn end_incapacitations(&mut self, timestamp: i64) {
        self.reset_grab(timestamp);
        for event in &mut self.incapacitations {
            if event.grab_source_id.take().is_some() || event.timestamp + event.duration > timestamp
            {
                event.duration = (timestamp - event.timestamp).max(0);
            }
        }
    }
}

impl Encounter {
    /// Freeze open grabs at the encounter's last combat packet for both live and saved views.
    pub fn snapshot_incapacitations(&mut self) {
        self.advance_grab_movements(self.last_combat_packet);
        for entity in self
            .entities
            .values_mut()
            .chain(self.current_boss.iter_mut())
        {
            for event in &mut entity.damage_stats.incapacitations {
                if event.grab_source_id.take().is_some() {
                    event.duration = (self.last_combat_packet - event.timestamp).max(0);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillInterruptCategory {
    Paralysis,
    Push,
    CrowdControl,
    Grab,
    Other,
}

impl SkillInterruptCategory {
    /// Classify a native `SkillCancelReason` byte into a player-facing involuntary
    /// interruption cause. Voluntary, system, and NPC-only cancel reasons return `None`.
    pub fn from_cancel_reason(reason: u8) -> Option<Self> {
        match reason {
            // ABNORMAL_SWAY (1)
            1 => Some(Self::Paralysis),
            // ABNORMAL_BRING_DOWN (2), ABNORMAL_ROOT_MOTION (3)
            2 | 3 => Some(Self::Push),
            // DEBUFF_NO_ACTION (8), ADD_STATUS_EFFECT (29)
            8 | 29 => Some(Self::CrowdControl),
            // GRAP_RELEASE (6), GRAP_FREEZE_STRONG (7)
            6 | 7 => Some(Self::Grab),
            // BE_KILLED (5), FORCE_TELEPORT_SKILL_BY_FALLEN_VOLUME (32), ABNORMAL_EVENT (40)
            5 | 32 | 40 => Some(Self::Other),
            // Voluntary, normal, system, and NPC-only reasons are excluded.
            _ => None,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct SkillInterrupts {
    pub paralysis: i64,
    pub push: i64,
    pub crowd_control: i64,
    pub grab: i64,
    pub other: i64,
}

impl SkillInterrupts {
    pub fn total(&self) -> i64 {
        self.paralysis + self.push + self.crowd_control + self.grab + self.other
    }

    pub fn is_empty(&self) -> bool {
        self.total() == 0
    }

    pub fn record(&mut self, category: SkillInterruptCategory) {
        match category {
            SkillInterruptCategory::Paralysis => self.paralysis += 1,
            SkillInterruptCategory::Push => self.push += 1,
            SkillInterruptCategory::CrowdControl => self.crowd_control += 1,
            SkillInterruptCategory::Grab => self.grab += 1,
            SkillInterruptCategory::Other => self.other += 1,
        }
    }

    pub fn merge(&mut self, other: Self) {
        self.paralysis += other.paralysis;
        self.push += other.push;
        self.crowd_control += other.crowd_control;
        self.grab += other.grab;
        self.other += other.other;
    }

    pub fn remove(&mut self, other: Self) {
        self.paralysis = self.paralysis.saturating_sub(other.paralysis).max(0);
        self.push = self.push.saturating_sub(other.push).max(0);
        self.crowd_control = self
            .crowd_control
            .saturating_sub(other.crowd_control)
            .max(0);
        self.grab = self.grab.saturating_sub(other.grab).max(0);
        self.other = self.other.saturating_sub(other.other).max(0);
    }
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
    #[serde(default, skip_serializing_if = "SkillInterrupts::is_empty")]
    pub interrupts: SkillInterrupts,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity_stats: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DamageData {
    pub skill_id: u32,
    pub skill_effect_id: u32,
    pub damage: i64,
    pub shield_damage: Option<i64>,
    pub modifier: i32,
    pub target_current_hp: i64,
    pub target_max_hp: i64,
    pub damage_attribute: Option<u8>,
    pub damage_type: u8,
    pub stagger: u32,
}

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub gauge1: u32,
    pub gauge2: u32,
    pub gauge3: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IncapacitatedEvent {
    #[serde(rename = "type")]
    pub event_type: IncapacitationEventType,
    pub timestamp: i64,
    pub duration: i64, // in a live meter, this might be retroactively updated to be shortened if the user uses get up or gets incapacitated with the same type again
    #[serde(skip)]
    pub grab_source_id: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum IncapacitationEventType {
    FALL_DOWN,
    CROWD_CONTROL,
    GRAB,
    PARALYSIS,
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
#[serde(rename_all = "camelCase", default)]
#[serde_as]
pub struct EncounterMisc {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boss_hp_log: Option<HashMap<String, Vec<BossHpLog>>>,
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
    // time spent in kazeros g2-2 starting zone
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intermission_start: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intermission_end: Option<i64>,
    // rdps contribution breakdown
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contribution_splits: Option<Vec<ContributionSplit>>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase", default)]
pub struct StatDamageContribution {
    pub damage_done_by_stat: i64,
    pub damage_done_by_stat_plus_value: i64,
}

impl StatDamageContribution {
    pub fn add(&mut self, damage_new: f64, damage_base: f64) {
        self.damage_done_by_stat += damage_base as i64;
        self.damage_done_by_stat_plus_value += damage_new as i64;
    }

    pub fn merge(&mut self, other: Self) {
        self.damage_done_by_stat += other.damage_done_by_stat;
        self.damage_done_by_stat_plus_value += other.damage_done_by_stat_plus_value;
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase", default)]
pub struct NpcWindowDamageMetrics {
    pub domination: StatDamageContribution,
    pub broken_bone: StatDamageContribution,
    pub npc_damage_taken: StatDamageContribution,
    pub stagger_combat_effect: StatDamageContribution,
    pub tracked_hits: i64,
    pub incomplete_hits: i64,
    pub missing_stagger_hits: i64,
    pub missing_domination_hits: i64,
    pub missing_weakness_hits: i64,
}

impl NpcWindowDamageMetrics {
    pub fn merge(&mut self, other: Self) {
        self.domination.merge(other.domination);
        self.broken_bone.merge(other.broken_bone);
        self.npc_damage_taken.merge(other.npc_damage_taken);
        self.stagger_combat_effect
            .merge(other.stagger_combat_effect);
        self.tracked_hits += other.tracked_hits;
        self.incomplete_hits += other.incomplete_hits;
        self.missing_stagger_hits += other.missing_stagger_hits;
        self.missing_domination_hits += other.missing_domination_hits;
        self.missing_weakness_hits += other.missing_weakness_hits;
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct ContributionSplit {
    pub npc_windows: NpcWindowDamageMetrics,
    pub atropine_damage_bonus: StatDamageContribution,
    pub attribute_atropine_attack_power_to_potion: bool,
    // Bitmask from NpcDamageAttribution, retained in saved/uploaded encounter JSON.
    pub npc_damage_attribution: u8,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub party_number: Option<i32>,
    pub damage_split_by_name: HashMap<String, i64>,
    pub damage_done_by_entity_skill_group: HashMap<String, HashMap<String, i64>>,
    pub damage_increase_by_entity_skill_group: HashMap<String, HashMap<String, i64>>,
    pub damage_done_without_ultimate_awakening: i64,
    pub hyper_awakening_damage: i64,
    pub damage_done_without_crits: i64,
    pub damage_done_with_all_crits: i64,
    pub damage_done_with_average_crits: i64,
    pub damage_done_from_crits: i64,
    pub damage_done_variance: f64,
    pub critical_hit_rate_adjusted_damage_raw: i64,
    pub critical_hit_rate_adjusted_damage_raw_capped: i64,
    pub additional_damage_1percent_damage: StatDamageContribution,
    pub critical_hit_rate_1percent_damage: StatDamageContribution,
    pub critical_damage_rate_1percent_damage: StatDamageContribution,
    pub evo_damage_1percent_damage: StatDamageContribution,
    pub weapon_power_1000_damage: StatDamageContribution,
    pub weapon_power_1percent_damage: StatDamageContribution,
    pub attack_power_1000_damage: StatDamageContribution,
    pub attack_power_1percent_damage: StatDamageContribution,
    pub main_stat_1000_damage: StatDamageContribution,
    pub raid_captain_efficiency: StatDamageContribution,
    pub blunt_thorn_efficiency: StatDamageContribution,
    pub supersonic_breakthrough_efficiency: StatDamageContribution,
    pub standing_striker_efficiency: StatDamageContribution,
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
pub struct EngravingData {
    pub id: u32,
    pub name: Option<String>,
    pub icon: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn death_clips_incapacitation_to_elapsed_time() {
        let mut stats = DamageStats::default();
        stats.incapacitations.push(IncapacitatedEvent {
            event_type: IncapacitationEventType::CROWD_CONTROL,
            timestamp: 1_000,
            duration: 5_000,
            grab_source_id: None,
        });
        stats.end_incapacitations(2_500);
        assert_eq!(stats.incapacitations[0].duration, 1_500);
    }

    #[test]
    fn death_clips_ongoing_incapacitation_behind_an_expired_event() {
        let mut stats = DamageStats::default();
        stats.incapacitations = vec![
            IncapacitatedEvent {
                event_type: IncapacitationEventType::CROWD_CONTROL,
                timestamp: 1_000,
                duration: 5_000,
                grab_source_id: None,
            },
            IncapacitatedEvent {
                event_type: IncapacitationEventType::FALL_DOWN,
                timestamp: 2_000,
                duration: 100,
                grab_source_id: None,
            },
        ];
        stats.end_incapacitations(2_500);
        assert_eq!(stats.incapacitations[0].duration, 1_500);
        assert_eq!(stats.incapacitations[1].duration, 100);
    }
}
