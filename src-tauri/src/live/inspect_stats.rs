use crate::data::{
    EXTERNAL_ABILITY_DATA, EXTERNAL_ARK_GRID_DATA, EXTERNAL_ARK_GRID_GEM_LEVELS_BY_OPTION_ID,
    EXTERNAL_ARK_PASSIVE_DATA, EXTERNAL_ARK_PASSIVE_KARMA_DATA, EXTERNAL_CARD_BOOK_DATA,
    EXTERNAL_ITEM_AMPLIFICATION_BASE_DATA, EXTERNAL_ITEM_DATA,
    EXTERNAL_ITEM_GRADE_STATIC_OPTION_DATA, EXTERNAL_ITEM_LEVEL_OPTION_DATA, SKILL_DATA,
    STAT_TYPE_MAP,
};
use crate::live::addon_type::AddonType;
use crate::live::stat_type::StatType;
use crate::models::ExternalResourceAddon;

use hashbrown::HashMap;
use meter_defs::defs::{ItemData, PKTPCInspectResult};

const DEFAULT_ARK_PASSIVE_KARMA_EVOLUTION_DAMAGE: f64 = 0.06;
const DEFAULT_ARK_PASSIVE_KARMA_BRAND_POWER: f64 = 0.06;
const DEFAULT_ARK_PASSIVE_KARMA_WEAPON_DAMAGE_MULTIPLIER: i64 = 250;
const DEFAULT_ARK_PASSIVE_KARMA_HYPER_DAMAGE: i64 = 1050;
const FLASH_ORB_UNIQUE_GROUP_ID: u32 = 523501;
const WIND_ORB_UNIQUE_GROUP_ID: u32 = 523201;
const STRENGTH_ORB_UNIQUE_GROUP_ID: u32 = 523401;
const DEFENSE_ORB_UNIQUE_GROUP_ID: u32 = 523301;
const SIDEREAL_WEAPON_HONE_MIN: u16 = 100;

#[derive(Debug, Clone, Copy)]
struct ParsedItemAddon {
    addon_type: u8,
    stat_type: u32,
    original_stat: u32,
    value: i64,
}

#[derive(Debug, Default, Clone)]
pub struct DerivedAbilityFeature {
    pub feature_type: String,
    pub level: u32,
    pub values: Vec<i64>,
}

#[derive(Debug, Default, Clone)]
pub struct InspectDerivedStats {
    pub stat_pairs: HashMap<u8, i64>,
    pub ally_attack_power_power: f64,
    pub ally_identity_damage_power: f64,
    pub ally_brand_power: f64,
    pub damage_conversion_type: Option<u8>,
    pub skill_attack_power_multiplier_by_skill: HashMap<u32, f64>,
    pub skill_status_effect_multiplier_by_skill: HashMap<u32, f64>,
    pub skill_group_status_effect_multiplier_by_group: HashMap<u32, f64>,
    pub ability_features: Vec<DerivedAbilityFeature>,
    pub buff_id_ownership: Vec<u32>,
    pub buff_unique_group_ownership: Vec<u32>,
    pub deferred_addons: Vec<ExternalResourceAddon>,
    pub item_build_debug: Vec<InspectItemBuildDebug>,
}

#[derive(Debug, Default, Clone)]
pub struct InspectItemBuildDebug {
    pub item_id: u32,
    pub item_name: String,
    pub data_type: u8,
    pub category: u32,
    pub raw_hone_level: u16,
    pub advanced_honing_level: u8,
    pub is_sidereal_weapon: bool,
    pub item_definition_found: bool,
    pub base_balance_level: u32,
    pub hone_adjusted: u32,
    pub advanced_balance_level_delta: u32,
    pub balance_level: u32,
    pub bonus_mult: f64,
    pub level_option_id: u32,
    pub static_option_ids: Vec<u32>,
    pub applied_option_found: bool,
    pub applied_option_kind: Option<String>,
    pub applied_option_id: Option<u32>,
    pub applied_weapon_power: i64,
    pub applied_strength: i64,
    pub applied_dexterity: i64,
    pub applied_intelligence: i64,
    pub applied_vitality: i64,
    pub applied_physical_defense: i64,
    pub applied_magic_defense: i64,
    pub ark_passive_line_count: usize,
    pub bracer_line_count: usize,
    pub quality_line_count: usize,
    pub gem_line_count: usize,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default)]
struct ResolvedItemBalanceLevel {
    base_balance_level: u32,
    hone_adjusted: u32,
    advanced_balance_level_delta: u32,
    balance_level: u32,
}

pub fn derive_inspect_stats(result: &PKTPCInspectResult) -> InspectDerivedStats {
    let raw_stat_pairs = result
        .stat_pairs
        .iter()
        .map(|stat| (stat.stat_type, stat.value))
        .collect::<HashMap<_, _>>();
    let mut derived = InspectDerivedStats::default();

    for item in &result.equipped_items {
        apply_item(item, &raw_stat_pairs, &mut derived);
    }
    for item in &result.equipped_gems {
        apply_item(item, &raw_stat_pairs, &mut derived);
    }

    apply_ark_passives(result, &raw_stat_pairs, &mut derived);
    apply_ark_passive_karma(result, &raw_stat_pairs, &mut derived);
    apply_cards(result, &raw_stat_pairs, &mut derived);
    apply_ark_grid(result, &raw_stat_pairs, &mut derived);
    apply_engravings(result, &mut derived);

    derived
}

