use crate::data::{
    RDPS_ADDITIONAL_IDENTITY_GROUP, SKILL_BUFF_DATA, SKILL_DATA, SKILL_EFFECT_DATA,
    SUPPORT_IDENTITY_GROUP, SUPPORT_MARKING_GROUP,
};
use crate::live::entity_tracker::{Entity, EntityTracker, InspectSnapshot, SkillRuntimeData};
use crate::live::player_stats::PlayerStats;
use crate::live::status_tracker::StatusEffectDetails;
use crate::live::{DEBUG_DUMP_DAMAGE_STATE_JSON, write_debug_json_dump};
use crate::models::{HitFlag, HitOption, PerLevelData};
use crate::utils::is_support_class;
use hashbrown::HashMap;
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct HitRdpsAttribution {
    pub rdps_type: u8,
    pub source_entity_id: u64,
    pub source_skill_id: u32,
    pub damage: i64,
    pub is_support: bool,
}

#[derive(Debug, Clone)]
pub struct HitEntityRdpsAttribution {
    pub source_entity_id: u64,
    pub damage: i64,
    pub is_support: bool,
}

#[derive(Debug, Clone)]
pub struct HitDebugSkillGroupAttribution {
    pub source_entity_id: u64,
    pub group_name: String,
    pub damage: i64,
    pub damage_increase: i64,
}

#[derive(Debug, Clone, Default)]
pub struct HitRdpsResult {
    pub crit_rate_raw: Option<f64>,
    pub crit_rate_capped: Option<f64>,
    pub crit_damage_multiplier: Option<f64>,
    pub rdps_damage_received: i64,
    pub rdps_damage_received_support: i64,
    pub entity_attributions: Vec<HitEntityRdpsAttribution>,
    pub attributions: Vec<HitRdpsAttribution>,
    pub debug_skill_group_attributions: Vec<HitDebugSkillGroupAttribution>,
}

#[derive(Debug, Clone)]
struct ContributionFactor {
    rdps_type: u8,
    source_entity_id: u64,
    source_skill_id: u32,
    factor: f64,
    is_support: bool,
}

const RDPS_TYPE_DAMAGE_BUFF: u8 = 1;
const RDPS_TYPE_TARGET_DEBUFF: u8 = 3;
const RDPS_TYPE_HYPER: u8 = 5;
const SUPPORT_IDENTITY_SOURCE_SKILLS: [u32; 3] = [21140, 21141, 21142];
const SUPPORT_IDENTITY_SKILL_GROUPS: [u32; 4] = [15000, 60000, 24000, 16000];

fn get_hit_crit_metrics(
    stats: &PlayerStats,
    damage_type: u8,
    can_crit: bool,
) -> Option<(f64, f64, f64)> {
    if !can_crit {
        return None;
    }

    let crit_rate_raw = stats.critical_hit_rate.value();
    let crit_rate_capped = crit_rate_raw.min(stats.critical_hit_rate_cap);
    let crit_damage_multiplier = (1.0 + stats.critical_damage_rate.value())
        * (1.0 + stats.critical_damage_rate_2.value())
        * (1.0
            + if damage_type == 0 {
                stats.physical_critical_damage_amplify.value()
            } else {
                stats.magical_critical_damage_amplify.value()
            });
    Some((crit_rate_raw, crit_rate_capped, crit_damage_multiplier))
}

pub fn compute_hit_rdps(
    attacker: &Entity,
    target: &Entity,
    damage: i64,
    skill_id: u32,
    skill_id_real: u32,
    skill_effect_id: u32,
    hit_option: &HitOption,
    hit_flag: &HitFlag,
    damage_attr: Option<u8>,
    damage_type: u8,
    is_hyper_awakening: bool,
    special: bool,
    se_on_source: &[StatusEffectDetails],
    se_on_target: &[StatusEffectDetails],
    event_timestamp: i64,
    entity_tracker: &EntityTracker,
    buffered_entities: Option<&HashMap<u64, Entity>>,
    buffered_owner_self_effects: Option<&HashMap<u64, Vec<StatusEffectDetails>>>,
) -> Option<HitRdpsResult> {
    if damage <= 0 || special {
        return None;
    }
    let debug_enabled = DEBUG_DUMP_DAMAGE_STATE_JSON
        && matches!(attacker.entity_type, crate::models::EntityType::Player);

    let (can_crit, is_affected_by_buffs) =
        resolve_skill_effect_flags(skill_effect_id, is_hyper_awakening);
    let attacker_snapshot = attacker
        .inspect_snapshot
        .as_ref()
        .or_else(|| get_buffered_or_live_snapshot(attacker.id, buffered_entities, entity_tracker));
    let Some(attacker_snapshot) = attacker_snapshot else {
        if debug_enabled {
            dump_rdps_hit_trace(
                "missing_attacker_snapshot",
                attacker,
                target,
                damage,
                skill_id,
                skill_id_real,
                skill_effect_id,
                hit_option,
                hit_flag,
                damage_attr,
                damage_type,
                is_hyper_awakening,
                event_timestamp,
                None,
                None,
                None,
                None,
                None,
                se_on_source,
                se_on_target,
                &[],
                None,
                None,
                None,
                &HitRdpsResult::default(),
            );
        }
        return None;
    };
    let mut stats = PlayerStats::default();
    stats.load_from_snapshot(attacker_snapshot, attacker.id, attacker.class_id);
    let stats_after_snapshot = if debug_enabled {
        Some(stats.debug_dump_value())
    } else {
        None
    };
    let mut runtime_state = attacker.runtime_state();
    runtime_state.identity_runtime_reliable =
        attacker.id != 0 && attacker.id == entity_tracker.local_entity_id;
    stats.apply_runtime_state(runtime_state);
    let runtime_data = attacker.skill_runtime_data.get(&skill_id_real).or_else(|| {
        get_buffered_or_live_skill_runtime_data(
            attacker.id,
            skill_id_real,
            buffered_entities,
            entity_tracker,
        )
    });
    stats.apply_skill_runtime_data(skill_effect_id, runtime_data);
    let stats_after_runtime = if debug_enabled {
        Some(stats.debug_dump_value())
    } else {
        None
    };
    if !is_affected_by_buffs && !is_hyper_awakening {
        let crit_metrics = get_hit_crit_metrics(&stats, damage_type, can_crit);
        let result = HitRdpsResult {
            crit_rate_raw: crit_metrics.map(|(crit_rate_raw, _, _)| crit_rate_raw),
            crit_rate_capped: crit_metrics.map(|(_, crit_rate_capped, _)| crit_rate_capped),
            crit_damage_multiplier: crit_metrics
                .map(|(_, _, crit_damage_multiplier)| crit_damage_multiplier),
            ..Default::default()
        };
        if debug_enabled {
            dump_rdps_hit_trace(
                "not_affected_by_buffs",
                attacker,
                target,
                damage,
                skill_id,
                skill_id_real,
                skill_effect_id,
                hit_option,
                hit_flag,
                damage_attr,
                damage_type,
                is_hyper_awakening,
                event_timestamp,
                Some(attacker_snapshot),
                runtime_data,
                stats_after_snapshot,
                stats_after_runtime,
                None,
                se_on_source,
                se_on_target,
                &[],
                None,
                None,
                None,
                &result,
            );
        }
        return Some(result);
    }
    let effective_damage_attr = stats.resolve_damage_attr(damage_attr);
    let skill_groups = get_skill_groups(skill_id);
    let skill_real_groups = if skill_id_real != skill_id {
        get_skill_groups(skill_id_real)
    } else {
        skill_groups
    };
    if can_crit
        && matches!(hit_option, HitOption::BACK_ATTACK)
        && is_directional_skill_any(skill_id, skill_id_real, runtime_data, 1)
    {
        stats.critical_hit_rate.add_self(0.1, "back_attack");
    }
    let mut contributions = Vec::new();
    append_source_contributions(
        &mut stats,
        &mut contributions,
        attacker.id,
        skill_id_real,
        hit_flag,
        effective_damage_attr,
        is_hyper_awakening,
        runtime_data,
        se_on_source,
        event_timestamp,
        entity_tracker,
        buffered_entities,
        buffered_owner_self_effects,
    );
    let mut damage_multiplier = 1.0;
    append_target_contributions(
        &mut stats,
        &mut contributions,
        attacker,
        skill_id,
        skill_effect_id,
        hit_option,
        hit_flag,
        can_crit,
        effective_damage_attr,
        damage_type,
        is_hyper_awakening,
        skill_groups,
        se_on_target,
        &mut damage_multiplier,
        event_timestamp,
        entity_tracker,
        buffered_entities,
        buffered_owner_self_effects,
    );
    stats.apply_dynamic_effects(
        skill_id,
        skill_id_real,
        skill_effect_id,
        skill_groups,
        skill_real_groups,
        runtime_data,
        Some(target),
        Some(hit_option),
        se_on_source,
        se_on_target,
        event_timestamp,
    );
    let stats_after_effects = if debug_enabled {
        Some(stats.debug_dump_value())
    } else {
        None
    };
    let crit_metrics = get_hit_crit_metrics(&stats, damage_type, can_crit);

    let total_attack_power = stats
        .calculate_final_attack_power(
            hit_option,
            hit_flag,
            effective_damage_attr,
            damage_type,
            is_hyper_awakening,
            is_affected_by_buffs,
            can_crit,
            can_crit,
        )
        .value();
    if total_attack_power <= 0.0 {
        let result = HitRdpsResult {
            crit_rate_raw: crit_metrics.map(|(crit_rate_raw, _, _)| crit_rate_raw),
            crit_rate_capped: crit_metrics.map(|(_, crit_rate_capped, _)| crit_rate_capped),
            crit_damage_multiplier: crit_metrics
                .map(|(_, _, crit_damage_multiplier)| crit_damage_multiplier),
            ..Default::default()
        };
        if debug_enabled {
            dump_rdps_hit_trace(
                "non_positive_total_attack_power",
                attacker,
                target,
                damage,
                skill_id,
                skill_id_real,
                skill_effect_id,
                hit_option,
                hit_flag,
                damage_attr,
                damage_type,
                is_hyper_awakening,
                event_timestamp,
                Some(attacker_snapshot),
                runtime_data,
                stats_after_snapshot,
                stats_after_runtime,
                stats_after_effects,
                se_on_source,
                se_on_target,
                &contributions,
                Some(damage_multiplier),
                Some(total_attack_power),
                None,
                &result,
            );
        }
        return Some(result);
    }

    let entity_portions = stats.get_damage_portions_contributed_from_all_entities(
        total_attack_power,
        hit_option,
        hit_flag,
        effective_damage_attr,
        damage_type,
        is_hyper_awakening,
        is_affected_by_buffs,
        can_crit,
        can_crit,
    );

    let mut result = HitRdpsResult::default();
    if let Some((crit_rate_raw, crit_rate_capped, crit_damage_multiplier)) = crit_metrics {
        result.crit_rate_raw = Some(crit_rate_raw);
        result.crit_rate_capped = Some(crit_rate_capped);
        result.crit_damage_multiplier = Some(crit_damage_multiplier);
    }
    if DEBUG_DUMP_DAMAGE_STATE_JSON {
        result.debug_skill_group_attributions = compute_debug_skill_group_attributions(
            &stats,
            total_attack_power,
            damage,
            &entity_portions,
            hit_option,
            hit_flag,
            effective_damage_attr,
            damage_type,
            is_hyper_awakening,
            is_affected_by_buffs,
            can_crit,
        );
    }
    for &(portion, entity_id) in &entity_portions {
        let entity_damage = (portion * damage as f64).round() as i64;
        if entity_id == attacker.id {
            continue;
        }
        if entity_damage <= 0 {
            continue;
        }

        result.rdps_damage_received += entity_damage;
        let grouped = contributions
            .iter()
            .filter(|contribution| contribution.source_entity_id == entity_id)
            .cloned()
            .collect::<Vec<_>>();
        let entity_is_support = grouped.iter().any(|contribution| contribution.is_support)
            || get_buffered_or_live_entity(entity_id, buffered_entities, entity_tracker)
                .is_some_and(|entity| is_support_class(&entity.class_id));
        result.entity_attributions.push(HitEntityRdpsAttribution {
            source_entity_id: entity_id,
            damage: entity_damage,
            is_support: entity_is_support,
        });
        let factor_sum = grouped
            .iter()
            .map(|entry| entry.factor.max(0.0))
            .sum::<f64>();
        if grouped.is_empty() || factor_sum <= 0.0 {
            if entity_is_support {
                result.rdps_damage_received_support += entity_damage;
            }
            continue;
        }

        for contribution in grouped {
            let attributed_damage =
                ((contribution.factor.max(0.0) / factor_sum) * entity_damage as f64).round() as i64;
            if attributed_damage <= 0 {
                continue;
            }
            if contribution.is_support {
                result.rdps_damage_received_support += attributed_damage;
            }
            result.attributions.push(HitRdpsAttribution {
                rdps_type: contribution.rdps_type,
                source_entity_id: contribution.source_entity_id,
                source_skill_id: contribution.source_skill_id,
                damage: attributed_damage,
                is_support: contribution.is_support,
            });
        }
    }

    if debug_enabled {
        dump_rdps_hit_trace(
            "ok",
            attacker,
            target,
            damage,
            skill_id,
            skill_id_real,
            skill_effect_id,
            hit_option,
            hit_flag,
            damage_attr,
            damage_type,
            is_hyper_awakening,
            event_timestamp,
            Some(attacker_snapshot),
            runtime_data,
            stats_after_snapshot,
            stats_after_runtime,
            stats_after_effects,
            se_on_source,
            se_on_target,
            &contributions,
            Some(damage_multiplier),
            Some(total_attack_power),
            Some(&entity_portions),
            &result,
        );
    }

    Some(result)
}

