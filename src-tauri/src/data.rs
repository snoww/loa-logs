use anyhow::{Result, anyhow, Context};
use hashbrown::{HashMap, HashSet};
use serde::de::DeserializeOwned;
use std::{fs::File, io::Read, ops::Deref, sync::OnceLock};

use crate::models::*;

pub static COMBAT_EFFECT_DATA: OnceLockWrapper<HashMap<i32, CombatEffectData>> =
    OnceLockWrapper::new();
pub static ENGRAVING_DATA: OnceLockWrapper<HashMap<u32, EngravingData>> = OnceLockWrapper::new();
pub static SKILL_BUFF_DATA: OnceLockWrapper<HashMap<u32, SkillBuffData>> = OnceLockWrapper::new();
pub static SKILL_DATA: OnceLockWrapper<HashMap<u32, SkillData>> = OnceLockWrapper::new();
pub static SKILL_EFFECT_DATA: OnceLockWrapper<HashMap<u32, SkillEffectData>> =
    OnceLockWrapper::new();
pub static SUPPORT_AP_GROUP: OnceLockWrapper<HashSet<u32>> = OnceLockWrapper::new();
pub static SUPPORT_IDENTITY_GROUP: OnceLockWrapper<HashSet<u32>> = OnceLockWrapper::new();
pub static STAT_TYPE_MAP: OnceLockWrapper<HashMap<String, u32>> = OnceLockWrapper::new();
pub static ESTHER_DATA: OnceLockWrapper<Vec<Esther>> = OnceLockWrapper::new();
pub static NPC_DATA: OnceLockWrapper<HashMap<u32, Npc>> = OnceLockWrapper::new();
pub static GEM_SKILL_MAP: OnceLockWrapper<HashMap<u32, Vec<u32>>> = OnceLockWrapper::new();
pub static RAID_MAP: OnceLockWrapper<HashMap<String, String>> = OnceLockWrapper::new();

pub struct OnceLockWrapper<T>(OnceLock<T>);

impl<T> OnceLockWrapper<T> {
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }

    pub fn set(&self, value: T) -> Result<()> {
        self.0
            .set(value)
            .map_err(|_| anyhow!("OnceLockWrapper already initialized"))
    }
}

impl<T> Deref for OnceLockWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.get().expect("OnceLockWrapper not initialized")
    }
}

pub struct AssetPreloader;

fn load<T: DeserializeOwned>(path: &str) -> Result<T> {
    let mut s = String::new();
    File::open(path).with_context(|| anyhow!("Missing file at: {path}"))?.read_to_string(&mut s)?;
    serde_json::from_str::<T>(&s).with_context(|| anyhow!("Error parsing JSON in {path}"))
}

impl AssetPreloader {
    pub fn new() -> Result<Self> {
        COMBAT_EFFECT_DATA.set(load("meter-data/CombatEffect.json")?)?;
        ENGRAVING_DATA.set(load("meter-data/Ability.json")?)?;
        SKILL_BUFF_DATA.set(load("meter-data/SkillBuff.json")?)?;
        SKILL_DATA.set(load("meter-data/Skill.json")?)?;
        SKILL_EFFECT_DATA.set(load("meter-data/SkillEffect.json")?)?;
        STAT_TYPE_MAP.set(load("meter-data/StatType.json")?)?;
        ESTHER_DATA.set(load("meter-data/Esther.json")?)?;
        NPC_DATA.set(load("meter-data/Npc.json")?)?;
        GEM_SKILL_MAP.set({
            let raw: HashMap<String, (String, String, Vec<u32>)> =
                load("meter-data/GemSkillGroup.json")?;
            raw.into_iter()
                .filter_map(|(key, entry)| key.parse::<u32>().ok().map(|id| (id, entry.2)))
                .collect()
        })?;
        RAID_MAP.set({
            let encounters: HashMap<String, HashMap<String, Vec<String>>> =
                load("meter-data/encounters.json")?;
            encounters
                .values()
                .flat_map(|raid| raid.iter())
                .flat_map(|(gate, bosses)| {
                    bosses.iter().map(move |boss| (boss.clone(), gate.clone()))
                })
                .collect()
        })?;
        SUPPORT_AP_GROUP.set(HashSet::from([
            101204, // bard
            101105, // paladin
            314004, // artist
            480030, // valkyrie
        ]))?;
        SUPPORT_IDENTITY_GROUP.set(HashSet::from([
            211400, // bard serenade of courage
            368000, // paladin holy aura
            310501, // artist moonfall
            480018, // valkyrie release light
        ]))?;

        Ok(Self)
    }
}
