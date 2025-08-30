use crate::live::debug_print;
use crate::live::entity_tracker::Entity;
use crate::live::skill_tracker::{CastEvent, SkillTracker};
use crate::live::stats_api::InspectInfo;
use crate::live::status_tracker::StatusEffectDetails;
use crate::live::data::*;
use crate::live::models::*;
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
        return if let Some(effect) = SKILL_EFFECT_DATA.get(&skill_effect_id) {
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
        };
    } else {
        return if let Some(skill) = SKILL_DATA.get(&skill_id) {
            if let Some(summon_source_skill) = skill.summon_source_skills.as_ref() {
                for source in summon_source_skill {
                    if skill_tracker
                        .skill_timestamp
                        .get(&(entity_id, *source))
                        .is_some()
                    {
                        if let Some(skill) = SKILL_DATA.get(source) {
                            return (
                                skill.name.clone().unwrap_or(skill.id.to_string()) + " (Summon)",
                                skill.icon.clone().unwrap_or_default(),
                                Some(summon_source_skill.clone()),
                                false,
                                skill.is_hyper_awakening,
                            );
                        }
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
        };
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

pub fn damage_gem_value_to_level(value: u32, tier: u8) -> u8 {
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

pub fn cooldown_gem_value_to_level(value: u32, tier: u8) -> u8 {
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

pub fn support_damage_gem_value_to_level(value: u32) -> u8 {
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

pub fn generate_intervals(start: i64, end: i64) -> Vec<i64> {
    if start >= end {
        return Vec::new();
    }

    (0..end - start).step_by(1_000).collect()
}

pub fn sum_in_range(vec: &[(i64, i64)], start: i64, end: i64) -> i64 {
    let start_idx = binary_search_left(vec, start);
    let end_idx = binary_search_left(vec, end + 1);

    vec[start_idx..end_idx]
        .iter()
        .map(|&(_, second)| second)
        .sum()
}

pub fn binary_search_left(vec: &[(i64, i64)], target: i64) -> usize {
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

pub fn calculate_average_dps(data: &[(i64, i64)], start_time: i64, end_time: i64) -> Vec<i64> {
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

pub fn get_skill_id(new_skill: u32, original_buff_id: u32) -> u32 {
    new_skill - 1_000_000_000 - original_buff_id
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

pub fn get_buff_names(player: &EncounterEntity, buffs: &HashMap<u32, StatusEffect>) -> Vec<String> {
    let mut names = Vec::new();
    for (id, _) in player.damage_stats.buffed_by.iter() {
        if let Some(buff) = buffs.get(id) {
            names.push(buff.source.name.clone());
        }
    }

    names
}

pub fn get_spec_from_ark_passive(node: &ArkPassiveNode) -> String {
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

pub fn get_damage_without_hyper_or_special(e: &EncounterEntity) -> i64 {
    let hyper = e.damage_stats.hyper_awakening_damage;
    let special = e
        .skills
        .values()
        .filter(|s| s.special.unwrap_or(false))
        .map(|s| s.total_damage)
        .sum::<i64>();
    e.damage_stats.damage_dealt - hyper - special
}

pub struct SupportBuffs {
    pub brand: f64,
    pub buff: f64,
    pub identity: f64,
    pub hyper: f64,
}