pub fn snapshot_owner_player_stats_for_buffs(
    owner_entity: &Entity,
    source_skill_id: Option<u32>,
    self_effects: &[StatusEffectDetails],
    eval_tick_ms: i64,
    entity_tracker: &EntityTracker,
) -> Option<PlayerStats> {
    let owner_snapshot = owner_entity.inspect_snapshot.as_ref()?;
    snapshot_owner_player_stats_from_snapshot(
        owner_entity,
        owner_snapshot,
        source_skill_id,
        self_effects,
        eval_tick_ms,
        entity_tracker,
    )
}

fn snapshot_owner_player_stats_from_snapshot(
    owner_entity: &Entity,
    owner_snapshot: &InspectSnapshot,
    source_skill_id: Option<u32>,
    self_effects: &[StatusEffectDetails],
    eval_tick_ms: i64,
    entity_tracker: &EntityTracker,
) -> Option<PlayerStats> {
    let mut stats = PlayerStats::default();
    stats.load_from_snapshot(owner_snapshot, owner_entity.id, owner_entity.class_id);

    let mut runtime_state = owner_entity.runtime_state();
    runtime_state.identity_runtime_reliable =
        owner_entity.id != 0 && owner_entity.id == entity_tracker.local_entity_id;
    stats.apply_runtime_state_snapshot(runtime_state);

    let source_runtime_data =
        source_skill_id.and_then(|skill_id| owner_entity.skill_runtime_data.get(&skill_id));
    stats.apply_skill_runtime_data(0, source_runtime_data);
    apply_owner_snapshot_self_buffs(&mut stats, self_effects, entity_tracker);

    if let Some(source_skill_id) = source_skill_id {
        let skill_groups = get_skill_groups(source_skill_id);
        stats.apply_dynamic_effects(
            source_skill_id,
            source_skill_id,
            0,
            skill_groups,
            skill_groups,
            source_runtime_data,
            None,
            None,
            self_effects,
            &[],
            eval_tick_ms,
        );
    }

    Some(stats)
}

fn apply_owner_snapshot_self_buffs(
    stats: &mut PlayerStats,
    self_effects: &[StatusEffectDetails],
    entity_tracker: &EntityTracker,
) {
    for effect in select_unique_group_effects(self_effects) {
        let Some(skill_buff) = SKILL_BUFF_DATA.get(&effect.status_effect_id) else {
            continue;
        };
        let source_entity_id = resolve_effect_source_id(&effect, skill_buff, entity_tracker, None);
        let effect_runtime_data = effect.source_skill_runtime_snapshot.as_ref();
        let Some(level_data) = get_level_data_resolved(
            skill_buff,
            effect.skill_level,
            effect_runtime_data,
            effect.stack_count,
        ) else {
            continue;
        };
        let buff_source = skill_buff
            .name
            .clone()
            .unwrap_or_else(|| skill_buff.id.to_string());

        for option in &level_data.passive_options {
            match option.option_type.as_str() {
                "stat" => stats.add_stat_from_source(
                    &option.key_stat,
                    i64::from(option.value),
                    source_entity_id,
                    &buff_source,
                ),
                "combat_effect" if option.key_index > 0 => {
                    stats.add_combat_effect_from_id(
                        option.key_index as u32,
                        source_entity_id,
                        &buff_source,
                    );
                }
                "attack_power_amplify_multiplier" => {
                    stats.add_attack_power_amplify_multiplier(
                        option.value as f64 / 10000.0,
                        source_entity_id,
                        &buff_source,
                    );
                }
                _ => {}
            }
        }
    }
}

