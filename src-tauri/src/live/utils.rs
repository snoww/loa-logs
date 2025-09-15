use crate::constants::DB_VERSION;
use crate::data::*;
use crate::live::entity_tracker::Entity;
use crate::live::skill_tracker::{CastEvent, SkillTracker};
use crate::live::stats_api::InspectInfo;
use crate::live::status_tracker::StatusEffectDetails;
use crate::models::*;
use flate2::write::GzEncoder;
use flate2::Compression;
use hashbrown::HashMap;
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

pub fn is_support(entity: &EncounterEntity) -> bool {
    if let Some(spec) = &entity.spec {
        is_support_spec(spec)
    } else {
        is_support_class(&entity.class_id)
    }
}

pub fn is_support_class(class_id: &u32) -> bool {
    matches!(class_id, 105 | 204 | 602 | 113)
}

pub fn is_support_spec(spec: &str) -> bool {
    matches!(
        spec,
        "Desperate Salvation" | "Full Bloom" | "Blessed Aura" | "Liberator"
    )
}

pub fn is_battle_item(skill_effect_id: &u32, _item_type: &str) -> bool {
    if let Some(item) = SKILL_EFFECT_DATA.get(skill_effect_id)
        && let Some(category) = item.item_type.as_ref() {
            return category == "useup";
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
        || buff_category == "supportbuff"
    {
        if let Some(buff_source_skills) = buff.source_skills.as_ref() {
            if let Some(source_skill) = source_skill {
                let skill = SKILL_DATA.get(&source_skill);
                get_summon_source_skill(skill, &mut status_effect);
            } else {
                // get the first skill that has a name, fall back to first if none
                let source_skill = {
                    let mut first_any = None;
                    let mut first_named = None;
                    for id in buff_source_skills {
                        if let Some(skill) = SKILL_DATA.get(id) {
                            if first_any.is_none() {
                                first_any = Some(skill);
                            }
                            if skill.name.is_some() {
                                first_named = Some(skill);
                                break; // break once skill with name found
                            }
                        }
                    }
                    first_named.or(first_any)
                };
                get_summon_source_skill(source_skill, &mut status_effect);
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
    } else if buff_category == "battleitem"
        && let Some(buff_source_item) = SKILL_EFFECT_DATA.get(&buff_id) {
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

    Some(status_effect)
}

fn get_summon_source_skill(skill: Option<&SkillData>, status_effect: &mut StatusEffect) {
    if let Some(skill) = skill {
        if let Some(summon_skills) = skill.summon_source_skills.as_ref() {
            let summon_source_skill = summon_skills.first().unwrap_or(&0);
            if *summon_source_skill > 0
                && let Some(summon_skill) = SKILL_DATA.get(summon_source_skill) {
                    status_effect.source.skill = Some(summon_skill.clone());
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

    if let Some(passive_option) = buff
        .per_level_data
        .get("1")
        .map(|data| &data.passive_options)
    {
        for option in passive_option {
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
                } else if ["critical_hit_rate", "criticalhit", "criticalhit_x"]
                    .contains(&key_stat_str)
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
            } else if option_type == "combat_effect"
                && let Some(combat_effect) = COMBAT_EFFECT_DATA.get(&option.key_index) {
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

pub type SkillDetails = (
    String,           // name
    String,           // icon
    Option<Vec<u32>>, // summon source skills
    bool,             // if skill is special, i.e. cannot crit or be buffed
    bool,             // is hyper awakening
);

pub fn get_skill_name_and_icon(
    skill_id: u32,
    skill_effect_id: u32,
    skill_tracker: &SkillTracker,
    entity_id: u64,
) -> SkillDetails {
    if (skill_id == 0) && (skill_effect_id == 0) {
        (
            "Bleed".to_string(),
            "buff_168.png".to_string(),
            None,
            false,
            false,
        )
    } else if (skill_effect_id != 0) && (skill_id == 0) {
        if let Some(effect) = SKILL_EFFECT_DATA.get(&skill_effect_id) {
            // if ValueJ is greater than 1,
            // 1 = esther, 2 = fixed, 3 = not used, 4 = orb power
            // these effects are not affected by crits or buffs
            let special = effect.values[9] > 0;

            if let Some(item_name) = effect.item_name.as_ref() {
                return (
                    item_name.clone(),
                    effect.icon.as_ref().cloned().unwrap_or_default(),
                    None,
                    special,
                    false,
                );
            }
            if let Some(source_skill) = effect.source_skills.as_ref() {
                if let Some(skill) = SKILL_DATA.get(source_skill.iter().min().unwrap_or(&0)) {
                    return (
                        skill.name.clone().unwrap_or(skill.id.to_string()),
                        skill.icon.clone().unwrap_or_default(),
                        None,
                        special,
                        skill.is_hyper_awakening,
                    );
                }
            } else if let Some(skill) = SKILL_DATA.get(&(skill_effect_id / 10)) {
                return (
                    skill.name.clone().unwrap_or(skill.id.to_string()),
                    skill.icon.clone().unwrap_or_default(),
                    None,
                    special,
                    skill.is_hyper_awakening,
                );
            }

            if effect.comment.is_empty() {
                (effect.id.to_string(), "".to_string(), None, special, false)
            } else {
                (effect.comment.clone(), "".to_string(), None, special, false)
            }
        } else {
            (skill_id.to_string(), "".to_string(), None, false, false)
        }
    } else if let Some(skill) = SKILL_DATA.get(&skill_id) {
        if let Some(summon_source_skill) = skill.summon_source_skills.as_ref() {
            for source in summon_source_skill {
                if skill_tracker
                    .skill_timestamp
                    .get(&(entity_id, *source))
                    .is_some()
                    && let Some(skill) = SKILL_DATA.get(source) {
                        return (
                            skill.name.clone().unwrap_or(skill.id.to_string()) + " (Summon)",
                            skill.icon.clone().unwrap_or_default(),
                            Some(summon_source_skill.clone()),
                            false,
                            skill.is_hyper_awakening,
                        );
                    }
            }
            if let Some(skill) = SKILL_DATA.get(summon_source_skill.iter().min().unwrap_or(&0))
            {
                (
                    skill.name.clone().unwrap_or(skill.id.to_string()) + " (Summon)",
                    skill.icon.clone().unwrap_or_default(),
                    Some(summon_source_skill.clone()),
                    false,
                    skill.is_hyper_awakening,
                )
            } else {
                (skill_id.to_string(), "".to_string(), None, false, false)
            }
        } else if let Some(source_skill) = skill.source_skills.as_ref() {
            if let Some(skill) = SKILL_DATA.get(source_skill.iter().min().unwrap_or(&0)) {
                (
                    skill.name.clone().unwrap_or(skill.id.to_string()),
                    skill.icon.clone().unwrap_or_default(),
                    None,
                    false,
                    skill.is_hyper_awakening,
                )
            } else {
                (skill_id.to_string(), "".to_string(), None, false, false)
            }
        } else {
            (
                skill.name.clone().unwrap_or(skill.id.to_string()),
                skill.icon.clone().unwrap_or_default(),
                None,
                false,
                skill.is_hyper_awakening,
            )
        }
    } else if let Some(skill) = SKILL_DATA.get(&(skill_id - (skill_id % 10))) {
        (
            skill.name.clone().unwrap_or(skill.id.to_string()),
            skill.icon.clone().unwrap_or_default(),
            None,
            false,
            skill.is_hyper_awakening,
        )
    } else {
        (skill_id.to_string(), "".to_string(), None, false, false)
    }
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
        113 => "Valkyrie",
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
        604 => "Wildsoul",
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

pub fn get_engravings(engraving_ids: &Option<Vec<u32>>) -> Option<Vec<String>> {
    let ids = match engraving_ids {
        Some(engravings) => engravings,
        None => return None,
    };
    let mut engravings: Vec<String> = Vec::new();

    for engraving_id in ids.iter() {
        if let Some(engraving_data) = ENGRAVING_DATA.get(engraving_id) {
            engravings.push(engraving_data.name.clone().unwrap_or("Unknown".to_string()));
        }
    }

    engravings.sort_unstable();
    Some(engravings)
}

pub fn is_hat_buff(buff_id: &u32) -> bool {
    matches!(buff_id, 362600 | 212305 | 319503 | 485100)
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
    damage_log: HashMap<String, Vec<(i64, i64)>>,
    cast_log: HashMap<String, HashMap<u32, Vec<i32>>>,
    boss_hp_log: HashMap<String, Vec<BossHpLog>>,
    raid_clear: bool,
    party_info: Vec<Vec<String>>,
    raid_difficulty: String,
    region: Option<String>,
    player_info: Option<HashMap<String, InspectInfo>>,
    meter_version: String,
    ntp_fight_start: i64,
    rdps_valid: bool,
    manual: bool,
    skill_cast_log: HashMap<u64, HashMap<u32, BTreeMap<i64, SkillCast>>>,
    skill_cooldowns: HashMap<u32, Vec<CastEvent>>,
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
        boss_hp_log
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
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
                    .iter()
                    .enumerate()
                    .map(|(index, party)| (index as i32, party.clone()))
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
        loadout_hash,
        combat_power,
        ark_passive_active,
        spec,
        ark_passive_data,
        support_ap,
        support_brand,
        support_identity,
        support_hyper
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25)",
        )
        .expect("failed to prepare entity statement");

    let fight_start = encounter.fight_start;
    let fight_end = encounter.last_combat_packet;

    // get average support buffs for supports
    let mut buffs = HashMap::new();
    for party in party_info.iter() {
        let party_members: Vec<_> = encounter
            .entities
            .iter()
            .filter(|(name, _)| party.contains(name))
            .map(|(name, entity)| entity)
            .collect();

        // specs are not determined for dps classes, but should be set for supports
        let party_without_support: Vec<_> = party_members
            .iter()
            .filter(|entity| !is_support(entity))
            .collect();

        if party_members.len() - party_without_support.len() == 1 {
            let party_damage_total: i64 = party_without_support
                .iter()
                .map(|e| get_damage_without_hyper_or_special(e))
                .sum();

            if party_damage_total <= 0 {
                continue;
            }

            let mut average_brand = 0.0;
            let mut average_buff = 0.0;
            let mut average_identity = 0.0;
            let mut average_hyper = 0.0;

            for player in party_without_support {
                let damage_dealt = get_damage_without_hyper_or_special(player) as f64;

                if damage_dealt <= 0.0 {
                    continue;
                }

                let party_damage_percent = damage_dealt / party_damage_total as f64;

                let brand_ratio = player.damage_stats.debuffed_by_support as f64 / damage_dealt;
                let buff_ratio = player.damage_stats.buffed_by_support as f64 / damage_dealt;
                let identity_ratio = player.damage_stats.buffed_by_identity as f64 / damage_dealt;

                average_brand += brand_ratio * party_damage_percent;
                average_buff += buff_ratio * party_damage_percent;
                average_identity += identity_ratio * party_damage_percent;
                average_hyper += (player.damage_stats.buffed_by_hat as f64
                    / player.damage_stats.damage_dealt as f64)
                    * party_damage_percent;
            }

            if let Some(support) = party_members.iter().find(|entity| is_support(entity)) {
                buffs.insert(
                    support.name.clone(),
                    SupportBuffs {
                        brand: average_brand,
                        buff: average_buff,
                        identity: average_identity,
                        hyper: average_hyper,
                    },
                );
            }
        }
    }

    for (_key, entity) in encounter.entities.iter_mut().filter(|(_, e)| {
        ((e.entity_type == EntityType::Player && e.class_id > 0)
            || e.name == encounter.local_player
            || e.entity_type == EntityType::Esther
            || (e.entity_type == EntityType::Boss && e.max_hp > 0))
            && e.damage_stats.damage_dealt > 0
    }) {
        if entity.entity_type == EntityType::Player {
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

            let spec = get_player_spec(entity, &encounter.encounter_damage_stats.buffs, false);
            entity.spec = Some(spec.clone());

            if let Some(info) = player_info
                .as_ref()
                .and_then(|stats| stats.get(&entity.name))
            {
                for gem in info.gems.iter().flatten() {
                    let skill_ids = if matches!(gem.gem_type, 34 | 35 | 65 | 63 | 61) {
                        GEM_SKILL_MAP
                            .get(&gem.skill_id)
                            .cloned()
                            .unwrap_or_default()
                    } else {
                        vec![gem.skill_id]
                    };

                    for skill_id in skill_ids {
                        if let Some(skill) = entity.skills.get_mut(&skill_id) {
                            match gem.gem_type {
                                27 | 35 => {
                                    // cooldown gems
                                    skill.gem_cooldown =
                                        Some(cooldown_gem_value_to_level(gem.value, gem.tier));
                                    skill.gem_tier = Some(gem.tier);
                                }
                                64 | 65 => {
                                    // support effect damage gems
                                    skill.gem_damage =
                                        Some(support_damage_gem_value_to_level(gem.value));
                                    skill.gem_tier_dmg = Some(gem.tier);
                                }
                                _ => {
                                    // damage gems
                                    skill.gem_damage =
                                        Some(damage_gem_value_to_level(gem.value, gem.tier));
                                    skill.gem_tier_dmg = Some(gem.tier);
                                }
                            }
                        }
                    }
                }

                entity.ark_passive_active = Some(info.ark_passive_enabled);

                let engravings = get_engravings(&info.engravings);
                if entity.class_id == 104
                    && engravings.as_ref().is_some_and(|engravings| {
                        engravings
                            .iter()
                            .any(|e| e == "Awakening" || e == "Drops of Ether")
                    })
                {
                    entity.spec = Some("Princess".to_string());
                } else if spec == "Unknown" {
                    // not reliable enough to be used on its own
                    if let Some(tree) = info.ark_passive_data.as_ref()
                        && let Some(enlightenment) = tree.enlightenment.as_ref() {
                            for node in enlightenment.iter() {
                                let spec = get_spec_from_ark_passive(node);
                                if spec != "Unknown" {
                                    entity.spec = Some(spec);
                                    break;
                                }
                            }
                        }
                }

                if entity.combat_power.is_none() {
                    entity.combat_power = info.combat_power.as_ref().map(|c| c.score);
                }

                entity.engraving_data = engravings;
                entity.ark_passive_data = info.ark_passive_data.clone();
                entity.loadout_hash = info.loadout_snapshot.clone();
            }
        }

        if entity.name == encounter.local_player {
            for (skill_id, events) in skill_cooldowns.iter() {
                if let Some(skill) = entity.skills.get_mut(skill_id) {
                    skill.time_available =
                        Some(get_total_available_time(events, fight_start, fight_end));
                }
            }
        }

        entity.damage_stats.dps = entity.damage_stats.damage_dealt / duration_seconds;

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
                    e.skill_cast_log = log.values().cloned().collect();
                });
            }
        }

        let compressed_skills = compress_json(&entity.skills);
        let compressed_damage_stats = compress_json(&entity.damage_stats);

        let damage_dealt = entity.damage_stats.damage_dealt;
        let damage_without_hyper =
            (damage_dealt - entity.damage_stats.hyper_awakening_damage) as f64;
        let support_buffs = buffs.get(&entity.name);

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
                entity.loadout_hash,
                entity.combat_power,
                entity.ark_passive_active,
                entity.spec,
                json!(entity.ark_passive_data),
                support_buffs
                    .map(|b| b.buff)
                    .unwrap_or(entity.damage_stats.buffed_by_support as f64 / damage_without_hyper),
                support_buffs.map(|b| b.brand).unwrap_or(
                    entity.damage_stats.debuffed_by_support as f64 / damage_without_hyper
                ),
                support_buffs.map(|b| b.identity).unwrap_or(
                    entity.damage_stats.buffed_by_identity as f64 / damage_without_hyper
                ),
                support_buffs
                    .map(|b| b.hyper)
                    .unwrap_or(entity.damage_stats.buffed_by_hat as f64 / damage_without_hyper),
            ])
            .expect("failed to insert entity");
    }

    let mut players = encounter
        .entities
        .values()
        .filter(|e| {
            ((e.entity_type == EntityType::Player && e.class_id != 0 && e.max_hp > 0)
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
        && player.entity_type == EntityType::Player
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

pub fn get_skill_id(new_skill: u32, original_buff_id: u32) -> u32 {
    new_skill - 1_000_000_000 - original_buff_id
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

pub fn get_player_spec(
    player: &EncounterEntity,
    buffs: &HashMap<u32, StatusEffect>,
    skip_min_check: bool,
) -> String {
    if !skip_min_check && player.skills.len() < 8 {
        return "Unknown".to_string();
    }

    match player.class.as_str() {
        "Berserker" => {
            // if has bloody rush
            if player.skills.contains_key(&16140)
                || player.skills.contains_key(&16145)
                || player.skills.contains_key(&16146)
                || player.skills.contains_key(&16147)
            {
                "Berserker Technique"
            } else {
                "Mayhem"
            }
        }
        "Destroyer" => {
            if player.skills.contains_key(&18090) {
                "Gravity Training"
            } else {
                "Rage Hammer"
            }
        }
        "Gunlancer" => {
            if player.skills.contains_key(&17200) && player.skills.contains_key(&17210) {
                "Lone Knight"
            } else if player.skills.contains_key(&17140) && player.skills.contains_key(&17110) {
                "Combat Readiness"
            } else {
                "Princess"
            }
        }
        "Paladin" => {
            // if has execution of judgement, judgement blade, or flash slash strength release tripod
            if player.skills.contains_key(&36250)
                || player.skills.contains_key(&36270)
                || player
                    .skills
                    .get(&36090)
                    .is_some_and(|s| s.tripod_index.is_some_and(|t| t.second == 3))
            {
                "Judgment"
            } else if player.skills.contains_key(&36200)
                || player.skills.contains_key(&36170)
                || player.skills.contains_key(&36800)
            {
                // if has heavenly blessing, wrath of god, or holy aura
                "Blessed Aura"
            } else {
                "Unknown"
            }
        }
        "Slayer" => {
            if player.skills.contains_key(&45004) {
                "Punisher"
            } else {
                "Predator"
            }
        }
        "Valkyrie" => {
            if player.skills.contains_key(&48060)
                || player.skills.contains_key(&48070)
                || player.skills.contains_key(&48500)
                || player.skills.contains_key(&48100)
            {
                // shining knight, final splendor, cataclysm, foresight slash
                "Shining Knight"
            } else if player.skills.contains_key(&48250)
                || player.skills.contains_key(&48270)
                || player.skills.contains_key(&48230)
                || player.skills.contains_key(&48220)
                || player.skills.contains_key(&48040)
                || player.skills.contains_key(&48041)
                || player.skills.contains_key(&48042)
            {
                // seraphic oath, seraphic leap,
                // circle of truth, truth's decree
                // release light
                "Liberator"
            } else {
                "Unknown"
            }
        }
        "Arcanist" => {
            if player.skills.contains_key(&19282) {
                "Order of the Emperor"
            } else {
                "Grace of the Empress"
            }
        }
        "Summoner" => {
            if player
                .skills
                .iter()
                .any(|(_, skill)| skill.name.contains("Kelsion"))
            {
                "Communication Overflow"
            } else {
                "Master Summoner"
            }
        }
        "Bard" => {
            // if has tempest skill, or vivace, or heavenly tune with crit tripod
            if (player.skills.contains_key(&21147)
                || player.skills.contains_key(&21148)
                || player.skills.contains_key(&21149))
                || player.skills.contains_key(&21310)
                || player
                    .skills
                    .get(&21160)
                    .is_some_and(|s| s.tripod_index.is_some_and(|t| t.third == 2))
            {
                return "True Courage".to_string();
            } else if player
                .skills
                .get(&21160)
                .is_some_and(|s| s.tripod_index.is_some_and(|t| t.third == 1))
            {
                // if heavenly tune has atk pwr tripod
                return "Desperate Salvation".to_string();
            }

            "Unknown"
        }
        "Sorceress" => {
            // if has arcane rupture
            if player.skills.contains_key(&37100) || player.skills.contains_key(&37101) {
                "Igniter"
            } else {
                "Reflux"
            }
        }
        "Wardancer" => {
            if player.skills.contains_key(&22340) {
                "Esoteric Skill Enhancement"
            } else {
                "First Intention"
            }
        }
        "Scrapper" => {
            if player.skills.contains_key(&23230) {
                "Ultimate Skill: Taijutsu"
            } else {
                "Shock Training"
            }
        }
        "Soulfist" => {
            if player.skills.contains_key(&24200) {
                "Energy Overflow"
            } else {
                "Robust Spirit"
            }
        }
        "Glaivier" => {
            if player.skills.contains_key(&34590) {
                "Pinnacle"
            } else {
                "Control"
            }
        }
        "Striker" => {
            if player.skills.contains_key(&39290) {
                "Deathblow"
            } else {
                "Esoteric Flurry"
            }
        }
        "Breaker" => {
            if player.skills.contains_key(&47020) {
                "Asura's Path"
            } else {
                "Brawl King Storm"
            }
        }
        "Deathblade" => {
            if player.skills.contains_key(&25038) {
                "Surge"
            } else {
                "Remaining Energy"
            }
        }
        "Shadowhunter" => {
            if player.skills.contains_key(&27860) {
                "Demonic Impulse"
            } else {
                "Perfect Suppression"
            }
        }
        "Reaper" => {
            let buff_names = get_buff_names(player, buffs);
            if buff_names.iter().any(|s| s.contains("Lunar Voice")) {
                "Lunar Voice"
            } else {
                "Hunger"
            }
        }
        "Souleater" => {
            if player.skills.contains_key(&46250) {
                "Night's Edge"
            } else {
                "Full Moon Harvester"
            }
        }
        "Sharpshooter" => {
            let buff_names = get_buff_names(player, buffs);
            if buff_names
                .iter()
                .any(|s| s.contains("Loyal Companion") || s.contains("Hawk Support"))
            {
                "Loyal Companion"
            } else {
                "Death Strike"
            }
        }
        "Deadeye" => {
            if player.skills.contains_key(&29300) {
                "Enhanced Weapon"
            } else {
                "Pistoleer"
            }
        }
        "Artillerist" => {
            if player.skills.contains_key(&30260) {
                "Barrage Enhancement"
            } else {
                "Firepower Enhancement"
            }
        }
        "Machinist" => {
            let buff_names = get_buff_names(player, buffs);
            if buff_names
                .iter()
                .any(|s| s.contains("Combat Mode") || s.contains("Evolutionary Legacy"))
            {
                "Evolutionary Legacy"
            } else {
                "Arthetinean Skill"
            }
        }
        "Gunslinger" => {
            if player.skills.contains_key(&38110) {
                "Peacemaker"
            } else {
                "Time to Hunt"
            }
        }
        "Artist" => {
            // dps if has cattle drive or shattering strike or rising moon
            // or sunsketch with crit tripod
            if player.skills.contains_key(&31940)
                || player.skills.contains_key(&31060)
                || player.skills.contains_key(&31145)
                || player
                    .skills
                    .get(&31400)
                    .is_some_and(|s| s.tripod_index.is_some_and(|t| t.third == 2))
            {
                return "Recurrence".to_string();
            } else if player
                .skills
                .get(&31400)
                .is_some_and(|s| s.tripod_index.is_some_and(|t| t.third == 1))
            {
                // if sunsketch has atk pwr tripod
                return "Full Bloom".to_string();
            }

            "Unknown"
        }
        "Aeromancer" => {
            if player.skills.contains_key(&32250) && player.skills.contains_key(&32260) {
                "Wind Fury"
            } else {
                "Drizzle"
            }
        }
        "Wildsoul" => {
            if player.skills.contains_key(&33400) || player.skills.contains_key(&33410) {
                "Ferality"
            } else {
                "Phantom Beast Awakening"
            }
        }
        _ => "Unknown",
    }
    .to_string()
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
        2360000 => "Judgment",
        2360010 => "Blessed Aura",
        2450000 => "Punisher",
        2450010 => "Predator",
        2480000 => "Shining Knight",
        2480100 => "Liberator",
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
        2330000 => "Ferality",
        2330100 => "Phantom Beast Awakening",
        _ => "Unknown",
    }
    .to_string()
}

pub fn boss_to_raid_map(boss: &str, max_hp: i64) -> Option<String> {
    match boss {
        "Phantom Legion Commander Brelshaza" => {
            if max_hp > 100_000_000_000 {
                Some("Act 2: Brelshaza G2".to_string())
            } else {
                Some("Brelshaza G6".to_string())
            }
        }
        _ => RAID_MAP.get(boss).cloned(),
    }
}

pub fn get_total_available_time(
    skill_cooldown: &Vec<CastEvent>,
    encounter_start: i64,
    encounter_end: i64,
) -> i64 {
    let mut total_available_time = 0;
    let mut current_available_from = encounter_start;

    for event in skill_cooldown {
        if event.timestamp > current_available_from {
            total_available_time += event.timestamp - current_available_from;
        }

        let cooldown_end = event.timestamp + event.cooldown_duration_ms;
        current_available_from = current_available_from.max(cooldown_end);
    }

    if encounter_end > current_available_from {
        total_available_time += encounter_end - current_available_from;
    }

    total_available_time
}

fn get_damage_without_hyper_or_special(e: &EncounterEntity) -> i64 {
    let hyper = e.damage_stats.hyper_awakening_damage;
    let special = e
        .skills
        .values()
        .filter(|s| s.special.unwrap_or(false))
        .map(|s| s.total_damage)
        .sum::<i64>();
    e.damage_stats.damage_dealt - hyper - special
}

struct SupportBuffs {
    brand: f64,
    buff: f64,
    identity: f64,
    hyper: f64,
}
