use crate::parser::entity_tracker::Entity;
use crate::parser::models::*;
use crate::parser::utils::is_support_class_id;
use hashbrown::HashMap;

pub fn get_buff_after_tripods(
    buff: &SkillBuffData,
    entity: &EncounterEntity,
    skill_id: u32,
    skill_effect_id: u32,
) -> SkillBuffData {
    let mut buff = buff.clone();
    let skill_effect_id = skill_effect_id as i32;
    let skill = &entity.skills.get(&skill_id);
    if let Some(skill) = skill.cloned() {
        if let Some(tripod_data) = skill.tripod_data {
            for tripod in tripod_data {
                for tripod_option in tripod.options {
                    let params = tripod_option.param;
                    let feature_type = tripod_option.effect_type;
                    let i0 = params.first().cloned().unwrap_or_default();
                    if feature_type == "change_buff_stat" {
                        if i0 == 0 || i0 == skill_effect_id {
                            let buff_id = params.get(1).cloned().unwrap_or_default();
                            if buff.id == buff_id {
                                let mut change_map: HashMap<i32, i32> = HashMap::new();
                                for i in (2..params.len()).step_by(2) {
                                    let stat_type = params.get(i).cloned();
                                    let value = params.get(i + 1).cloned();
                                    if let (Some(stat_type), Some(value)) = (stat_type, value) {
                                        change_map.insert(stat_type, value);
                                    }
                                }
                                for passive_option in buff.passive_option.iter_mut() {
                                    let change = change_map.get(
                                        &(STAT_TYPE_MAP[passive_option.key_stat.as_str()] as i32),
                                    );
                                    if passive_option.option_type == "stat" {
                                        if let Some(change) = change.cloned() {
                                            if tripod_option.param_type == "absolute" {
                                                passive_option.value += change;
                                            } else {
                                                passive_option.value = (passive_option.value as f32
                                                    * (1.0 + change as f32 / 100.0))
                                                    .round()
                                                    as i32;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else if feature_type == "add_buff_stat" {
                        if i0 == 0 || i0 == skill_effect_id {
                            let buff_id = params.get(1).cloned().unwrap_or_default();
                            if buff.id == buff_id {
                                let key_stat = params.get(2).cloned();
                                let value = params.get(3).cloned();
                                if let (Some(key_stat), Some(value)) = (key_stat, value) {
                                    buff.passive_option.push(PassiveOption {
                                        option_type: "stat".to_string(),
                                        key_stat: STAT_TYPE_MAP_TRA
                                            .get(&(key_stat as u32))
                                            .unwrap()
                                            .to_string(),
                                        key_index: 0,
                                        value,
                                    });
                                }
                            }
                        }
                    } else if feature_type == "change_buff_param" {
                        if let Some(status_effect_values) = buff.status_effect_values.as_mut() {
                            if i0 == 0 || i0 == skill_effect_id {
                                let buff_id = params.get(1).cloned().unwrap_or_default();
                                if buff.id == buff_id {
                                    if params.get(2).cloned().unwrap_or_default() == 0 {
                                        buff.status_effect_values = Some(params[3..].to_vec());
                                    } else {
                                        let mut new_values: Vec<i32> = vec![];
                                        for i in 0..status_effect_values.len().max(params.len() - 3)
                                        {
                                            if params.get(i + 3).is_some() {
                                                let old_value = status_effect_values
                                                    .get(i)
                                                    .cloned()
                                                    .unwrap_or_default();
                                                let new_value = (old_value as f32
                                                    * (1.0
                                                        + params
                                                            .get(i + 3)
                                                            .cloned()
                                                            .unwrap_or_default()
                                                            as f32
                                                            / 100.0)
                                                        .round())
                                                    as i32;
                                                new_values.push(new_value);
                                            }
                                        }
                                        buff.status_effect_values = Some(new_values);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    buff
}

pub fn get_crit_multiplier_from_combat_effect(
    ce: &CombatEffectData,
    ce_condition_data: CombatEffectConditionData,
) -> f64 {
    let mut crit_damage_rate = 0.0;

    ce.effects
        .iter()
        .filter(|effect| {
            effect
                .actions
                .iter()
                .any(|action| action.action_type == "modify_critical_multiplier")
        })
        .for_each(|effect| {
            if is_combat_effect_condition_valid(
                effect,
                ce_condition_data.self_entity,
                ce_condition_data.target_entity,
                ce_condition_data.caster_entity,
                ce_condition_data.skill,
                ce_condition_data.hit_option,
                ce_condition_data.target_count,
            ) {
                effect
                    .actions
                    .iter()
                    .filter(|action| action.action_type == "modify_critical_multiplier")
                    .for_each(|action| {
                        if action.action_type == "modify_critical_multiplier" {
                            let val =
                                action.args.first().cloned().unwrap_or_default() as f64 / 100.0;
                            crit_damage_rate += val;
                        }
                    })
            }
        });

    crit_damage_rate
}

pub fn is_combat_effect_condition_valid(
    effect: &CombatEffectDetail,
    self_entity: &Entity,
    target_entity: &Entity,
    caster_entity: &Entity,
    skill: Option<&SkillData>,
    hit_option: i32,
    target_count: i32,
) -> bool {
    let mut is_valid = true;
    for condition in effect.conditions.iter() {
        if !is_valid {
            break;
        }

        let actor = &condition.actor;
        match condition.condition_type.as_str() {
            "target_count" => {
                if target_count != condition.arg {
                    is_valid = false;
                }
            }
            "current_skill" => {
                if let Some(skill) = skill {
                    if skill.id != condition.arg {
                        is_valid = false;
                    }
                } else {
                    is_valid = false;
                }
            }
            "pc" => {
                if actor == "self" {
                    if self_entity.entity_type != EntityType::PLAYER {
                        is_valid = false;
                    }
                } else if actor == "target" {
                    if target_entity.entity_type != EntityType::PLAYER {
                        is_valid = false;
                    }
                } else if actor == "caster" {
                    if caster_entity.entity_type != EntityType::PLAYER {
                        is_valid = false;
                    }
                } else {
                    is_valid = false;
                }
            }
            "skill_identity_category" => {
                if let Some(skill) = skill {
                    if let Some(identity_category) = &skill.identity_category {
                        if *IDENTITY_CATEGORY.get(identity_category.as_str()).unwrap()
                            != condition.arg
                        {
                            is_valid = false;
                        }
                    } else {
                        is_valid = false;
                    }
                } else {
                    is_valid = false;
                }
            }
            "abnormal_move_immune" => {
                if target_entity.entity_type != EntityType::BOSS || !target_entity.push_immune {
                    is_valid = false;
                }
            }
            "abnormal_move_all" | "abnormal_move" | "abnormal_status" => {
                is_valid = false;
            }
            "current_skill_group" => {
                if let Some(skill) = skill {
                    if let Some(groups) = &skill.groups {
                        if !groups.contains(&condition.arg) {
                            is_valid = false;
                        }
                    } else {
                        is_valid = false;
                    }
                } else {
                    is_valid = false;
                }
            }
            "hp_less" => {
                let entity = match actor.as_str() {
                    "self" => Some(&self_entity),
                    "target" => Some(&target_entity),
                    "caster" => Some(&caster_entity),
                    _ => None,
                };

                if let Some(entity) = entity {
                    if let (Some(hp), Some(max_hp)) = (entity.stats.get(&1), entity.stats.get(&27))
                    {
                        if (*hp / *max_hp) >= (condition.arg as i64 / 100) {
                            is_valid = false;
                        }
                    } else {
                        is_valid = false;
                    }
                } else {
                    is_valid = false;
                }
            }
            "npc_scaled_level_less" => {
                if actor == "target" {
                    if target_entity.entity_type == EntityType::BOSS
                        && target_entity.balance_level > condition.arg as u16
                    {
                        is_valid = false;
                    }
                } else {
                    is_valid = false;
                }
            }
            "npc_grade_less" => {
                if actor == "target" {
                    if let Some(grade) = NPC_GRADE.get(target_entity.grade.as_str()).cloned() {
                        if target_entity.entity_type == EntityType::BOSS && grade > condition.arg {
                            is_valid = false;
                        }
                    } else {
                        is_valid = false;
                    }
                } else {
                    is_valid = false;
                }
            }
            "npc_grade_greater" => {
                if actor == "target" {
                    if let Some(grade) = NPC_GRADE.get(target_entity.grade.as_str()).cloned() {
                        if target_entity.entity_type == EntityType::BOSS && grade < condition.arg {
                            is_valid = false;
                        }
                    } else {
                        is_valid = false;
                    }
                } else {
                    is_valid = false;
                }
            }
            "identity_stance" => {
                if actor == "self" {
                    if self_entity.entity_type != EntityType::PLAYER
                        || self_entity.stance as i32 != condition.arg
                    {
                        is_valid = false;
                    }
                } else {
                    is_valid = false;
                }
            }
            "directional_attack" => {
                if (hit_option + 1) & condition.arg == 0 {
                    is_valid = false;
                }
            }
            _ => {
                is_valid = false;
            }
        }
    }

    is_valid
}

pub fn apply_rdps(
    damage_owner: &mut EncounterEntity,
    source_entity: Option<&mut EncounterEntity>,
    skill_id: u32,
    delta: f64,
) {
    let delta = delta.round() as i64;
    let skill = damage_owner.skills.get_mut(&skill_id).unwrap();
    if let Some(source) = source_entity {
        source.damage_stats.rdps_damage_given += delta;

        if is_support_class_id(source.class_id) {
            damage_owner.damage_stats.rdps_damage_received_support += delta;
            skill.rdps_damage_received_support += delta;
        }
    }

    damage_owner.damage_stats.rdps_damage_received += delta;
    skill.rdps_damage_received += delta;
}
