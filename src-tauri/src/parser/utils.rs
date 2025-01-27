use crate::constants::DB_VERSION;
use crate::parser::entity_tracker::Entity;
use crate::parser::models::*;
use crate::parser::skill_tracker::SkillTracker;
use crate::parser::stats_api::PlayerStats;
use crate::parser::status_tracker::StatusEffectDetails;
use flate2::write::GzEncoder;
use flate2::Compression;
use hashbrown::{HashMap, HashSet};
use moka::sync::Cache;
use rusqlite::{params, Transaction};
use serde::Serialize;
use serde_json::json;
use std::cmp::{max, Ordering, Reverse};
use std::collections::BTreeMap;
use std::io::Write;

pub fn encounter_entity_from_entity(entity: &Entity) -> EncounterEntity {
    let mut e = EncounterEntity {
        id: entity.id,
        name: entity.name.clone(),
        entity_type: entity.entity_type,
        npc_id: entity.npc_id,
        class_id: entity.class_id,
        class: get_class_from_id(&entity.class_id),
        gear_score: entity.gear_level,
        ..Default::default()
    };

    if entity.character_id > 0 {
        e.character_id = entity.character_id;
    }

    e
}

pub fn update_player_entity(old: &mut EncounterEntity, new: &Entity) {
    old.id = new.id;
    old.character_id = new.character_id;
    old.name.clone_from(&new.name);
    old.class_id = new.class_id;
    old.class = get_class_from_id(&new.class_id);
    old.gear_score = new.gear_level;
}

pub fn is_support_class_id(class_id: u32) -> bool {
    class_id == 105 || class_id == 204 || class_id == 602
}

pub fn is_battle_item(skill_effect_id: &u32, _item_type: &str) -> bool {
    if let Some(item) = SKILL_EFFECT_DATA.get(skill_effect_id) {
        if let Some(category) = item.item_type.as_ref() {
            return category == "useup";
        }
    }
    false
}

pub fn get_status_effect_data(buff_id: u32, source_skill: Option<u32>) -> Option<StatusEffect> {
    let buff = SKILL_BUFF_DATA.get(&buff_id);
    if buff.is_none() || buff.unwrap().icon_show_type.clone().unwrap_or_default() == "none" {
        return None;
    }

    let buff = buff.unwrap();
    let buff_category = if buff.buff_category.clone().unwrap_or_default() == "ability"
        && [501, 502, 503, 504, 505].contains(&buff.unique_group)
    {
        "dropsofether".to_string()
    } else {
        buff.buff_category.clone().unwrap_or_default()
    };
    let mut status_effect = StatusEffect {
        target: {
            if buff.target == "none" {
                StatusEffectTarget::OTHER
            } else if buff.target == "self" {
                StatusEffectTarget::SELF
            } else {
                StatusEffectTarget::PARTY
            }
        },
        category: buff.category.clone(),
        buff_category: buff_category.clone(),
        buff_type: get_status_effect_buff_type_flags(buff),
        unique_group: buff.unique_group,
        source: StatusEffectSource {
            name: buff.name.clone()?,
            desc: buff.desc.clone()?,
            icon: buff.icon.clone()?,
            ..Default::default()
        },
    };

    if buff_category == "classskill"
        || buff_category == "arkpassive"
        || buff_category == "identity"
        || (buff_category == "ability" && buff.unique_group != 0)
    {
        if let Some(buff_source_skills) = buff.source_skills.as_ref() {
            if let Some(source_skill) = source_skill {
                let skill = SKILL_DATA.get(&source_skill);
                get_summon_source_skill(skill, &mut status_effect);
            } else {
                let source_skill = buff_source_skills.first().unwrap_or(&0);
                let skill = SKILL_DATA.get(source_skill);
                get_summon_source_skill(skill, &mut status_effect);
            }
        } else if let Some(buff_source_skill) = SKILL_DATA.get(&(buff_id / 10)) {
            status_effect.source.skill = Some(buff_source_skill.clone());
        } else if let Some(buff_source_skill) = SKILL_DATA.get(&((buff_id / 100) * 10)) {
            status_effect.source.skill = Some(buff_source_skill.clone());
        } else {
            let skill_id = buff.unique_group / 10;
            let buff_source_skill = SKILL_DATA.get(&skill_id);
            status_effect.source.skill = buff_source_skill.cloned();
        }
    } else if buff_category == "set" && buff.set_name.is_some() {
        status_effect.source.set_name.clone_from(&buff.set_name);
    } else if buff_category == "battleitem" {
        if let Some(buff_source_item) = SKILL_EFFECT_DATA.get(&buff_id) {
            if let Some(item_name) = buff_source_item.item_name.as_ref() {
                status_effect.source.name.clone_from(item_name);
            }
            if let Some(item_desc) = buff_source_item.item_desc.as_ref() {
                status_effect.source.desc.clone_from(item_desc);
            }
            if let Some(icon) = buff_source_item.icon.as_ref() {
                status_effect.source.icon.clone_from(icon);
            }
        }
    }

    Some(status_effect)
}

fn get_summon_source_skill(skill: Option<&SkillData>, status_effect: &mut StatusEffect) {
    if let Some(skill) = skill {
        if let Some(summon_skills) = skill.summon_source_skills.as_ref() {
            let summon_source_skill = summon_skills.first().unwrap_or(&0);
            if *summon_source_skill > 0 {
                if let Some(summon_skill) = SKILL_DATA.get(summon_source_skill) {
                    status_effect.source.skill = Some(summon_skill.clone());
                }
            }
        } else {
            status_effect.source.skill = Some(skill.clone());
        }
    }
}

