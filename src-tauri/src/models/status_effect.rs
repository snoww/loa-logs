use bitflags::bitflags;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::AsRefStr;

use crate::models::SkillData;

#[derive(Debug, Clone, Serialize, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusEffectSource {
    pub name: String,
    pub desc: String,
    pub icon: String,
    pub skill: Option<SkillData>,
    pub set_name: Option<String>,
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

#[derive(Debug, Default, Clone)]
pub struct StatusEffectDetails {
    pub instance_id: u32,
    pub status_effect_id: u32,
    pub custom_id: u32,
    pub target_id: u64,
    pub source_id: u64,
    pub target_type: StatusEffectTargetType,
    pub db_target_type: String,
    pub value: u64,
    pub stack_count: u8,
    pub category: StatusEffectCategory,
    pub buff_category: StatusEffectBuffCategory,
    pub show_type: StatusEffectShowType,
    pub status_effect_type: StatusEffectType,
    pub expiration_delay: f32,
    pub expire_at: Option<DateTime<Utc>>,
    pub end_tick: u64,
    pub timestamp: DateTime<Utc>,
    pub name: String,
    pub unique_group: u32,
}

impl StatusEffectDetails {
    pub fn is_valid_for_raid(&self) -> bool {
    (self.buff_category == StatusEffectBuffCategory::BattleItem
        || self.buff_category == StatusEffectBuffCategory::Bracelet
        || self.buff_category == StatusEffectBuffCategory::Elixir
        || self.buff_category == StatusEffectBuffCategory::Etc)
        && self.category == StatusEffectCategory::Debuff
        && self.show_type == StatusEffectShowType::All
}
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum StatusEffectTarget {
    #[default]
    OTHER,
    PARTY,
    SELF,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum StatusEffectTargetType {
    #[default]
    Party = 0,
    Local = 1,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq, AsRefStr)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum StatusEffectCategory {
    Buff,
    Debuff,
    #[default]
    #[serde(other)]
    Other
}

#[derive(Debug, Default, Copy, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StatusEffectShowType {
    Other = 0,
    #[default]
    #[serde(other)]
    All = 1,
}

#[derive(Debug, Default, Copy, Clone, Deserialize, PartialEq, Eq, AsRefStr)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum StatusEffectBuffCategory {
    Pet,
    Cook,
    Bracelet,
    Etc,
    BattleItem,
    Elixir,
    Ability,
    DropsOfEther,
    ClassSkill,
    ArkPassive,
    Identity,
    SupportBuff,
    Set,
    #[default]
    #[serde(other)]
    Other
}

#[derive(Debug, Default, Copy, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StatusEffectType {
    Shield,
    Freeze,
    Fear,
    Stun,
    Sleep,
    Earthquake,
    Electrocution,
    PolymorphPc,
    ForcedMove,
    MindControl,
    Paralyzation,
    WeakenDefense,
    WeakenResistance,
    SkillDamageAmplify,
    BeattackedDamageAmplify,
    SkillDamageAmplifyAttack,
    DirectionalAttackAmplify,
    InstantStatAmplify,
    AttackPowerAmplify,
    InstantStatAmplifyByContents,
    EvolutionTypeDamage,
    MoveSpeedDown,
    AllSpeedDown,
    ResetCooldown,
    ChangeAiPoint,
    AiPointAmplify,
    IncreaseIdentityGauge,
    #[default]
    #[serde(other)]
    Other,
}

impl StatusEffectType {
    pub fn is_hard_crowd_control(&self) -> bool {
        matches!(
            self,
            Self::Freeze
                | Self::Fear
                | Self::Stun
                | Self::Sleep
                | Self::Earthquake
                | Self::Electrocution
                | Self::PolymorphPc
                | Self::ForcedMove
                | Self::MindControl
                | Self::Paralyzation
        )
    }

    pub fn is_damage_amplify(&self) -> bool {
        matches!(
            self,
            Self::WeakenDefense
                | Self::WeakenResistance
                | Self::SkillDamageAmplify
                | Self::BeattackedDamageAmplify
                | Self::SkillDamageAmplifyAttack
                | Self::DirectionalAttackAmplify
                | Self::InstantStatAmplify
                | Self::AttackPowerAmplify
                | Self::InstantStatAmplifyByContents
                | Self::EvolutionTypeDamage
        )
    }
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
