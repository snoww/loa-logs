use std::io::{Read, Write};
use std::str::FromStr;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use hashbrown::HashMap;
use rusqlite::types::{FromSql, FromSqlResult, ValueRef};
use serde::de::DeserializeOwned;
use serde::Serialize;
use anyhow::Result;

use crate::live::models::*;

pub fn map_encounter(row: &rusqlite::Row) -> rusqlite::Result<(Encounter, bool)> {
    let misc_str: String = row.get(12).unwrap_or_default();
    let misc = serde_json::from_str::<EncounterMisc>(misc_str.as_str())
        .map(Some)
        .unwrap_or_default();

    let mut compressed = false;
    let mut boss_hp_log: HashMap<String, Vec<BossHpLog>> = HashMap::new();

    if let Some(misc) = misc.as_ref() {
        let version = misc
            .version
            .clone()
            .unwrap_or_default()
            .split('.')
            .map(|x| x.parse::<i32>().unwrap_or_default())
            .collect::<Vec<_>>();

        if version[0] > 1
            || (version[0] == 1 && version[1] >= 14)
            || (version[0] == 1 && version[1] == 13 && version[2] >= 5)
        {
            compressed = true;
        }

        if !compressed {
            boss_hp_log = misc.boss_hp_log.clone().unwrap_or_default();
        }
    }

    let buffs: HashMap<u32, StatusEffect>;
    let debuffs: HashMap<u32, StatusEffect>;
    let applied_shield_buffs: HashMap<u32, StatusEffect>;
    
    if compressed {
        let raw_bytes: Vec<u8> = row.get(10).unwrap_or_default();
        let mut decompress = GzDecoder::new(raw_bytes.as_slice());
        let mut buff_string = String::new();
        decompress
            .read_to_string(&mut buff_string)
            .expect("could not decompress buffs");
        buffs = serde_json::from_str::<HashMap<u32, StatusEffect>>(buff_string.as_str())
            .unwrap_or_default();

        let raw_bytes: Vec<u8> = row.get(11).unwrap_or_default();
        let mut decompress = GzDecoder::new(raw_bytes.as_slice());
        let mut debuff_string = String::new();
        decompress
            .read_to_string(&mut debuff_string)
            .expect("could not decompress debuffs");
        debuffs =
            serde_json::from_str::<HashMap<u32, StatusEffect>>(debuff_string.as_str())
                .unwrap_or_default();

        let raw_bytes: Vec<u8> = row.get(19).unwrap_or_default();
        let mut decompress = GzDecoder::new(raw_bytes.as_slice());
        let mut applied_shield_buff_string = String::new();
        decompress
            .read_to_string(&mut applied_shield_buff_string)
            .expect("could not decompress applied_shield_buffs");
        applied_shield_buffs = serde_json::from_str::<HashMap<u32, StatusEffect>>(
            applied_shield_buff_string.as_str(),
        )
        .unwrap_or_default();

        let raw_bytes: Vec<u8> = row.get(20).unwrap_or_default();
        let mut decompress = GzDecoder::new(raw_bytes.as_slice());
        let mut boss_string = String::new();
        decompress
            .read_to_string(&mut boss_string)
            .expect("could not decompress boss_hp_log");
        boss_hp_log =
            serde_json::from_str::<HashMap<String, Vec<BossHpLog>>>(boss_string.as_str())
                .unwrap_or_default();
    } else {
        let buff_str: String = row.get(10).unwrap_or_default();
        buffs = serde_json::from_str::<HashMap<u32, StatusEffect>>(buff_str.as_str())
            .unwrap_or_default();
        let debuff_str: String = row.get(11).unwrap_or_default();
        debuffs = serde_json::from_str::<HashMap<u32, StatusEffect>>(debuff_str.as_str())
            .unwrap_or_default();
        let applied_shield_buff_str: String = row.get(19).unwrap_or_default();
        applied_shield_buffs = serde_json::from_str::<HashMap<u32, StatusEffect>>(
            applied_shield_buff_str.as_str(),
        )
        .unwrap_or_default();
    }

    let total_shielding = row.get(17).unwrap_or_default();
    let total_effective_shielding = row.get(18).unwrap_or_default();
    let encounter = Encounter {
        last_combat_packet: row.get(0)?,
        fight_start: row.get(1)?,
        local_player: row.get(2).unwrap_or("You".to_string()),
        current_boss_name: row.get(3)?,
        duration: row.get(4)?,
        encounter_damage_stats: EncounterDamageStats {
            total_damage_dealt: row.get(5)?,
            top_damage_dealt: row.get(6)?,
            total_damage_taken: row.get(7)?,
            top_damage_taken: row.get(8)?,
            dps: row.get(9)?,
            buffs,
            debuffs,
            misc,
            total_shielding,
            total_effective_shielding,
            applied_shield_buffs,
            boss_hp_log,
            ..Default::default()
        },
        difficulty: row.get(13)?,
        favorite: row.get(14)?,
        cleared: row.get(15)?,
        boss_only_damage: row.get(16)?,
        ..Default::default()
    };

    Ok((encounter, compressed))
}