pub fn get_status_effect_buff_type_flags(buff: &SkillBuffData) -> u32 {
    let dmg_buffs = [
        "weaken_defense",
        "weaken_resistance",
        "skill_damage_amplify",
        "beattacked_damage_amplify",
        "skill_damage_amplify_attack",
        "directional_attack_amplify",
        "instant_stat_amplify",
        "attack_power_amplify",
        "instant_stat_amplify_by_contents",
        "evolution_type_damage",
    ];

    let mut buff_type = StatusEffectBuffTypeFlags::NONE;
    if dmg_buffs.contains(&buff.buff_type.as_str()) {
        buff_type |= StatusEffectBuffTypeFlags::DMG;
    } else if ["move_speed_down", "all_speed_down"].contains(&buff.buff_type.as_str()) {
        buff_type |= StatusEffectBuffTypeFlags::MOVESPEED;
    } else if buff.buff_type == "reset_cooldown" {
        buff_type |= StatusEffectBuffTypeFlags::COOLDOWN;
    } else if ["change_ai_point", "ai_point_amplify"].contains(&buff.buff_type.as_str()) {
        buff_type |= StatusEffectBuffTypeFlags::STAGGER;
    } else if buff.buff_type == "increase_identity_gauge" {
        buff_type |= StatusEffectBuffTypeFlags::RESOURCE;
    }

    for option in buff.passive_options.iter() {
        let key_stat_str = option.key_stat.as_str();
        let option_type = option.option_type.as_str();
        if option_type == "stat" {
            let stat = STAT_TYPE_MAP.get(key_stat_str);
            if stat.is_none() {
                continue;
            }
            let stat = stat.unwrap().to_owned();
            if ["mastery", "mastery_x", "paralyzation_point_rate"].contains(&key_stat_str) {
                buff_type |= StatusEffectBuffTypeFlags::STAGGER;
            } else if ["rapidity", "rapidity_x", "cooldown_reduction"].contains(&key_stat_str) {
                buff_type |= StatusEffectBuffTypeFlags::COOLDOWN;
            } else if [
                "max_mp",
                "max_mp_x",
                "max_mp_x_x",
                "normal_mp_recovery",
                "combat_mp_recovery",
                "normal_mp_recovery_rate",
                "combat_mp_recovery_rate",
                "resource_recovery_rate",
            ]
            .contains(&key_stat_str)
            {
                buff_type |= StatusEffectBuffTypeFlags::RESOURCE;
            } else if [
                "con",
                "con_x",
                "max_hp",
                "max_hp_x",
                "max_hp_x_x",
                "normal_hp_recovery",
                "combat_hp_recovery",
                "normal_hp_recovery_rate",
                "combat_hp_recovery_rate",
                "self_recovery_rate",
                "drain_hp_dam_rate",
                "vitality",
            ]
            .contains(&key_stat_str)
            {
                buff_type |= StatusEffectBuffTypeFlags::HP;
            } else if STAT_TYPE_MAP["def"] <= stat && stat <= STAT_TYPE_MAP["magical_inc_rate"]
                || ["endurance", "endurance_x"].contains(&option.key_stat.as_str())
            {
                if buff.category == "buff" && option.value >= 0
                    || buff.category == "debuff" && option.value <= 0
                {
                    buff_type |= StatusEffectBuffTypeFlags::DMG;
                } else {
                    buff_type |= StatusEffectBuffTypeFlags::DEFENSE;
                }
            } else if STAT_TYPE_MAP["move_speed"] <= stat
                && stat <= STAT_TYPE_MAP["vehicle_move_speed_rate"]
            {
                buff_type |= StatusEffectBuffTypeFlags::MOVESPEED;
            }
            if [
                "attack_speed",
                "attack_speed_rate",
                "rapidity",
                "rapidity_x",
            ]
            .contains(&key_stat_str)
            {
                buff_type |= StatusEffectBuffTypeFlags::ATKSPEED;
            } else if ["critical_hit_rate", "criticalhit", "criticalhit_x"].contains(&key_stat_str)
            {
                buff_type |= StatusEffectBuffTypeFlags::CRIT;
            } else if STAT_TYPE_MAP["attack_power_sub_rate_1"] <= stat
                && stat <= STAT_TYPE_MAP["skill_damage_sub_rate_2"]
                || STAT_TYPE_MAP["fire_dam_rate"] <= stat
                    && stat <= STAT_TYPE_MAP["elements_dam_rate"]
                || [
                    "str",
                    "agi",
                    "int",
                    "str_x",
                    "agi_x",
                    "int_x",
                    "char_attack_dam",
                    "attack_power_rate",
                    "skill_damage_rate",
                    "attack_power_rate_x",
                    "skill_damage_rate_x",
                    "hit_rate",
                    "dodge_rate",
                    "critical_dam_rate",
                    "awakening_dam_rate",
                    "attack_power_addend",
                    "weapon_dam",
                ]
                .contains(&key_stat_str)
            {
                if buff.category == "buff" && option.value >= 0
                    || buff.category == "debuff" && option.value <= 0
                {
                    buff_type |= StatusEffectBuffTypeFlags::DMG;
                } else {
                    buff_type |= StatusEffectBuffTypeFlags::DEFENSE;
                }
            }
        } else if option_type == "skill_critical_ratio" {
            buff_type |= StatusEffectBuffTypeFlags::CRIT;
        } else if [
            "skill_damage",
            "class_option",
            "skill_group_damage",
            "skill_critical_damage",
            "skill_penetration",
        ]
        .contains(&option_type)
        {
            if buff.category == "buff" && option.value >= 0
                || buff.category == "debuff" && option.value <= 0
            {
                buff_type |= StatusEffectBuffTypeFlags::DMG;
            } else {
                buff_type |= StatusEffectBuffTypeFlags::DEFENSE;
            }
        } else if ["skill_cooldown_reduction", "skill_group_cooldown_reduction"]
            .contains(&option_type)
        {
            buff_type |= StatusEffectBuffTypeFlags::COOLDOWN;
        } else if ["skill_mana_reduction", "mana_reduction"].contains(&option_type) {
            buff_type |= StatusEffectBuffTypeFlags::RESOURCE;
        } else if option_type == "combat_effect" {
            if let Some(combat_effect) = COMBAT_EFFECT_DATA.get(&option.key_index) {
                for effect in combat_effect.effects.iter() {
                    for action in effect.actions.iter() {
                        if [
                            "modify_damage",
                            "modify_final_damage",
                            "modify_critical_multiplier",
                            "modify_penetration",
                            "modify_penetration_when_critical",
                            "modify_penetration_addend",
                            "modify_penetration_addend_when_critical",
                            "modify_damage_shield_multiplier",
                        ]
                        .contains(&action.action_type.as_str())
                        {
                            buff_type |= StatusEffectBuffTypeFlags::DMG;
                        } else if action.action_type == "modify_critical_ratio" {
                            buff_type |= StatusEffectBuffTypeFlags::CRIT;
                        }
                    }
                }
            }
        }
    }

    buff_type.bits()
}