fn apply_engravings(result: &PKTPCInspectResult, derived: &mut InspectDerivedStats) {
    for engraving in &result.engraving_datas {
        let Some(ability) = EXTERNAL_ABILITY_DATA.get(&engraving.id) else {
            continue;
        };
        let level = u32::from(engraving.level);
        let Some(level_data) = ability.levels.get(&level) else {
            continue;
        };
        match normalize_feature_type(&ability.feature_type) {
            "matt_critical" => {
                if let Some(value) = level_data.values.get(2) {
                    add_flat_stat("critical_dam_rate", *value, &HashMap::new(), derived);
                }
            }
            "dagger_critical" => {
                if let Some(value) = level_data.values.first() {
                    add_flat_stat("critical_hit_rate", *value, &HashMap::new(), derived);
                }
            }
            "spirit_absorption" => {
                if let Some(value) = level_data.values.first() {
                    add_flat_stat("move_speed_rate", *value, &HashMap::new(), derived);
                    add_flat_stat("attack_speed_rate", *value, &HashMap::new(), derived);
                }
            }
            "gravity_glove" => {
                if let Some(value) = level_data.values.first() {
                    add_flat_stat("attack_speed_rate", -*value, &HashMap::new(), derived);
                }
            }
            "troop_leader" => {
                add_ability_feature(&ability.feature_type, level, &level_data.values, derived);
            }
            "ether_boy" => {
                push_unique_u32(
                    &mut derived.buff_unique_group_ownership,
                    FLASH_ORB_UNIQUE_GROUP_ID,
                );
                push_unique_u32(
                    &mut derived.buff_unique_group_ownership,
                    WIND_ORB_UNIQUE_GROUP_ID,
                );
                push_unique_u32(
                    &mut derived.buff_unique_group_ownership,
                    STRENGTH_ORB_UNIQUE_GROUP_ID,
                );
                push_unique_u32(
                    &mut derived.buff_unique_group_ownership,
                    DEFENSE_ORB_UNIQUE_GROUP_ID,
                );
            }
            _ => {}
        }
    }
}

fn apply_item(
    item: &ItemData,
    raw_stat_pairs: &HashMap<u8, i64>,
    derived: &mut InspectDerivedStats,
) {
    let Some(item_id) = item.u32_1 else {
        return;
    };
    let Some(item_data) = item.item_data_typed.as_ref() else {
        return;
    };

    let mut item_debug = InspectItemBuildDebug {
        item_id,
        data_type: item_data.b_0,
        raw_hone_level: item.u16_0.unwrap_or_default(),
        level_option_id: 0,
        ..Default::default()
    };

    let Some(item_definition) = EXTERNAL_ITEM_DATA.get(&item_id) else {
        item_debug
            .issues
            .push("missing_item_definition".to_string());
        derived.item_build_debug.push(item_debug);
        return;
    };

    item_debug.item_definition_found = true;
    item_debug.item_name = item_definition.name.clone();
    item_debug.category = item_definition.category;
    let hone_level = item.u16_0.unwrap_or_default();
    let advanced_honing_level = item_data
        .equippable_item_data
        .as_ref()
        .and_then(|equippable| {
            equippable
                .sub_p_k_t_inventory_item_list_result_5_5_15
                .as_ref()
                .map(|advanced| advanced.b_0.max(advanced.b_1))
        })
        .unwrap_or_default();
    let resolved_balance_level =
        resolve_item_balance_level(item_definition, hone_level, advanced_honing_level);
    item_debug.raw_hone_level = hone_level;
    item_debug.advanced_honing_level = advanced_honing_level;
    item_debug.is_sidereal_weapon = item_definition.grade.eq_ignore_ascii_case("esther");
    item_debug.base_balance_level = resolved_balance_level.base_balance_level;
    item_debug.hone_adjusted = resolved_balance_level.hone_adjusted;
    item_debug.advanced_balance_level_delta = resolved_balance_level.advanced_balance_level_delta;
    item_debug.balance_level = resolved_balance_level.balance_level;
    item_debug.level_option_id = item_definition.level_option_id;
    item_debug.static_option_ids = item_definition.static_option_ids.clone();
    let should_parse_extended_item_data =
        item_definition.tier > 3 || item_definition.grade.eq_ignore_ascii_case("esther");

    if should_parse_extended_item_data {
        if let Some(equippable) = item_data.equippable_item_data.as_ref() {
            let ark_passive_data = &equippable.sub_p_k_t_inventory_item_list_result_5_3_32;
            if ark_passive_data.b_0 == 1 {
                item_debug.ark_passive_line_count = ark_passive_data
                    .bytearraylist_0
                    .as_deref()
                    .map_or(0, |entries| entries.len());
                for chunk in split_fixed_chunks(
                    ark_passive_data
                        .bytearraylist_0
                        .as_deref()
                        .unwrap_or_default(),
                    30,
                ) {
                    apply_ark_passive_addon(chunk, raw_stat_pairs, derived);
                }
            }

            let bracer_data = &equippable.sub_p_k_t_inventory_item_list_result_5_2_74;
            if bracer_data.b_0 != 0
                && let Some(bracer) = bracer_data
                    .sub_p_k_t_inventory_item_list_result_6_6_73
                    .as_ref()
            {
                item_debug.bracer_line_count =
                    bracer.bytearraylist_0.len() + bracer.bytearraylist_1.len();
                for chunk in split_fixed_chunks(bracer.bytearraylist_0.as_slice(), 30) {
                    apply_bracer_addon(chunk, raw_stat_pairs, derived);
                }
                for chunk in split_fixed_chunks(bracer.bytearraylist_1.as_slice(), 30) {
                    apply_bracer_addon(chunk, raw_stat_pairs, derived);
                }
            }

            for chunk in split_fixed_chunks(equippable.quality_lines.as_slice(), 29) {
                item_debug.quality_line_count += 1;
                apply_quality_addon(chunk, raw_stat_pairs, derived);
            }
        }

        if item_data.b_0 == 7
            && let Some(gem_bytes) = item_data.bytearraylist_3.as_deref()
        {
            item_debug.gem_line_count = split_fixed_chunks(gem_bytes, 9).count();
            for chunk in split_fixed_chunks(gem_bytes, 9) {
                if !apply_gem_addon(chunk, raw_stat_pairs, derived) {
                    item_debug.issues.push("unresolved_gem_layout".to_string());
                }
            }
        }
    } else if item_data.equippable_item_data.is_some() || item_data.b_0 == 7 {
        item_debug
            .issues
            .push("skipped_non_lal_extended_item_parse".to_string());
    }

    let mut bonus_mult = 1.0;
    if advanced_honing_level > 0 && item_definition.item_amplification_base_id != 0 {
        if let Some(level) = EXTERNAL_ITEM_AMPLIFICATION_BASE_DATA
            .get(&item_definition.item_amplification_base_id)
            .and_then(|base| base.levels.get(&(advanced_honing_level as u32)))
        {
            bonus_mult += level.stage_bonus_stat_rate as f64 / 10000.0;
        }
    }
    item_debug.bonus_mult = bonus_mult;

    if item_definition.level_option_id != 0
        && apply_item_level_option(
            item_definition.level_option_id,
            resolved_balance_level.balance_level,
            bonus_mult,
            raw_stat_pairs,
            derived,
        )
    {
        if let Some(level_data) = get_item_level_option(
            item_definition.level_option_id,
            resolved_balance_level.balance_level,
        ) {
            fill_item_option_debug(
                &mut item_debug,
                "level_option",
                item_definition.level_option_id,
                level_data,
                bonus_mult,
            );
        } else {
            item_debug.applied_option_kind = Some("level_option".to_string());
            item_debug.applied_option_id = Some(item_definition.level_option_id);
            item_debug
                .issues
                .push("level_option_missing_balance_level".to_string());
        }
        derived.item_build_debug.push(item_debug);
        return;
    }

    for option_id in &item_definition.static_option_ids {
        if apply_item_level_option(
            *option_id,
            resolved_balance_level.balance_level,
            bonus_mult,
            raw_stat_pairs,
            derived,
        ) {
            if let Some(level_data) =
                get_item_level_option(*option_id, resolved_balance_level.balance_level)
            {
                fill_item_option_debug(
                    &mut item_debug,
                    "static_level_option",
                    *option_id,
                    level_data,
                    bonus_mult,
                );
            } else {
                item_debug.applied_option_kind = Some("static_level_option".to_string());
                item_debug.applied_option_id = Some(*option_id);
                item_debug.issues.push(format!(
                    "static_level_option_missing_balance_level:{option_id}"
                ));
            }
            continue;
        }

        if let Some(option_data) = EXTERNAL_ITEM_GRADE_STATIC_OPTION_DATA.get(option_id) {
            apply_item_option_level(option_data, bonus_mult, raw_stat_pairs, derived);
            for addon in &option_data.addons {
                apply_external_addon(addon, raw_stat_pairs, derived);
            }
            fill_item_option_debug(
                &mut item_debug,
                "static_option",
                *option_id,
                option_data,
                bonus_mult,
            );
        }
    }

    if (item_definition.level_option_id != 0 || !item_definition.static_option_ids.is_empty())
        && !item_debug.applied_option_found
    {
        item_debug.issues.push("no_matching_option_row".to_string());
    }

    derived.item_build_debug.push(item_debug);
}