fn append_source_contributions(
    stats: &mut PlayerStats,
    contributions: &mut Vec<ContributionFactor>,
    attacker_id: u64,
    skill_id_real: u32,
    hit_flag: &HitFlag,
    damage_attr: Option<u8>,
    is_hyper_awakening: bool,
    current_skill_runtime_data: Option<&SkillRuntimeData>,
    se_on_source: &[StatusEffectDetails],
    event_timestamp: i64,
    entity_tracker: &EntityTracker,
    buffered_entities: Option<&HashMap<u64, Entity>>,
    buffered_owner_self_effects: Option<&HashMap<u64, Vec<StatusEffectDetails>>>,
) {
    let selected_effects = select_unique_group_effects(se_on_source);
    let attacker_attack_power = stats
        .calculate_attack_power_pre_multipliers()
        .value()
        .max(1.0);

    for effect in selected_effects {
        let Some(skill_buff) = SKILL_BUFF_DATA.get(&effect.status_effect_id) else {
            continue;
        };
        let source_entity_id =
            resolve_effect_source_id(&effect, skill_buff, entity_tracker, buffered_entities);
        if source_entity_id == 0 {
            continue;
        }
        let is_attributable_source =
            is_player_source_entity_id(source_entity_id, buffered_entities, entity_tracker);
        let is_self_source = source_entity_id == attacker_id;

        let effect_runtime_data = effect.source_skill_runtime_snapshot.as_ref();
        let Some(level_data) = get_level_data_resolved(
            skill_buff,
            effect.skill_level,
            effect_runtime_data,
            effect.stack_count,
        ) else {
            continue;
        };
        let source_skill_id = source_skill_id_from_effect(&effect);
        let source_entity =
            get_buffered_or_live_entity(source_entity_id, buffered_entities, entity_tracker);
        let source_snapshot =
            get_buffered_or_live_snapshot(source_entity_id, buffered_entities, entity_tracker);
        let source_class_id = source_entity
            .map(|entity| entity.class_id)
            .unwrap_or_default();
        let source_player_stats: Option<Arc<PlayerStats>> = buffered_owner_self_effects
            .and_then(|_| {
                rebuild_missing_effect_owner_snapshot(
                    &effect,
                    source_entity_id,
                    source_entity,
                    buffered_entities,
                    buffered_owner_self_effects,
                    event_timestamp,
                    entity_tracker,
                )
            })
            .map(Arc::new)
            .or_else(|| effect.owner_player_stats_snapshot.clone())
            .or_else(|| {
                source_snapshot.and_then(|snapshot| {
                    source_entity.map(|entity| {
                        Arc::new(load_player_stats_from_snapshot(
                            snapshot,
                            source_entity_id,
                            entity.class_id,
                        ))
                    })
                })
            });
        let is_support = is_attributable_source
            && is_support_source(
                source_entity_id,
                skill_buff,
                entity_tracker,
                buffered_entities,
            );
        let buff_source = skill_buff
            .name
            .clone()
            .unwrap_or_else(|| skill_buff.id.to_string());

        for option in &level_data.passive_options {
            match option.option_type.as_str() {
                "stat" => apply_source_passive_stat(
                    stats,
                    option,
                    skill_buff,
                    source_entity_id,
                    source_class_id,
                    source_skill_id,
                    source_player_stats.as_deref(),
                    &buff_source,
                ),
                "combat_effect" if option.key_index > 0 => stats.add_combat_effect_from_id(
                    option.key_index as u32,
                    source_entity_id,
                    &buff_source,
                ),
                "attack_power_amplify_multiplier" => stats.add_attack_power_amplify_multiplier(
                    option.value as f64 / 10000.0,
                    source_entity_id,
                    &buff_source,
                ),
                "class_option" if option.key_index > 0 => {
                    if let Some(source_entity) = source_entity {
                        stats.add_external_addon_from_source(
                            "class_option",
                            &option.key_stat,
                            option.key_index as u32,
                            i64::from(option.value),
                            source_entity_id,
                            source_entity.class_id,
                            &buff_source,
                        );
                    }
                }
                _ => {}
            }
        }

        if skill_buff.buff_type == "attack_power_amplify" && !is_hyper_awakening {
            let Some(source_player_stats) = source_player_stats.as_ref() else {
                continue;
            };
            let Some(status_effect_values) = level_data.status_effect_values.as_ref() else {
                continue;
            };
            let Some(percent_buff_raw) = status_effect_values.first() else {
                continue;
            };
            let percent_buff = *percent_buff_raw as f64 / 10000.0;
            if percent_buff <= 0.0 {
                continue;
            }

            let source_base_attack_power =
                source_player_stats.calculate_base_attack_power().value();
            if source_base_attack_power <= 0.0 {
                continue;
            }

            let percent_buff = percent_buff
                * (1.0
                    + get_ally_attack_power_power(source_player_stats)
                    + get_skill_attack_power_multiplier(source_player_stats, source_skill_id));
            let buff_value = source_base_attack_power * percent_buff;
            let factor = (source_base_attack_power * percent_buff) / attacker_attack_power;
            if factor > 0.0 {
                stats.attack_power_addend.add(
                    buff_value,
                    attacker_id,
                    source_entity_id,
                    buff_source.clone(),
                );
                if is_attributable_source && !is_self_source {
                    contributions.push(ContributionFactor {
                        rdps_type: RDPS_TYPE_DAMAGE_BUFF,
                        source_entity_id,
                        source_skill_id,
                        factor,
                        is_support,
                    });
                }
            }
        }

        if skill_buff.id == 192833
            && stats
                .get_skill_identity_category(skill_id_real, current_skill_runtime_data)
                .as_deref()
                .is_some_and(|category| category.eq_ignore_ascii_case("arcana_normal"))
        {
            stats
                .critical_hit_rate
                .add(0.2, attacker_id, source_entity_id, buff_source.clone());
        }

        let (normal_damage_factor, hyper_damage_factor) = get_source_passive_factors(
            &level_data.passive_options,
            hit_flag,
            damage_attr,
            is_hyper_awakening,
        );
        let normal_damage_factor = normal_damage_factor
            * get_source_damage_multiplier(
                skill_buff,
                source_skill_id,
                source_class_id,
                source_player_stats.as_deref(),
            );
        if normal_damage_factor > 0.0 {
            if is_attributable_source && !is_self_source {
                contributions.push(ContributionFactor {
                    rdps_type: RDPS_TYPE_DAMAGE_BUFF,
                    source_entity_id,
                    source_skill_id,
                    factor: normal_damage_factor,
                    is_support,
                });
            }
        }
        if hyper_damage_factor > 0.0 {
            if is_attributable_source && !is_self_source {
                contributions.push(ContributionFactor {
                    rdps_type: RDPS_TYPE_HYPER,
                    source_entity_id,
                    source_skill_id,
                    factor: hyper_damage_factor,
                    is_support,
                });
            }
        }
    }
}

fn apply_source_passive_stat(
    stats: &mut PlayerStats,
    option: &crate::models::PassiveOption,
    skill_buff: &crate::models::SkillBuffData,
    source_entity_id: u64,
    source_class_id: u32,
    source_skill_id: u32,
    source_player_stats: Option<&PlayerStats>,
    buff_source: &str,
) {
    let mut value = i64::from(option.value);
    if matches!(
        option.key_stat.as_str(),
        "skill_damage_sub_rate_1" | "skill_damage_sub_rate_2"
    ) {
        if let Some(source_player_stats) = source_player_stats {
            if is_identity_skill_buff(skill_buff) {
                value = ((value as f64)
                    * get_identity_buff_multiplier(
                        source_player_stats,
                        source_class_id,
                        source_skill_id,
                    ))
                .round() as i64;
            } else if option
                .key_stat
                .eq_ignore_ascii_case("skill_damage_sub_rate_2")
            {
                value = ((value as f64)
                    * (1.0
                        + get_ally_identity_damage_power(source_player_stats)
                        + get_skill_status_effect_multiplier(source_player_stats, source_skill_id)))
                .round() as i64;
            }
        }
    }
    stats.add_stat_from_source(&option.key_stat, value, source_entity_id, buff_source);
}

fn append_target_contributions(
    stats: &mut PlayerStats,
    contributions: &mut Vec<ContributionFactor>,
    attacker: &Entity,
    skill_id: u32,
    skill_effect_id: u32,
    hit_option: &HitOption,
    hit_flag: &HitFlag,
    can_crit: bool,
    damage_attr: Option<u8>,
    damage_type: u8,
    is_hyper_awakening: bool,
    _skill_groups: &[u32],
    se_on_target: &[StatusEffectDetails],
    damage_multiplier: &mut f64,
    event_timestamp: i64,
    entity_tracker: &EntityTracker,
    buffered_entities: Option<&HashMap<u64, Entity>>,
    buffered_owner_self_effects: Option<&HashMap<u64, Vec<StatusEffectDetails>>>,
) {
    if is_hyper_awakening {
        return;
    }

    let selected_effects =
        select_target_effects(attacker, se_on_target, entity_tracker, buffered_entities);
    for effect in selected_effects {
        let Some(skill_buff) = SKILL_BUFF_DATA.get(&effect.status_effect_id) else {
            continue;
        };
        let source_entity_id =
            resolve_effect_source_id(&effect, skill_buff, entity_tracker, buffered_entities);
        if source_entity_id == 0 {
            continue;
        }
        let is_self_source = is_same_player(
            attacker,
            source_entity_id,
            buffered_entities,
            entity_tracker,
        );
        let is_attributable_source =
            is_player_source_entity_id(source_entity_id, buffered_entities, entity_tracker);
        if !should_apply_target_effect(
            skill_buff,
            attacker,
            source_entity_id,
            entity_tracker,
            buffered_entities,
        ) {
            continue;
        }
        let source_skill_id = source_skill_id_from_effect(&effect);
        let raw_level_data = get_level_data(skill_buff, effect.skill_level);
        let effect_runtime_data = effect.source_skill_runtime_snapshot.as_ref();
        let Some(level_data) = get_level_data_resolved(
            skill_buff,
            effect.skill_level,
            effect_runtime_data,
            effect.stack_count,
        ) else {
            continue;
        };
        let is_support = is_attributable_source
            && is_support_source(
                source_entity_id,
                skill_buff,
                entity_tracker,
                buffered_entities,
            );
        let source_entity =
            get_buffered_or_live_entity(source_entity_id, buffered_entities, entity_tracker);
        let source_snapshot =
            get_buffered_or_live_snapshot(source_entity_id, buffered_entities, entity_tracker);
        let source_player_stats: Option<Arc<PlayerStats>> = buffered_owner_self_effects
            .and_then(|_| {
                rebuild_missing_effect_owner_snapshot(
                    &effect,
                    source_entity_id,
                    source_entity,
                    buffered_entities,
                    buffered_owner_self_effects,
                    event_timestamp,
                    entity_tracker,
                )
            })
            .map(Arc::new)
            .or_else(|| effect.owner_player_stats_snapshot.clone())
            .or_else(|| {
                source_snapshot.and_then(|snapshot| {
                    source_entity.map(|entity| {
                        Arc::new(load_player_stats_from_snapshot(
                            snapshot,
                            source_entity_id,
                            entity.class_id,
                        ))
                    })
                })
            });
        for option in &level_data.passive_options {
            apply_target_passive_option(option, damage_type, damage_attr, damage_multiplier);
        }
        if skill_buff.buff_type == "skill_damage_amplify" {
            let Some(status_effect_values) = level_data.status_effect_values.as_ref() else {
                continue;
            };
            let Some(raw_status_effect_values) =
                raw_level_data.and_then(|level_data| level_data.status_effect_values.as_ref())
            else {
                continue;
            };
            if status_effect_values.len() > 1 {
                let skill_id_to_buff = raw_status_effect_values
                    .first()
                    .copied()
                    .unwrap_or_default() as u32;
                let skill_effect_id_to_buff =
                    raw_status_effect_values.get(4).copied().unwrap_or_default() as u32;
                if (skill_id_to_buff == 0 || skill_id_to_buff == skill_id)
                    && (skill_effect_id_to_buff == 0 || skill_effect_id_to_buff == skill_effect_id)
                {
                    let Some(source_player_stats) = source_player_stats.as_ref() else {
                        continue;
                    };
                    let mut factor = status_effect_values[1] as f64 / 10000.0;
                    let mut multiplier = 1.0
                        + get_skill_status_effect_multiplier(source_player_stats, source_skill_id);
                    if SUPPORT_MARKING_GROUP.contains(&skill_buff.unique_group) {
                        multiplier += get_ally_brand_power(source_player_stats);
                    }
                    factor *= multiplier;
                    if factor > 0.0 {
                        stats.skill_damage_amplify.add(
                            factor,
                            attacker.id,
                            source_entity_id,
                            skill_buff
                                .name
                                .clone()
                                .unwrap_or_else(|| skill_buff.id.to_string()),
                        );
                        if is_attributable_source && !is_self_source {
                            contributions.push(ContributionFactor {
                                rdps_type: RDPS_TYPE_TARGET_DEBUFF,
                                source_entity_id,
                                source_skill_id,
                                factor,
                                is_support,
                            });
                        }
                    }
                }
            }
        }

        let direct_factor = get_target_direct_factor(
            skill_buff,
            &level_data,
            hit_option,
            hit_flag,
            can_crit,
            damage_attr,
            damage_type,
        );
        apply_target_direct_stats(
            stats,
            attacker.id,
            source_entity_id,
            skill_buff,
            &level_data,
            hit_option,
            hit_flag,
            damage_attr,
            damage_type,
        );
        if direct_factor > 0.0 {
            if is_attributable_source && !is_self_source {
                contributions.push(ContributionFactor {
                    rdps_type: RDPS_TYPE_TARGET_DEBUFF,
                    source_entity_id,
                    source_skill_id,
                    factor: direct_factor,
                    is_support,
                });
            }
        }
    }
}

