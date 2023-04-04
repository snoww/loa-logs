use hashbrown::HashSet;
use serde::Serialize;

use super::models::EntityType;

#[derive(Debug)]
pub struct LogInitEnv<'a> {
    pub player_id: &'a str,
}

#[derive(Debug)]
pub struct LogPhaseTransition {
    pub raid_result: RaidResult,
}

#[derive(Debug)]
pub struct LogNewPc<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub class_id: i32,
    pub class: &'a str,
    pub level: i32,
    pub gear_score: f64,
    pub current_hp: i64,
    pub max_hp: i64,
    pub entity_type: EntityType,
}

#[derive(Debug)]
pub struct LogNewNpc<'a> {
    pub id: &'a str,
    pub npc_id: i32,
    pub name: &'a str,
    pub current_hp: i64,
    pub max_hp: i64,
    pub entity_type: EntityType,
}

#[derive(Debug)]
pub struct LogDeath<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub killer_id: &'a str,
    pub killer_name: &'a str,
}

#[derive(Debug)]
pub struct LogSkillStart<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub skill_id: i32,
    pub skill_name: &'a str,
}

#[derive(Debug)]
pub struct LogSkillStage {
    pub id: String,
    pub name: String,
    pub skill_id: i32,
    pub skill_name: String,
    pub skill_stage: i32,
}

#[derive(Debug)]
pub struct LogDamage<'a> {
    pub source_id: &'a str,
    pub source_name: &'a str,
    pub skill_id: i32,
    pub skill_name: &'a str,
    pub skill_effect_id: i32,
    pub skill_effect: &'a str,
    pub target_id: &'a str,
    pub target_name: &'a str,
    pub damage: i64,
    pub damage_mod: i32,
    pub current_hp: i64,
    pub max_hp: i64,
    pub effects_on_source: HashSet<i32>,
    pub effects_on_target: HashSet<i32>,
}

#[derive(Debug)]
pub struct LogHeal {
    pub id: String,
    pub name: String,
    pub heal_amount: i32,
    pub current_hp: i64,
}

#[derive(Debug)]
pub struct LogCounterAttack<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub target_id: &'a str,
    pub target_name: &'a str,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum RaidResult {
    RAID_RESULT,
    GUARDIAN_DEAD,
    RAID_END,
    UNKNOWN,
}

#[derive(Debug, PartialEq)]
#[repr(i32)]
pub enum HitOption {
    NONE,
    BACK_ATTACK,
    FRONTAL_ATTACK,
    FLANK_ATTACK,
    MAX,
}

#[derive(Debug, PartialEq)]
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