pub fn map_encounter_preview(row: &rusqlite::Row) -> rusqlite::Result<EncounterPreview> {
    let classes: String = row.get(9).unwrap_or_default();

    let (classes, names): (Vec<i32>, Vec<String>) = classes
        .split(',')
        .map(|s| {
            let info: Vec<&str> = s.split(':').collect();
            if info.len() != 2 {
                return (101, "Unknown".to_string());
            }
            (info[0].parse::<i32>().unwrap_or(101), info[1].to_string())
        })
        .unzip();

    Ok(EncounterPreview {
        id: row.get(0)?,
        fight_start: row.get(1)?,
        boss_name: row.get(2)?,
        duration: row.get(3)?,
        classes,
        names,
        difficulty: row.get(4)?,
        favorite: row.get(5)?,
        cleared: row.get(6)?,
        local_player: row.get(7)?,
        my_dps: row.get(8).unwrap_or(0),
        spec: row.get(10).unwrap_or_default(),
        support_ap: row.get(11).unwrap_or_default(),
        support_brand: row.get(12).unwrap_or_default(),
        support_identity: row.get(13).unwrap_or_default(),
        support_hyper: row.get(14).unwrap_or_default(),
    })
}

pub fn map_entity(row: &rusqlite::Row, is_compressed: bool) -> rusqlite::Result<EncounterEntity> {

    let (skills, damage_stats) = if is_compressed {
        
        let CompressedJson(skills): CompressedJson<HashMap<u32, Skill>> = row.get(7)?;
        let CompressedJson(damage_stats): CompressedJson<DamageStats> = row.get(8)?;
        (skills, damage_stats)
    } else {
        let JsonColumn(skills): JsonColumn<HashMap<u32, Skill>> = row.get(7)?;
        let JsonColumn(damage_stats): JsonColumn<DamageStats> = row.get(8)?;

        (skills, damage_stats)
    };

    let JsonColumn(skill_stats): JsonColumn<SkillStats> = row.get(9)?;

    let entity_type: String = row.get(11).unwrap_or_default();

    let JsonColumn(engravings): JsonColumn<Option<Vec<String>>> = row.get(14)?;

    let spec: Option<String> = row.get(15).unwrap_or_default();
    let ark_passive_active: Option<bool> = row.get(16).unwrap_or_default();

    let JsonColumn(ark_passive_data): JsonColumn<Option<ArkPassiveData>> = row.get(17)?;

    Ok(EncounterEntity {
        name: row.get(0)?,
        class_id: row.get(1)?,
        class: row.get(2)?,
        gear_score: row.get(3)?,
        current_hp: row.get(4)?,
        max_hp: row.get(5)?,
        is_dead: row.get(6)?,
        skills,
        damage_stats,
        skill_stats,
        entity_type: EntityType::from_str(entity_type.as_str())
            .unwrap_or(EntityType::UNKNOWN),
        npc_id: row.get(12)?,
        character_id: row.get(13).unwrap_or_default(),
        engraving_data: engravings,
        spec,
        ark_passive_active,
        ark_passive_data,
        loadout_hash: row.get(18).unwrap_or_default(),
        combat_power: row.get(19).unwrap_or_default(),
        ..Default::default()
    })
}

pub fn decode<T>(bytes: Vec<u8>) -> Result<T> 
where T: ?Sized + DeserializeOwned {
    let mut decompress = GzDecoder::new(bytes.as_slice());
    let mut buffer = vec![];
    decompress.read_to_end(&mut buffer);

    let value = serde_json::from_slice(&buffer)?;

    Ok(value)
}

pub fn compress_json<T>(value: &T) -> Vec<u8>
    where
        T: ?Sized + Serialize,
{
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    let bytes = serde_json::to_vec(value).expect("unable to serialize json");
    encoder.write_all(&bytes).expect("unable to write json to buffer");
    encoder.finish().expect("unable to compress json")
}

pub struct CompressedJson<T>(pub T);

impl<T> FromSql for CompressedJson<T>
where
    T: DeserializeOwned,
{
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Blob(bytes) => {
                let mut decompress = GzDecoder::new(bytes);
                let mut buffer = Vec::new();
                decompress.read_to_end(&mut buffer).map_err(|e| {
                    rusqlite::types::FromSqlError::Other(Box::new(e))
                })?;

                let parsed: T = serde_json::from_slice(&buffer).map_err(|e| {
                    rusqlite::types::FromSqlError::Other(Box::new(e))
                })?;
                Ok(CompressedJson(parsed))
            }
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

pub struct JsonColumn<T>(pub T);

impl<T> FromSql for JsonColumn<T>
where
    T: DeserializeOwned,
{
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Text(text) => {
                let parsed = serde_json::from_slice(text).map_err(|e| {
                    rusqlite::types::FromSqlError::Other(Box::new(e))
                })?;
                Ok(JsonColumn(parsed))
            }
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}