fn apply_ark_passives(
    result: &PKTPCInspectResult,
    raw_stat_pairs: &HashMap<u8, i64>,
    derived: &mut InspectDerivedStats,
) {
    let mut goddess_blessing_level = 0u32;
    let mut goddess_blessing_values: Option<Vec<i64>> = None;
    for section in &result.ark_passive_tree_data_inspect.ark_passive_node_datas {
        for node in section {
            let Some(passive) = EXTERNAL_ARK_PASSIVE_DATA.get(&node.ark_passive_id) else {
                continue;
            };
            let level = node.points.unwrap_or(1);
            if let Some(level_data) = passive.levels.get(&level) {
                for addon in &level_data.addons {
                    if addon.addon_type == "ability_feature"
                        && let Some(ability) = EXTERNAL_ABILITY_DATA.get(&addon.key_index)
                        && let Some(ability_level) = ability.levels.get(&(addon.key_value as u32))
                    {
                        add_ability_feature(
                            &ability.feature_type,
                            addon.key_value as u32,
                            &ability_level.values,
                            derived,
                        );
                        if ability
                            .feature_type
                            .eq_ignore_ascii_case("goddess_blessing")
                            && (addon.key_value as u32) > goddess_blessing_level
                        {
                            goddess_blessing_level = addon.key_value as u32;
                            goddess_blessing_values = Some(ability_level.values.clone());
                        }
                    }
                    apply_external_addon(addon, raw_stat_pairs, derived);
                }
            }
        }
    }
    if let Some(values) = goddess_blessing_values {
        for buff_id in values.into_iter().take(2).filter(|value| *value > 0) {
            push_unique_u32(&mut derived.buff_id_ownership, buff_id as u32);
        }
    }
}

fn apply_ark_passive_karma(
    result: &PKTPCInspectResult,
    raw_stat_pairs: &HashMap<u8, i64>,
    derived: &mut InspectDerivedStats,
) {
    if result.stigma_layout_datas.is_empty() {
        add_multiplier_stat(
            "evolution_dam_rate",
            DEFAULT_ARK_PASSIVE_KARMA_EVOLUTION_DAMAGE,
            raw_stat_pairs,
            derived,
        );
        derived.ally_brand_power += DEFAULT_ARK_PASSIVE_KARMA_BRAND_POWER;
        add_flat_stat(
            "weapon_dam_x",
            DEFAULT_ARK_PASSIVE_KARMA_WEAPON_DAMAGE_MULTIPLIER,
            raw_stat_pairs,
            derived,
        );
        add_flat_stat(
            "ultimate_awakening_dam_rate",
            DEFAULT_ARK_PASSIVE_KARMA_HYPER_DAMAGE,
            raw_stat_pairs,
            derived,
        );
        return;
    }

    for stigma in &result.stigma_layout_datas {
        let Some(karma) = EXTERNAL_ARK_PASSIVE_KARMA_DATA.get(&stigma.stigma_id) else {
            continue;
        };
        if let Some(rank) = karma.ranks.get(&stigma.stigma_rank) {
            for addon in &rank.addons {
                apply_external_addon(addon, raw_stat_pairs, derived);
            }
        }
        if let Some(level) = karma.levels.get(&stigma.stigma_level) {
            for addon in &level.addons {
                apply_external_addon(addon, raw_stat_pairs, derived);
            }
        }
    }
}

