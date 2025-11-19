use crate::data::*;
use crate::live::entity_tracker::Entity;
use crate::live::skill_tracker::SkillTracker;
use crate::live::status_tracker::StatusEffectDetails;
use crate::models::*;
use crate::utils::*;
use hashbrown::HashMap;

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

pub fn is_battle_item(skill_effect_id: &u32, _item_type: &str) -> bool {
    if let Some(item) = SKILL_EFFECT_DATA.get(skill_effect_id)
        && let Some(category) = item.item_type.as_ref()
    {
        return category == "useup";
    }
    false
}

pub fn get_status_effect_data(buff_id: u32, source_skill: Option<u32>) -> Option<StatusEffect> {
    let buff = SKILL_BUFF_DATA.get(&buff_id)?;

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
        && let Some(buff_source_item) = SKILL_EFFECT_DATA.get(&buff_id)
    {
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
                && let Some(summon_skill) = SKILL_DATA.get(summon_source_skill)
            {
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
                && let Some(combat_effect) = COMBAT_EFFECT_DATA.get(&option.key_index)
            {
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
            if let Some(source_skills) = effect.source_skills.as_ref()
                && !source_skills.is_empty()
            {
                let source_skill = if source_skills.len() == 1 {
                    source_skills.first().cloned().unwrap_or_default()
                } else {
                    // take skill_effect_id e.g. 370015
                    // get base skill id -> 37000
                    let skill_effect_base = (skill_effect_id - (skill_effect_id % 1000)) / 10;
                    // get first skill that is furthest away from base (i.e. weapon attack)
                    source_skills
                        .iter()
                        .filter(|id| (**id as i32 - skill_effect_base as i32).abs() < 10000)
                        .max()
                        .cloned()
                        .unwrap_or_default()
                };
                if let Some(skill) = SKILL_DATA.get(&source_skill) {
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
        if let Some(summon_source_skills) = skill.summon_source_skills.as_ref()
            && !summon_source_skills.is_empty()
        {
            for source in summon_source_skills {
                if skill_tracker
                    .skill_timestamp
                    .get(&(entity_id, *source))
                    .is_some()
                    && let Some(skill) = SKILL_DATA.get(source)
                {
                    return (
                        skill.name.clone().unwrap_or(skill.id.to_string()) + " (Summon)",
                        skill.icon.clone().unwrap_or_default(),
                        Some(summon_source_skills.clone()),
                        false,
                        skill.is_hyper_awakening,
                    );
                }
            }
            if let Some(skill) = SKILL_DATA.get(summon_source_skills.iter().min().unwrap_or(&0)) {
                (
                    skill.name.clone().unwrap_or(skill.id.to_string()) + " (Summon)",
                    skill.icon.clone().unwrap_or_default(),
                    Some(summon_source_skills.clone()),
                    false,
                    skill.is_hyper_awakening,
                )
            } else {
                (skill_id.to_string(), "".to_string(), None, false, false)
            }
        } else if let Some(source_skills) = skill.source_skills.as_ref()
            && !source_skills.is_empty()
        {
            if let Some(skill) = SKILL_DATA.get(source_skills.iter().min().unwrap_or(&0)) {
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

pub fn is_hat_buff(buff_id: &u32) -> bool {
    matches!(buff_id, 362600 | 212305 | 319503 | 319504 | 485100)
}

pub fn is_hyper_hat_buff(buff_id: &u32) -> bool {
    matches!(buff_id, 362601 | 212306 | 319506 | 485101)
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

pub fn update_current_boss_name(boss_name: &str) -> String {
    match boss_name {
        "Chaos Lightning Dragon Jade" => "Argeos",
        "Vicious Argeos" | "Ruthless Lakadroff" | "Untrue Crimson Yoho" | "Despicable Skolakia" => {
            "Behemoth, the Storm Commander"
        }
        "Krathoios's Tail" => "Krathoios",
        _ => boss_name,
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