fn get_source_passive_factors(
    passive_options: &[crate::models::PassiveOption],
    hit_flag: &HitFlag,
    damage_attr: Option<u8>,
    is_hyper_awakening: bool,
) -> (f64, f64) {
    let mut normal_damage_factor = 0.0;
    let mut hyper_damage_factor = 0.0;
    let is_critical = is_critical_hit(hit_flag);

    for option in passive_options {
        let factor = option.value as f64 / 10000.0;
        if factor == 0.0 {
            continue;
        }

        match option.key_stat.as_str() {
            "ultimate_awakening_dam_rate" | "awakening_dam_rate" if is_hyper_awakening => {
                hyper_damage_factor += factor;
            }
            "attack_power_rate"
            | "attack_power_sub_rate_1"
            | "attack_power_sub_rate_2"
            | "skill_damage_sub_rate_1"
            | "skill_damage_sub_rate_2"
            | "skill_damage_rate"
            | "evolution_dam_rate"
                if !is_hyper_awakening =>
            {
                normal_damage_factor += factor;
            }
            "critical_dam_rate" if !is_hyper_awakening && is_critical => {
                normal_damage_factor += factor;
            }
            "fire_dam_rate" if !is_hyper_awakening && damage_attr == Some(1) => {
                normal_damage_factor += factor;
            }
            "ice_dam_rate" if !is_hyper_awakening && damage_attr == Some(2) => {
                normal_damage_factor += factor;
            }
            "electricity_dam_rate" if !is_hyper_awakening && damage_attr == Some(3) => {
                normal_damage_factor += factor;
            }
            "earth_dam_rate" if !is_hyper_awakening && damage_attr == Some(5) => {
                normal_damage_factor += factor;
            }
            "dark_dam_rate" if !is_hyper_awakening && damage_attr == Some(6) => {
                normal_damage_factor += factor;
            }
            "holy_dam_rate" if !is_hyper_awakening && damage_attr == Some(7) => {
                normal_damage_factor += factor;
            }
            _ => {}
        }
    }

    (normal_damage_factor, hyper_damage_factor)
}

fn resolve_skill_effect_flags(skill_effect_id: u32, is_hyper_awakening: bool) -> (bool, bool) {
    if is_hyper_awakening {
        return (false, false);
    }

    let Some(skill_effect) = SKILL_EFFECT_DATA.get(&skill_effect_id) else {
        return (true, true);
    };
    let Some(flags) = skill_effect.values.get(9).copied() else {
        return (true, true);
    };

    let can_crit = (flags & (1 << 1)) == 0 && (flags & (1 << 2)) == 0;
    if can_crit {
        (true, true)
    } else {
        (false, false)
    }
}

fn get_skill_groups(skill_id: u32) -> &'static [u32] {
    SKILL_DATA
        .get(&skill_id)
        .and_then(|skill| skill.groups.as_deref())
        .unwrap_or(&[])
}

fn get_buffered_or_live_entity<'a>(
    entity_id: u64,
    buffered_entities: Option<&'a HashMap<u64, Entity>>,
    entity_tracker: &'a EntityTracker,
) -> Option<&'a Entity> {
    buffered_entities
        .and_then(|entities| entities.get(&entity_id))
        .or_else(|| entity_tracker.get_entity_ref(entity_id))
}

fn get_buffered_or_live_snapshot<'a>(
    entity_id: u64,
    buffered_entities: Option<&'a HashMap<u64, Entity>>,
    entity_tracker: &'a EntityTracker,
) -> Option<&'a InspectSnapshot> {
    get_buffered_or_live_entity(entity_id, buffered_entities, entity_tracker)
        .and_then(|entity| entity.inspect_snapshot.as_ref())
        .or_else(|| entity_tracker.get_inspect_snapshot(entity_id))
}

fn is_player_source_entity_id(
    entity_id: u64,
    buffered_entities: Option<&HashMap<u64, Entity>>,
    entity_tracker: &EntityTracker,
) -> bool {
    get_buffered_or_live_entity(entity_id, buffered_entities, entity_tracker)
        .is_some_and(|entity| matches!(entity.entity_type, crate::models::EntityType::Player))
}

fn rebuild_missing_effect_owner_snapshot(
    effect: &StatusEffectDetails,
    source_entity_id: u64,
    source_entity: Option<&Entity>,
    buffered_entities: Option<&HashMap<u64, Entity>>,
    buffered_owner_self_effects: Option<&HashMap<u64, Vec<StatusEffectDetails>>>,
    event_timestamp: i64,
    entity_tracker: &EntityTracker,
) -> Option<PlayerStats> {
    let owner_self_effects = buffered_owner_self_effects?.get(&source_entity_id)?;
    let source_entity = source_entity?;
    let owner_snapshot =
        get_buffered_or_live_snapshot(source_entity_id, buffered_entities, entity_tracker)?;
    snapshot_owner_player_stats_from_snapshot(
        source_entity,
        owner_snapshot,
        effect.source_skill_id,
        owner_self_effects,
        event_timestamp,
        entity_tracker,
    )
}

fn get_buffered_or_live_skill_runtime_data<'a>(
    entity_id: u64,
    skill_id: u32,
    buffered_entities: Option<&'a HashMap<u64, Entity>>,
    entity_tracker: &'a EntityTracker,
) -> Option<&'a crate::live::entity_tracker::SkillRuntimeData> {
    if let Some(buffered_entities) = buffered_entities {
        return buffered_entities
            .get(&entity_id)
            .and_then(|entity| entity.skill_runtime_data.get(&skill_id));
    }

    entity_tracker.get_skill_runtime_data(entity_id, skill_id)
}

fn is_same_player(
    attacker: &Entity,
    source_entity_id: u64,
    buffered_entities: Option<&HashMap<u64, Entity>>,
    entity_tracker: &EntityTracker,
) -> bool {
    if attacker.id == source_entity_id {
        return true;
    }

    get_buffered_or_live_entity(source_entity_id, buffered_entities, entity_tracker).is_some_and(
        |source_entity| {
            attacker.character_id != 0
                && source_entity.character_id != 0
                && attacker.character_id == source_entity.character_id
        },
    )
}

fn should_apply_target_effect(
    skill_buff: &crate::models::SkillBuffData,
    attacker: &Entity,
    source_entity_id: u64,
    entity_tracker: &EntityTracker,
    buffered_entities: Option<&HashMap<u64, Entity>>,
) -> bool {
    if is_party_wide_skill_buff(skill_buff) {
        if is_same_party_target_effect_source(
            attacker,
            source_entity_id,
            entity_tracker,
            buffered_entities,
        ) {
            return true;
        }
        return false;
    }
    if is_self_target_skill_buff(skill_buff)
        && !is_same_player(
            attacker,
            source_entity_id,
            buffered_entities,
            entity_tracker,
        )
    {
        return false;
    }
    true
}

pub fn filter_target_effects_for_attacker(
    attacker: &Entity,
    se_on_target: &[StatusEffectDetails],
    entity_tracker: &EntityTracker,
    buffered_entities: Option<&HashMap<u64, Entity>>,
) -> Vec<StatusEffectDetails> {
    if se_on_target.is_empty() {
        return Vec::new();
    }
    se_on_target
        .iter()
        .filter(|effect| {
            let Some(skill_buff) = SKILL_BUFF_DATA.get(&effect.status_effect_id) else {
                return true;
            };
            // Skip the source resolution when the buff can't be filtered out anyway.
            if !is_party_wide_skill_buff(skill_buff) && !is_self_target_skill_buff(skill_buff) {
                return true;
            }
            let source_entity_id =
                resolve_effect_source_id(effect, skill_buff, entity_tracker, buffered_entities);
            should_apply_target_effect(
                skill_buff,
                attacker,
                source_entity_id,
                entity_tracker,
                buffered_entities,
            )
        })
        .cloned()
        .collect()
}

fn apply_target_passive_option(
    option: &crate::models::PassiveOption,
    damage_type: u8,
    damage_attr: Option<u8>,
    damage_multiplier: &mut f64,
) {
    if !option.option_type.eq_ignore_ascii_case("stat") {
        return;
    }

    if matches!(
        option.key_stat.as_str(),
        "skill_damage_sub_rate_1" | "skill_damage_sub_rate_2" | "critical_hit_rate"
    ) {
        return;
    }

    let value = option.value as f64 / 10000.0;
    if value == 0.0 {
        return;
    }

    if damage_type == 1
        && matches!(
            option.key_stat.as_str(),
            "magical_inc_sub_rate_1" | "magical_inc_sub_rate_2" | "magical_inc_rate"
        )
    {
        *damage_multiplier *= 1.0 + value;
        return;
    }

    if damage_type == 0
        && matches!(
            option.key_stat.as_str(),
            "physical_inc_sub_rate_1" | "physical_inc_sub_rate_2" | "physical_inc_rate"
        )
    {
        *damage_multiplier *= 1.0 + value;
        return;
    }

    if matches!(
        (option.key_stat.as_str(), damage_attr),
        ("fire_res_rate", Some(1))
            | ("ice_res_rate", Some(2))
            | ("electricity_res_rate", Some(3))
            | ("earth_res_rate", Some(5))
            | ("dark_res_rate", Some(6))
            | ("holy_res_rate", Some(7))
    ) {
        *damage_multiplier *= 1.0 - value;
    }
}