fn apply_cards(
    result: &PKTPCInspectResult,
    raw_stat_pairs: &HashMap<u8, i64>,
    derived: &mut InspectDerivedStats,
) {
    for card_set in EXTERNAL_CARD_BOOK_DATA.values() {
        let mut card_count = 0u32;
        let mut awakening_sum = 0u32;

        for set_card in &card_set.card_ids {
            for card in &result.card_datas {
                if card.id == *set_card {
                    card_count += 1;
                    awakening_sum += card.awakening_level;
                }
            }
        }

        for level in &card_set.levels {
            if awakening_sum >= level.required_awakening_level_sum
                && card_count >= level.required_card_count
            {
                for addon in &level.addons {
                    apply_external_addon(addon, raw_stat_pairs, derived);
                }
                if let Some(damage_attr) = damage_attr_from_name(&level.damage_attr) {
                    derived.damage_conversion_type = Some(damage_attr);
                }
            }
        }
    }
}

fn apply_ark_grid(
    result: &PKTPCInspectResult,
    raw_stat_pairs: &HashMap<u8, i64>,
    derived: &mut InspectDerivedStats,
) {
    let mut pending_ability_points: HashMap<u32, i64> = HashMap::new();
    let entry_count = usize::min(
        usize::from(result.ark_grid_cores.num),
        usize::min(
            result.ark_grid_cores.core_entries.len(),
            result.ark_grid_cores.core_ids.len(),
        ),
    );

    for index in 0..entry_count {
        let Some(core) = EXTERNAL_ARK_GRID_DATA
            .cores
            .get(&result.ark_grid_cores.core_ids[index])
        else {
            continue;
        };
        let gem_entries = &result.ark_grid_cores.core_entries[index];
        let aggregate_rank = gem_entries
            .iter()
            .filter(|entry| entry.enabled == 1)
            .map(|entry| entry.order_rank)
            .sum::<u32>();

        for option_slot in &core.options {
            if aggregate_rank <= option_slot.required_points {
                continue;
            }
            let Some(core_option) = EXTERNAL_ARK_GRID_DATA
                .core_options
                .get(&option_slot.option_id)
            else {
                continue;
            };
            for addon in &core_option.addons {
                if addon.addon_type == "ability_point" {
                    *pending_ability_points.entry(addon.key_index).or_default() += addon.key_value;
                } else {
                    apply_external_addon(addon, raw_stat_pairs, derived);
                }
            }
        }

        for gem_entry in gem_entries.iter().filter(|entry| entry.enabled == 1) {
            for gem_value in &gem_entry.values {
                let Some(levels) =
                    EXTERNAL_ARK_GRID_GEM_LEVELS_BY_OPTION_ID.get(&gem_value.option_id)
                else {
                    continue;
                };
                if let Some(level) = levels.iter().find(|level| level.level == gem_value.rank) {
                    let addon = ExternalResourceAddon {
                        addon_type: level.addon_type.clone(),
                        stat_type: level.stat_type.clone(),
                        key_index: level.addon_index,
                        key_value: if level.addon_value_override != 0 {
                            level.addon_value_override
                        } else {
                            level.addon_value
                        },
                    };
                    apply_external_addon(&addon, raw_stat_pairs, derived);
                }
            }
        }
    }

    for (ability_id, total_points) in pending_ability_points {
        if total_points <= 0 || total_points % 5 != 0 {
            continue;
        }
        apply_ability_level_addons(
            ability_id,
            (total_points / 5) as u32,
            raw_stat_pairs,
            derived,
        );
    }
}

fn apply_external_addon(
    addon: &ExternalResourceAddon,
    raw_stat_pairs: &HashMap<u8, i64>,
    derived: &mut InspectDerivedStats,
) {
    match addon.addon_type.as_str() {
        "stat" => apply_named_stat(
            addon.stat_type.as_str(),
            addon.key_value,
            raw_stat_pairs,
            derived,
        ),
        "ability_point" => {
            let ability_id = addon
                .stat_type
                .parse::<u32>()
                .ok()
                .filter(|ability_id| *ability_id != 0)
                .unwrap_or(addon.key_index);
            apply_ability_level_addons(ability_id, 1, raw_stat_pairs, derived)
        }
        "attack_power_amplify_multiplier" => {
            derived.ally_attack_power_power += addon.key_value as f64 / 10000.0;
        }
        "skill_group_status_effect_stat_multiplier" => {
            *derived
                .skill_group_status_effect_multiplier_by_group
                .entry(addon.key_index)
                .or_default() += addon.key_value as f64 / 10000.0;
        }
        "skill_attack_power_amplify_multiplier" => {
            *derived
                .skill_attack_power_multiplier_by_skill
                .entry(addon.key_index)
                .or_default() += addon.key_value as f64 / 10000.0;
        }
        "skill_status_effect_stat_multiplier" => {
            *derived
                .skill_status_effect_multiplier_by_skill
                .entry(addon.key_index)
                .or_default() += addon.key_value as f64 / 10000.0;
        }
        "ability_feature" => {
            if let Some(ability) = EXTERNAL_ABILITY_DATA.get(&addon.key_index)
                && let Some(level) = ability.levels.get(&(addon.key_value as u32))
            {
                add_ability_feature(
                    &ability.feature_type,
                    addon.key_value as u32,
                    &level.values,
                    derived,
                );
            } else {
                derived.deferred_addons.push(addon.clone());
            }
        }
        "ark_passive_point"
        | "party_without_self_heal"
        | "party_without_self_shield"
        | "skill_group_cooldown_reduction"
        | "identity_gauge"
        | "mana_reduction" => {}
        _ => derived.deferred_addons.push(addon.clone()),
    }
}