pub fn get_skill_name_and_icon(
    skill_id: &u32,
    skill_effect_id: &u32,
    skill_name: String,
    skill_tracker: &SkillTracker,
    entity_id: u64,
) -> (String, String, Option<Vec<u32>>) {
    if (*skill_id == 0) && (*skill_effect_id == 0) {
        ("Bleed".to_string(), "buff_168.png".to_string(), None)
    } else if (*skill_effect_id != 0) && (*skill_effect_id == *skill_id) {
        return if let Some(effect) = SKILL_EFFECT_DATA.get(skill_effect_id) {
            if let Some(item_name) = effect.item_name.as_ref() {
                return (
                    item_name.clone(),
                    effect.icon.as_ref().cloned().unwrap_or_default(),
                    None,
                );
            }
            if let Some(source_skill) = effect.source_skills.as_ref() {
                if let Some(skill) = SKILL_DATA.get(source_skill.iter().min().unwrap_or(&0)) {
                    return (
                        skill.name.clone().unwrap_or_default(),
                        skill.icon.clone().unwrap_or_default(),
                        None,
                    );
                }
            } else if let Some(skill) = SKILL_DATA.get(&(skill_effect_id / 10)) {
                return (
                    skill.name.clone().unwrap_or_default(),
                    skill.icon.clone().unwrap_or_default(),
                    None,
                );
            }
            (effect.comment.clone(), "".to_string(), None)
        } else {
            (skill_name, "".to_string(), None)
        };
    } else {
        return if let Some(skill) = SKILL_DATA.get(skill_id) {
            if let Some(summon_source_skill) = skill.summon_source_skills.as_ref() {
                for source in summon_source_skill {
                    if skill_tracker
                        .skill_timestamp
                        .get(&(entity_id, *source))
                        .is_some()
                    {
                        if let Some(skill) = SKILL_DATA.get(source) {
                            return (
                                skill.name.clone().unwrap_or_default() + " (Summon)",
                                skill.icon.clone().unwrap_or_default(),
                                Some(summon_source_skill.clone()),
                            );
                        }
                    }
                }
                if let Some(skill) = SKILL_DATA.get(summon_source_skill.iter().min().unwrap_or(&0)) {
                    (
                        skill.name.clone().unwrap_or_default() + " (Summon)",
                        skill.icon.clone().unwrap_or_default(),
                        Some(summon_source_skill.clone()),
                    )
                } else {
                    (skill_name, "".to_string(), None)
                }
            } else if let Some(source_skill) = skill.source_skills.as_ref() {
                if let Some(skill) = SKILL_DATA.get(source_skill.iter().min().unwrap_or(&0)) {
                    (
                        skill.name.clone().unwrap_or_default(),
                        skill.icon.clone().unwrap_or_default(),
                        None,
                    )
                } else {
                    (skill_name, "".to_string(), None)
                }
            } else {
                (
                    skill.name.clone().unwrap_or_default(),
                    skill.icon.clone().unwrap_or_default(),
                    None,
                )
            }
        } else if let Some(skill) = SKILL_DATA.get(&(skill_id - (skill_id % 10))) {
            (
                skill.name.clone().unwrap_or_default(),
                skill.icon.clone().unwrap_or_default(),
                None,
            )
        } else {
            (skill_name, "".to_string(), None)
        };
    }
}

pub fn get_skill_name(skill_id: &u32) -> String {
    SKILL_DATA
        .get(skill_id)
        .map_or(skill_id.to_string(), |skill| {
            if skill.name.is_none() {
                skill_id.to_string()
            } else {
                skill.name.clone().unwrap_or_default()
            }
        })
}

pub fn get_skill(skill_id: &u32) -> Option<SkillData> {
    SKILL_DATA.get(skill_id).cloned()
}

pub fn get_class_from_id(class_id: &u32) -> String {
    let class = match class_id {
        0 => "",
        101 => "Warrior (Male)",
        102 => "Berserker",
        103 => "Destroyer",
        104 => "Gunlancer",
        105 => "Paladin",
        111 => "Female Warrior",
        112 => "Slayer",
        201 => "Mage",
        202 => "Arcanist",
        203 => "Summoner",
        204 => "Bard",
        205 => "Sorceress",
        301 => "Martial Artist (Female)",
        302 => "Wardancer",
        303 => "Scrapper",
        304 => "Soulfist",
        305 => "Glaivier",
        311 => "Martial Artist (Male)",
        312 => "Striker",
        313 => "Breaker",
        401 => "Assassin",
        402 => "Deathblade",
        403 => "Shadowhunter",
        404 => "Reaper",
        405 => "Souleater",
        501 => "Gunner (Male)",
        502 => "Sharpshooter",
        503 => "Deadeye",
        504 => "Artillerist",
        505 => "Machinist",
        511 => "Gunner (Female)",
        512 => "Gunslinger",
        601 => "Specialist",
        602 => "Artist",
        603 => "Aeromancer",
        604 => "Alchemist",
        _ => "Unknown",
    };

    class.to_string()
}

fn damage_gem_value_to_level(value: u32, tier: u8) -> u8 {
    if tier == 4 {
        match value {
            4400 => 10,
            4000 => 9,
            3600 => 8,
            3200 => 7,
            2800 => 6,
            2400 => 5,
            2000 => 4,
            1600 => 3,
            1200 => 2,
            800 => 1,
            _ => 0,
        }
    } else {
        match value {
            4000 => 10,
            3000 => 9,
            2400 => 8,
            2100 => 7,
            1800 => 6,
            1500 => 5,
            1200 => 4,
            900 => 3,
            600 => 2,
            300 => 1,
            _ => 0,
        }
    }
}

fn cooldown_gem_value_to_level(value: u32, tier: u8) -> u8 {
    if tier == 4 {
        match value {
            2400 => 10,
            2200 => 9,
            2000 => 8,
            1800 => 7,
            1600 => 6,
            1400 => 5,
            1200 => 4,
            1000 => 3,
            800 => 2,
            600 => 1,
            _ => 0,
        }
    } else {
        match value {
            2000 => 10,
            1800 => 9,
            1600 => 8,
            1400 => 7,
            1200 => 6,
            1000 => 5,
            800 => 4,
            600 => 3,
            400 => 2,
            200 => 1,
            _ => 0,
        }
    }
}

fn support_damage_gem_value_to_level(value: u32) -> u8 {
    match value {
        1000 => 10,
        900 => 9,
        800 => 8,
        700 => 7,
        600 => 6,
        500 => 5,
        400 => 4,
        300 => 3,
        200 => 2,
        100 => 1,
        _ => 0,
    }
}

fn gem_skill_id_to_skill_ids(skill_id: u32) -> Vec<u32> {
    match skill_id {
        13000 | 13001 => vec![18011, 18030], // destroyer hypergravity skills
        23000 => vec![
            20311, 20310, 20070, 20071, 20080, 20081, 20170, 20181, 20280, 20281,
        ], // summoner elemental damage
        41000 => vec![25038, 25035, 25036, 25037, 25400, 25401, 25402], // db surge skill
        42000 | 42001 => vec![
            27800, 27030, 27810, 27820, 27830, 27840, 27850, 27860, 27940, 27960,
        ], // sh transformation skills
        51001 => vec![28159, 28160, 28161, 28162, 28170], // sharpshooter bird skill
        53000 | 53001 => vec![30240, 30250, 30260, 30270, 30290], // arty barrage skills
        54000 | 54001 => vec![
            35720, 35750, 35760, 35761, 35770, 35771, 35780, 35781, 35790, 35800,
        ], // machinist transformation skills
        62000 => vec![32040, 32041],         // aeromancer sun shower
        24000 => vec![21140, 21141, 21142, 21143, 21130, 21131, 21132, 21133], // bard serenade skills
        47000 => vec![47950], // bk breaker identity
        60000 => vec![
            31050, 31051, 31110, 31120, 31121, 31130, 31131, 31140, 31141,
        ], // artist moonfall
        19030 => vec![19290, 19030, 19300], // arcana evokes
        _ => vec![skill_id],
    }
}

