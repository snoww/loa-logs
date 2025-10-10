use crate::models::{utils::int_or_string_as_string, StatusEffectBuffCategory, StatusEffectCategory, StatusEffectShowType, StatusEffectType};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillData {
    pub id: i32,
    pub name: Option<String>,
    #[serde(rename = "type", default)]
    #[serde(deserialize_with = "int_or_string_as_string")]
    pub skill_type: String,
    pub desc: Option<String>,
    pub class_id: u32,
    pub icon: Option<String>,
    pub identity_category: Option<String>,
    #[serde(alias = "groups")]
    pub groups: Option<Vec<i32>>,
    pub summon_source_skills: Option<Vec<u32>>,
    pub source_skills: Option<Vec<u32>>,
    #[serde(default)]
    pub is_hyper_awakening: bool,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillEffectData {
    pub id: i32,
    pub comment: String,
    #[serde(skip)]
    pub stagger: i32,
    pub source_skills: Option<Vec<u32>>,
    pub directional_mask: Option<i32>,
    pub item_name: Option<String>,
    pub item_desc: Option<String>,
    pub item_type: Option<String>,
    pub icon: Option<String>,
    pub values: Vec<i32>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillBuffData {
    pub id: i32,
    pub name: Option<String>,
    pub desc: Option<String>,
    pub icon: Option<String>,
    pub icon_show_type: StatusEffectShowType,
    pub duration: i32,
    pub category: StatusEffectCategory,
    #[serde(rename(deserialize = "type"))]
    pub buff_type: StatusEffectType,
    pub status_effect_values: Option<Vec<i32>>,
    pub buff_category: StatusEffectBuffCategory,
    pub target: String,
    pub unique_group: u32,
    #[serde(rename(deserialize = "overlap"))]
    pub overlap_flag: i32,
    pub per_level_data: HashMap<String, PerLevelData>,
    pub source_skills: Option<Vec<u32>>,
    pub set_name: Option<String>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PerLevelData {
    pub passive_options: Vec<PassiveOption>,
    // pub status_effect_values: Vec<i32>
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PassiveOption {
    #[serde(rename(deserialize = "type"))]
    pub option_type: PassiveOptionType,
    pub key_stat: StatType,
    pub key_index: i32,
    pub value: i32,
}

#[derive(Debug, Default, Deserialize, Clone, Copy, PartialEq, Hash, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StatType {
    // Stagger-related
    Mastery = 20,
    MasteryX = 26,
    ParalyzationPointRate = 54,

    // Cooldown-related
    Rapidity = 18,
    RapidityX = 24,
    CooldownReduction = 53,

    // Resource-related
    MaxMp = 28,
    MaxMpX = 30,
    MaxMpXX = 32,
    NormalMpRecovery = 37,
    CombatMpRecovery = 38,
    NormalMpRecoveryRate = 39,
    CombatMpRecoveryRate = 40,
    ResourceRecoveryRate = 149,

    // HP-related
    Con = 6,
    ConX = 10,
    MaxHp = 27,
    MaxHpX = 29,
    MaxHpXX = 31,
    NormalHpRecovery = 33,
    CombatHpRecovery = 34,
    NormalHpRecoveryRate = 35,
    CombatHpRecoveryRate = 36,
    SelfRecoveryRate = 41,
    DrainHpDamRate = 42,
    Vitality = 137,

    // Defense-related
    Endurance = 19,
    EnduranceX = 25,
    Def = 55,
    Res = 56,
    DefX = 57,
    ResX = 58,
    DefXX = 59,
    ResXX = 60,
    DefPenRate = 67,
    ResPenRate = 68,
    PhysicalIncRate = 69,
    MagicalIncRate = 70,

    // Attack speed-related
    AttackSpeed = 77,
    AttackSpeedRate = 78,

    // Offensive stats
    Str = 3,
    Agi = 4,
    Int = 5,
    StrX = 7,
    AgiX = 8,
    IntX = 9,
    CharAttackDam = 47,
    AttackPowerRate = 49,
    SkillDamageRate = 50,
    AttackPowerRateX = 51,
    SkillDamageRateX = 52,
    HitRate = 72,
    DodgeRate = 73,
    CriticalDamRate = 76,
    AwakeningDamRate = 110,
    AttackPowerAddend = 123,
    WeaponDam = 151,

    // Additional damage subrate / elemental damage stats
    AttackPowerSubRate1 = 141,
    AttackPowerSubRate2 = 142,
    PhysicalIncSubRate1 = 143,
    PhysicalIncSubRate2 = 144,
    MagicalIncSubRate1 = 145,
    MagicalIncSubRate2 = 146,
    SkillDamageSubRate1 = 147,
    SkillDamageSubRate2 = 148,
    FireDamRate = 87,
    WaterDamRate = 88,
    EarthDamRate = 91,
    WindDamRate = 89,
    LightDamRate = 93,
    DarkDamRate = 92,
    ElementsDamRate = 94,

    // Critical-related
    CriticalHitRate = 74,
    Criticalhit = 15,
    CriticalhitX = 21,

    // Movement speed
    MoveSpeed = 79,
    MoveSpeedRate = 80,
    PropMoveSpeed = 81,
    PropMoveSpeedRate = 82,
    VehicleMoveSpeed = 83,
    MountMoveSpeed = 85,
    VehicleMoveSpeedRate = 84,
    ShipMoveSpeedRate = 86,

    // Rest
    Hp = 1,
    Mp = 2,
    Specialty = 16,
    Oppression = 17,
    CriticalResRate = 75,
    SelfShieldRate = 71,
    ElectricityResRate = 97,
    EarthResRate = 99,
    DarkResRate = 100,
    HolyResRate = 101,
    ElementsResRate = 102,
    SelfCcTimeRate = 105,
    EnemyCcTimeRate = 106,
    IdentityValue1 = 107,
    IdentityValue2 = 108,
    IdentityValue3 = 109,
    ItemDropRate = 111,
    GoldRate = 112,
    ExpRate = 113,
    NpcSpeciesHumanoidDamRate = 125,
    NpcSpeciesDevilDamRate = 126,
    NpcSpeciesSubstanceDamRate = 127,
    NpcSpeciesUndeadDamRate = 128,
    NpcSpeciesPlantDamRate = 129,
    NpcSpeciesInsectDamRate = 130,
    NpcSpeciesSpiritDamRate = 131,
    NpcSpeciesWildBeastDamRate = 132,
    NpcSpeciesMechanicDamRate = 133,
    NpcSpeciesAncientDamRate = 134,
    NpcSpeciesGodDamRate = 135,
    NpcSpeciesArchfiendDamRate = 136,
    ShipBooterSpeed = 138,
    ShipWreckSpeedRate = 139,
    IslandSpeedRate = 140,
    SkillEffectDamAddend = 48,

    #[default]
    #[serde(other)]
    Other = 0,
}

impl StatType {
   pub fn is_damage_stat(&self) -> bool {
        matches!(
            self,
            Self::AttackPowerSubRate1
                | Self::AttackPowerSubRate2
                | Self::SkillDamageSubRate1
                | Self::SkillDamageSubRate2
                | Self::FireDamRate
                | Self::WaterDamRate
                | Self::EarthDamRate
                | Self::WindDamRate
                | Self::LightDamRate
                | Self::DarkDamRate
                | Self::ElementsDamRate
        )
    }

    pub fn is_stag_stat(&self) -> bool {
        matches!(
            self,
            Self::Mastery
            | Self::MasteryX
            | Self::ParalyzationPointRate)
    }

    pub fn is_defense_stat(&self) -> bool {
        matches!(
            self,
            Self::Def
                | Self::Res
                | Self::DefX
                | Self::ResX
                | Self::DefXX
                | Self::ResXX
                | Self::DefPenRate
                | Self::ResPenRate
                | Self::PhysicalIncRate
                | Self::MagicalIncRate
        )
    }

    pub fn is_movement_stat(&self) -> bool {
        matches!(
            self,
            Self::MoveSpeed
                | Self::MoveSpeedRate
                | Self::PropMoveSpeed
                | Self::PropMoveSpeedRate
                | Self::VehicleMoveSpeed
                | Self::VehicleMoveSpeedRate
                | Self::MountMoveSpeed
        )
    }

    pub fn is_endurance_stat(&self) -> bool {
        matches!(
            self,
            Self::Endurance
                | Self::EnduranceX)
    }

    pub fn is_offensive_stat(&self) -> bool {
        matches!(
            self,
            Self::Str
                | Self::StrX
                | Self::Agi
                | Self::AgiX
                | Self::Int
                | Self::IntX
                | Self::CharAttackDam
                | Self::AttackPowerRate
                | Self::AttackPowerRateX
                | Self::SkillDamageRate
                | Self::SkillDamageRateX
                | Self::HitRate
                | Self::DodgeRate
                | Self::CriticalDamRate
                | Self::AwakeningDamRate
                | Self::AttackPowerAddend
                | Self::WeaponDam
        )
    }

    pub fn is_cooldown_stat(&self) -> bool {
        matches!(
            self,
            Self::Rapidity
            | Self::RapidityX
            | Self::CooldownReduction)
    }

    pub fn is_resource_stat(&self) -> bool {
        matches!(
            self,
            Self::MaxMp
                | Self::MaxMpX
                | Self::MaxMpXX
                | Self::NormalMpRecovery
                | Self::CombatMpRecovery
                | Self::NormalMpRecoveryRate
                | Self::CombatMpRecoveryRate
                | Self::ResourceRecoveryRate
        )
    }

    pub fn is_crit_stat(&self) -> bool {
        matches!(
            self,
            Self::CriticalHitRate
                | Self::Criticalhit
                | Self::CriticalhitX)
    }

    pub fn is_atk_speed_stat(&self) -> bool {
        matches!(
            self,
            Self::AttackSpeed
                | Self::AttackSpeedRate
                | Self::Rapidity
                | Self::RapidityX)
    }

    pub fn is_hp_stat(&self) -> bool {
        matches!(
            self,
            Self::Con
                | Self::ConX
                | Self::MaxHp
                | Self::MaxHpX
                | Self::MaxHpXX
                | Self::NormalHpRecovery
                | Self::CombatHpRecovery
                | Self::NormalHpRecoveryRate
                | Self::CombatHpRecoveryRate
                | Self::SelfRecoveryRate
                | Self::DrainHpDamRate
                | Self::Vitality
        )
    }
}

#[derive(Debug, Default, Deserialize, Clone, Copy, PartialEq, Hash, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PassiveOptionType {
    Stat,
    SkillManaReduction,
    ManaReduction,
    SkillCooldownReduction,
    SkillGroupCooldownReduction,
    SkillDamage,
    ClassOption,
    SkillGroupDamage,
    SkillCriticalDamage,
    SkillCriticalRatio,
    SkillPenetration,
    CombatEffect,
    #[default]
    #[serde(other)]
    Other,
}

impl PassiveOptionType {
    pub fn is_resource(&self) -> bool {
         matches!(
            self,
            Self::SkillManaReduction
                | Self::ManaReduction) 
    }

    pub fn is_cooldown_reduction(&self) -> bool {
        matches!(
            self,
            Self::SkillCooldownReduction
            | Self::SkillGroupCooldownReduction) 
    }

    pub fn is_skill_option(&self) -> bool {
        matches!(
            self,
                Self::SkillDamage
                | Self::ClassOption
                | Self::SkillGroupDamage
                | Self::SkillCriticalDamage
                | Self::SkillPenetration
        )
    }
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct CombatEffectData {
    pub effects: Vec<CombatEffectDetail>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct CombatEffectDetail {
    pub ratio: i32,
    pub cooldown: i32,
    pub conditions: Vec<CombatEffectCondition>,
    pub actions: Vec<CombatEffectAction>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct CombatEffectCondition {
    #[serde(rename(deserialize = "type"))]
    pub condition_type: String,
    pub actor_type: String,
    pub arg: i32,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct CombatEffectAction {
    pub action_type: CombatEffectActionType,
    pub actor_type: String,
    pub args: Vec<i32>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum CombatEffectActionType {
    ModifyCriticalRatio,
    ModifyDamage,
    ModifyFinalDamage,
    ModifyCriticalMultiplier,
    ModifyPenetration,
    ModifyPenetrationWhenCritical,
    ModifyPenetrationAddend,
    ModifyPenetrationAddendWhenCritical,
    ModifyDamageShieldMultiplier,
    #[default]
    #[serde(other)]
    Other,
}

impl CombatEffectActionType {
      pub fn is_damage_modifier(&self) -> bool {
        matches!(
            self,
            Self::ModifyDamage
                | Self::ModifyFinalDamage
                | Self::ModifyCriticalMultiplier
                | Self::ModifyPenetration
                | Self::ModifyPenetrationWhenCritical
                | Self::ModifyPenetrationAddend
                | Self::ModifyPenetrationAddendWhenCritical
                | Self::ModifyDamageShieldMultiplier
        )
    }

    pub fn is_crit_modifier(&self) -> bool {
        matches!(self, Self::ModifyCriticalRatio)
    }
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Npc {
    pub id: i32,
    pub name: Option<String>,
    pub grade: String,
    #[serde(rename = "type")]
    pub npc_type: String,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Esther {
    pub name: String,
    pub icon: String,
    pub skills: Vec<i32>,
    #[serde(alias = "npcs")]
    pub npc_ids: Vec<u32>,
}