fn add_ability_feature(
    feature_type: &str,
    level: u32,
    values: &[i64],
    derived: &mut InspectDerivedStats,
) {
    let normalized = normalize_feature_type(feature_type);
    if let Some(existing) = derived
        .ability_features
        .iter_mut()
        .find(|feature| normalize_feature_type(&feature.feature_type) == normalized)
    {
        if level > existing.level {
            existing.feature_type = feature_type.to_string();
            existing.level = level;
            existing.values = values.to_vec();
        }
        return;
    }
    derived.ability_features.push(DerivedAbilityFeature {
        feature_type: feature_type.to_string(),
        level,
        values: values.to_vec(),
    });
}

fn normalize_feature_type(feature_type: &str) -> &str {
    feature_type.strip_prefix("ap_").unwrap_or(feature_type)
}

fn push_unique_u32(values: &mut Vec<u32>, value: u32) {
    if value != 0 && !values.contains(&value) {
        values.push(value);
    }
}

fn apply_numeric_addon(
    addon_type: u8,
    stat_or_key: u32,
    value: i64,
    raw_stat_pairs: &HashMap<u8, i64>,
    derived: &mut InspectDerivedStats,
) {
    match AddonType::from_raw(addon_type) {
        Some(AddonType::STAT) => {
            let _ = raw_stat_pairs;
            if let Ok(stat_id) = u8::try_from(stat_or_key) {
                *derived.stat_pairs.entry(stat_id).or_default() += value;
            }
        }
        Some(AddonType::ABILITY_POINT) => {
            apply_ability_level_addons(stat_or_key, 1, raw_stat_pairs, derived)
        }
        Some(AddonType::ATTACK_POWER_AMPLIFY_MULTIPLIER) => {
            derived.ally_attack_power_power += value as f64 / 10000.0;
        }
        Some(AddonType::ARK_PASSIVE_POINT) => {}
        Some(AddonType::SKILL_STATUS_EFFECT_STAT_MULTIPLIER) => {
            *derived
                .skill_status_effect_multiplier_by_skill
                .entry(stat_or_key)
                .or_default() += value as f64 / 10000.0;
        }
        Some(AddonType::SKILL_GROUP_STATUS_EFFECT_STAT_MULTIPLIER) => {
            *derived
                .skill_group_status_effect_multiplier_by_group
                .entry(stat_or_key)
                .or_default() += value as f64 / 10000.0;
        }
        Some(AddonType::SKILL_ATTACK_POWER_AMPLIFY_MULTIPLIER) => {
            *derived
                .skill_attack_power_multiplier_by_skill
                .entry(stat_or_key)
                .or_default() += value as f64 / 10000.0;
        }
        _ => {}
    }
}

fn apply_parsed_item_addon(
    addon: ParsedItemAddon,
    raw_stat_pairs: &HashMap<u8, i64>,
    derived: &mut InspectDerivedStats,
) {
    match AddonType::from_raw(addon.addon_type) {
        Some(AddonType::STAT) => {
            let _ = raw_stat_pairs;
            let stat_id = u8::try_from(addon.stat_type)
                .unwrap_or_else(|_| panic!("invalid item stat id: {}", addon.stat_type));
            assert!(
                StatType::from_raw(u32::from(stat_id)).is_some(),
                "undefined item stat id: {}",
                addon.stat_type
            );
            *derived.stat_pairs.entry(stat_id).or_default() += addon.value;
        }
        Some(AddonType::ABILITY_POINT) => derived.deferred_addons.push(ExternalResourceAddon {
            addon_type: "ability_point".to_string(),
            stat_type: addon.stat_type.to_string(),
            key_index: addon.original_stat,
            key_value: addon.value,
        }),
        Some(AddonType::COMBAT_EFFECT) => derived.deferred_addons.push(ExternalResourceAddon {
            addon_type: "combat_effect".to_string(),
            stat_type: addon.stat_type.to_string(),
            key_index: addon.original_stat,
            key_value: addon.value,
        }),
        Some(AddonType::CLASS_OPTION) => derived.deferred_addons.push(ExternalResourceAddon {
            addon_type: "class_option".to_string(),
            stat_type: addon.stat_type.to_string(),
            key_index: addon.original_stat,
            key_value: addon.value,
        }),
        Some(AddonType::ATTACK_POWER_AMPLIFY_MULTIPLIER) => {
            derived.ally_attack_power_power += addon.value as f64 / 10000.0;
        }
        Some(AddonType::ARK_PASSIVE_POINT) => {}
        Some(AddonType::SKILL_STATUS_EFFECT_STAT_MULTIPLIER) => {
            *derived
                .skill_status_effect_multiplier_by_skill
                .entry(addon.original_stat)
                .or_default() += addon.value as f64 / 10000.0;
        }
        Some(AddonType::SKILL_GROUP_STATUS_EFFECT_STAT_MULTIPLIER) => {
            *derived
                .skill_group_status_effect_multiplier_by_group
                .entry(addon.original_stat)
                .or_default() += addon.value as f64 / 10000.0;
        }
        Some(AddonType::SKILL_ATTACK_POWER_AMPLIFY_MULTIPLIER) => {
            *derived
                .skill_attack_power_multiplier_by_skill
                .entry(addon.original_stat)
                .or_default() += addon.value as f64 / 10000.0;
        }
        Some(AddonType::PARTY_WITHOUT_SELF_HEAL | AddonType::PARTY_WITHOUT_SELF_SHIELD) => {}
        _ => panic!(
            "unhandled parsed item addon type: {} / stat {} / original {} / value {}",
            addon.addon_type, addon.stat_type, addon.original_stat, addon.value
        ),
    }
}