pub fn get_engravings(
    class_id: u32,
    engravings: &Option<Vec<u32>>,
) -> (Vec<String>, Option<Vec<String>>) {
    let engravings = match engravings {
        Some(engravings) => engravings,
        None => return (vec![], None),
    };

    let mut class_engravings: Vec<String> = Vec::new();
    let mut other_engravings: Vec<String> = Vec::new();

    for engraving_id in engravings.iter() {
        if let Some(engraving_data) = ENGRAVING_DATA.get(engraving_id) {
            let player_engraving = engraving_data.name.clone();
            if is_class_engraving(class_id, engraving_data.id) {
                class_engravings.push(player_engraving.clone().unwrap_or("Unknown".to_string()));
            } else {
                other_engravings.push(player_engraving.unwrap_or("Unknown".to_string()));
            }
        }
    }

    other_engravings.sort_unstable();
    let sorted_engravings: Vec<String> = class_engravings
        .iter()
        .cloned()
        .chain(other_engravings)
        .collect();

    if sorted_engravings.is_empty() {
        (class_engravings, None)
    } else {
        (class_engravings, Some(sorted_engravings))
    }
}

fn is_class_engraving(class_id: u32, engraving_id: u32) -> bool {
    match engraving_id {
        125 | 188 => class_id == 102, // mayhem, berserker's technique
        196 | 197 => class_id == 103, // rage hammer, gravity training
        224 | 225 => class_id == 104, // combat readiness, lone knight
        282 | 283 => class_id == 105, // judgement, blessed aura
        309 | 320 => class_id == 112, // predator, punisher
        200 | 201 => class_id == 202, // empress's grace, order of the emperor
        198 | 199 => class_id == 203, // master summoner, communication overflow
        194 | 195 => class_id == 204, // true courage, desperate salvation
        293 | 294 => class_id == 205, // igniter, reflux
        189 | 127 => class_id == 302, // first intention, esoteric skill enhancement
        190 | 191 => class_id == 303, // ultimate skill: taijutsu, shock training
        256 | 257 => class_id == 304, // energy overflow, robust spirit
        276 | 277 => class_id == 305, // pinnacle, control
        291 | 292 => class_id == 312, // deathblow, esoteric flurry
        314 | 315 => class_id == 313, // brawl king storm, asura's path
        278 | 279 => class_id == 402, // remaining energy, surge
        280 | 281 => class_id == 403, // perfect suppression, demonic impulse
        286 | 287 => class_id == 404, // hunger, lunar voice
        311 | 312 => class_id == 405, // full moon harvester, night's edge
        258 | 259 => class_id == 502, // loyal companion, death strike
        192 | 129 => class_id == 503, // pistoleer, enhanced weapon
        130 | 193 => class_id == 504, // firepower enhancement, barrage enhancement
        284 | 285 => class_id == 505, // arthetinean skill, evolutionary legacy
        289 | 290 => class_id == 512, // peacemaker, time to hunt
        305 | 306 => class_id == 602, // recurrence, full bloom
        307 | 308 => class_id == 603, // wind fury, drizzle
        _ => false,
    }
}

pub fn is_hyper_awakening_skill(skill_id: u32) -> bool {
    matches!(
        skill_id,
        16720
            | 16730
            | 18240
            | 18250
            | 17250
            | 17260
            | 36230
            | 36240
            | 45820
            | 45830
            | 19360
            | 19370
            | 20370
            | 20350
            | 21320
            | 21330
            | 37380
            | 37390
            | 22360
            | 22370
            | 23400
            | 23410
            | 24300
            | 24310
            | 34620
            | 34630
            | 39340
            | 39350
            | 47300
            | 47310
            | 25410
            | 25420
            | 27910
            | 27920
            | 26940
            | 26950
            | 46620
            | 46630
            | 29360
            | 29370
            | 30320
            | 30330
            | 35810
            | 35820
            | 38320
            | 38330
            | 31920
            | 31930
            | 32290
            | 32280
    )
}

pub fn is_hat_buff(buff_id: &u32) -> bool {
    matches!(buff_id, 362600 | 212305 | 319503)
}

fn generate_intervals(start: i64, end: i64) -> Vec<i64> {
    if start >= end {
        return Vec::new();
    }

    (0..end - start).step_by(1_000).collect()
}

fn sum_in_range(vec: &[(i64, i64)], start: i64, end: i64) -> i64 {
    let start_idx = binary_search_left(vec, start);
    let end_idx = binary_search_left(vec, end + 1);

    vec[start_idx..end_idx]
        .iter()
        .map(|&(_, second)| second)
        .sum()
}

fn binary_search_left(vec: &[(i64, i64)], target: i64) -> usize {
    let mut left = 0;
    let mut right = vec.len();

    while left < right {
        let mid = left + (right - left) / 2;
        match vec[mid].0.cmp(&target) {
            Ordering::Less => left = mid + 1,
            _ => right = mid,
        }
    }

    left
}

fn calculate_average_dps(data: &[(i64, i64)], start_time: i64, end_time: i64) -> Vec<i64> {
    let step = 5;
    let mut results = vec![0; ((end_time - start_time) / step + 1) as usize];
    let mut current_sum = 0;
    let mut data_iter = data.iter();
    let mut current_data = data_iter.next();

    for t in (start_time..=end_time).step_by(step as usize) {
        while let Some((timestamp, value)) = current_data {
            if *timestamp / 1000 <= t {
                current_sum += value;
                current_data = data_iter.next();
            } else {
                break;
            }
        }

        results[((t - start_time) / step) as usize] = current_sum / (t - start_time + 1);
    }

    results
}

pub fn check_tripod_index_change(before: Option<TripodIndex>, after: Option<TripodIndex>) -> bool {
    if before.is_none() && after.is_none() {
        return false;
    }

    if before.is_none() || after.is_none() {
        return true;
    }

    let before = before.unwrap();
    let after = after.unwrap();

    before != after
}

pub fn check_tripod_level_change(before: Option<TripodLevel>, after: Option<TripodLevel>) -> bool {
    if before.is_none() && after.is_none() {
        return false;
    }

    if before.is_none() || after.is_none() {
        return true;
    }

    let before = before.unwrap();
    let after = after.unwrap();

    before != after
}

