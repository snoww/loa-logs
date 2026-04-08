use hashbrown::{HashMap, HashSet};
use serde::Serialize;

use crate::models::*;

#[derive(Debug)]
pub enum UiPayload<'a> {
    None,
    InvalidDamage,
    Data {
        snapshot: EncounterSnapshot<'a>,
        party_info: Option<&'a [Vec<String>]>
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncounterSnapshot<'a> {
    pub last_combat_packet: i64,
    pub fight_start: i64,
    pub local_player: &'a str,
    pub entities: HashMap<&'a String, EncounterSnapshotEntity<'a>>,
    pub current_boss_name: &'a str,
    pub current_boss: Option<EncounterCurrentBoss<'a>>,
    pub encounter_damage_stats: EncounterSnapshotDamageStats<'a>,
    pub duration: i64,
    pub difficulty: Option<&'a str>,
    pub boss_only_damage: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<&'a str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncounterSnapshotEntity<'a> {
    pub id: u64,
    pub character_id: u64,
    pub npc_id: u32,
    pub hp_bars: Option<u32>,
    pub name: &'a str,
    pub entity_type: EntityType,
    pub class_id: u32,
    pub class: &'a str,
    pub gear_score: f32,
    pub current_hp: i64,
    pub max_hp: i64,
    pub current_shield: u64,
    pub is_dead: bool,
    pub skills: HashMap<&'a u32, EncounterSnapshotEntitySkills<'a>>,
    pub damage_stats: EncounterSnapshotEntityDamageStats<'a>,
    pub skill_stats: &'a SkillStats,
}

impl<'a> From<&'a EncounterEntity> for EncounterSnapshotEntity<'a> {
    fn from(e: &'a EncounterEntity) -> Self {
        Self {
            id: e.id,
            character_id: e.character_id,
            npc_id: e.npc_id,
            hp_bars: e.hp_bars,
            name: &e.name,
            entity_type: e.entity_type,
            class_id: e.class_id,
            class: &e.class,
            gear_score: e.gear_score,
            current_hp: e.current_hp,
            max_hp: e.max_hp,
            current_shield: e.current_shield,
            is_dead: e.is_dead,
            skills: e.skills.iter()
                .map(|(k, s)| (k, s.into()))
                .collect(),
            damage_stats: (&e.damage_stats).into(),
            skill_stats: &e.skill_stats,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")] 
pub struct EncounterSnapshotEntitySkills<'a> {
    pub id: u32,
    pub name: &'a str,
    pub icon: &'a str,
    pub total_damage: i64,
    pub max_damage: i64,
    pub max_damage_cast: i64,
    pub buffed_by: &'a HashMap<u32, i64>,
    pub debuffed_by: &'a HashMap<u32, i64>,
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
    #[serde(default)]
    pub stagger: i64,
    #[serde(default)]
    pub is_hyper_awakening: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub special: Option<bool>,
    #[serde(skip)]
    pub last_timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_available: Option<i64>, // total time skill was available to cast
    pub rdps_received: &'a HashMap<u8, HashMap<u32, i64>>,
    pub rdps_contributed: &'a HashMap<u8, i64>,
}

impl<'a> From<&'a Skill> for EncounterSnapshotEntitySkills<'a> {
    fn from(skill: &'a Skill) -> Self {
        Self {
            id: skill.id,
            name: &skill.name,
            icon: &skill.icon,
            total_damage: skill.total_damage,
            max_damage: skill.max_damage,
            max_damage_cast: skill.max_damage_cast,
            buffed_by: &skill.buffed_by,
            debuffed_by: &skill.debuffed_by,
            buffed_by_support: skill.buffed_by_support,
            buffed_by_identity: skill.buffed_by_identity,
            buffed_by_hat: skill.buffed_by_hat,
            debuffed_by_support: skill.debuffed_by_support,
            casts: skill.casts,
            hits: skill.hits,
            crits: skill.crits,
            adjusted_crit: skill.adjusted_crit,
            crit_damage: skill.crit_damage,
            back_attacks: skill.back_attacks,
            front_attacks: skill.front_attacks,
            back_attack_damage: skill.back_attack_damage,
            front_attack_damage: skill.front_attack_damage,
            dps: skill.dps,
            tripod_index: skill.tripod_index,
            tripod_level: skill.tripod_level,
            gem_cooldown: skill.gem_cooldown,
            gem_tier: skill.gem_tier,
            gem_damage: skill.gem_damage,
            gem_tier_dmg: skill.gem_tier_dmg,
            stagger: skill.stagger,
            is_hyper_awakening: skill.is_hyper_awakening,
            special: skill.special,
            last_timestamp: skill.last_timestamp,
            time_available: skill.time_available,
            rdps_received: &skill.rdps_received,
            rdps_contributed: &skill.rdps_contributed,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncounterSnapshotEntityDamageStats<'a> {
    pub damage_dealt: i64,
    pub hyper_awakening_damage: i64,
    pub damage_taken: i64,
    pub buffed_by: &'a HashMap<u32, i64>,
    pub debuffed_by: &'a HashMap<u32, i64>,
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
    pub shields_given_by: &'a HashMap<u32, u64>,
    pub shields_received_by: &'a HashMap<u32, u64>,
    pub damage_absorbed_by: &'a HashMap<u32, u64>,
    pub damage_absorbed_on_others_by: &'a HashMap<u32, u64>,
    pub deaths: i64,
    pub death_time: i64,
    pub dps: i64,
    #[serde(default)]
    pub stagger: i64,
    #[serde(skip)]
    pub buffed_damage: i64,
    #[serde(default)]
    pub unbuffed_damage: i64,
    #[serde(default)]
    pub unbuffed_dps: i64,
}

impl<'a> From<&'a DamageStats> for EncounterSnapshotEntityDamageStats<'a> {
    fn from(d: &'a DamageStats) -> Self {
        Self {
            damage_dealt: d.damage_dealt,
            hyper_awakening_damage: d.hyper_awakening_damage,
            damage_taken: d.damage_taken,
            buffed_by: &d.buffed_by,
            debuffed_by: &d.debuffed_by,
            buffed_by_support: d.buffed_by_support,
            buffed_by_identity: d.buffed_by_identity,
            debuffed_by_support: d.debuffed_by_support,
            buffed_by_hat: d.buffed_by_hat,
            crit_damage: d.crit_damage,
            back_attack_damage: d.back_attack_damage,
            front_attack_damage: d.front_attack_damage,
            shields_given: d.shields_given,
            shields_received: d.shields_received,
            damage_absorbed: d.damage_absorbed,
            damage_absorbed_on_others: d.damage_absorbed_on_others,
            shields_given_by: &d.shields_given_by,
            shields_received_by: &d.shields_received_by,
            damage_absorbed_by: &d.damage_absorbed_by,
            damage_absorbed_on_others_by: &d.damage_absorbed_on_others_by,
            deaths: d.deaths,
            death_time: d.death_time,
            dps: d.dps,
            stagger: d.stagger,
            buffed_damage: d.buffed_damage,
            unbuffed_damage: d.unbuffed_damage,
            unbuffed_dps: d.unbuffed_dps,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncounterSnapshotDamageStats<'a> {
    pub total_damage_dealt: i64,
    pub top_damage_dealt: i64,
    pub total_damage_taken: i64,
    pub top_damage_taken: i64,
    pub dps: i64,
    pub buffs: &'a HashMap<u32, StatusEffect>,
    pub debuffs: &'a HashMap<u32, StatusEffect>,
    pub total_shielding: u64,
    pub total_effective_shielding: u64,
    pub applied_shield_buffs: &'a HashMap<u32, StatusEffect>,
    #[serde(skip)]
    pub unknown_buffs: &'a HashSet<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub misc: Option<&'a EncounterMisc>,
}

impl<'a> From<&'a EncounterDamageStats> for EncounterSnapshotDamageStats<'a> {
    fn from(stats: &'a EncounterDamageStats) -> Self {
        Self {
            total_damage_dealt: stats.total_damage_dealt,
            top_damage_dealt: stats.top_damage_dealt,
            total_damage_taken: stats.total_damage_taken,
            top_damage_taken: stats.top_damage_taken,
            dps: stats.dps,
            buffs: &stats.buffs,
            debuffs: &stats.debuffs,
            total_shielding: stats.total_shielding,
            total_effective_shielding: stats.total_effective_shielding,
            applied_shield_buffs: &stats.applied_shield_buffs,
            unknown_buffs: &stats.unknown_buffs,
            misc: stats.misc.as_ref(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncounterCurrentBoss<'a> {
    pub id: u64,
    pub npc_id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hp_bars: Option<u32>,
    pub name: &'a str,
    pub entity_type: EntityType,
    pub current_hp: i64,
    pub max_hp: i64,
    pub current_shield: u64,
    pub is_dead: bool,
    pub skills: &'a HashMap<u32, Skill>,
    pub damage_stats: &'a DamageStats,
    pub skill_stats: &'a SkillStats,
}