fn apply_ark_passive_addon(
    bytes: &[u8],
    raw_stat_pairs: &HashMap<u8, i64>,
    derived: &mut InspectDerivedStats,
) {
    let addon = parse_ark_passive_addon(bytes);
    apply_parsed_item_addon(addon, raw_stat_pairs, derived);
}

fn apply_bracer_addon(
    bytes: &[u8],
    raw_stat_pairs: &HashMap<u8, i64>,
    derived: &mut InspectDerivedStats,
) {
    let addon = parse_bracer_addon(bytes);
    apply_parsed_item_addon(addon, raw_stat_pairs, derived);
}

fn apply_quality_addon(
    bytes: &[u8],
    raw_stat_pairs: &HashMap<u8, i64>,
    derived: &mut InspectDerivedStats,
) {
    let addon = parse_quality_addon(bytes);
    apply_parsed_item_addon(addon, raw_stat_pairs, derived);
}

fn parse_ark_passive_addon(bytes: &[u8]) -> ParsedItemAddon {
    assert!(
        bytes.len() >= 30,
        "invalid ark passive addon byte length: {}",
        bytes.len()
    );
    let addon_type = bytes[0];
    let _max_value = read_i32(bytes, 5) as i64;
    let mut value = read_i32(bytes, 9) as i64;
    let original_stat = read_u32(bytes, 13);
    let _item_grade_option_id = read_u32(bytes, 17);
    let _min_value = read_i32(bytes, 25) as i64;
    let mut stat_type = original_stat;

    match AddonType::from_raw(addon_type) {
        Some(AddonType::STAT) => {}
        Some(AddonType::ARK_PASSIVE_POINT) => {}
        Some(AddonType::COMBAT_EFFECT) => {
            value = match original_stat {
                x if x == StatType::OUTGOING_DAMAGE_0 as u32 => 55,
                x if x == StatType::OUTGOING_DAMAGE_1 as u32 => 120,
                x if x == StatType::OUTGOING_DAMAGE_2 as u32 => 200,
                _ => panic!("unhandled ark passive combat effect stat: {original_stat} [{value}]"),
            };
            stat_type = StatType::OUTGOING_DAMAGE as u32;
        }
        Some(AddonType::CLASS_OPTION) => {
            value = match original_stat {
                x if x == StatType::SUPPORT_GAUGE_0 as u32 => 160,
                x if x == StatType::SUPPORT_GAUGE_1 as u32 => 360,
                x if x == StatType::SUPPORT_GAUGE_2 as u32 => 600,
                _ => panic!("unhandled ark passive class option stat: {original_stat} [{value}]"),
            };
            stat_type = StatType::SUPPORT_GAUGE as u32;
        }
        Some(AddonType::SKILL_GROUP_STATUS_EFFECT_STAT_MULTIPLIER) => {}
        Some(AddonType::ATTACK_POWER_AMPLIFY_MULTIPLIER) => {
            stat_type = StatType::ALLY_ATK_POWER_ENHANCEMENT_EFFECT as u32;
        }
        Some(AddonType::PARTY_WITHOUT_SELF_SHIELD) => {
            stat_type = StatType::SHIELD_FOR_PARTY_MEMBERS as u32;
        }
        Some(AddonType::PARTY_WITHOUT_SELF_HEAL) => {
            stat_type = StatType::RECOVERY_FOR_PARTY_MEMBERS as u32;
        }
        _ => panic!(
            "unhandled ark passive addon type: {addon_type} / stat {original_stat} / value {value}"
        ),
    }

    ParsedItemAddon {
        addon_type,
        stat_type,
        original_stat,
        value,
    }
}

fn parse_bracer_addon(bytes: &[u8]) -> ParsedItemAddon {
    assert!(
        bytes.len() >= 30,
        "invalid bracer addon byte length: {}",
        bytes.len()
    );
    let addon_type = bytes[0];
    let _max_value = read_i32(bytes, 5) as i64;
    let value = read_i32(bytes, 9) as i64;
    let original_stat = read_u32(bytes, 13);
    let _min_value = read_i32(bytes, 25) as i64;
    let mut stat_type = original_stat;

    match AddonType::from_raw(addon_type) {
        Some(AddonType::STAT) => {}
        Some(AddonType::ABILITY_POINT) => {}
        Some(AddonType::COMBAT_EFFECT) => {}
        Some(AddonType::SKILL_GROUP_STATUS_EFFECT_STAT_MULTIPLIER) => {}
        Some(AddonType::ATTACK_POWER_AMPLIFY_MULTIPLIER) => {
            stat_type = StatType::ALLY_ATK_POWER_ENHANCEMENT_EFFECT as u32;
        }
        _ => panic!(
            "unhandled bracer addon type: {addon_type} / stat {original_stat} / value {value}"
        ),
    }

    ParsedItemAddon {
        addon_type,
        stat_type,
        original_stat,
        value,
    }
}

fn parse_quality_addon(bytes: &[u8]) -> ParsedItemAddon {
    assert!(
        bytes.len() >= 29,
        "invalid quality addon byte length: {}",
        bytes.len()
    );
    let addon_type = bytes[0];
    assert!(
        AddonType::from_raw(addon_type) == Some(AddonType::STAT),
        "unhandled quality addon type: {addon_type}"
    );

    ParsedItemAddon {
        addon_type,
        stat_type: read_u32(bytes, 13),
        original_stat: read_u32(bytes, 13),
        value: read_i32(bytes, 5) as i64,
    }
}