const WINDOW_MS: i64 = 5_000;
const WINDOW_S: i64 = 5;

#[allow(clippy::too_many_arguments)]
pub fn insert_data(
    tx: &Transaction,
    mut encounter: Encounter,
    prev_stagger: i32,
    damage_log: HashMap<String, Vec<(i64, i64)>>,
    identity_log: HashMap<String, IdentityLog>,
    cast_log: HashMap<String, HashMap<u32, Vec<i32>>>,
    boss_hp_log: HashMap<String, Vec<BossHpLog>>,
    stagger_log: Vec<(i32, f32)>,
    mut stagger_intervals: Vec<(i32, i32)>,
    raid_clear: bool,
    party_info: Vec<Vec<String>>,
    raid_difficulty: String,
    region: Option<String>,
    player_info: Option<HashMap<String, PlayerStats>>,
    meter_version: String,
    ntp_fight_start: i64,
    rdps_valid: bool,
    manual: bool,
    skill_cast_log: HashMap<u64, HashMap<u32, BTreeMap<i64, SkillCast>>>,
) -> i64 {
    let mut encounter_stmt = tx
        .prepare_cached(
            "
    INSERT INTO encounter (
        last_combat_packet,
        total_damage_dealt,
        top_damage_dealt,
        total_damage_taken,
        top_damage_taken,
        dps,
        buffs,
        debuffs,
        total_shielding,
        total_effective_shielding,
        applied_shield_buffs,
        misc,
        version,
        boss_hp_log,
        stagger_log
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        )
        .expect("failed to prepare encounter statement");

    encounter.duration = encounter.last_combat_packet - encounter.fight_start;
    let duration_seconds = max(encounter.duration / 1000, 1);
    encounter.encounter_damage_stats.dps =
        encounter.encounter_damage_stats.total_damage_dealt / duration_seconds;

    let misc: EncounterMisc = EncounterMisc {
        raid_clear: if raid_clear { Some(true) } else { None },
        party_info: if party_info.is_empty() {
            None
        } else {
            Some(
                party_info
                    .into_iter()
                    .enumerate()
                    .map(|(index, party)| (index as i32, party))
                    .collect(),
            )
        },
        region,
        version: Some(meter_version),
        rdps_valid: Some(rdps_valid),
        rdps_message: if rdps_valid {
            None
        } else {
            Some("invalid_stats".to_string())
        },
        ntp_fight_start: Some(ntp_fight_start),
        manual_save: Some(manual),
        ..Default::default()
    };

    let mut stagger_stats: Option<StaggerStats> = None;
    if !stagger_log.is_empty() {
        if prev_stagger > 0 && prev_stagger != encounter.encounter_damage_stats.max_stagger {
            // never finished staggering the boss, calculate average from whatever stagger has been done
            let stagger_start_s = ((encounter.encounter_damage_stats.stagger_start
                - encounter.fight_start)
                / 1000) as i32;
            let stagger_duration = stagger_log.last().unwrap().0 - stagger_start_s;
            if stagger_duration > 0 {
                stagger_intervals.push((stagger_duration, prev_stagger));
            }
        }

        let (total_stagger_time, total_stagger_dealt) = stagger_intervals.iter().fold(
            (0, 0),
            |(total_time, total_stagger), (time, stagger)| {
                (total_time + time, total_stagger + stagger)
            },
        );

        if total_stagger_time > 0 {
            let stagger = StaggerStats {
                average: (total_stagger_dealt as f64 / total_stagger_time as f64)
                    / encounter.encounter_damage_stats.max_stagger as f64
                    * 100.0,
                staggers_per_min: (total_stagger_dealt as f64 / (total_stagger_time as f64 / 60.0))
                    / encounter.encounter_damage_stats.max_stagger as f64,
                log: stagger_log,
            };
            stagger_stats = Some(stagger);
        }
    }

    let compressed_boss_hp = compress_json(&boss_hp_log);
    let compressed_buffs = compress_json(&encounter.encounter_damage_stats.buffs);
    let compressed_debuffs = compress_json(&encounter.encounter_damage_stats.debuffs);
    let compressed_shields = compress_json(&encounter.encounter_damage_stats.applied_shield_buffs);

    encounter_stmt
        .execute(params![
            encounter.last_combat_packet,
            encounter.encounter_damage_stats.total_damage_dealt,
            encounter.encounter_damage_stats.top_damage_dealt,
            encounter.encounter_damage_stats.total_damage_taken,
            encounter.encounter_damage_stats.top_damage_taken,
            encounter.encounter_damage_stats.dps,
            compressed_buffs,
            compressed_debuffs,
            encounter.encounter_damage_stats.total_shielding,
            encounter.encounter_damage_stats.total_effective_shielding,
            compressed_shields,
            json!(misc),
            DB_VERSION,
            compressed_boss_hp,
            json!(stagger_stats),
        ])
        .expect("failed to insert encounter");

    let last_insert_id = tx.last_insert_rowid();

    let mut entity_stmt = tx
        .prepare_cached(
            "
    INSERT INTO entity (
        name,
        encounter_id,
        npc_id,
        entity_type,
        class_id,
        class,
        gear_score,
        current_hp,
        max_hp,
        is_dead,
        skills,
        damage_stats,
        skill_stats,
        dps,
        character_id,
        engravings,
        gear_hash,
        ark_passive_active,
        spec,
        ark_passive_data
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
        )
        .expect("failed to prepare entity statement");

    let fight_start = encounter.fight_start;
    let fight_end = encounter.last_combat_packet;

    for (_key, entity) in encounter.entities.iter_mut().filter(|(_, e)| {
        ((e.entity_type == EntityType::PLAYER && e.class_id != 0 && e.max_hp > 0)
            || e.name == encounter.local_player
            || e.entity_type == EntityType::ESTHER
            || (e.entity_type == EntityType::BOSS && e.max_hp > 0))
            && e.damage_stats.damage_dealt > 0
    }) {
        if entity.entity_type == EntityType::PLAYER {
            let intervals = generate_intervals(fight_start, fight_end);
            if let Some(damage_log) = damage_log.get(&entity.name) {
                if !intervals.is_empty() {
                    for interval in intervals {
                        let start = fight_start + interval - WINDOW_MS;
                        let end = fight_start + interval + WINDOW_MS;

                        let damage = sum_in_range(damage_log, start, end);
                        entity
                            .damage_stats
                            .dps_rolling_10s_avg
                            .push(damage / (WINDOW_S * 2));
                    }
                }
                let fight_start_sec = encounter.fight_start / 1000;
                let fight_end_sec = encounter.last_combat_packet / 1000;
                entity.damage_stats.dps_average =
                    calculate_average_dps(damage_log, fight_start_sec, fight_end_sec);
            }

            entity.spec = Some(get_player_spec(
                entity,
                &encounter.encounter_damage_stats.buffs,
            ));

            if let Some(info) = player_info
                .as_ref()
                .and_then(|stats| stats.get(&entity.name))
            {
                for gem in info.gems.iter().flatten() {
                    for skill_id in gem_skill_id_to_skill_ids(gem.skill_id) {
                        if let Some(skill) = entity.skills.get_mut(&skill_id) {
                            match gem.gem_type {
                                5 | 34 => {
                                    // damage gem
                                    skill.gem_damage =
                                        Some(damage_gem_value_to_level(gem.value, gem.tier));
                                    skill.gem_tier_dmg = Some(gem.tier);
                                }
                                27 | 35 => {
                                    // cooldown gem
                                    skill.gem_cooldown =
                                        Some(cooldown_gem_value_to_level(gem.value, gem.tier));
                                    skill.gem_tier = Some(gem.tier);
                                }
                                64 | 65 => {
                                    // support identity gem??
                                    skill.gem_damage =
                                        Some(support_damage_gem_value_to_level(gem.value));
                                    skill.gem_tier_dmg = Some(gem.tier);
                                }
                                _ => {}
                            }
                        }
                    }
                }

                entity.ark_passive_active = Some(info.ark_passive_enabled);

                let (class, other) = get_engravings(entity.class_id, &info.engravings);
                entity.engraving_data = other;
                if info.ark_passive_enabled {
                    // not reliable enough
                    // if let Some(tree) = info.ark_passive_data.as_ref() {
                    //     if let Some(enlightenment) = tree.enlightenment.as_ref() {
                    //         for node in enlightenment.iter() {
                    //             let spec = get_spec_from_ark_passive(node);
                    //             if spec != "Unknown" {
                    //                 entity.spec = Some(spec);
                    //                 break;
                    //             }
                    //         }
                    //     }
                    // }
                    entity.ark_passive_data = info.ark_passive_data.clone();
                } else if class.len() == 1 {
                    entity.spec = Some(class[0].clone());
                }
            }

            entity.damage_stats.dps = entity.damage_stats.damage_dealt / duration_seconds;
        }

        for (_, skill) in entity.skills.iter_mut() {
            skill.dps = skill.total_damage / duration_seconds;
        }

        for (_, cast_log) in cast_log.iter().filter(|&(s, _)| *s == entity.name) {
            for (skill, log) in cast_log {
                entity.skills.entry(*skill).and_modify(|e| {
                    e.cast_log.clone_from(log);
                });
            }
        }

        for (_, skill_cast_log) in skill_cast_log.iter().filter(|&(s, _)| *s == entity.id) {
            for (skill, log) in skill_cast_log {
                entity.skills.entry(*skill).and_modify(|e| {
                    let average_cast = e.total_damage as f64 / e.casts as f64;
                    let filter = average_cast * 0.05;
                    let mut adj_hits = 0;
                    let mut adj_crits = 0;
                    for cast in log.values() {
                        for hit in cast.hits.iter() {
                            if hit.damage as f64 > filter {
                                adj_hits += 1;
                                if hit.crit {
                                    adj_crits += 1;
                                }
                            }
                        }
                    }
                    
                    if adj_hits > 0 { 
                        e.adjusted_crit = Some(adj_crits as f64 / adj_hits as f64);
                    }
                    
                    e.max_damage_cast = log
                        .values()
                        .map(|cast| cast.hits.iter().map(|hit| hit.damage).sum::<i64>())
                        .max()
                        .unwrap_or_default();
                    e.skill_cast_log = log
                        .iter()
                        .map(|(_, skill_casts)| skill_casts.clone())
                        .collect();
                });
            }
        }

        if let Some(identity_log) = identity_log.get(&entity.name) {
            if entity.name == encounter.local_player && identity_log.len() >= 2 {
                let mut total_identity_gain = 0;
                let data = identity_log;
                let duration_seconds = (data[data.len() - 1].0 - data[0].0) / 1000;
                let max = match entity.class.as_str() {
                    "Summoner" => 7_000.0,
                    "Souleater" => 3_000.0,
                    _ => 10_000.0,
                };
                let stats: String = match entity.class.as_str() {
                    "Arcanist" => {
                        let mut cards: HashMap<u32, u32> = HashMap::new();
                        let mut log: Vec<(i32, (f32, u32, u32))> = Vec::new();
                        for i in 1..data.len() {
                            let (t1, prev) = data[i - 1];
                            let (t2, curr) = data[i];

                            // don't count clown cards draws as card draws
                            if curr.1 != 0 && curr.1 != prev.1 && prev.1 != 19284 {
                                cards.entry(curr.1).and_modify(|e| *e += 1).or_insert(1);
                            }
                            if curr.2 != 0 && curr.2 != prev.2 && prev.2 != 19284 {
                                cards.entry(curr.2).and_modify(|e| *e += 1).or_insert(1);
                            }

                            if t2 > t1 && curr.0 > prev.0 {
                                total_identity_gain += curr.0 - prev.0;
                            }

                            let relative_time = ((t2 - fight_start) as f32 / 1000.0) as i32;
                            // calculate percentage, round to 2 decimal places
                            let percentage = if curr.0 >= max as u32 {
                                100.0
                            } else {
                                (((curr.0 as f32 / max) * 100.0) * 100.0).round() / 100.0
                            };
                            log.push((relative_time, (percentage, curr.1, curr.2)));
                        }

                        let avg_per_s = (total_identity_gain as f64 / duration_seconds as f64)
                            / max as f64
                            * 100.0;
                        let identity_stats = IdentityArcanist {
                            average: avg_per_s,
                            card_draws: cards,
                            log,
                        };

                        serde_json::to_string(&identity_stats).unwrap()
                    }
                    "Artist" | "Bard" => {
                        let mut log: Vec<(i32, (f32, u32))> = Vec::new();

                        for i in 1..data.len() {
                            let (t1, i1) = data[i - 1];
                            let (t2, i2) = data[i];

                            if t2 <= t1 {
                                continue;
                            }

                            if i2.0 > i1.0 {
                                total_identity_gain += i2.0 - i1.0;
                            }

                            let relative_time = ((t2 - fight_start) as f32 / 1000.0) as i32;
                            // since bard and artist have 3 bubbles, i.1 is the number of bubbles
                            // we scale percentage to 3 bubbles
                            // current bubble + max * number of bubbles
                            let percentage: f32 =
                                ((((i2.0 as f32 + max * i2.1 as f32) / max) * 100.0) * 100.0)
                                    .round()
                                    / 100.0;
                            log.push((relative_time, (percentage, i2.1)));
                        }

                        let avg_per_s = (total_identity_gain as f64 / duration_seconds as f64)
                            / max as f64
                            * 100.0;
                        let identity_stats = IdentityArtistBard {
                            average: avg_per_s,
                            log,
                        };
                        serde_json::to_string(&identity_stats).unwrap()
                    }
                    _ => {
                        let mut log: Vec<(i32, f32)> = Vec::new();
                        for i in 1..data.len() {
                            let (t1, i1) = data[i - 1];
                            let (t2, i2) = data[i];

                            if t2 <= t1 {
                                continue;
                            }

                            if i2.0 > i1.0 {
                                total_identity_gain += i2.0 - i1.0;
                            }

                            let relative_time = ((t2 - fight_start) as f32 / 1000.0) as i32;
                            let percentage =
                                (((i2.0 as f32 / max) * 100.0) * 100.0).round() / 100.0;
                            log.push((relative_time, percentage));
                        }

                        let avg_per_s = (total_identity_gain as f64 / duration_seconds as f64)
                            / max as f64
                            * 100.0;
                        let identity_stats = IdentityGeneric {
                            average: avg_per_s,
                            log,
                        };
                        serde_json::to_string(&identity_stats).unwrap()
                    }
                };

                entity.skill_stats.identity_stats = Some(stats);
            }
        }

        let compressed_skills = compress_json(&entity.skills);
        let compressed_damage_stats = compress_json(&entity.damage_stats);

        entity_stmt
            .execute(params![
                entity.name,
                last_insert_id,
                entity.npc_id,
                entity.entity_type.to_string(),
                entity.class_id,
                entity.class,
                entity.gear_score,
                entity.current_hp,
                entity.max_hp,
                entity.is_dead,
                compressed_skills,
                compressed_damage_stats,
                json!(entity.skill_stats),
                entity.damage_stats.dps,
                entity.character_id,
                json!(entity.engraving_data),
                entity.gear_hash,
                entity.ark_passive_active,
                entity.spec,
                json!(entity.ark_passive_data)
            ])
            .expect("failed to insert entity");
    }

    let mut players = encounter
        .entities
        .values()
        .filter(|e| {
            ((e.entity_type == EntityType::PLAYER && e.class_id != 0 && e.max_hp > 0)
                || e.name == encounter.local_player)
                && e.damage_stats.damage_dealt > 0
        })
        .collect::<Vec<_>>();
    let local_player_dps = players
        .iter()
        .find(|e| e.name == encounter.local_player)
        .map(|e| e.damage_stats.dps)
        .unwrap_or_default();
    players.sort_unstable_by_key(|e| Reverse(e.damage_stats.damage_dealt));
    let preview_players = players
        .into_iter()
        .map(|e| format!("{}:{}", e.class_id, e.name))
        .collect::<Vec<_>>()
        .join(",");

    let mut encounter_preview_stmt = tx
        .prepare_cached(
            "
    INSERT INTO encounter_preview (
        id,
        fight_start,
        current_boss,
        duration,
        players,
        difficulty,
        local_player,
        my_dps,
        cleared,
        boss_only_damage
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        )
        .expect("failed to prepare encounter preview statement");
    encounter_preview_stmt
        .execute(params![
            last_insert_id,
            encounter.fight_start,
            encounter.current_boss_name,
            encounter.duration,
            preview_players,
            raid_difficulty,
            encounter.local_player,
            local_player_dps,
            raid_clear,
            encounter.boss_only_damage
        ])
        .expect("failed to insert encounter preview");

    last_insert_id
}

pub fn map_status_effect(se: &StatusEffectDetails, custom_id_map: &mut HashMap<u32, u32>) -> u32 {
    if se.custom_id > 0 {
        custom_id_map.insert(se.custom_id, se.status_effect_id);
        se.custom_id
    } else {
        se.status_effect_id
    }
}

pub fn is_valid_player(player: &EncounterEntity) -> bool {
    player.gear_score >= 0.0
        && player.entity_type == EntityType::PLAYER
        && player.character_id != 0
        && player.class_id != 0
        && player.name != "You"
        && player
            .name
            .chars()
            .next()
            .unwrap_or_default()
            .is_uppercase()
}

pub fn get_new_id(source_skill: u32) -> u32 {
    source_skill + 1_000_000_000
}

pub fn get_skill_id(new_skill: u32) -> u32 {
    new_skill - 1_000_000_000
}

pub fn compress_json<T>(value: &T) -> Vec<u8>
where
    T: ?Sized + Serialize,
{
    let mut e = GzEncoder::new(Vec::new(), Compression::default());
    let bytes = serde_json::to_vec(value).expect("unable to serialize json");
    e.write_all(&bytes).expect("unable to write json to buffer");
    e.finish().expect("unable to compress json")
}

pub fn update_current_boss_name(boss_name: &str) -> String {
    match boss_name {
        "Chaos Lightning Dragon Jade" => "Argeos",
        "Vicious Argeos" | "Ruthless Lakadroff" | "Untrue Crimson Yoho" | "Despicable Skolakia" => {
            "Behemoth, the Storm Commander"
        }
        _ => boss_name,
    }
    .to_string()
}

fn get_player_spec(player: &EncounterEntity, buffs: &HashMap<u32, StatusEffect>) -> String {
    if player.skills.len() < 8 { 
        return "Unknown".to_string();
    }
    
    match player.class.as_str() {
        "Berserker" => {
            if player.skills.contains_key(&16140)
            {
                "Berserker Technique".to_string()
            } else {
                "Mayhem".to_string()
            }
        }
        "Destroyer" => {
            if player.skills.contains_key(&18090) {
                "Gravity Training".to_string()
            } else {
                "Rage Hammer".to_string()
            }
        }
        "Gunlancer" => {
            if player.skills.contains_key(&17200) && player.skills.contains_key(&17210) {
                "Lone Knight".to_string()
            } else if player.skills.contains_key(&17140) {
                "Combat Readiness".to_string()
            } else {
                "Princess".to_string()
            }
        }
        "Paladin" => {
            if (player.skills.contains_key(&36050)
                || player.skills.contains_key(&36080)
                || player.skills.contains_key(&36150)
                || player.skills.contains_key(&36100))
                && player.skills.contains_key(&36200)
                && player.skills.contains_key(&36170)
            {
                "Blessed Aura".to_string()
            } else {
                "Judgement".to_string()
            }
        }
        "Slayer" => {
            if player.skills.contains_key(&45004) {
                "Punisher".to_string()
            } else {
                "Predator".to_string()
            }
        }
        "Arcanist" => {
            if player.skills.contains_key(&19282) {
                "Order of the Emperor".to_string()
            } else {
                "Grace of the Empress".to_string()
            }
        }
        "Summoner" => {
            if player
                .skills
                .iter()
                .any(|(_, skill)| skill.name.contains("Kelsion"))
            {
                "Communication Overflow".to_string()
            } else {
                "Master Summoner".to_string()
            }
        }
        "Bard" => {
            if player.skills.contains_key(&21250) && player.skills.contains_key(&21080) {
                "Desperate Salvation".to_string()
            } else {
                "True Courage".to_string()
            }
        }
        "Sorceress" => {
            if player.skills.contains_key(&37350)
                && player.skills.contains_key(&37270)
                && player.skills.contains_key(&37330)
            {
                "Igniter".to_string()
            } else {
                "Reflux".to_string()
            }
        }
        "Wardancer" => {
            if player.skills.contains_key(&22340) {
                "Esoteric Skill Enhancement".to_string()
            } else {
                "First Intention".to_string()
            }
        }
        "Scrapper" => {
            if player.skills.contains_key(&23230) {
                "Ultimate Skill: Taijutsu".to_string()
            } else {
                "Shock Training".to_string()
            }
        }
        "Soulfist" => {
            if player.skills.contains_key(&24200) {
                "Energy Overflow".to_string()
            } else {
                "Robust Spirit".to_string()
            }
        }
        "Glaivier" => {
            if player.skills.contains_key(&34590) {
                "Pinnacle".to_string()
            } else {
                "Control".to_string()
            }
        }
        "Striker" => {
            if player.skills.contains_key(&39110) {
                "Esoteric Flurry".to_string()
            } else {
                "Deathblow".to_string()
            }
        }
        "Breaker" => {
            if player.skills.contains_key(&47020) {
                "Asura's Path".to_string()
            } else {
                "Brawl King Storm".to_string()
            }
        }
        "Deathblade" => {
            if player.skills.contains_key(&25038) {
                "Surge".to_string()
            } else {
                "Remaining Energy".to_string()
            }
        }
        "Shadowhunter" => {
            if player.skills.contains_key(&27860) {
                "Demonic Impulse".to_string()
            } else {
                "Perfect Suppression".to_string()
            }
        }
        "Reaper" => {
            let buff_names = get_buff_names(player, buffs);
            if buff_names.iter().any(|s| s.contains("Lunar Voice")) {
                "Lunar Voice".to_string()
            } else {
                "Hunger".to_string()
            }
        }
        "Souleater" => {
            if player.skills.contains_key(&46250) {
                "Night's Edge".to_string()
            } else {
                "Full Moon Harvester".to_string()
            }
        }
        "Sharpshooter" => {
            let buff_names = get_buff_names(player, buffs);
            if buff_names.iter().any(|s| s.contains("Loyal Companion") || s.contains("Hawk Support")) {
                "Loyal Companion".to_string()
            } else {
                "Death Strike".to_string()
            }
        }
        "Deadeye" => {
            if player.skills.contains_key(&29300) {
                "Enhanced Weapon".to_string()
            } else {
                "Pistoleer".to_string()
            }
        }
        "Artillerist" => {
            if player.skills.contains_key(&30260) {
                "Barrage Enhancement".to_string()
            } else {
                "Firepower Enhancement".to_string()
            }
        }
        "Machinist" => {
            let buff_names = get_buff_names(player, buffs);
            if buff_names.iter().any(|s| s.contains("Combat Mode") || s.contains("Evolutionary Legacy")) {
                "Evolutionary Legacy".to_string()
            } else {
                "Arthetinean Skill".to_string()
            }
        }
        "Gunslinger" => {
            if player.skills.contains_key(&38110) {
                "Peacemaker".to_string()
            } else {
                "Time to Hunt".to_string()
            }
        }
        "Artist" => {
            if player.skills.contains_key(&31400)
                && player.skills.contains_key(&31410)
                && player.skills.contains_key(&31420)
            {
                "Full Bloom".to_string()
            } else {
                "Recurrence".to_string()
            }
        }
        "Aeromancer" => {
            if player.skills.contains_key(&32250) && player.skills.contains_key(&32260) {
                "Wind Fury".to_string()
            } else {
                "Drizzle".to_string()
            }
        }
        _ => "Unknown".to_string(),
    }
}

fn get_buff_names(player: &EncounterEntity, buffs: &HashMap<u32, StatusEffect>) -> Vec<String> {
    let mut names = Vec::new();
    for (id, _) in player.damage_stats.buffed_by.iter() {
        if let Some(buff) = buffs.get(id) {
            names.push(buff.source.name.clone());
        }
    }

    names
}

fn get_spec_from_ark_passive(node: &ArkPassiveNode) -> String {
    match node.id {
        2160000 => "Berserker Technique",
        2160010 => "Mayhem",
        2170000 => "Lone Knight",
        2170010 => "Combat Readiness",
        2180000 => "Rage Hammer",
        2180010 => "Gravity Training",
        2360000 => "Judgement",
        2360010 => "Blessed Aura",
        2450000 => "Punisher",
        2450010 => "Predator",
        2230000 => "Ultimate Skill: Taijutsu",
        2230100 => "Shock Training",
        2220000 => "First Intention",
        2220100 => "Esoteric Skill Enhancement",
        2240000 => "Energy Overflow",
        2240100 => "Robust Spirit",
        2340000 => "Control",
        2340100 => "Pinnacle",
        2470000 => "Brawl King Storm",
        2470100 => "Asura's Path",
        2390000 => "Esoteric Flurry",
        2390010 => "Deathblow",
        2300000 => "Barrage Enhancement",
        2300100 => "Firepower Enhancement",
        2290000 => "Enhanced Weapon",
        2290100 => "Pistoleer",
        2280000 => "Death Strike",
        2280100 => "Loyal Companion",
        2350000 => "Evolutionary Legacy",
        2350100 => "Arthetinean Skill",
        2380000 => "Peacemaker",
        2380100 => "Time to Hunt",
        2370000 => "Igniter",
        2370100 => "Reflux",
        2190000 => "Grace of the Empress",
        2190100 => "Order of the Emperor",
        2200000 => "Communication Overflow",
        2200100 => "Master Summoner",
        2210000 => "Desperate Salvation",
        2210100 => "True Courage",
        2270000 => "Demonic Impulse",
        2270600 => "Perfect Suppression",
        2250000 => "Surge",
        2250600 => "Remaining Energy",
        2260000 => "Lunar Voice",
        2260600 => "Hunger",
        2460000 => "Full Moon Harvester",
        2460600 => "Night's Edge",
        2320000 => "Wind Fury",
        2320600 => "Drizzle",
        2310000 => "Full Bloom",
        2310600 => "Recurrence",
        _ => "Unknown",
    }
    .to_string()
}