fn is_same_party_target_effect_source(
    attacker: &Entity,
    source_entity_id: u64,
    entity_tracker: &EntityTracker,
    buffered_entities: Option<&HashMap<u64, Entity>>,
) -> bool {
    if entity_tracker.are_same_party_entities(attacker.id, source_entity_id) {
        return true;
    }

    get_buffered_or_live_entity(source_entity_id, buffered_entities, entity_tracker).is_some_and(
        |source_entity| {
            entity_tracker
                .are_same_party_characters(attacker.character_id, source_entity.character_id)
        },
    )
}

fn is_party_wide_skill_buff(skill_buff: &crate::models::SkillBuffData) -> bool {
    skill_buff.target.eq_ignore_ascii_case("party")
        || skill_buff.target.eq_ignore_ascii_case("self_party")
        || (skill_buff.target.eq_ignore_ascii_case("none")
            && skill_buff
                .icon_show_type
                .as_deref()
                .is_some_and(|icon| icon.eq_ignore_ascii_case("caster_or_party")))
}

fn is_self_target_skill_buff(skill_buff: &crate::models::SkillBuffData) -> bool {
    skill_buff.target.eq_ignore_ascii_case("self")
}

fn get_target_direct_factor(
    skill_buff: &crate::models::SkillBuffData,
    level_data: &PerLevelData,
    hit_option: &HitOption,
    _hit_flag: &HitFlag,
    can_crit: bool,
    damage_attr: Option<u8>,
    damage_type: u8,
) -> f64 {
    let Some(status_effect_values) = level_data.status_effect_values.as_ref() else {
        return 0.0;
    };

    match skill_buff.buff_type.as_str() {
        "directional_attack_amplify" => match hit_option {
            HitOption::FRONTAL_ATTACK => {
                get_status_effect_factor_with_divisor(status_effect_values, 0, 100.0)
            }
            HitOption::BACK_ATTACK => {
                get_status_effect_factor_with_divisor(status_effect_values, 4, 100.0)
            }
            _ => 0.0,
        },
        "instant_stat_amplify" => {
            let mut factor = get_status_effect_factor(status_effect_values, 7);
            factor += -get_status_effect_factor(status_effect_values, 2) * 0.5;
            factor += -get_status_effect_factor(status_effect_values, 3) * 0.5;
            if can_crit {
                factor += get_status_effect_factor(status_effect_values, 0);
                factor += get_status_effect_factor(status_effect_values, 1);
                factor += match damage_type {
                    0 => get_status_effect_factor(status_effect_values, 9),
                    1 => get_status_effect_factor(status_effect_values, 10),
                    _ => 0.0,
                };
            }
            factor
        }
        "instant_stat_amplify_by_contents" => {
            if damage_attr.is_some_and(|attr| {
                status_effect_values
                    .get(1)
                    .copied()
                    .is_some_and(|value| value as u8 == attr)
            }) {
                -get_status_effect_factor(status_effect_values, 2)
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

fn apply_target_direct_stats(
    stats: &mut PlayerStats,
    attacker_id: u64,
    source_entity_id: u64,
    skill_buff: &crate::models::SkillBuffData,
    level_data: &PerLevelData,
    _hit_option: &HitOption,
    _hit_flag: &HitFlag,
    _damage_attr: Option<u8>,
    _damage_type: u8,
) {
    let Some(status_effect_values) = level_data.status_effect_values.as_ref() else {
        return;
    };
    let buff_source = skill_buff
        .name
        .clone()
        .unwrap_or_else(|| skill_buff.id.to_string());

    match skill_buff.buff_type.as_str() {
        "directional_attack_amplify" => {
            let front_value = get_status_effect_factor_with_divisor(status_effect_values, 0, 100.0);
            let back_value = get_status_effect_factor_with_divisor(status_effect_values, 4, 100.0);
            if front_value != 0.0 {
                stats.front_attack_amplify.add(
                    front_value,
                    attacker_id,
                    source_entity_id,
                    buff_source.clone(),
                );
            }
            if back_value != 0.0 {
                stats.back_attack_amplify.add(
                    back_value,
                    attacker_id,
                    source_entity_id,
                    buff_source.clone(),
                );
            }
        }
        "instant_stat_amplify" => {
            let crit_rate = get_status_effect_factor(status_effect_values, 0);
            if crit_rate != 0.0 {
                stats.critical_hit_rate.add(
                    crit_rate,
                    attacker_id,
                    source_entity_id,
                    buff_source.clone(),
                );
            }
            let crit_damage = get_status_effect_factor(status_effect_values, 1);
            if crit_damage != 0.0 {
                stats.critical_damage_rate.add(
                    crit_damage,
                    attacker_id,
                    source_entity_id,
                    buff_source.clone(),
                );
            }
            stats.outgoing_dmg_stat_amp.add(
                get_status_effect_factor(status_effect_values, 7),
                attacker_id,
                source_entity_id,
                buff_source.clone(),
            );
            let physical_crit_damage_amp = get_status_effect_factor(status_effect_values, 9);
            if physical_crit_damage_amp != 0.0 {
                stats.physical_critical_damage_amplify.add(
                    physical_crit_damage_amp,
                    attacker_id,
                    source_entity_id,
                    buff_source.clone(),
                );
            }
            let magical_crit_damage_amp = get_status_effect_factor(status_effect_values, 10);
            if magical_crit_damage_amp != 0.0 {
                stats.magical_critical_damage_amplify.add(
                    magical_crit_damage_amp,
                    attacker_id,
                    source_entity_id,
                    buff_source.clone(),
                );
            }
            stats.physical_defense_break.add(
                -get_status_effect_factor(status_effect_values, 2) * 0.5,
                attacker_id,
                source_entity_id,
                buff_source.clone(),
            );
            stats.magical_defense_break.add(
                -get_status_effect_factor(status_effect_values, 3) * 0.5,
                attacker_id,
                source_entity_id,
                buff_source.clone(),
            );
        }
        "instant_stat_amplify_by_contents" => {
            if let Some(index) = status_effect_values
                .get(1)
                .copied()
                .and_then(|value| damage_attr_index(value as u8))
            {
                stats.damage_attr_amplifications[index].add(
                    -get_status_effect_factor(status_effect_values, 2),
                    attacker_id,
                    source_entity_id,
                    buff_source,
                );
            }
        }
        _ => {}
    }
}

fn get_ally_attack_power_power(player_stats: &PlayerStats) -> f64 {
    player_stats.ally_attack_power_power.value()
}

fn get_ally_brand_power(player_stats: &PlayerStats) -> f64 {
    player_stats.ally_brand_power.value()
}

fn get_skill_attack_power_multiplier(player_stats: &PlayerStats, source_skill_id: u32) -> f64 {
    player_stats.get_skill_attack_power_multiplier(source_skill_id)
}

fn get_skill_status_effect_multiplier(player_stats: &PlayerStats, source_skill_id: u32) -> f64 {
    player_stats
        .get_skill_status_effect_multiplier(source_skill_id, get_skill_groups(source_skill_id))
}

fn get_source_damage_multiplier(
    skill_buff: &crate::models::SkillBuffData,
    source_skill_id: u32,
    source_class_id: u32,
    source_player_stats: Option<&PlayerStats>,
) -> f64 {
    let Some(source_player_stats) = source_player_stats else {
        return 1.0;
    };

    if is_identity_skill_buff(skill_buff) {
        return get_identity_buff_multiplier(source_player_stats, source_class_id, source_skill_id);
    }
    if source_skill_has_identity_group(source_skill_id) {
        return 1.0
            + get_ally_identity_damage_power(source_player_stats)
            + get_skill_status_effect_multiplier(source_player_stats, source_skill_id);
    }
    1.0
}

fn is_critical_hit(hit_flag: &HitFlag) -> bool {
    matches!(hit_flag, HitFlag::CRITICAL | HitFlag::DOT_CRITICAL)
}

fn get_status_effect_factor(status_effect_values: &[i32], index: usize) -> f64 {
    status_effect_values.get(index).copied().unwrap_or_default() as f64 / 10000.0
}

fn get_status_effect_factor_with_divisor(
    status_effect_values: &[i32],
    index: usize,
    divisor: f64,
) -> f64 {
    status_effect_values.get(index).copied().unwrap_or_default() as f64 / divisor
}

fn damage_attr_index(damage_attr: u8) -> Option<usize> {
    match damage_attr {
        0..=7 => Some(damage_attr as usize),
        _ => None,
    }
}

fn get_damage_splits(dmg: f64, factors: &[f64]) -> Vec<f64> {
    let n = factors.len();
    let mut pieces = vec![0.0; n + 1];
    let mut shapley = vec![0.0; n];

    let mut total_factor = 1.0;
    for factor in factors {
        total_factor *= 1.0 + *factor;
    }
    let base_damage = 1.0 / total_factor;

    let mut factorial = vec![1.0; n + 1];
    for idx in 1..=n {
        factorial[idx] = idx as f64 * factorial[idx - 1];
    }

    let max_mask = 1usize << n;
    for idx in 0..n {
        let mut sum = 0.0;
        for mask in 0..max_mask {
            if (mask & (1 << idx)) != 0 {
                continue;
            }

            let mut prod = 1.0;
            let mut subset_size = 0usize;
            for factor_idx in 0..n {
                if (mask & (1 << factor_idx)) != 0 {
                    prod *= 1.0 + factors[factor_idx];
                    subset_size += 1;
                }
            }
            sum += prod * factorial[subset_size] * factorial[n - subset_size - 1];
        }
        shapley[idx] = base_damage * factors[idx] * sum / factorial[n];
    }

    pieces[0] = base_damage * dmg;
    for (idx, value) in shapley.into_iter().enumerate() {
        pieces[idx + 1] = value * dmg;
    }
    pieces
}

fn compute_debug_skill_group_attributions(
    stats: &PlayerStats,
    total_attack_power: f64,
    damage: i64,
    entity_portions: &[(f64, u64)],
    hit_option: &HitOption,
    hit_flag: &HitFlag,
    damage_attr: Option<u8>,
    damage_type: u8,
    is_hyper_awakening: bool,
    is_affected_by_buffs: bool,
    can_crit: bool,
) -> Vec<HitDebugSkillGroupAttribution> {
    let mut output = Vec::new();
    for &(entity_damage_portion, entity_id) in entity_portions {
        if entity_id == 0 || entity_id == stats.owner_id || entity_damage_portion <= 0.0 {
            continue;
        }

        let mut cloned_stats = stats.clone();
        let mut stat_contributions = Vec::<(f64, String, f64, Vec<(f64, String)>)>::new();
        for stat_idx in stats.iterate_stat_datas() {
            let source_stat = stats.get_stat_data_ref(stat_idx);
            if source_stat.get_value_for_entity_id(entity_id) <= 0.0 {
                continue;
            }

            let modification = source_stat.get_modification_for_entity_id(entity_id);
            let mut sum_value = 0.0;
            let mut original_values = Vec::with_capacity(modification.values.len());
            let mut mod_values = Vec::with_capacity(modification.values.len());
            for value in &modification.values {
                original_values.push(value.value);
                if value.value == 0.0 {
                    continue;
                }
                sum_value += value.value;
                mod_values.push((value.value, value.source.clone()));
            }
            if sum_value == 0.0 {
                continue;
            }

            {
                let modification = cloned_stats
                    .get_stat_data_ref_mut(stat_idx)
                    .get_modification_for_entity_id_mut(entity_id);
                for value in &mut modification.values {
                    value.value = 0.0;
                }
            }

            let attack_power_without = cloned_stats
                .calculate_final_attack_power(
                    hit_option,
                    hit_flag,
                    damage_attr,
                    damage_type,
                    is_hyper_awakening,
                    is_affected_by_buffs,
                    can_crit,
                    true,
                )
                .value();

            {
                let modification = cloned_stats
                    .get_stat_data_ref_mut(stat_idx)
                    .get_modification_for_entity_id_mut(entity_id);
                for (index, value) in modification.values.iter_mut().enumerate() {
                    value.value = original_values.get(index).copied().unwrap_or_default();
                }
            }

            if attack_power_without <= 0.0 {
                continue;
            }

            let delta = total_attack_power - attack_power_without;
            let contribution = delta / attack_power_without;
            if contribution > 0.0 {
                stat_contributions.push((
                    contribution,
                    stats.get_stat_data_name(stat_idx),
                    sum_value,
                    mod_values,
                ));
            }
        }

        if stat_contributions.is_empty() {
            continue;
        }

        let factors = stat_contributions
            .iter()
            .map(|(contribution, _, _, _)| *contribution)
            .collect::<Vec<_>>();
        let splits = stats.get_damage_splits(1.0, &factors);
        let self_portion = splits.first().copied().unwrap_or_default();
        let denominator = 1.0 - self_portion;
        if denominator <= 0.0 {
            continue;
        }
        let scalar = entity_damage_portion / denominator;

        for (index, (stat_factor, stat_name, sum_value, mod_values)) in
            stat_contributions.into_iter().enumerate()
        {
            let stat_shapley = splits.get(index + 1).copied().unwrap_or_default();
            for (value, source) in mod_values {
                if value == 0.0 {
                    continue;
                }

                let name = format!("{stat_name}/{source}");
                let contribution = stat_shapley * (value / sum_value);
                let damage_contribution = (contribution * damage as f64 * scalar) as i64;
                let damage_increase = (stat_factor * value / sum_value * damage as f64) as i64;
                if damage_contribution == 0 && damage_increase == 0 {
                    continue;
                }

                output.push(HitDebugSkillGroupAttribution {
                    source_entity_id: entity_id,
                    group_name: name,
                    damage: damage_contribution,
                    damage_increase,
                });
            }
        }
    }

    output
}

#[allow(clippy::too_many_arguments)]
fn dump_rdps_hit_trace(
    reason: &str,
    attacker: &Entity,
    target: &Entity,
    damage: i64,
    skill_id: u32,
    skill_id_real: u32,
    skill_effect_id: u32,
    hit_option: &HitOption,
    hit_flag: &HitFlag,
    damage_attr: Option<u8>,
    damage_type: u8,
    is_hyper_awakening: bool,
    event_timestamp: i64,
    attacker_snapshot: Option<&InspectSnapshot>,
    runtime_data: Option<&SkillRuntimeData>,
    stats_after_snapshot: Option<Value>,
    stats_after_runtime: Option<Value>,
    stats_after_effects: Option<Value>,
    se_on_source: &[StatusEffectDetails],
    se_on_target: &[StatusEffectDetails],
    contributions: &[ContributionFactor],
    damage_multiplier: Option<f64>,
    total_attack_power: Option<f64>,
    entity_portions: Option<&[(f64, u64)]>,
    result: &HitRdpsResult,
) {
    let label = format!(
        "{}-{}-{}-{}",
        attacker.name, event_timestamp, skill_id_real, reason
    );
    write_debug_json_dump(
        "rdps-hit",
        &label,
        &json!({
            "reason": reason,
            "context": {
                "event_timestamp": event_timestamp,
                "damage": damage,
                "skill_id": skill_id,
                "skill_id_real": skill_id_real,
                "skill_effect_id": skill_effect_id,
                "hit_option": format!("{hit_option:?}"),
                "hit_flag": format!("{hit_flag:?}"),
                "damage_attr": damage_attr,
                "damage_type": damage_type,
                "is_hyper_awakening": is_hyper_awakening,
            },
            "attacker": entity_debug_value(attacker),
            "target": entity_debug_value(target),
            "attacker_snapshot": attacker_snapshot.map(inspect_snapshot_debug_value),
            "runtime_data": skill_runtime_data_debug_value(runtime_data),
            "status_effects_on_source": se_on_source.iter().map(status_effect_debug_value).collect::<Vec<_>>(),
            "status_effects_on_target": se_on_target.iter().map(status_effect_debug_value).collect::<Vec<_>>(),
            "stats_after_snapshot": stats_after_snapshot,
            "stats_after_runtime": stats_after_runtime,
            "stats_after_effects": stats_after_effects,
            "contributions": contributions.iter().map(contribution_factor_debug_value).collect::<Vec<_>>(),
            "damage_multiplier": damage_multiplier.map(debug_json_f64),
            "total_attack_power": total_attack_power.map(debug_json_f64),
            "entity_portions": entity_portions.map(|portions| {
                portions.iter().map(|(portion, entity_id)| {
                    json!({
                        "entity_id": entity_id,
                        "portion": debug_json_f64(*portion),
                    })
                }).collect::<Vec<_>>()
            }),
            "result": hit_rdps_result_debug_value(result),
        }),
    );
}

fn entity_debug_value(entity: &Entity) -> Value {
    let runtime_state = entity.runtime_state();
    json!({
        "id": entity.id,
        "name": entity.name,
        "class_id": entity.class_id,
        "character_id": entity.character_id,
        "entity_type": format!("{:?}", entity.entity_type),
        "stance": entity.stance,
        "level": entity.level,
        "gear_level": debug_json_f64(entity.gear_level as f64),
        "owner_id": entity.owner_id,
        "npc_grade": entity.grade,
        "hp": runtime_state.current_hp,
        "max_hp": runtime_state.max_hp,
        "mp": runtime_state.current_mp,
        "max_mp": runtime_state.max_mp,
        "combat_mp_recovery": runtime_state.combat_mp_recovery,
        "inspect_stale": entity.inspect_stale,
        "has_inspect_snapshot": entity.inspect_snapshot.is_some(),
        "combat_state": {
            "identity_gauge1": entity.identity_gauge1,
            "identity_gauge2": entity.identity_gauge2,
            "identity_gauge3": entity.identity_gauge3,
            "identity_gauge1_prev": entity.identity_gauge1_prev,
            "identity_gauge2_prev": entity.identity_gauge2_prev,
            "identity_gauge3_prev": entity.identity_gauge3_prev,
            "identity_last_skill_start_id": entity.identity_last_skill_start_id,
            "identity_last_skill_start_at_ms": entity.identity_last_skill_start_at_ms,
            "destroyer_recent_consumed_cores": entity.destroyer_recent_consumed_cores,
            "destroyer_recent_consumed_at_ms": entity.destroyer_recent_consumed_at_ms,
        },
    })
}

fn inspect_item_build_debug_value(
    item: &crate::live::inspect_stats::InspectItemBuildDebug,
) -> Value {
    json!({
        "item_id": item.item_id,
        "item_name": item.item_name,
        "data_type": item.data_type,
        "category": item.category,
        "raw_hone_level": item.raw_hone_level,
        "advanced_honing_level": item.advanced_honing_level,
        "is_sidereal_weapon": item.is_sidereal_weapon,
        "item_definition_found": item.item_definition_found,
        "base_balance_level": item.base_balance_level,
        "hone_adjusted": item.hone_adjusted,
        "advanced_balance_level_delta": item.advanced_balance_level_delta,
        "balance_level": item.balance_level,
        "bonus_mult": debug_json_f64(item.bonus_mult),
        "level_option_id": item.level_option_id,
        "static_option_ids": item.static_option_ids,
        "applied_option_found": item.applied_option_found,
        "applied_option_kind": item.applied_option_kind,
        "applied_option_id": item.applied_option_id,
        "applied_weapon_power": item.applied_weapon_power,
        "applied_strength": item.applied_strength,
        "applied_dexterity": item.applied_dexterity,
        "applied_intelligence": item.applied_intelligence,
        "applied_vitality": item.applied_vitality,
        "applied_physical_defense": item.applied_physical_defense,
        "applied_magic_defense": item.applied_magic_defense,
        "ark_passive_line_count": item.ark_passive_line_count,
        "bracer_line_count": item.bracer_line_count,
        "quality_line_count": item.quality_line_count,
        "gem_line_count": item.gem_line_count,
        "issues": item.issues,
    })
}

fn inspect_snapshot_debug_value(snapshot: &InspectSnapshot) -> Value {
    json!({
        "gear_level": debug_json_f64(snapshot.gear_level as f64),
        "stat_pairs": snapshot.stat_pairs.iter().map(|(stat_type, value)| {
            json!({
                "stat_type": stat_type,
                "value": value,
            })
        }).collect::<Vec<_>>(),
        "derived_stats": {
            "stat_pairs": snapshot.derived_stats.stat_pairs.iter().map(|(stat_type, value)| {
                json!({
                    "stat_type": stat_type,
                    "value": value,
                })
            }).collect::<Vec<_>>(),
            "ally_attack_power_power": debug_json_f64(snapshot.derived_stats.ally_attack_power_power),
            "ally_identity_damage_power": debug_json_f64(snapshot.derived_stats.ally_identity_damage_power),
            "ally_brand_power": debug_json_f64(snapshot.derived_stats.ally_brand_power),
            "damage_conversion_type": snapshot.derived_stats.damage_conversion_type,
            "skill_attack_power_multiplier_by_skill": snapshot.derived_stats.skill_attack_power_multiplier_by_skill.iter().map(|(skill_id, value)| {
                json!({
                    "skill_id": skill_id,
                    "value": debug_json_f64(*value),
                })
            }).collect::<Vec<_>>(),
            "skill_status_effect_multiplier_by_skill": snapshot.derived_stats.skill_status_effect_multiplier_by_skill.iter().map(|(skill_id, value)| {
                json!({
                    "skill_id": skill_id,
                    "value": debug_json_f64(*value),
                })
            }).collect::<Vec<_>>(),
            "skill_group_status_effect_multiplier_by_group": snapshot.derived_stats.skill_group_status_effect_multiplier_by_group.iter().map(|(group_id, value)| {
                json!({
                    "group_id": group_id,
                    "value": debug_json_f64(*value),
                })
            }).collect::<Vec<_>>(),
            "ability_features": snapshot.derived_stats.ability_features.iter().map(|feature| {
                json!({
                    "feature_type": feature.feature_type,
                    "level": feature.level,
                    "values": feature.values,
                })
            }).collect::<Vec<_>>(),
            "item_build_debug": snapshot.derived_stats.item_build_debug.iter().map(inspect_item_build_debug_value).collect::<Vec<_>>(),
            "buff_id_ownership": snapshot.derived_stats.buff_id_ownership,
            "buff_unique_group_ownership": snapshot.derived_stats.buff_unique_group_ownership,
            "deferred_addons": snapshot.derived_stats.deferred_addons.iter().map(|addon| format!("{addon:?}")).collect::<Vec<_>>(),
        },
        "addon_values": snapshot.addon_values.iter().map(|value| {
            json!({
                "addon_type": value.addon_type,
                "value": value.value,
            })
        }).collect::<Vec<_>>(),
        "engravings": snapshot.engravings.iter().map(|engraving| {
            json!({
                "id": engraving.id,
                "unknown": engraving.unknown,
                "level": engraving.level,
            })
        }).collect::<Vec<_>>(),
        "equipped_items": snapshot.equipped_items.iter().map(|item| {
            json!({
                "unique_id": item.unique_id,
                "raw_item_id": item.raw_item_id,
                "raw_hone_level": item.raw_hone_level,
                "raw_slot_index": item.raw_slot_index,
                "data_type": item.data_type,
                "has_equippable_item_data": item.has_equippable_item_data,
                "has_ark_grid_gem_data": item.has_ark_grid_gem_data,
            })
        }).collect::<Vec<_>>(),
        "equipped_gems": snapshot.equipped_gems.iter().map(|item| {
            json!({
                "unique_id": item.unique_id,
                "raw_item_id": item.raw_item_id,
                "raw_hone_level": item.raw_hone_level,
                "raw_slot_index": item.raw_slot_index,
                "data_type": item.data_type,
                "has_equippable_item_data": item.has_equippable_item_data,
                "has_ark_grid_gem_data": item.has_ark_grid_gem_data,
            })
        }).collect::<Vec<_>>(),
        "cards": snapshot.cards.iter().map(|card| {
            json!({
                "id": card.id,
                "awakening_level": card.awakening_level,
            })
        }).collect::<Vec<_>>(),
        "stigma_layouts": snapshot.stigma_layouts.iter().map(|layout| {
            json!({
                "stigma_id": layout.stigma_id,
                "stigma_level": layout.stigma_level,
                "stigma_rank": layout.stigma_rank,
            })
        }).collect::<Vec<_>>(),
        "ark_grid_cores": snapshot.ark_grid_cores.iter().map(|core| {
            json!({
                "core_id": core.core_id,
                "base_id": core.base_id,
                "options": core.options.iter().map(|option| {
                    json!({
                        "willpower_rank": option.willpower_rank,
                        "item_id": option.item_id,
                        "enabled": option.enabled,
                        "order_rank": option.order_rank,
                        "slot_index": option.slot_index,
                        "values": option.values.iter().map(|value| {
                            json!({
                                "option_id": value.option_id,
                                "rank": value.rank,
                            })
                        }).collect::<Vec<_>>(),
                    })
                }).collect::<Vec<_>>(),
            })
        }).collect::<Vec<_>>(),
        "ark_passive_data": snapshot.ark_passive_data.as_ref().map(|data| format!("{data:?}")),
    })
}

fn skill_runtime_data_debug_value(runtime_data: Option<&SkillRuntimeData>) -> Value {
    match runtime_data {
        Some(runtime_data) => json!({
            "skill_level": runtime_data.skill_level,
            "skill_option_data": runtime_data.skill_option_data.as_ref().map(|data| json!({
                "layer_index": data.layer_index,
                "start_stage_index": data.start_stage_index,
                "transit_index": data.transit_index,
                "stage_start_time": data.stage_start_time,
                "farmost_dist": data.farmost_dist.map(|value| debug_json_f64(value as f64)),
                "tripod_index": data.tripod_index,
                "tripod_level": data.tripod_level,
            })),
            "last_cast_at_ms": runtime_data.last_cast_at_ms,
            "last_start_at_ms": runtime_data.last_start_at_ms,
            "identity_gauge1_at_start": runtime_data.identity_gauge1_at_start,
            "identity_gauge2_at_start": runtime_data.identity_gauge2_at_start,
            "identity_gauge3_at_start": runtime_data.identity_gauge3_at_start,
            "cached_critical_hit_damage_bonus": debug_json_f64(runtime_data.cached_critical_hit_damage_bonus),
            "cached_critical_rate_bonus": debug_json_f64(runtime_data.cached_critical_rate_bonus),
            "cached_attack_speed_bonus": debug_json_f64(runtime_data.cached_attack_speed_bonus),
            "cached_critical_hit_damage_bonus_per_skill_effect": runtime_data.cached_critical_hit_damage_bonus_per_skill_effect.iter().map(|(skill_effect_id, value)| {
                json!({
                    "skill_effect_id": skill_effect_id,
                    "value": debug_json_f64(*value),
                })
            }).collect::<Vec<_>>(),
            "cached_critical_rate_bonus_per_skill_effect": runtime_data.cached_critical_rate_bonus_per_skill_effect.iter().map(|(skill_effect_id, value)| {
                json!({
                    "skill_effect_id": skill_effect_id,
                    "value": debug_json_f64(*value),
                })
            }).collect::<Vec<_>>(),
        }),
        None => Value::Null,
    }
}

fn status_effect_debug_value(effect: &StatusEffectDetails) -> Value {
    json!({
        "instance_id": effect.instance_id,
        "status_effect_id": effect.status_effect_id,
        "custom_id": effect.custom_id,
        "target_id": effect.target_id,
        "source_id": effect.source_id,
        "source_skill_id": effect.source_skill_id,
        "target_type": format!("{:?}", effect.target_type),
        "db_target_type": effect.db_target_type,
        "skill_level": effect.skill_level,
        "buff_type_flags": effect.buff_type_flags,
        "value": effect.value,
        "stack_count": effect.stack_count,
        "category": format!("{:?}", effect.category),
        "buff_category": format!("{:?}", effect.buff_category),
        "show_type": format!("{:?}", effect.show_type),
        "status_effect_type": format!("{:?}", effect.status_effect_type),
        "expiration_delay": debug_json_f64(effect.expiration_delay as f64),
        "expire_at": effect.expire_at.map(|value| value.to_rfc3339()),
        "end_tick": effect.end_tick,
        "timestamp": effect.timestamp.to_rfc3339(),
        "name": effect.name,
        "unique_group": effect.unique_group,
        "has_owner_player_stats_snapshot": effect.owner_player_stats_snapshot.is_some(),
        "has_source_skill_runtime_snapshot": effect.source_skill_runtime_snapshot.is_some(),
    })
}

fn contribution_factor_debug_value(contribution: &ContributionFactor) -> Value {
    json!({
        "rdps_type": contribution.rdps_type,
        "source_entity_id": contribution.source_entity_id,
        "source_skill_id": contribution.source_skill_id,
        "factor": debug_json_f64(contribution.factor),
        "is_support": contribution.is_support,
    })
}

fn hit_rdps_result_debug_value(result: &HitRdpsResult) -> Value {
    json!({
        "rdps_damage_received": result.rdps_damage_received,
        "rdps_damage_received_support": result.rdps_damage_received_support,
        "entity_attributions": result.entity_attributions.iter().map(|attribution| {
            json!({
                "source_entity_id": attribution.source_entity_id,
                "damage": attribution.damage,
                "is_support": attribution.is_support,
            })
        }).collect::<Vec<_>>(),
        "attributions": result.attributions.iter().map(|attribution| {
            json!({
                "rdps_type": attribution.rdps_type,
                "source_entity_id": attribution.source_entity_id,
                "source_skill_id": attribution.source_skill_id,
                "damage": attribution.damage,
                "is_support": attribution.is_support,
            })
        }).collect::<Vec<_>>(),
        "debug_skill_group_attributions": result.debug_skill_group_attributions.iter().map(|attribution| {
            json!({
                "source_entity_id": attribution.source_entity_id,
                "group_name": attribution.group_name,
                "damage": attribution.damage,
                "damage_increase": attribution.damage_increase,
            })
        }).collect::<Vec<_>>(),
    })
}

fn debug_json_f64(value: f64) -> Value {
    if value.is_finite() {
        json!(value)
    } else {
        json!({
            "non_finite": format!("{value:?}"),
        })
    }
}

fn get_level_data(
    skill_buff: &crate::models::SkillBuffData,
    skill_level: u8,
) -> Option<&PerLevelData> {
    skill_buff
        .per_level_data
        .get(&skill_level.max(1).to_string())
        .or_else(|| skill_buff.per_level_data.get("1"))
}

fn get_level_data_resolved(
    skill_buff: &crate::models::SkillBuffData,
    skill_level: u8,
    runtime_data: Option<&crate::live::entity_tracker::SkillRuntimeData>,
    stack_count: u8,
) -> Option<PerLevelData> {
    let mut level_data = get_level_data(skill_buff, skill_level)?.clone();
    if let Some(runtime_data) = runtime_data {
        if let Some((values, relative)) =
            runtime_data.buff_param_changes.get(&(skill_buff.id as u32))
        {
            let params = level_data.status_effect_values.get_or_insert_with(Vec::new);
            if params.len() < values.len() {
                params.resize(values.len(), 0);
            }
            for (index, value) in values.iter().enumerate() {
                params[index] = if *relative {
                    (params[index] as f64 * (1.0 + (*value as f64 / 100.0))).round() as i32
                } else {
                    *value as i32
                };
            }
        }
        if let Some(changes) = runtime_data.buff_stat_changes.get(&(skill_buff.id as u32)) {
            for option in &mut level_data.passive_options {
                if let Some((value, relative)) = changes.get(&option.key_stat) {
                    option.value = if *relative {
                        (option.value as f64 * (1.0 + (*value as f64 / 100.0))).round() as i32
                    } else {
                        *value as i32
                    };
                }
            }
        }
        if let Some(added_stats) = runtime_data.buff_added_stats.get(&(skill_buff.id as u32)) {
            level_data
                .passive_options
                .extend(added_stats.iter().cloned());
        }
    }
    let stacks = i32::from(stack_count.max(1));
    if let Some(values) = level_data.status_effect_values.as_mut() {
        for value in values {
            *value *= stacks;
        }
    }
    for option in &mut level_data.passive_options {
        if matches!(
            option.option_type.as_str(),
            "stat" | "attack_power_amplify_multiplier"
        ) {
            option.value *= stacks;
        }
    }
    Some(level_data)
}

fn source_skill_id_from_effect(effect: &StatusEffectDetails) -> u32 {
    effect
        .source_skill_id
        .filter(|skill_id| *skill_id > 0)
        .unwrap_or((effect.status_effect_id / 10).max(1))
}

fn resolve_effect_source_id(
    effect: &StatusEffectDetails,
    skill_buff: &crate::models::SkillBuffData,
    entity_tracker: &EntityTracker,
    buffered_entities: Option<&HashMap<u64, Entity>>,
) -> u64 {
    let owns_effect = |entity: &Entity| {
        entity.inspect_snapshot.as_ref().is_some_and(|snapshot| {
            snapshot
                .derived_stats
                .buff_id_ownership
                .contains(&effect.status_effect_id)
                || (skill_buff.unique_group > 0
                    && snapshot
                        .derived_stats
                        .buff_unique_group_ownership
                        .contains(&skill_buff.unique_group))
        })
    };

    if effect.source_id != 0
        && let Some(source_entity) =
            get_buffered_or_live_entity(effect.source_id, buffered_entities, entity_tracker)
    {
        if matches!(
            source_entity.entity_type,
            crate::models::EntityType::Boss
                | crate::models::EntityType::Guardian
                | crate::models::EntityType::Monster
                | crate::models::EntityType::Npc
                | crate::models::EntityType::Esther
        ) {
            return effect.source_id;
        }
        if matches!(source_entity.entity_type, crate::models::EntityType::Player)
            && owns_effect(source_entity)
        {
            return effect.source_id;
        }
    }

    if let Some(buffered_entities) = buffered_entities
        && let Some(entity) = buffered_entities.values().find(|entity| {
            matches!(entity.entity_type, crate::models::EntityType::Player) && owns_effect(entity)
        })
    {
        return entity.id;
    }

    entity_tracker
        .entities
        .values()
        .find(|entity| {
            matches!(entity.entity_type, crate::models::EntityType::Player) && owns_effect(entity)
        })
        .map(|entity| entity.id)
        .unwrap_or(effect.source_id)
}

fn is_directional_skill(
    skill_id: u32,
    runtime_data: Option<&crate::live::entity_tracker::SkillRuntimeData>,
    mask: i32,
) -> bool {
    let base_mask = SKILL_DATA
        .get(&skill_id)
        .map(|skill| skill.directional_mask)
        .unwrap_or_default();
    let runtime_mask = runtime_data
        .and_then(|runtime| runtime.cached_directional_mask)
        .unwrap_or(base_mask);
    (runtime_mask & mask) != 0
}

fn is_directional_skill_any(
    skill_id: u32,
    skill_id_real: u32,
    runtime_data: Option<&crate::live::entity_tracker::SkillRuntimeData>,
    mask: i32,
) -> bool {
    is_directional_skill(skill_id, runtime_data, mask)
        || (skill_id_real != skill_id && is_directional_skill(skill_id_real, runtime_data, mask))
}

fn select_unique_group_effects(status_effects: &[StatusEffectDetails]) -> Vec<StatusEffectDetails> {
    let mut selected_by_group = std::collections::BTreeMap::new();
    let mut selected = Vec::new();

    for status_effect in status_effects {
        if status_effect.unique_group == 0 {
            selected.push(status_effect.clone());
            continue;
        }

        let entry = selected_by_group
            .entry(status_effect.unique_group)
            .or_insert_with(|| status_effect.clone());
        if status_effect.status_effect_id < entry.status_effect_id {
            *entry = status_effect.clone();
        }
    }

    selected.extend(selected_by_group.into_values());
    selected
}

fn select_target_effects(
    attacker: &Entity,
    status_effects: &[StatusEffectDetails],
    entity_tracker: &EntityTracker,
    buffered_entities: Option<&HashMap<u64, Entity>>,
) -> Vec<StatusEffectDetails> {
    let mut selected_by_group = std::collections::BTreeMap::new();
    let mut selected = Vec::new();

    for status_effect in status_effects {
        let Some(skill_buff) = SKILL_BUFF_DATA.get(&status_effect.status_effect_id) else {
            continue;
        };
        if status_effect.unique_group == 0 {
            selected.push(status_effect.clone());
            continue;
        }

        let source_entity_id =
            resolve_effect_source_id(status_effect, skill_buff, entity_tracker, buffered_entities);
        let should_group = !is_party_wide_skill_buff(skill_buff)
            || is_same_party_target_effect_source(
                attacker,
                source_entity_id,
                entity_tracker,
                buffered_entities,
            );
        if !should_group {
            selected.push(status_effect.clone());
            continue;
        }

        let entry = selected_by_group
            .entry(status_effect.unique_group)
            .or_insert_with(|| status_effect.clone());
        if status_effect.status_effect_id < entry.status_effect_id {
            *entry = status_effect.clone();
        }
    }

    selected.extend(selected_by_group.into_values());
    selected
}

fn is_support_source(
    source_entity_id: u64,
    skill_buff: &crate::models::SkillBuffData,
    entity_tracker: &EntityTracker,
    buffered_entities: Option<&HashMap<u64, Entity>>,
) -> bool {
    skill_buff
        .buff_category
        .as_deref()
        .is_some_and(|category| category == "supportbuff")
        || get_buffered_or_live_entity(source_entity_id, buffered_entities, entity_tracker)
            .is_some_and(|entity| is_support_class(&entity.class_id))
}

fn is_identity_skill_buff(skill_buff: &crate::models::SkillBuffData) -> bool {
    SUPPORT_IDENTITY_GROUP.contains(&skill_buff.unique_group)
        || RDPS_ADDITIONAL_IDENTITY_GROUP.contains(&skill_buff.unique_group)
        || skill_buff
            .source_skills
            .as_ref()
            .and_then(|skills| skills.first())
            .is_some_and(|skill_id| SUPPORT_IDENTITY_SOURCE_SKILLS.contains(skill_id))
}

fn source_skill_has_identity_group(source_skill_id: u32) -> bool {
    SKILL_DATA
        .get(&source_skill_id)
        .and_then(|skill| skill.groups.as_ref())
        .is_some_and(|groups| {
            groups
                .iter()
                .any(|group_id| SUPPORT_IDENTITY_SKILL_GROUPS.contains(group_id))
        })
}

fn get_identity_buff_multiplier(
    player_stats: &PlayerStats,
    source_class_id: u32,
    source_skill_id: u32,
) -> f64 {
    let spec_bonus = match source_class_id {
        105 | 602 => player_stats.spec_bonus_identity_1.value(),
        204 | 113 => player_stats.spec_bonus_identity_2.value(),
        _ => 0.0,
    };
    (1.0 + spec_bonus.max(0.0))
        * (1.0
            + get_ally_identity_damage_power(player_stats)
            + get_skill_status_effect_multiplier(player_stats, source_skill_id))
}

fn get_ally_identity_damage_power(player_stats: &PlayerStats) -> f64 {
    player_stats.ally_identity_damage_power.value()
}

fn load_player_stats_from_snapshot(
    snapshot: &InspectSnapshot,
    owner_id: u64,
    class_id: u32,
) -> PlayerStats {
    let mut player_stats = PlayerStats::default();
    player_stats.load_from_snapshot(snapshot, owner_id, class_id);
    if DEBUG_DUMP_DAMAGE_STATE_JSON {
        write_debug_json_dump(
            "inspect-item-build",
            &format!("owner-{}-class-{}", owner_id, class_id),
            &json!({
                "owner_id": owner_id,
                "class_id": class_id,
                "item_build_debug": snapshot.derived_stats.item_build_debug.iter().map(inspect_item_build_debug_value).collect::<Vec<_>>(),
            }),
        );
        write_debug_json_dump(
            "player-stats-calc",
            &format!("owner-{}-class-{}", owner_id, class_id),
            &json!({
                "owner_id": owner_id,
                "class_id": class_id,
                "snapshot": inspect_snapshot_debug_value(snapshot),
                "player_stats": player_stats.debug_dump_value(),
            }),
        );
    }
    player_stats
}