fn resolve_item_balance_level(
    item_definition: &crate::models::ExternalItemData,
    hone_level: u16,
    advanced_honing_level: u8,
) -> ResolvedItemBalanceLevel {
    let hone_adjusted = u32::from(hone_level.saturating_sub(SIDEREAL_WEAPON_HONE_MIN));
    let base_balance_level = if item_definition.grade.eq_ignore_ascii_case("esther")
        && hone_level >= SIDEREAL_WEAPON_HONE_MIN
    {
        get_sidereal_weapon_balance_level(hone_adjusted)
    } else {
        item_definition
            .balance_level
            .saturating_add(hone_adjusted.saturating_mul(5))
    };
    let mut resolved = ResolvedItemBalanceLevel {
        base_balance_level,
        hone_adjusted,
        advanced_balance_level_delta: 0,
        balance_level: base_balance_level,
    };

    if item_definition.item_amplification_base_id != 0
        && advanced_honing_level > 0
        && let Some(item_amp_base) =
            EXTERNAL_ITEM_AMPLIFICATION_BASE_DATA.get(&item_definition.item_amplification_base_id)
        && let Some(level) = item_amp_base.levels.get(&(advanced_honing_level as u32))
    {
        resolved.advanced_balance_level_delta = level.balance_level.max(0) as u32;
        resolved.balance_level = resolved
            .balance_level
            .saturating_add(resolved.advanced_balance_level_delta);
    }

    resolved
}

fn get_sidereal_weapon_balance_level(hone_level: u32) -> u32 {
    match hone_level {
        0 => 1100,
        1 => 1200,
        2 => 1300,
        3 => 1400,
        4 => 1500,
        5 => 1600,
        6 => 1675,
        7 => 1695,
        8 => 1715,
        9 => 1745,
        10 => 1775,
        11 => 1800,
        12 => 1830,
        _ => panic!("unsupported sidereal weapon hone level: {hone_level}"),
    }
}

fn get_item_level_option(
    option_id: u32,
    balance_level: u32,
) -> Option<&'static crate::models::ExternalItemOptionLevel> {
    EXTERNAL_ITEM_LEVEL_OPTION_DATA
        .get(&option_id)
        .and_then(|level_option| level_option.levels.get(&balance_level))
}

fn fill_item_option_debug<T: ItemOptionLevelLike>(
    item_debug: &mut InspectItemBuildDebug,
    applied_option_kind: &str,
    applied_option_id: u32,
    option_data: &T,
    bonus_mult: f64,
) {
    item_debug.applied_option_found = true;
    item_debug.applied_option_kind = Some(applied_option_kind.to_string());
    item_debug.applied_option_id = Some(applied_option_id);
    item_debug.applied_weapon_power =
        scale_item_option_stat(option_data.weapon_power(), bonus_mult);
    item_debug.applied_strength = scale_item_option_stat(option_data.strength(), bonus_mult);
    item_debug.applied_dexterity = scale_item_option_stat(option_data.dexterity(), bonus_mult);
    item_debug.applied_intelligence =
        scale_item_option_stat(option_data.intelligence(), bonus_mult);
    item_debug.applied_vitality = scale_item_option_stat(option_data.vitality(), bonus_mult);
    item_debug.applied_physical_defense =
        scale_item_option_stat(option_data.physical_defense(), bonus_mult);
    item_debug.applied_magic_defense =
        scale_item_option_stat(option_data.magic_defense(), bonus_mult);
}

fn apply_item_level_option(
    option_id: u32,
    balance_level: u32,
    bonus_mult: f64,
    raw_stat_pairs: &HashMap<u8, i64>,
    derived: &mut InspectDerivedStats,
) -> bool {
    let Some(level_option) = EXTERNAL_ITEM_LEVEL_OPTION_DATA.get(&option_id) else {
        return false;
    };
    if let Some(level_stats) = level_option.levels.get(&balance_level) {
        apply_item_option_level(level_stats, bonus_mult, raw_stat_pairs, derived);
    }
    true
}

fn apply_ability_level_addons(
    ability_id: u32,
    level: u32,
    raw_stat_pairs: &HashMap<u8, i64>,
    derived: &mut InspectDerivedStats,
) {
    let Some(ability) = EXTERNAL_ABILITY_DATA.get(&ability_id) else {
        return;
    };
    let Some(level_data) = ability.levels.get(&level) else {
        return;
    };
    for addon in &level_data.addons {
        apply_external_addon(addon, raw_stat_pairs, derived);
    }
}

fn apply_gem_addon(
    gem_line: &[u8],
    raw_stat_pairs: &HashMap<u8, i64>,
    derived: &mut InspectDerivedStats,
) -> bool {
    let Some((ability_offset, addon_offset, value_offset)) = resolve_gem_layout(gem_line) else {
        return false;
    };
    if gem_line.len() < ability_offset + 4 || gem_line.len() < value_offset + 4 {
        return false;
    }

    let addon_type = gem_line[addon_offset];
    let ability_id = read_u32(gem_line, ability_offset);
    let value = read_i32(gem_line, value_offset) as i64;
    apply_numeric_addon(addon_type, ability_id, value, raw_stat_pairs, derived);
    true
}

