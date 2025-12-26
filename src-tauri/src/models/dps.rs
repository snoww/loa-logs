use anyhow::Result;
use hashbrown::HashMap;
use crate::models::{Entity, RdpsData};

#[derive(Debug)]
pub struct DamageEventContext<'a> {
    pub damage: i64,
    pub stagger: i64,
    pub skill_id: u32,
    pub skill_effect_id: u32,
    pub target_current_hp: i64,
    pub target_max_hp: i64,
    pub rdps_data: Vec<RdpsData>,
    pub dmg_src_entity: &'a Entity,
    pub proj_entity: &'a Entity,
    pub dmg_target_entity: &'a Entity,
    pub se_on_source_ids: Vec<u32>,
    pub se_on_target_ids: Vec<u32>,
    pub hit_flag: HitFlag,
    pub hit_option: HitOption,
    pub timestamp: i64,
    pub character_id_to_name: &'a HashMap<u64, String>,
}

#[derive(Debug, Clone, Copy)]
pub struct HitInfo(pub HitOption, pub HitFlag);

impl TryFrom<i32> for HitInfo {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self> {
        let flag_raw = (value & 0xF) as u8;
        let flag = match flag_raw {
            0 => HitFlag::Normal,
            1 => HitFlag::Critical,
            2 => HitFlag::Miss,
            3 => HitFlag::Invincible,
            4 => HitFlag::Dot,
            5 => HitFlag::Immune,
            6 => HitFlag::ImmuneSilenced,
            7 => HitFlag::FontSilenced,
            8 => HitFlag::DotCritical,
            9 => HitFlag::Dodge,
            10 => HitFlag::Reflect,
            11 => HitFlag::DamageShare,
            12 => HitFlag::DodgeHit,
            13 => HitFlag::Max,
            _ => return Err(anyhow::anyhow!("Invalid HitFlag value: {}", flag_raw)),
        };

        let option_raw = ((value >> 4) & 0x7) - 1;
        let option = match option_raw {
            -1 => HitOption::None,
            0 => HitOption::BackAttack,
            1 => HitOption::FrontalAttack,
            2 => HitOption::FlankAttack,
            3 => HitOption::Max,
            _ => return Err(anyhow::anyhow!("Invalid HitOption value: {}", option_raw)),
        };

        Ok(Self(option, flag))
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum HitOption {
    None = 0,
    BackAttack = 1,
    FrontalAttack = 2,
    FlankAttack = 3,
    Max = 4,
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum HitFlag {
    Normal = 0,
    Critical = 1,
    Miss = 2,
    Invincible = 3,
    Dot = 4,
    Immune = 5,
    ImmuneSilenced = 6,
    FontSilenced = 7,
    DotCritical = 8,
    Dodge = 9,
    Reflect = 10,
    DamageShare = 11,
    DodgeHit = 12,
    Max = 13,
    Unknown = 14
}
