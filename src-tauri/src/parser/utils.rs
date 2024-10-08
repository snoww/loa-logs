use crate::parser::entity_tracker::Entity;
use crate::parser::models::*;
use crate::parser::skill_tracker::SkillTracker;
use crate::parser::stats_api::{Engraving, PlayerStats};
use crate::parser::status_tracker::StatusEffectDetails;
use flate2::write::GzEncoder;
use flate2::Compression;
use hashbrown::HashMap;
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

pub fn is_battle_item(skill_effect_id: u32, _item_type: &str) -> bool {
    if let Some(item) = SKILL_EFFECT_DATA.get(&skill_effect_id) {
        if let Some(category) = item.item_category.as_ref() {
            return category == "useup_battle_item_common_attack";
        }
    }
    false
}

pub fn get_status_effect_data(buff_id: u32, source_skill: Option<u32>) -> Option<StatusEffect> {
    let buff = SKILL_BUFF_DATA.get(&buff_id);
    if buff.is_none() || buff.unwrap().icon_show_type == "none" {
        return None;
    }

    let buff = buff.unwrap();
    let buff_category = if buff.buff_category == "ability"
        && [501, 502, 503, 504, 505].contains(&buff.unique_group)
    {
        "dropsofether".to_string()
    } else {
        buff.buff_category.clone()
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
            name: buff.name.clone(),
            desc: buff.desc.clone(),
            icon: buff.icon.clone(),
            ..Default::default()
        },
    };

    if buff_category == "classskill"
        || buff_category == "identity"
        || (buff_category == "ability" && buff.unique_group != 0)
    {
        if let Some(buff_source_skills) = buff.source_skill.as_ref() {
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
        if let Some(summon_skills) = skill.summon_source_skill.as_ref() {
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

    for option in buff.passive_option.iter() {
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
            if let Some(source_skill) = effect.source_skill.as_ref() {
                if let Some(skill) = SKILL_DATA.get(source_skill.first().unwrap_or(&0)) {
                    return (skill.name.clone(), skill.icon.clone(), None);
                }
            } else if let Some(skill) = SKILL_DATA.get(&(skill_effect_id / 10)) {
                return (skill.name.clone(), skill.icon.clone(), None);
            }
            (effect.comment.clone(), "".to_string(), None)
        } else {
            (skill_name, "".to_string(), None)
        };
    } else {
        return if let Some(skill) = SKILL_DATA.get(skill_id) {
            if let Some(summon_source_skill) = skill.summon_source_skill.as_ref() {
                for source in summon_source_skill {
                    if skill_tracker
                        .skill_timestamp
                        .get(&(entity_id, *source))
                        .is_some()
                    {
                        if let Some(skill) = SKILL_DATA.get(source) {
                            return (
                                skill.name.clone() + " (Summon)",
                                skill.icon.clone(),
                                Some(summon_source_skill.clone()),
                            );
                        }
                    }
                }
                if let Some(skill) = SKILL_DATA.get(summon_source_skill.first().unwrap_or(&0)) {
                    (
                        skill.name.clone() + " (Summon)",
                        skill.icon.clone(),
                        Some(summon_source_skill.clone()),
                    )
                } else {
                    (skill_name, "".to_string(), None)
                }
            } else if let Some(source_skill) = skill.source_skill.as_ref() {
                if let Some(skill) = SKILL_DATA.get(source_skill.first().unwrap_or(&0)) {
                    (skill.name.clone(), skill.icon.clone(), None)
                } else {
                    (skill_name, "".to_string(), None)
                }
            } else {
                (skill.name.clone(), skill.icon.clone(), None)
            }
        } else if let Some(skill) = SKILL_DATA.get(&(skill_id - (skill_id % 10))) {
            (skill.name.clone(), skill.icon.clone(), None)
        } else {
            (skill_name, "".to_string(), None)
        };
    }
}

pub fn get_skill_name(skill_id: &u32) -> String {
    SKILL_DATA
        .get(skill_id)
        .map_or(skill_id.to_string(), |skill| { 
            if skill.name.is_empty() {
                skill_id.to_string()
            } else {
                skill.name.clone()
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

fn damage_gem_value_to_level(value: u32) -> u8 {
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

fn cooldown_gem_value_to_level(value: u32) -> u8 {
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

fn gem_skill_id_to_skill_ids(skill_id: u32) -> Vec<u32> {
    match skill_id {
        13000 | 13001 => vec![18011, 18030], // destroyer hypergravity skills
        23000 => vec![
            20311, 20310, 20070, 20071, 20080, 20081, 20170, 20181, 20280, 20281,
        ], // summoner elemental damage
        41000 => vec![25038, 25035, 25036, 25037], // db surge skill
        42000 | 42001 => vec![27800, 27030, 27810, 27820, 27830, 27840, 27850, 27860], // sh transformation skills
        51001 => vec![28159, 28160, 28161, 28162, 28170], // sharpshooter bird skill
        53000 | 53001 => vec![30240, 30250, 30260, 30270, 30290], // arty barrage skills
        54000 | 54001 => vec![
            35720, 35750, 35760, 35761, 35770, 35771, 35780, 35781, 35790, 35800,
        ], // machinist transformation skills
        62000 => vec![32040, 32041],                      // aeromancer sun shower
        _ => vec![skill_id],
    }
}

pub fn get_engravings(
    class_id: u32,
    engravings: &Option<Vec<Engraving>>,
) -> Option<PlayerEngravings> {
    let engravings = match engravings {
        Some(engravings) => engravings,
        None => return None,
    };

    let mut class_engravings: Vec<PlayerEngraving> = Vec::new();
    let mut other_engravings: Vec<PlayerEngraving> = Vec::new();

    for e in engravings.iter() {
        if let Some(engraving_data) = ENGRAVING_DATA.get(&e.id) {
            let player_engraving = PlayerEngraving {
                name: engraving_data.name.clone(),
                id: e.id,
                level: e.level,
                icon: engraving_data.icon.clone(),
            };
            if is_class_engraving(class_id, engraving_data.id) {
                class_engravings.push(player_engraving);
            } else {
                other_engravings.push(player_engraving);
            }
        }
    }

    class_engravings.sort_by(|a, b| b.level.cmp(&a.level).then_with(|| a.id.cmp(&b.id)));
    other_engravings.sort_by(|a, b| b.level.cmp(&a.level).then_with(|| a.id.cmp(&b.id)));

    let class = if class_engravings.is_empty() {
        None
    } else {
        Some(class_engravings)
    };
    let other = if other_engravings.is_empty() {
        None
    } else {
        Some(other_engravings)
    };

    if class.is_none() && other.is_none() {
        None
    } else {
        Some(PlayerEngravings {
            class_engravings: class,
            other_engravings: other,
        })
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
    player_stats: Option<Cache<String, PlayerStats>>,
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
        gear_hash
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
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
        }

        entity.damage_stats.dps = entity.damage_stats.damage_dealt / duration_seconds;

        if let Some(stats) = player_stats
            .as_ref()
            .and_then(|stats| stats.get(&entity.name))
        {
            for gem in stats.gems.iter().flatten() {
                for skill_id in gem_skill_id_to_skill_ids(gem.skill_id) {
                    if let Some(skill) = entity.skills.get_mut(&skill_id) {
                        match gem.gem_type {
                            5 | 34 => {
                                // damage gem
                                skill.gem_damage = Some(damage_gem_value_to_level(gem.value))
                            }
                            27 | 35 => {
                                // cooldown gem
                                skill.gem_cooldown = Some(cooldown_gem_value_to_level(gem.value))
                            }
                            _ => {}
                        }
                    }
                }
            }

            entity.engraving_data = get_engravings(entity.class_id, &stats.engravings);
            entity.gear_hash = Some(stats.hash.clone());
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