fn resolve_gem_layout(gem_line: &[u8]) -> Option<(usize, usize, usize)> {
    let mut ability_offset = None;
    for idx in 0..=gem_line.len().saturating_sub(4) {
        let skill_id = read_u32(gem_line, idx);
        if SKILL_DATA.contains_key(&skill_id) {
            ability_offset = Some(idx);
            break;
        }
    }
    let ability_offset = ability_offset?;

    let mut value_offset = None;
    for idx in 0..=gem_line.len().saturating_sub(4) {
        if idx >= ability_offset && idx < ability_offset + 4 {
            continue;
        }
        let value = read_u32(gem_line, idx);
        if (300..=4400).contains(&value) {
            value_offset = Some(idx);
            break;
        }
    }
    let value_offset = value_offset?;

    let addon_offset = (0..gem_line.len()).find(|idx| {
        !(*idx >= ability_offset && *idx < ability_offset + 4)
            && !(*idx >= value_offset && *idx < value_offset + 4)
    })?;

    Some((ability_offset, addon_offset, value_offset))
}

fn apply_item_option_level<T: ItemOptionLevelLike>(
    level: &T,
    bonus_mult: f64,
    raw_stat_pairs: &HashMap<u8, i64>,
    derived: &mut InspectDerivedStats,
) {
    add_flat_stat(
        "weapon_dam",
        scale_item_option_stat(level.weapon_power(), bonus_mult),
        raw_stat_pairs,
        derived,
    );
    add_flat_stat(
        "int",
        scale_item_option_stat(level.intelligence(), bonus_mult),
        raw_stat_pairs,
        derived,
    );
    add_flat_stat(
        "str",
        scale_item_option_stat(level.strength(), bonus_mult),
        raw_stat_pairs,
        derived,
    );
    add_flat_stat(
        "agi",
        scale_item_option_stat(level.dexterity(), bonus_mult),
        raw_stat_pairs,
        derived,
    );
    add_flat_stat(
        "con",
        scale_item_option_stat(level.vitality(), bonus_mult),
        raw_stat_pairs,
        derived,
    );
    add_flat_stat(
        "def",
        scale_item_option_stat(level.physical_defense(), bonus_mult),
        raw_stat_pairs,
        derived,
    );
    add_flat_stat(
        "res",
        scale_item_option_stat(level.magic_defense(), bonus_mult),
        raw_stat_pairs,
        derived,
    );
}

fn scale_item_option_stat(value: i64, bonus_mult: f64) -> i64 {
    (value as f64 * bonus_mult) as i64
}

fn apply_named_stat(
    stat_name: &str,
    value: i64,
    raw_stat_pairs: &HashMap<u8, i64>,
    derived: &mut InspectDerivedStats,
) {
    match stat_name {
        "stigma_power_rate" => {
            derived.ally_brand_power += value as f64 / 10000.0;
        }
        "none" | "" => {}
        _ => add_flat_stat(stat_name, value, raw_stat_pairs, derived),
    }
}

fn add_multiplier_stat(
    stat_name: &str,
    value: f64,
    raw_stat_pairs: &HashMap<u8, i64>,
    derived: &mut InspectDerivedStats,
) {
    add_flat_stat(
        stat_name,
        (value * 10000.0).round() as i64,
        raw_stat_pairs,
        derived,
    );
}

fn add_flat_stat(
    stat_name: &str,
    value: i64,
    raw_stat_pairs: &HashMap<u8, i64>,
    derived: &mut InspectDerivedStats,
) {
    let _ = raw_stat_pairs;
    let Some(stat_id) = STAT_TYPE_MAP
        .get(stat_name)
        .copied()
        .and_then(|stat_id| u8::try_from(stat_id).ok())
    else {
        return;
    };
    *derived.stat_pairs.entry(stat_id).or_default() += value;
}

fn damage_attr_from_name(name: &str) -> Option<u8> {
    match name.to_ascii_lowercase().as_str() {
        "fire" => Some(1),
        "ice" => Some(2),
        "electricity" => Some(3),
        "wind" => Some(4),
        "earth" => Some(5),
        "dark" => Some(6),
        "holy" => Some(7),
        _ => None,
    }
}

fn split_fixed_chunks(bytes: &[u8], chunk_len: usize) -> impl Iterator<Item = &[u8]> {
    bytes
        .chunks_exact(chunk_len)
        .filter(|chunk| !chunk.iter().all(|value| *value == 0))
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    let mut value = [0u8; 4];
    value.copy_from_slice(&bytes[offset..offset + 4]);
    u32::from_le_bytes(value)
}

fn read_i32(bytes: &[u8], offset: usize) -> i32 {
    let mut value = [0u8; 4];
    value.copy_from_slice(&bytes[offset..offset + 4]);
    i32::from_le_bytes(value)
}

trait ItemOptionLevelLike {
    fn weapon_power(&self) -> i64;
    fn physical_defense(&self) -> i64;
    fn magic_defense(&self) -> i64;
    fn strength(&self) -> i64;
    fn dexterity(&self) -> i64;
    fn intelligence(&self) -> i64;
    fn vitality(&self) -> i64;
}

impl ItemOptionLevelLike for crate::models::ExternalItemOptionLevel {
    fn weapon_power(&self) -> i64 {
        self.weapon_power
    }
    fn physical_defense(&self) -> i64 {
        self.physical_defense
    }
    fn magic_defense(&self) -> i64 {
        self.magic_defense
    }
    fn strength(&self) -> i64 {
        self.strength
    }
    fn dexterity(&self) -> i64 {
        self.dexterity
    }
    fn intelligence(&self) -> i64 {
        self.intelligence
    }
    fn vitality(&self) -> i64 {
        self.vitality
    }
}

impl ItemOptionLevelLike for crate::models::ExternalItemGradeStaticOptionData {
    fn weapon_power(&self) -> i64 {
        self.weapon_power
    }
    fn physical_defense(&self) -> i64 {
        self.physical_defense
    }
    fn magic_defense(&self) -> i64 {
        self.magic_defense
    }
    fn strength(&self) -> i64 {
        self.strength
    }
    fn dexterity(&self) -> i64 {
        self.dexterity
    }
    fn intelligence(&self) -> i64 {
        self.intelligence
    }
    fn vitality(&self) -> i64 {
        self.vitality
    }
}
