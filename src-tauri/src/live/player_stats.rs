use crate::data::{
    COMBAT_EFFECT_DATA, EXTERNAL_ABILITY_DATA, EXTERNAL_ADDON_SKILL_FEATURE_DATA,
    EXTERNAL_ITEM_CLASS_OPTION_DATA, SKILL_DATA, identity_category_matches, stat_type_name_from_id,
};
use crate::live::entity_tracker::{InspectSnapshot, SkillRuntimeData};
use crate::live::status_tracker::StatusEffectDetails;
use crate::models::{CombatEffectAction, CombatEffectCondition, HitFlag, HitOption};
use hashbrown::{HashMap, HashSet};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt;
use std::sync::OnceLock;

const DAMAGE_ATTR_SLOTS: usize = 8;
const DEFAULT_CRITICAL_DAMAGE_RATE: f64 = 1.0;
const MOVE_SPEED_ATTACK_SPEED_CAP: f64 = 0.4;
const DESTROYER_RELEASE_IDENTITY_CATEGORY_ID: &str = "7";
const DESTROYER_RELEASE_IDENTITY_CATEGORY: &str = "destroyer_release";
const DESTROYER_HYPERGRAVITY_VORTEX_SKILL_GROUP_ID: u32 = 2160060;
const DESTROYER_RECENT_CONSUMED_CORE_WINDOW_MS: i64 = 5_000;
const ROSTER_MAIN_STAT_BONUS: f64 = 1930.0;
const ROSTER_CRITICAL_HIT_BONUS: f64 = 69.0;
const SKIN_MAIN_STAT_MULTIPLIER_CAP: f64 = 0.08;
const PET_MAIN_STAT_MULTIPLIER: f64 = 0.011057;
const PET_SKILL_DAMAGE_RATE: f64 = 0.01;
const FIXED_STAT_DATA_COUNT: usize = 44;
const DAMAGE_SPLIT_CACHED_MAX_FACTORS: usize = 20;
pub const STAT_PRIORITY_SUPPORT: i32 = 0;
pub const STAT_PRIORITY_DEFAULT: i32 = 100;

#[derive(Default)]
struct DamageSplitScratch {
    compact_factors: Vec<f64>,
    compact_indices: Vec<usize>,
    subset_prod: Vec<f64>,
    subset_size: Vec<usize>,
    weights: Vec<f64>,
    factorial: Vec<f64>,
}

thread_local! {
    static DAMAGE_SPLIT_SCRATCH: RefCell<DamageSplitScratch> = RefCell::new(DamageSplitScratch::default());
}

fn damage_split_weight_cache() -> &'static Vec<Vec<f64>> {
    static CACHE: OnceLock<Vec<Vec<f64>>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut factorial = vec![1.0; DAMAGE_SPLIT_CACHED_MAX_FACTORS + 1];
        for index in 1..=DAMAGE_SPLIT_CACHED_MAX_FACTORS {
            factorial[index] = factorial[index - 1] * index as f64;
        }

        let mut cache = Vec::with_capacity(DAMAGE_SPLIT_CACHED_MAX_FACTORS + 1);
        cache.push(Vec::new());
        for count in 1..=DAMAGE_SPLIT_CACHED_MAX_FACTORS {
            let mut weights = vec![0.0; count];
            for subset_size in 0..count {
                weights[subset_size] =
                    factorial[subset_size] * factorial[count - subset_size - 1] / factorial[count];
            }
            cache.push(weights);
        }
        cache
    })
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OperationType {
    #[default]
    Additive,
    Multiplicative,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct IdentityGaugeSnapshot {
    pub identity_gauge1: u32,
    pub identity_gauge2: u32,
    pub identity_gauge3: u32,
}

#[derive(Debug, Clone, Default)]
pub struct StatDataValue {
    pub value: f64,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct StatModification {
    pub values: Vec<StatDataValue>,
    pub source_entity_id: u64,
    pub source_priority: i32,
}

impl Default for StatModification {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            source_entity_id: 0,
            source_priority: STAT_PRIORITY_DEFAULT,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StatData {
    pub self_values: Vec<StatDataValue>,
    pub modified_values: Vec<StatModification>,
    pub operation_type: OperationType,
}

impl StatData {
    fn compare_stat_data_value(lhs: &StatDataValue, rhs: &StatDataValue) -> Ordering {
        lhs.source
            .cmp(&rhs.source)
            .then_with(|| lhs.value.total_cmp(&rhs.value))
    }

    fn compare_stat_modification(lhs: &StatModification, rhs: &StatModification) -> Ordering {
        lhs.source_priority
            .cmp(&rhs.source_priority)
            .then_with(|| lhs.source_entity_id.cmp(&rhs.source_entity_id))
    }

    fn sort_self_values(&mut self) {
        self.self_values.sort_by(Self::compare_stat_data_value);
    }

    fn sort_modification_values(modification: &mut StatModification) {
        modification.values.sort_by(Self::compare_stat_data_value);
    }

    fn sort_modifications(&mut self) {
        self.modified_values
            .sort_by(Self::compare_stat_modification);
    }

    pub fn value(&self) -> f64 {
        let mut value = self.self_value();
        match self.operation_type {
            OperationType::Additive => {
                for modification in &self.modified_values {
                    value += modification.value(self.operation_type);
                }
            }
            OperationType::Multiplicative => {
                value += 1.0;
                for modification in &self.modified_values {
                    value *= 1.0 + modification.value(self.operation_type);
                }
                value -= 1.0;
            }
        }
        value
    }

    pub fn self_value(&self) -> f64 {
        match self.operation_type {
            OperationType::Additive => self.self_values.iter().map(|value| value.value).sum(),
            OperationType::Multiplicative => {
                let mut value = 1.0;
                for entry in &self.self_values {
                    value *= 1.0 + entry.value;
                }
                value - 1.0
            }
        }
    }

    pub fn get_value(&self) -> f64 {
        self.value()
    }

    pub fn modification_count(&self) -> usize {
        self.modified_values.len()
    }

    pub fn add_self(&mut self, value: f64, source: impl Into<String>) {
        if value == 0.0 {
            return;
        }
        let source = source.into();
        if self.operation_type == OperationType::Additive
            && let Some(existing) = self
                .self_values
                .iter_mut()
                .find(|entry| entry.source == source)
        {
            existing.value += value;
            return;
        }
        self.self_values.push(StatDataValue { value, source });
        self.sort_self_values();
    }

    pub fn add(
        &mut self,
        value: f64,
        stats_owner_id: u64,
        value_owner_id: u64,
        source: impl Into<String>,
    ) {
        self.add_with_priority(
            value,
            stats_owner_id,
            value_owner_id,
            source,
            STAT_PRIORITY_DEFAULT,
        );
    }

    pub fn add_with_priority(
        &mut self,
        value: f64,
        stats_owner_id: u64,
        value_owner_id: u64,
        source: impl Into<String>,
        source_priority: i32,
    ) {
        if stats_owner_id == value_owner_id || value_owner_id == 0 {
            self.add_self(value, source);
            return;
        }
        self.add_modification_with_priority(value, value_owner_id, source, source_priority);
    }

    pub fn add_modification(
        &mut self,
        value: f64,
        source_entity_id: u64,
        source: impl Into<String>,
    ) {
        self.add_modification_with_priority(value, source_entity_id, source, STAT_PRIORITY_DEFAULT);
    }

    pub fn add_modification_with_priority(
        &mut self,
        value: f64,
        source_entity_id: u64,
        source: impl Into<String>,
        source_priority: i32,
    ) {
        if value == 0.0 {
            return;
        }
        let source = source.into();
        if let Some(index) = self
            .modified_values
            .iter()
            .position(|modification| modification.source_entity_id == source_entity_id)
        {
            let modification = &mut self.modified_values[index];
            let mut should_sort_modifications = false;
            if source_priority < modification.source_priority {
                modification.source_priority = source_priority;
                should_sort_modifications = true;
            }
            if self.operation_type == OperationType::Additive
                && let Some(existing) = modification
                    .values
                    .iter_mut()
                    .find(|entry| entry.source == source)
            {
                existing.value += value;
                if should_sort_modifications {
                    self.sort_modifications();
                }
                return;
            }
            modification.values.push(StatDataValue { value, source });
            Self::sort_modification_values(modification);
            if should_sort_modifications {
                self.sort_modifications();
            }
            return;
        }
        self.modified_values.push(StatModification {
            values: vec![StatDataValue { value, source }],
            source_entity_id,
            source_priority,
        });
        self.sort_modifications();
    }

    pub fn get_value_for_entity_id(&self, entity_id: u64) -> f64 {
        if entity_id == 0 {
            return self.self_value();
        }
        self.modified_values
            .iter()
            .find(|modification| modification.source_entity_id == entity_id)
            .map(|modification| modification.value(self.operation_type))
            .unwrap_or_default()
    }

    pub fn set_self(&mut self, value: f64, source: impl Into<String>) {
        self.self_values.clear();
        self.self_values.push(StatDataValue {
            value,
            source: source.into(),
        });
    }

    pub fn set_modification(
        &mut self,
        value: f64,
        source_entity_id: u64,
        source: impl Into<String>,
    ) {
        self.set_modification_with_priority(value, source_entity_id, source, STAT_PRIORITY_DEFAULT);
    }

    pub fn set_modification_with_priority(
        &mut self,
        value: f64,
        source_entity_id: u64,
        source: impl Into<String>,
        source_priority: i32,
    ) {
        let source = source.into();
        if let Some(index) = self
            .modified_values
            .iter()
            .position(|modification| modification.source_entity_id == source_entity_id)
        {
            let modification = &mut self.modified_values[index];
            modification.values.clear();
            modification.values.push(StatDataValue { value, source });
            if source_priority < modification.source_priority {
                modification.source_priority = source_priority;
                self.sort_modifications();
            }
            return;
        }
        self.modified_values.push(StatModification {
            values: vec![StatDataValue { value, source }],
            source_entity_id,
            source_priority,
        });
        self.sort_modifications();
    }

    pub fn set(
        &mut self,
        value: f64,
        stats_owner_id: u64,
        value_owner_id: u64,
        source: impl Into<String>,
    ) {
        self.set_with_priority(
            value,
            stats_owner_id,
            value_owner_id,
            source,
            STAT_PRIORITY_DEFAULT,
        );
    }

    pub fn set_with_priority(
        &mut self,
        value: f64,
        stats_owner_id: u64,
        value_owner_id: u64,
        source: impl Into<String>,
        source_priority: i32,
    ) {
        if stats_owner_id == value_owner_id || value_owner_id == 0 {
            self.set_self(value, source);
            return;
        }
        self.set_modification_with_priority(value, value_owner_id, source, source_priority);
    }

    pub fn operation_type(&self) -> OperationType {
        self.operation_type
    }

    pub fn get_operation_type(&self) -> OperationType {
        self.operation_type()
    }

    pub fn set_operation_type(&mut self, operation_type: OperationType) {
        self.operation_type = operation_type;
    }

    pub fn copy_from(&mut self, source: &Self) {
        *self = source.clone();
    }

    pub fn clear(&mut self) {
        self.self_values.clear();
        self.modified_values.clear();
        self.operation_type = OperationType::Additive;
    }

    pub fn clamp(&mut self, min: f64, max: f64) {
        if self.operation_type != OperationType::Additive {
            panic!("Not supported for multiplicative yet!");
        }
        let sum = self.value();
        if sum < min {
            self.add_self(min - sum, "clamp");
            return;
        }
        if sum <= max {
            return;
        }
        let mut to_fix = sum - max;
        for modification in self.modified_values.iter_mut().rev() {
            for value in modification.values.iter_mut().rev() {
                if value.value <= 0.0 {
                    continue;
                }
                let fix_allowed = value.value.min(to_fix);
                value.value -= fix_allowed;
                to_fix -= fix_allowed;
                if to_fix <= 0.0 {
                    return;
                }
            }
        }
        for value in self.self_values.iter_mut().rev() {
            if value.value <= 0.0 {
                continue;
            }
            let fix_allowed = value.value.min(to_fix);
            value.value -= fix_allowed;
            to_fix -= fix_allowed;
            if to_fix <= 0.0 {
                return;
            }
        }
        if to_fix > 0.0 {
            self.add_self(-to_fix, "clamp");
        }
    }

    pub fn zero_for_entity(&mut self, stats_owner_id: u64, entity_id: u64) {
        if entity_id == 0 {
            self.self_values.clear();
            return;
        }
        if stats_owner_id == entity_id {
            self.self_values.clear();
            return;
        }
        if let Some(modification) = self
            .modified_values
            .iter_mut()
            .find(|modification| modification.source_entity_id == entity_id)
        {
            modification.values.clear();
        }
    }

    /// Take ownership of the values vec representing `entity_id`'s contribution,
    /// leaving an empty vec in its place. An empty vec is semantically equivalent
    /// to "no contribution from this source," so this is the cheap way to compute
    /// "what would attack power be without this entity?" without cloning the stat.
    /// Returns None if there's nothing to take.
    pub fn take_values_for_entity(
        &mut self,
        stats_owner_id: u64,
        entity_id: u64,
    ) -> Option<Vec<StatDataValue>> {
        if entity_id == 0 || stats_owner_id == entity_id {
            (!self.self_values.is_empty()).then(|| std::mem::take(&mut self.self_values))
        } else {
            self.modified_values
                .iter_mut()
                .find(|modification| modification.source_entity_id == entity_id)
                .filter(|modification| !modification.values.is_empty())
                .map(|modification| std::mem::take(&mut modification.values))
        }
    }

    /// Restore the values vec previously taken by `take_values_for_entity`.
    pub fn restore_values_for_entity(
        &mut self,
        stats_owner_id: u64,
        entity_id: u64,
        values: Vec<StatDataValue>,
    ) {
        if entity_id == 0 || stats_owner_id == entity_id {
            self.self_values = values;
        } else if let Some(modification) = self
            .modified_values
            .iter_mut()
            .find(|modification| modification.source_entity_id == entity_id)
        {
            modification.values = values;
        }
    }

    pub fn contributing_entities(&self, out: &mut HashSet<u64>) {
        for modification in &self.modified_values {
            if !modification.values.is_empty() {
                out.insert(modification.source_entity_id);
            }
        }
    }

    pub fn get_modification_at(&self, index: usize) -> &StatModification {
        &self.modified_values[index]
    }

    pub fn get_modification_for_entity_id(&self, entity_id: u64) -> &StatModification {
        self.modified_values
            .iter()
            .find(|modification| modification.source_entity_id == entity_id)
            .unwrap_or_else(|| panic!("Modification for entity {entity_id} not found."))
    }

    pub fn get_modification_for_entity_id_mut(&mut self, entity_id: u64) -> &mut StatModification {
        self.modified_values
            .iter_mut()
            .find(|modification| modification.source_entity_id == entity_id)
            .unwrap_or_else(|| panic!("Modification for entity {entity_id} not found."))
    }

    pub fn copy(&mut self, source: &Self) {
        self.copy_from(source);
    }

    pub fn multiplied_by_scalar(&self, scalar: f64) -> Self {
        let mut out = self.clone();
        for value in &mut out.self_values {
            value.value *= scalar;
        }
        for modification in &mut out.modified_values {
            for value in &mut modification.values {
                value.value *= scalar;
            }
        }
        out
    }

    pub fn divided_by_scalar(&self, scalar: f64) -> Self {
        let mut out = self.clone();
        for value in &mut out.self_values {
            value.value /= scalar;
        }
        for modification in &mut out.modified_values {
            for value in &mut modification.values {
                value.value /= scalar;
            }
        }
        out
    }

    pub fn multiplied_by_stat(&self, right: &Self) -> Self {
        let mut out = Self::default();
        out.add_self(self.value() * right.value(), "base");
        out
    }

    pub fn mad(&self, right: &Self) -> Self {
        self.added(&self.multiplied_by_stat(right))
    }

    pub fn added(&self, right: &Self) -> Self {
        let mut out = self.clone();
        for value in &right.self_values {
            out.add_self(value.value, value.source.clone());
        }
        for modification in &right.modified_values {
            for value in &modification.values {
                out.add_modification_with_priority(
                    value.value,
                    modification.source_entity_id,
                    value.source.clone(),
                    modification.source_priority,
                );
            }
        }
        out
    }

    pub fn subtracted(&self, right: &Self) -> Self {
        let mut out = self.clone();
        for value in &right.self_values {
            out.add_self(-value.value, value.source.clone());
        }
        for modification in &right.modified_values {
            for value in &modification.values {
                out.add_modification_with_priority(
                    -value.value,
                    modification.source_entity_id,
                    value.source.clone(),
                    modification.source_priority,
                );
            }
        }
        out
    }
}

impl StatModification {
    pub fn get_value(&self, operation_type: OperationType) -> f64 {
        self.value(operation_type)
    }

    fn value(&self, operation_type: OperationType) -> f64 {
        match operation_type {
            OperationType::Additive => self.values.iter().map(|value| value.value).sum(),
            OperationType::Multiplicative => {
                let mut value = 1.0;
                for entry in &self.values {
                    value *= 1.0 + entry.value;
                }
                value - 1.0
            }
        }
    }
}

impl fmt::Display for StatData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[derive(Debug, Clone, Default)]
pub struct RuntimeState {
    pub identity_runtime_reliable: bool,
    pub identity_stance: u8,
    pub identity_gauge1: u32,
    pub identity_gauge2: u32,
    pub identity_gauge3: u32,
    pub identity_gauge1_prev: u32,
    pub identity_gauge2_prev: u32,
    pub identity_gauge3_prev: u32,
    pub last_skill_start_id: u32,
    pub last_skill_start_at_ms: i64,
    pub destroyer_recent_consumed_cores: i32,
    pub destroyer_recent_consumed_at_ms: i64,
    pub current_hp: i64,
    pub max_hp: i64,
    pub current_mp: i64,
    pub max_mp: i64,
    pub combat_mp_recovery: i64,
    pub skill_start_identity_gauge_snapshots: HashMap<u32, IdentityGaugeSnapshot>,
}

impl RuntimeState {
    pub fn create() -> Self {
        let mut state = Self::default();
        state.clear();
        state
    }

    pub fn clear(&mut self) {
        self.identity_runtime_reliable = false;
        self.identity_stance = 0;
        self.identity_gauge1 = 0;
        self.identity_gauge2 = 0;
        self.identity_gauge3 = 0;
        self.identity_gauge1_prev = 0;
        self.identity_gauge2_prev = 0;
        self.identity_gauge3_prev = 0;
        self.last_skill_start_id = 0;
        self.last_skill_start_at_ms = 0;
        self.destroyer_recent_consumed_cores = 0;
        self.destroyer_recent_consumed_at_ms = 0;
        self.current_hp = 0;
        self.max_hp = 0;
        self.current_mp = 0;
        self.max_mp = 0;
        self.combat_mp_recovery = 0;
        self.skill_start_identity_gauge_snapshots.clear();
    }

    pub fn copy_from(&mut self, source: &Self) {
        *self = source.clone();
    }
}

#[derive(Debug, Clone)]
pub struct AbilityFeatureState {
    pub feature_type: String,
    pub level: u32,
    pub values: Vec<i64>,
    pub owner_id: u64,
}

#[derive(Debug, Clone)]
pub struct ActiveCombatEffect {
    pub combat_effect_id: u32,
    pub effect: crate::models::CombatEffectDetail,
    pub owner_id: u64,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct ActiveAddonSkillFeature {
    pub feature_id: u32,
    pub owner_id: u64,
}

#[derive(Debug, Clone, Copy)]
enum CompositeTarget {
    CriticalRate,
    CriticalDamage,
    MoveSpeedDamage,
    CriticalHitToDamage,
    CriticalRateCapOverToEvolution,
}

#[derive(Debug, Clone, Copy)]
enum CompositeOwnership {
    PreserveSource,
    ForceSelf,
}

#[derive(Debug, Clone)]
pub struct PlayerStats {
    pub owner_id: u64,
    pub runtime_state: RuntimeState,
    pub weapon_power: StatData,
    pub weapon_dam_x: StatData,
    pub attack_power_base_multiplier: StatData,
    pub attack_power_rate: StatData,
    pub str_stat: StatData,
    pub dex_stat: StatData,
    pub int_stat: StatData,
    pub str_multiplier_stat: StatData,
    pub dex_multiplier_stat: StatData,
    pub int_multiplier_stat: StatData,
    pub critical_hit_stat: StatData,
    pub move_speed_rate: StatData,
    pub attack_speed_rate: StatData,
    pub ally_identity_damage_power: StatData,
    pub ally_attack_power_power: StatData,
    pub ally_brand_power: StatData,
    pub evolution_damage: StatData,
    pub modify_damage_combat_effect: StatData,
    pub spec_bonus_identity_1: StatData,
    pub spec_bonus_identity_2: StatData,
    pub spec_bonus_identity_3: StatData,
    pub critical_hit_rate: StatData,
    pub critical_hit_rate_cap: f64,
    pub critical_damage_rate: StatData,
    pub critical_damage_rate_2: StatData,
    pub attack_power_addend: StatData,
    pub attack_power_addend_2: StatData,
    pub attack_power_sub_rate_1: StatData,
    pub attack_power_sub_rate_2: StatData,
    pub skill_damage_sub_rate_1: StatData,
    pub skill_damage_sub_rate_2: StatData,
    pub skill_damage_rate: StatData,
    pub ultimate_awakening_damage_rate: StatData,
    pub move_speed_to_damage_rate: StatData,
    pub critical_hit_to_damage_rate: StatData,
    pub physical_defense_break: StatData,
    pub magical_defense_break: StatData,
    pub outgoing_dmg_stat_amp: StatData,
    pub skill_damage_amplify: StatData,
    pub front_attack_amplify: StatData,
    pub back_attack_amplify: StatData,
    pub physical_critical_damage_amplify: StatData,
    pub magical_critical_damage_amplify: StatData,
    pub damage_attr_rates: Vec<StatData>,
    pub damage_attr_amplifications: Vec<StatData>,
    pub damage_conversion_type: Option<u8>,
    pub evolution_damage_bonus_from_blunt_thorn: StatData,
    pub evolution_damage_bonus_from_supersonic_breakthrough: StatData,
    pub skill_status_effect_multiplier: HashMap<u32, f64>,
    pub skill_attack_power_multiplier: HashMap<u32, f64>,
    pub skill_group_status_effect_multiplier: HashMap<u32, f64>,
    pub active_combat_effects: Vec<ActiveCombatEffect>,
    pub active_addon_skill_features: Vec<ActiveAddonSkillFeature>,
    pub active_ability_features: Vec<AbilityFeatureState>,
}

impl Default for PlayerStats {
    fn default() -> Self {
        let mut stats = Self {
            owner_id: 0,
            runtime_state: RuntimeState::create(),
            weapon_power: StatData::default(),
            weapon_dam_x: StatData::default(),
            attack_power_base_multiplier: StatData::default(),
            attack_power_rate: StatData::default(),
            str_stat: StatData::default(),
            dex_stat: StatData::default(),
            int_stat: StatData::default(),
            str_multiplier_stat: StatData::default(),
            dex_multiplier_stat: StatData::default(),
            int_multiplier_stat: StatData::default(),
            critical_hit_stat: StatData::default(),
            move_speed_rate: StatData::default(),
            attack_speed_rate: StatData::default(),
            ally_identity_damage_power: StatData::default(),
            ally_attack_power_power: StatData::default(),
            ally_brand_power: StatData::default(),
            evolution_damage: StatData::default(),
            modify_damage_combat_effect: StatData::default(),
            spec_bonus_identity_1: StatData::default(),
            spec_bonus_identity_2: StatData::default(),
            spec_bonus_identity_3: StatData::default(),
            critical_hit_rate: StatData::default(),
            critical_hit_rate_cap: 1.0,
            critical_damage_rate: StatData::default(),
            critical_damage_rate_2: StatData::default(),
            attack_power_addend: StatData::default(),
            attack_power_addend_2: StatData::default(),
            attack_power_sub_rate_1: StatData::default(),
            attack_power_sub_rate_2: StatData::default(),
            skill_damage_sub_rate_1: StatData::default(),
            skill_damage_sub_rate_2: StatData::default(),
            skill_damage_rate: StatData::default(),
            ultimate_awakening_damage_rate: StatData::default(),
            move_speed_to_damage_rate: StatData::default(),
            critical_hit_to_damage_rate: StatData::default(),
            physical_defense_break: StatData::default(),
            magical_defense_break: StatData::default(),
            outgoing_dmg_stat_amp: StatData::default(),
            skill_damage_amplify: StatData::default(),
            front_attack_amplify: StatData::default(),
            back_attack_amplify: StatData::default(),
            physical_critical_damage_amplify: StatData::default(),
            magical_critical_damage_amplify: StatData::default(),
            damage_attr_rates: vec![StatData::default(); DAMAGE_ATTR_SLOTS],
            damage_attr_amplifications: vec![StatData::default(); DAMAGE_ATTR_SLOTS],
            damage_conversion_type: None,
            evolution_damage_bonus_from_blunt_thorn: StatData::default(),
            evolution_damage_bonus_from_supersonic_breakthrough: StatData::default(),
            skill_status_effect_multiplier: HashMap::new(),
            skill_attack_power_multiplier: HashMap::new(),
            skill_group_status_effect_multiplier: HashMap::new(),
            active_combat_effects: Vec::new(),
            active_addon_skill_features: Vec::new(),
            active_ability_features: Vec::new(),
        };
        stats.clear();
        stats
    }
}

impl PlayerStats {
    fn compare_combat_effect_condition(
        lhs: &CombatEffectCondition,
        rhs: &CombatEffectCondition,
    ) -> Ordering {
        lhs.condition_type
            .cmp(&rhs.condition_type)
            .then_with(|| lhs.actor_type.cmp(&rhs.actor_type))
            .then_with(|| lhs.arg.cmp(&rhs.arg))
    }

    fn compare_combat_effect_action(
        lhs: &CombatEffectAction,
        rhs: &CombatEffectAction,
    ) -> Ordering {
        lhs.action_type
            .cmp(&rhs.action_type)
            .then_with(|| lhs.actor_type.cmp(&rhs.actor_type))
            .then_with(|| lhs.args.len().cmp(&rhs.args.len()))
            .then_with(|| {
                lhs.args
                    .iter()
                    .zip(rhs.args.iter())
                    .map(|(lhs_arg, rhs_arg)| lhs_arg.cmp(rhs_arg))
                    .find(|ordering| *ordering != Ordering::Equal)
                    .unwrap_or(Ordering::Equal)
            })
    }

    fn compare_combat_effect_detail(
        lhs: &crate::models::CombatEffectDetail,
        rhs: &crate::models::CombatEffectDetail,
    ) -> Ordering {
        lhs.ratio
            .cmp(&rhs.ratio)
            .then_with(|| lhs.cooldown.cmp(&rhs.cooldown))
            .then_with(|| lhs.conditions.len().cmp(&rhs.conditions.len()))
            .then_with(|| {
                lhs.conditions
                    .iter()
                    .zip(rhs.conditions.iter())
                    .map(|(lhs_condition, rhs_condition)| {
                        Self::compare_combat_effect_condition(lhs_condition, rhs_condition)
                    })
                    .find(|ordering| *ordering != Ordering::Equal)
                    .unwrap_or(Ordering::Equal)
            })
            .then_with(|| lhs.actions.len().cmp(&rhs.actions.len()))
            .then_with(|| {
                lhs.actions
                    .iter()
                    .zip(rhs.actions.iter())
                    .map(|(lhs_action, rhs_action)| {
                        Self::compare_combat_effect_action(lhs_action, rhs_action)
                    })
                    .find(|ordering| *ordering != Ordering::Equal)
                    .unwrap_or(Ordering::Equal)
            })
    }

    fn sort_active_combat_effects(entries: &mut Vec<ActiveCombatEffect>) {
        entries.sort_by(|lhs, rhs| {
            lhs.owner_id
                .cmp(&rhs.owner_id)
                .then_with(|| lhs.combat_effect_id.cmp(&rhs.combat_effect_id))
                .then_with(|| lhs.source.cmp(&rhs.source))
                .then_with(|| Self::compare_combat_effect_detail(&lhs.effect, &rhs.effect))
        });
    }

    fn sort_active_addon_skill_features(entries: &mut Vec<ActiveAddonSkillFeature>) {
        entries.sort_by(|lhs, rhs| {
            lhs.owner_id
                .cmp(&rhs.owner_id)
                .then_with(|| lhs.feature_id.cmp(&rhs.feature_id))
        });
    }

    fn sort_active_ability_features(entries: &mut Vec<AbilityFeatureState>) {
        entries.sort_by(|lhs, rhs| {
            lhs.owner_id
                .cmp(&rhs.owner_id)
                .then_with(|| {
                    normalize_feature_type(&lhs.feature_type)
                        .cmp(&normalize_feature_type(&rhs.feature_type))
                })
                .then_with(|| lhs.level.cmp(&rhs.level))
        });
    }

    pub fn create() -> Self {
        Self::default()
    }

    pub fn copy(&mut self, source: &Self) {
        *self = source.clone();
    }

    pub fn clear(&mut self) {
        self.runtime_state.clear();
        self.skill_status_effect_multiplier.clear();
        self.skill_attack_power_multiplier.clear();
        self.skill_group_status_effect_multiplier.clear();
        self.active_combat_effects.clear();
        self.active_addon_skill_features.clear();
        self.active_ability_features.clear();
        self.damage_conversion_type = None;
        self.evolution_damage_bonus_from_blunt_thorn.clear();
        self.evolution_damage_bonus_from_supersonic_breakthrough
            .clear();

        self.for_each_stat_mut(|stat| stat.clear());
        self.modify_damage_combat_effect.operation_type = OperationType::Multiplicative;
        self.critical_hit_rate_cap = 1.0;
        self.critical_damage_rate
            .set_self(DEFAULT_CRITICAL_DAMAGE_RATE, "base");
    }

    pub fn apply_runtime_state(&mut self, runtime_state: RuntimeState) {
        self.runtime_state = runtime_state;
    }

    pub fn capture_runtime_state_snapshot(&self) -> RuntimeState {
        self.runtime_state.clone()
    }

    pub fn apply_runtime_state_snapshot(&mut self, runtime_snapshot: RuntimeState) {
        self.runtime_state = runtime_snapshot;
    }

    pub fn clear_runtime_state(&mut self) {
        self.runtime_state.clear();
    }

    pub fn copy_runtime_state_from(&mut self, source: &Self) {
        self.runtime_state.copy_from(&source.runtime_state);
    }

    pub fn record_skill_start(&mut self, skill_id: u32, timestamp: i64) {
        const DESTROYER_VORTEX_GRAVITY_SKILL_ID: u32 = 18011;
        const DESTROYER_HYPERGRAVITY_BASIC_ATTACK_SKILL_ID: u32 = 18030;

        self.runtime_state.last_skill_start_id = skill_id;
        self.runtime_state.last_skill_start_at_ms = timestamp;
        if skill_id == 0 {
            return;
        }
        self.runtime_state
            .skill_start_identity_gauge_snapshots
            .insert(
                skill_id,
                IdentityGaugeSnapshot {
                    identity_gauge1: self.runtime_state.identity_gauge1,
                    identity_gauge2: self.runtime_state.identity_gauge2,
                    identity_gauge3: self.runtime_state.identity_gauge3,
                },
            );
        if skill_id == DESTROYER_VORTEX_GRAVITY_SKILL_ID
            || skill_id == DESTROYER_HYPERGRAVITY_BASIC_ATTACK_SKILL_ID
        {
            self.runtime_state.destroyer_recent_consumed_cores = 0;
            self.runtime_state.destroyer_recent_consumed_at_ms = 0;
        }
    }

    pub fn record_identity_gauge_change(
        &mut self,
        class_id: u32,
        identity_gauge1: u32,
        identity_gauge2: u32,
        identity_gauge3: u32,
        timestamp: i64,
    ) {
        const DESTROYER_CLASS_ID: u32 = 103;
        const DESTROYER_VORTEX_GRAVITY_SKILL_ID: u32 = 18011;
        const DESTROYER_HYPERGRAVITY_BASIC_ATTACK_SKILL_ID: u32 = 18030;
        const RECENT_DESTROYER_SKILL_START_WINDOW_MS: i64 = 4_000;

        let previous_gauge2 = self.runtime_state.identity_gauge2;
        self.runtime_state.identity_gauge1_prev = self.runtime_state.identity_gauge1;
        self.runtime_state.identity_gauge2_prev = self.runtime_state.identity_gauge2;
        self.runtime_state.identity_gauge3_prev = self.runtime_state.identity_gauge3;
        self.runtime_state.identity_gauge1 = identity_gauge1;
        self.runtime_state.identity_gauge2 = identity_gauge2;
        self.runtime_state.identity_gauge3 = identity_gauge3;
        if class_id != DESTROYER_CLASS_ID {
            return;
        }
        let recent_destroyer_skill = self.runtime_state.last_skill_start_id
            == DESTROYER_VORTEX_GRAVITY_SKILL_ID
            || self.runtime_state.last_skill_start_id
                == DESTROYER_HYPERGRAVITY_BASIC_ATTACK_SKILL_ID;
        if !recent_destroyer_skill
            || timestamp < self.runtime_state.last_skill_start_at_ms
            || timestamp - self.runtime_state.last_skill_start_at_ms
                > RECENT_DESTROYER_SKILL_START_WINDOW_MS
        {
            return;
        }
        if previous_gauge2 <= identity_gauge2 {
            return;
        }
        let consumed_cores = (previous_gauge2 - identity_gauge2) as i32;
        if consumed_cores <= 0 {
            return;
        }
        self.runtime_state.destroyer_recent_consumed_cores = consumed_cores;
        self.runtime_state.destroyer_recent_consumed_at_ms = timestamp;
    }

    pub fn set_identity_stance(&mut self, stance: u8) {
        self.runtime_state.identity_stance = stance;
    }

    pub fn load_from_snapshot(&mut self, snapshot: &InspectSnapshot, owner_id: u64, class_id: u32) {
        let runtime_state = self.capture_runtime_state_snapshot();
        self.clear();
        self.apply_runtime_state_snapshot(runtime_state);
        self.owner_id = owner_id;
        for (stat_id, value) in &snapshot.derived_stats.stat_pairs {
            if let Some(name) = stat_name_from_id(*stat_id) {
                self.add_stat_by_name(&name, *value, owner_id, "inspect_derived");
            }
        }

        self.ally_attack_power_power
            .add_self(snapshot.derived_stats.ally_attack_power_power, "inspect");
        self.ally_identity_damage_power
            .add_self(snapshot.derived_stats.ally_identity_damage_power, "inspect");
        self.ally_brand_power
            .add_self(snapshot.derived_stats.ally_brand_power, "inspect");
        for (skill_id, multiplier) in &snapshot
            .derived_stats
            .skill_attack_power_multiplier_by_skill
        {
            *self
                .skill_attack_power_multiplier
                .entry(*skill_id)
                .or_default() += *multiplier;
        }
        for (skill_id, multiplier) in &snapshot
            .derived_stats
            .skill_status_effect_multiplier_by_skill
        {
            *self
                .skill_status_effect_multiplier
                .entry(*skill_id)
                .or_default() += *multiplier;
        }
        for (group_id, multiplier) in &snapshot
            .derived_stats
            .skill_group_status_effect_multiplier_by_group
        {
            *self
                .skill_group_status_effect_multiplier
                .entry(*group_id)
                .or_default() += *multiplier;
        }
        for addon in &snapshot.derived_stats.deferred_addons {
            self.handle_external_addon(addon, owner_id, class_id, "inspect_deferred");
        }
        for feature in &snapshot.derived_stats.ability_features {
            self.add_ability_feature(
                &feature.feature_type,
                feature.level,
                &feature.values,
                owner_id,
            );
        }
        self.damage_conversion_type = snapshot.derived_stats.damage_conversion_type;
        self.apply_lal_base_inspect_bonuses();
        self.apply_lal_stat_pair_fallback(&snapshot.stat_pairs);
    }

    pub fn apply_skill_runtime_data(
        &mut self,
        skill_effect_id: u32,
        runtime_data: Option<&SkillRuntimeData>,
    ) {
        let Some(runtime_data) = runtime_data else {
            return;
        };
        self.critical_hit_rate
            .add_self(runtime_data.cached_critical_rate_bonus, "skill_tripods");
        self.critical_damage_rate.add_self(
            runtime_data.cached_critical_hit_damage_bonus,
            "skill_tripods",
        );
        self.attack_speed_rate
            .add_self(runtime_data.cached_attack_speed_bonus, "skill_tripods");
        if let Some(value) = runtime_data
            .cached_critical_rate_bonus_per_skill_effect
            .get(&skill_effect_id)
        {
            self.critical_hit_rate.add_self(*value, "skill_tripods");
        }
        if let Some(value) = runtime_data
            .cached_critical_hit_damage_bonus_per_skill_effect
            .get(&skill_effect_id)
        {
            self.critical_damage_rate.add_self(*value, "skill_tripods");
        }
    }

    pub fn apply_dynamic_effects(
        &mut self,
        skill_id: u32,
        skill_real_id: u32,
        skill_effect_id: u32,
        skill_groups: &[u32],
        skill_real_groups: &[u32],
        runtime_data: Option<&SkillRuntimeData>,
        target_entity: Option<&crate::live::entity_tracker::Entity>,
        hit_option: Option<&HitOption>,
        self_effects: &[StatusEffectDetails],
        target_effects: &[StatusEffectDetails],
        eval_tick_ms: i64,
    ) {
        self.evaluate_combat_effects(
            skill_id,
            skill_real_id,
            skill_effect_id,
            skill_groups,
            skill_real_groups,
            runtime_data,
            target_entity,
            hit_option,
            self_effects,
            target_effects,
        );
        self.evaluate_addon_skill_features(
            skill_id,
            skill_real_id,
            skill_groups,
            skill_real_groups,
            runtime_data,
        );
        self.evaluate_addon_ability_features(
            skill_id,
            skill_real_id,
            skill_groups,
            skill_real_groups,
            runtime_data,
            eval_tick_ms,
        );
    }

    pub fn evaluate_dynamic_effects(
        &mut self,
        skill_id: u32,
        skill_real_id: u32,
        skill_effect_id: u32,
        skill_groups: &[u32],
        skill_real_groups: &[u32],
        runtime_data: Option<&SkillRuntimeData>,
        target_entity: Option<&crate::live::entity_tracker::Entity>,
        hit_option: Option<&HitOption>,
        self_effects: &[StatusEffectDetails],
        target_effects: &[StatusEffectDetails],
        eval_tick_ms: i64,
    ) {
        self.apply_dynamic_effects(
            skill_id,
            skill_real_id,
            skill_effect_id,
            skill_groups,
            skill_real_groups,
            runtime_data,
            target_entity,
            hit_option,
            self_effects,
            target_effects,
            eval_tick_ms,
        );
    }

    pub fn add_stat_from_source(
        &mut self,
        name: &str,
        stat_value: i64,
        owner_id: u64,
        source: &str,
    ) {
        self.add_stat_by_name_with_priority(
            name,
            stat_value,
            owner_id,
            source,
            STAT_PRIORITY_DEFAULT,
        );
    }

    pub fn add_stat_from_source_with_priority(
        &mut self,
        name: &str,
        stat_value: i64,
        owner_id: u64,
        source: &str,
        source_priority: i32,
    ) {
        self.add_stat_by_name_with_priority(name, stat_value, owner_id, source, source_priority);
    }

    pub fn add_stat(&mut self, name: &str, stat_value: i64, owner_id: u64, source: &str) {
        self.add_stat_by_name_with_priority(
            name,
            stat_value,
            owner_id,
            source,
            STAT_PRIORITY_DEFAULT,
        );
    }

    pub fn add_value(&mut self, stat_data: &mut StatData, value: f64, owner_id: u64, source: &str) {
        stat_data.add(value, self.owner_id, owner_id, source);
    }

    pub fn add_value_with_priority(
        &mut self,
        stat_data: &mut StatData,
        value: f64,
        owner_id: u64,
        source: &str,
        source_priority: i32,
    ) {
        stat_data.add_with_priority(value, self.owner_id, owner_id, source, source_priority);
    }

    pub fn add_combat_effect_from_id(
        &mut self,
        combat_effect_id: u32,
        owner_id: u64,
        source: &str,
    ) {
        if let Some(combat_effect) = COMBAT_EFFECT_DATA.get(&(combat_effect_id as i32)) {
            for effect in &combat_effect.effects {
                self.active_combat_effects.push(ActiveCombatEffect {
                    combat_effect_id,
                    effect: effect.clone(),
                    owner_id,
                    source: source.to_string(),
                });
            }
            Self::sort_active_combat_effects(&mut self.active_combat_effects);
        }
    }

    pub fn add_combat_effect(&mut self, combat_effect_id: u32, owner_id: u64, source: &str) {
        self.add_combat_effect_from_id(combat_effect_id, owner_id, source);
    }

    pub fn handle_combat_effect(&mut self, combat_effect_id: u32, owner_id: u64, source: &str) {
        self.add_combat_effect_from_id(combat_effect_id, owner_id, source);
    }

    pub fn add_attack_power_amplify_multiplier(&mut self, value: f64, owner_id: u64, source: &str) {
        self.add_attack_power_amplify_multiplier_with_priority(
            value,
            owner_id,
            source,
            STAT_PRIORITY_DEFAULT,
        );
    }

    pub fn add_attack_power_amplify_multiplier_with_priority(
        &mut self,
        value: f64,
        owner_id: u64,
        source: &str,
        source_priority: i32,
    ) {
        self.ally_attack_power_power.add_with_priority(
            value,
            self.owner_id,
            owner_id,
            source,
            source_priority,
        );
    }

    pub fn add_external_addon_from_source(
        &mut self,
        addon_type: &str,
        stat_type: &str,
        key_index: u32,
        key_value: i64,
        owner_id: u64,
        class_id: u32,
        source: &str,
    ) {
        self.add_external_addon_from_source_with_priority(
            addon_type,
            stat_type,
            key_index,
            key_value,
            owner_id,
            class_id,
            source,
            STAT_PRIORITY_DEFAULT,
        );
    }

    pub fn add_external_addon_from_source_with_priority(
        &mut self,
        addon_type: &str,
        stat_type: &str,
        key_index: u32,
        key_value: i64,
        owner_id: u64,
        class_id: u32,
        source: &str,
        source_priority: i32,
    ) {
        let addon = crate::models::ExternalResourceAddon {
            addon_type: addon_type.to_string(),
            stat_type: stat_type.to_string(),
            key_index,
            key_value,
        };
        self.handle_external_addon_with_priority(
            &addon,
            owner_id,
            class_id,
            source,
            source_priority,
        );
    }

    pub fn handle_addon(
        &mut self,
        addon: &crate::models::ExternalResourceAddon,
        owner_id: u64,
        class_id: u32,
        source: &str,
    ) {
        self.handle_external_addon(addon, owner_id, class_id, source);
    }

    pub fn add_ability_feature(
        &mut self,
        feature_type: &str,
        level: u32,
        values: &[i64],
        owner_id: u64,
    ) {
        let normalized = normalize_feature_type(feature_type);
        if let Some(existing) = self
            .active_ability_features
            .iter_mut()
            .find(|feature| normalize_feature_type(&feature.feature_type) == normalized)
        {
            if level > existing.level {
                existing.feature_type = feature_type.to_string();
                existing.level = level;
                existing.values = values.to_vec();
                existing.owner_id = owner_id;
            }
            return;
        }
        self.active_ability_features.push(AbilityFeatureState {
            feature_type: feature_type.to_string(),
            level,
            values: values.to_vec(),
            owner_id,
        });
        Self::sort_active_ability_features(&mut self.active_ability_features);
    }

    pub fn add_addon_skill_feature(&mut self, feature_id: u32, owner_id: u64) {
        self.active_addon_skill_features
            .push(ActiveAddonSkillFeature {
                feature_id,
                owner_id,
            });
        Self::sort_active_addon_skill_features(&mut self.active_addon_skill_features);
    }

    fn evaluate_combat_effects(
        &mut self,
        skill_id: u32,
        skill_real_id: u32,
        skill_effect_id: u32,
        skill_groups: &[u32],
        skill_real_groups: &[u32],
        runtime_data: Option<&SkillRuntimeData>,
        target_entity: Option<&crate::live::entity_tracker::Entity>,
        hit_option: Option<&HitOption>,
        self_effects: &[StatusEffectDetails],
        target_effects: &[StatusEffectDetails],
    ) {
        let mut active_effects = self.active_combat_effects.clone();
        if let Some(runtime_data) = runtime_data {
            for (effect_skill_effect_id, effect_ids) in &runtime_data.added_chain_combat_effects {
                if *effect_skill_effect_id != 0 && *effect_skill_effect_id != skill_effect_id {
                    continue;
                }
                for effect_id in effect_ids {
                    if let Some(combat_effect) = COMBAT_EFFECT_DATA.get(&(*effect_id as i32)) {
                        for effect in &combat_effect.effects {
                            active_effects.push(ActiveCombatEffect {
                                combat_effect_id: *effect_id,
                                effect: effect.clone(),
                                owner_id: self.owner_id,
                                source: "skill_tripods".to_string(),
                            });
                        }
                    }
                }
            }

            for (effect_skill_effect_id, changes) in &runtime_data.changed_combat_effects {
                if *effect_skill_effect_id != 0 && *effect_skill_effect_id != skill_effect_id {
                    continue;
                }
                for active in &mut active_effects {
                    for change in changes {
                        if change.combat_effect_id != active.combat_effect_id {
                            continue;
                        }
                        for action in &mut active.effect.actions {
                            let changed_count = usize::min(change.values.len(), action.args.len());
                            for index in 0..changed_count {
                                if change.relative {
                                    action.args[index] = (action.args[index] as f64
                                        * (1.0 + (change.values[index] as f64 * 0.01)))
                                        as i32;
                                } else {
                                    action.args[index] += change.values[index] as i32;
                                }
                            }
                        }
                    }
                }
            }
        }
        Self::sort_active_combat_effects(&mut active_effects);

        for active in active_effects {
            if runtime_data.is_some_and(|runtime| {
                runtime
                    .removed_chain_combat_effects
                    .contains(&active.combat_effect_id)
            }) {
                continue;
            }
            if !self.combat_effect_conditions_satisfied(
                &active.effect.conditions,
                skill_id,
                skill_real_id,
                skill_effect_id,
                skill_groups,
                skill_real_groups,
                runtime_data,
                target_entity,
                hit_option,
                self_effects,
                target_effects,
            ) {
                continue;
            }
            for action in &active.effect.actions {
                self.evaluate_combat_effect_action(action, active.owner_id, &active.source);
            }
        }
    }

    fn combat_effect_conditions_satisfied(
        &self,
        conditions: &[CombatEffectCondition],
        skill_id: u32,
        skill_real_id: u32,
        skill_effect_id: u32,
        skill_groups: &[u32],
        skill_real_groups: &[u32],
        runtime_data: Option<&SkillRuntimeData>,
        target_entity: Option<&crate::live::entity_tracker::Entity>,
        hit_option: Option<&HitOption>,
        self_effects: &[StatusEffectDetails],
        target_effects: &[StatusEffectDetails],
    ) -> bool {
        let identity_category = self.resolve_skill_identity_category(skill_id, runtime_data);
        let identity_category_real =
            self.resolve_skill_identity_category(skill_real_id, runtime_data);
        let has_identity_gauge_runtime_state = self.has_reliable_identity_gauge_runtime_state();
        for condition in conditions {
            let satisfied = match condition.condition_type.as_str() {
                "current_skill" => {
                    skill_id == condition.arg as u32 || skill_real_id == condition.arg as u32
                }
                "current_skill_group" => {
                    let group_id = condition.arg as u32;
                    skill_groups.contains(&group_id) || skill_real_groups.contains(&group_id)
                }
                "skill_identity_category" => {
                    let target = condition.arg.to_string();
                    identity_category
                        .as_deref()
                        .is_some_and(|category| identity_category_matches(category, &target))
                        || identity_category_real
                            .as_deref()
                            .is_some_and(|category| identity_category_matches(category, &target))
                }
                "identity_stance" => self.runtime_state.identity_stance == condition.arg as u8,
                "identity_element_value_less" => {
                    if !has_identity_gauge_runtime_state {
                        true
                    } else {
                        let max_identity_percent = condition.arg as f64 / 1000.0;
                        (self
                            .try_get_skill_start_identity_gauge_snapshot(
                                skill_id,
                                skill_real_id,
                                runtime_data,
                            )
                            .map(|snapshot| snapshot.identity_gauge1)
                            .unwrap_or(self.runtime_state.identity_gauge1)
                            as f64
                            / 10000.0)
                            < max_identity_percent
                    }
                }
                "identity_element_value" => {
                    if !has_identity_gauge_runtime_state {
                        true
                    } else {
                        self.get_identity_element_value(skill_id, skill_real_id, runtime_data)
                            .is_some_and(|value| value == condition.arg)
                    }
                }
                "directional_attack" => hit_option.is_some_and(|option| match condition.arg {
                    1 => matches!(option, HitOption::BACK_ATTACK),
                    2 => matches!(option, HitOption::FRONTAL_ATTACK),
                    3 => matches!(option, HitOption::FLANK_ATTACK),
                    _ => false,
                }),
                "skill_effect_id" => skill_effect_id == condition.arg as u32,
                "not_skill_effect_id" => skill_effect_id != condition.arg as u32,
                "status_effect_on" => match condition.actor_type.as_str() {
                    "self" => self_effects
                        .iter()
                        .any(|effect| effect.status_effect_id == condition.arg as u32),
                    "target" => target_effects
                        .iter()
                        .any(|effect| effect.status_effect_id == condition.arg as u32),
                    _ => false,
                },
                "directional_skill_effect" => {
                    let base_mask = SKILL_DATA
                        .get(&skill_id)
                        .map(|skill| skill.directional_mask)
                        .unwrap_or_default();
                    let base_mask_real = SKILL_DATA
                        .get(&skill_real_id)
                        .map(|skill| skill.directional_mask)
                        .unwrap_or_default();
                    let runtime_mask =
                        runtime_data.and_then(|runtime| runtime.cached_directional_mask);
                    ((base_mask != 0 || base_mask_real != 0) && runtime_mask.unwrap_or(1) != 0)
                        || runtime_mask.is_some_and(|mask| mask > 0)
                }
                "npc_grade_greater" | "abnormal_move_immune" => true,
                "hp_less" => {
                    if condition.actor_type.eq_ignore_ascii_case("target") {
                        false
                    } else {
                        self.runtime_state.max_hp > 0
                            && (self.runtime_state.current_hp as f64
                                / self.runtime_state.max_hp as f64)
                                < condition.arg as f64
                    }
                }
                "npc_grade_less" => target_entity
                    .and_then(|entity| Self::npc_grade_rank(&entity.grade))
                    .is_some_and(|grade| grade < condition.arg),
                "abnormal_status" => false,
                _ => true,
            };
            if !satisfied {
                return false;
            }
        }
        true
    }

    fn evaluate_combat_effect_action(
        &mut self,
        action: &CombatEffectAction,
        owner_id: u64,
        source: &str,
    ) {
        let Some(value) = action.args.first().copied() else {
            return;
        };
        match action.action_type.as_str() {
            "modify_critical_multiplier" => self.critical_damage_rate.add(
                value as f64 / 10000.0,
                self.owner_id,
                owner_id,
                source,
            ),
            "modify_critical_ratio" => {
                self.critical_hit_rate
                    .add(value as f64 / 10000.0, self.owner_id, owner_id, source)
            }
            "modify_damage_when_critical" => self.critical_damage_rate_2.add(
                value as f64 / 10000.0,
                self.owner_id,
                owner_id,
                source,
            ),
            "modify_evolution_damage_multiplier" => {
                self.evolution_damage
                    .add(value as f64 / 10000.0, self.owner_id, owner_id, source)
            }
            "modify_damage" => {
                self.modify_damage_combat_effect.add(
                    value as f64 / 10000.0,
                    self.owner_id,
                    owner_id,
                    source,
                );
            }
            _ => {}
        }
    }

    fn evaluate_addon_skill_features(
        &mut self,
        skill_id: u32,
        skill_real_id: u32,
        skill_groups: &[u32],
        skill_real_groups: &[u32],
        runtime_data: Option<&SkillRuntimeData>,
    ) {
        let mut active_features = self.active_addon_skill_features.clone();
        if let Some(runtime_data) = runtime_data {
            for feature_id in &runtime_data.addon_skill_feature_ids {
                active_features.push(ActiveAddonSkillFeature {
                    feature_id: *feature_id,
                    owner_id: self.owner_id,
                });
            }
        }

        for active in active_features {
            let Some(feature) = EXTERNAL_ADDON_SKILL_FEATURE_DATA.get(&active.feature_id) else {
                continue;
            };
            let mut conditions_satisfied = true;
            if feature.skill_id != 0 {
                conditions_satisfied &=
                    feature.skill_id == skill_id || feature.skill_id == skill_real_id;
            } else if feature.skill_group_id != 0 {
                conditions_satisfied &= skill_groups.contains(&feature.skill_group_id)
                    || skill_real_groups.contains(&feature.skill_group_id);
            }
            if !conditions_satisfied {
                continue;
            }
            self.apply_addon_skill_feature(feature, active.owner_id);
        }
    }

    fn apply_addon_skill_feature(
        &mut self,
        feature: &crate::models::ExternalAddonSkillFeature,
        owner_id: u64,
    ) {
        match feature.feature_type.as_str() {
            "change_dam_critical_rate" => {
                if feature.parameters.len() > 1
                    && feature.parameter_type.eq_ignore_ascii_case("absolute")
                {
                    self.critical_hit_rate.add(
                        feature.parameters[1] as f64 / 10000.0,
                        self.owner_id,
                        owner_id,
                        feature.name.clone(),
                    );
                }
            }
            "change_dam_critical" => {
                if feature.parameters.len() > 1
                    && (feature.parameter_type.eq_ignore_ascii_case("absolute")
                        || feature.parameter_type.eq_ignore_ascii_case("relative"))
                {
                    self.critical_damage_rate.add(
                        feature.parameters[1] as f64 / 10000.0,
                        self.owner_id,
                        owner_id,
                        feature.name.clone(),
                    );
                }
            }
            "change_attack_stage_speed" => {
                if let Some(value) = feature.parameters.first() {
                    self.attack_speed_rate.add(
                        *value as f64 / 100.0,
                        self.owner_id,
                        owner_id,
                        feature.name.clone(),
                    );
                }
            }
            _ => {}
        }
    }

    fn evaluate_addon_ability_features(
        &mut self,
        skill_id: u32,
        skill_real_id: u32,
        skill_groups: &[u32],
        skill_real_groups: &[u32],
        runtime_data: Option<&SkillRuntimeData>,
        eval_tick_ms: i64,
    ) {
        let skill_identity_category = self.resolve_skill_identity_category(skill_id, runtime_data);
        let skill_real_identity_category =
            self.resolve_skill_identity_category(skill_real_id, runtime_data);
        let skill_level = runtime_data
            .map(|runtime| runtime.skill_level.max(1))
            .unwrap_or(1) as u32;
        let mut critical_rate_cap_over_to_evo_damage_multiplier = 0.0f64;
        let mut critical_rate_cap_over_to_damage_multiplier_cap = 0.0f64;
        let mut critical_hit_to_damage_rates = Vec::<(f64, String)>::new();
        let matches_skill_or_real_id =
            |target_skill_id: u32| skill_id == target_skill_id || skill_real_id == target_skill_id;
        let matches_skill_or_real_group = |target_group_id: u32| {
            skill_groups.contains(&target_group_id) || skill_real_groups.contains(&target_group_id)
        };
        let matches_identity_category = |expected: &[&str]| {
            skill_identity_category.as_deref().is_some_and(|category| {
                expected
                    .iter()
                    .any(|expected| identity_category_matches(category, expected))
            }) || skill_real_identity_category
                .as_deref()
                .is_some_and(|category| {
                    expected
                        .iter()
                        .any(|expected| identity_category_matches(category, expected))
                })
        };

        for active in self.active_ability_features.clone() {
            match normalize_feature_type(&active.feature_type) {
                "matt_critical" => {
                    if active.values.len() > 2 {
                        self.critical_damage_rate.add(
                            active.values[2] as f64 / 10000.0,
                            self.owner_id,
                            active.owner_id,
                            active.feature_type.clone(),
                        );
                    }
                }
                "dagger_critical" => {
                    if let Some(value) = active.values.first() {
                        self.critical_hit_rate.add(
                            *value as f64 / 10000.0,
                            self.owner_id,
                            active.owner_id,
                            active.feature_type.clone(),
                        );
                    }
                }
                "spirit_absorption" => {
                    if let Some(value) = active.values.first() {
                        let rate = *value as f64 / 10000.0;
                        self.move_speed_rate.add(
                            rate,
                            self.owner_id,
                            active.owner_id,
                            active.feature_type.clone(),
                        );
                        self.attack_speed_rate.add(
                            rate,
                            self.owner_id,
                            active.owner_id,
                            active.feature_type.clone(),
                        );
                    }
                }
                "gravity_glove" => {
                    if let Some(value) = active.values.first() {
                        self.attack_speed_rate.add(
                            -(*value as f64 / 10000.0),
                            self.owner_id,
                            active.owner_id,
                            active.feature_type.clone(),
                        );
                    }
                }
                "blocky_thorn" => {
                    if active.values.len() > 2 {
                        self.critical_hit_rate_cap = active.values[0] as f64 / 10000.0;
                        critical_rate_cap_over_to_evo_damage_multiplier =
                            active.values[1] as f64 / 10000.0;
                        critical_rate_cap_over_to_damage_multiplier_cap =
                            active.values[2] as f64 / 10000.0;
                    }
                }
                "identity_soul_eater_tricall" => {
                    if let Some(value) = active.values.first() {
                        self.critical_hit_rate.add(
                            *value as f64 / 10000.0,
                            self.owner_id,
                            active.owner_id,
                            active.feature_type.clone(),
                        );
                    }
                }
                "identity_summoner_senior" => {
                    if matches_identity_category(&[
                        "26",
                        "27",
                        "summoner_normal",
                        "summoner_ancient",
                    ]) && active.values.len() > 2
                    {
                        self.critical_hit_rate.add(
                            active.values[2] as f64 / 10000.0,
                            self.owner_id,
                            active.owner_id,
                            active.feature_type.clone(),
                        );
                    }
                }
                "identity_hawkeye_secondcolleague" => {
                    if matches_identity_category(&["25", "hawkeye_summon"])
                        && !active.values.is_empty()
                    {
                        self.critical_hit_rate.add(
                            active.values[0] as f64 / 10000.0,
                            self.owner_id,
                            active.owner_id,
                            active.feature_type.clone(),
                        );
                    }
                }
                "identity_infighter_male_sura" => {
                    if (matches_skill_or_real_id(47020) || matches_skill_or_real_group(36000))
                        && active.values.len() > 2
                    {
                        critical_hit_to_damage_rates.push((
                            active.values[2] as f64 / 10000.0,
                            active.feature_type.clone(),
                        ));
                    }
                }
                "identity_infighter_male_dusk" => {
                    if skill_id == 47950 {
                        self.critical_hit_rate.add(
                            0.15,
                            self.owner_id,
                            active.owner_id,
                            active.feature_type.clone(),
                        );
                    }
                    if active.values.len() > 5 && skill_id == active.values[5] as u32 {
                        self.critical_hit_rate.add(
                            0.15,
                            self.owner_id,
                            active.owner_id,
                            active.feature_type.clone(),
                        );
                    }
                }
                "identity_weather_artist_storm" => {
                    if active.values.len() > 4 {
                        let atk_speed_to_crit_dmg_percent = active.values[3] as f64 / 10000.0;
                        let move_speed_to_crit_rate_percent = active.values[4] as f64 / 10000.0;
                        self.add_to_composite_stat_with_function(
                            &self.move_speed_rate.clone(),
                            MOVE_SPEED_ATTACK_SPEED_CAP,
                            1000.0,
                            move_speed_to_crit_rate_percent,
                            &active.feature_type,
                            CompositeTarget::CriticalRate,
                            CompositeOwnership::ForceSelf,
                        );
                        self.add_to_composite_stat_with_function(
                            &self.attack_speed_rate.clone(),
                            MOVE_SPEED_ATTACK_SPEED_CAP,
                            1000.0,
                            atk_speed_to_crit_dmg_percent,
                            &active.feature_type,
                            CompositeTarget::CriticalDamage,
                            CompositeOwnership::ForceSelf,
                        );
                    }
                }
                "mana_incinerator" => {
                    if active.values.len() > 2 {
                        let cap_evo_dmg = active.values[2] as f64 / 10000.0;
                        let mana_cost_to_evo_dmg = active.values[1] as f64 / 10000.0;
                        let mana_per_point = active.values[0].max(1) as i32;
                        let mana_cost = SKILL_DATA
                            .get(&skill_real_id)
                            .and_then(|skill| skill.levels.get(&skill_level))
                            .or_else(|| {
                                SKILL_DATA
                                    .get(&skill_id)
                                    .and_then(|skill| skill.levels.get(&skill_level))
                            })
                            .map(|level| level.mana_cost)
                            .unwrap_or_default();
                        let bonus_evo_dmg = ((mana_cost / mana_per_point) as f64
                            * mana_cost_to_evo_dmg)
                            .min(cap_evo_dmg);
                        self.evolution_damage.add(
                            bonus_evo_dmg,
                            self.owner_id,
                            active.owner_id,
                            active.feature_type.clone(),
                        );
                    }
                }
                "sonic_boom" => {
                    if active.values.len() > 3 {
                        let default_percent = active.values[0] as f64 / 10000.0;
                        let over_cap_bonus = active.values[1] as f64 / 10000.0;
                        let over_cap_percent = active.values[2] as f64 / 10000.0;
                        let cap_evo_dmg = active.values[3] as f64 / 10000.0;
                        let move_speed_bonus = self.move_speed_rate.value();
                        let attack_speed_bonus = self.attack_speed_rate.value();
                        let mut local_evo_dmg = StatData::default();
                        let base_bonus = ((move_speed_bonus.min(MOVE_SPEED_ATTACK_SPEED_CAP)
                            + attack_speed_bonus.min(MOVE_SPEED_ATTACK_SPEED_CAP))
                            * default_percent)
                            .max(0.0);
                        local_evo_dmg.add_self(base_bonus, active.feature_type.clone());
                        if move_speed_bonus > MOVE_SPEED_ATTACK_SPEED_CAP
                            && attack_speed_bonus > MOVE_SPEED_ATTACK_SPEED_CAP
                        {
                            local_evo_dmg.add_self(over_cap_bonus, active.feature_type.clone());
                            let mut attack_speed_accumulator = 0.0;
                            let attack_speed_evo_left = cap_evo_dmg - local_evo_dmg.value();
                            Self::add_self_to_stat_with_function(
                                &mut local_evo_dmg,
                                &self.attack_speed_rate,
                                1000.0,
                                attack_speed_evo_left,
                                |value| {
                                    attack_speed_accumulator += value;
                                    if attack_speed_accumulator > MOVE_SPEED_ATTACK_SPEED_CAP {
                                        (attack_speed_accumulator - MOVE_SPEED_ATTACK_SPEED_CAP)
                                            .min(value)
                                            * over_cap_percent
                                    } else {
                                        0.0
                                    }
                                },
                                &active.feature_type,
                            );
                            let mut move_speed_accumulator = 0.0;
                            let move_speed_evo_left = cap_evo_dmg - local_evo_dmg.value();
                            Self::add_self_to_stat_with_function(
                                &mut local_evo_dmg,
                                &self.move_speed_rate,
                                1000.0,
                                move_speed_evo_left,
                                |value| {
                                    move_speed_accumulator += value;
                                    if move_speed_accumulator > MOVE_SPEED_ATTACK_SPEED_CAP {
                                        (move_speed_accumulator - MOVE_SPEED_ATTACK_SPEED_CAP)
                                            .min(value)
                                            * over_cap_percent
                                    } else {
                                        0.0
                                    }
                                },
                                &active.feature_type,
                            );
                        }
                        self.evolution_damage = self.evolution_damage.added(&local_evo_dmg);
                        self.evolution_damage_bonus_from_supersonic_breakthrough = local_evo_dmg;
                    }
                }
                "troop_leader" => {
                    if let Some(value) = active.values.first() {
                        self.add_to_composite_stat_with_function(
                            &self.move_speed_rate.clone(),
                            MOVE_SPEED_ATTACK_SPEED_CAP,
                            1000.0,
                            *value as f64 / 10000.0,
                            &active.feature_type,
                            CompositeTarget::MoveSpeedDamage,
                            CompositeOwnership::ForceSelf,
                        );
                    }
                }
                "identity_warlord_lonely_knight" => {
                    if matches_identity_category(&["41", "warlord_lance"])
                        && active.values.len() > 4
                    {
                        critical_hit_to_damage_rates.push((
                            active.values[4] as f64 / 10000.0,
                            active.feature_type.clone(),
                        ));
                    }
                }
                "identity_holyknight_female_radiant" => {
                    if active.values.len() > 7 {
                        critical_hit_to_damage_rates.push((
                            active.values[7] as f64 / 10000.0,
                            format!("{}.AllDamage", active.feature_type),
                        ));
                    }
                    if matches_identity_category(&["71", "holyknight_female_identity_x"])
                        && active.values.len() > 2
                    {
                        critical_hit_to_damage_rates.push((
                            active.values[2] as f64 / 10000.0,
                            format!("{}.FinalSplendor", active.feature_type),
                        ));
                    }
                }
                "identity_lancemaster_climax" => {}
                "identity_destroyer_angry_hammer" => {
                    if matches_identity_category(&[
                        DESTROYER_RELEASE_IDENTITY_CATEGORY_ID,
                        DESTROYER_RELEASE_IDENTITY_CATEGORY,
                    ]) && active.values.len() > 1
                    {
                        let consumed_cores = if self.has_reliable_identity_gauge_runtime_state() {
                            let Some(consumed_cores) =
                                self.get_recent_destroyer_consumed_cores(eval_tick_ms)
                            else {
                                continue;
                            };
                            consumed_cores
                        } else {
                            3
                        };
                        self.critical_hit_rate.add(
                            consumed_cores as f64 * (active.values[0] as f64 / 10000.0),
                            self.owner_id,
                            active.owner_id,
                            active.feature_type.clone(),
                        );
                        self.critical_damage_rate.add(
                            consumed_cores as f64 * (active.values[1] as f64 / 10000.0),
                            self.owner_id,
                            active.owner_id,
                            active.feature_type.clone(),
                        );
                    }
                }
                "identity_destroyer_gravity_up" => {
                    let is_destroyer_basic_attack =
                        matches_skill_or_real_group(DESTROYER_HYPERGRAVITY_VORTEX_SKILL_GROUP_ID);
                    if is_destroyer_basic_attack && active.values.len() > 1 {
                        self.critical_hit_rate.add(
                            active.values[1] as f64 / 10000.0,
                            self.owner_id,
                            active.owner_id,
                            active.feature_type.clone(),
                        );
                    }
                }
                "identity_destroyer_transform" => {
                    if self.try_is_destroyer_transform_mode() == Some(true)
                        && !active.values.is_empty()
                    {
                        critical_hit_to_damage_rates.push((
                            active.values[0] as f64 / 10000.0,
                            active.feature_type.clone(),
                        ));
                    }
                }
                "identity_blaster_free_bomb_ardment" => {
                    if matches_skill_or_real_group(2300400) && active.values.len() > 1 {
                        self.critical_hit_rate.add(
                            active.values[1] as f64 / 10000.0,
                            self.owner_id,
                            active.owner_id,
                            active.feature_type.clone(),
                        );
                    }
                }
                "identity_devil_hunter_female_hunt_time" => {
                    if let Some(value) = active.values.first() {
                        self.critical_hit_rate.add(
                            *value as f64 / 10000.0,
                            self.owner_id,
                            active.owner_id,
                            active.feature_type.clone(),
                        );
                    }
                }
                _ => {}
            }
        }

        for (rate, source) in critical_hit_to_damage_rates {
            self.add_to_composite_stat_with_function(
                &self.critical_hit_rate.clone(),
                self.critical_hit_rate_cap,
                rate,
                rate,
                &source,
                CompositeTarget::CriticalHitToDamage,
                CompositeOwnership::PreserveSource,
            );
        }

        if critical_rate_cap_over_to_evo_damage_multiplier > 0.0 {
            self.add_to_composite_stat_over_threshold(
                &self.critical_hit_rate.clone(),
                self.critical_hit_rate_cap,
                critical_rate_cap_over_to_damage_multiplier_cap,
                critical_rate_cap_over_to_evo_damage_multiplier,
                "blocky_thorn",
                CompositeTarget::CriticalRateCapOverToEvolution,
                CompositeOwnership::PreserveSource,
            );
        }
    }

    fn resolve_skill_identity_category(
        &self,
        skill_id: u32,
        runtime_data: Option<&SkillRuntimeData>,
    ) -> Option<String> {
        runtime_data
            .and_then(|runtime| runtime.cached_identity_category.clone())
            .or_else(|| {
                SKILL_DATA
                    .get(&skill_id)
                    .and_then(|skill| skill.identity_category.clone())
            })
    }

    fn try_get_skill_start_identity_gauge_snapshot(
        &self,
        skill_id: u32,
        skill_real_id: u32,
        runtime_data: Option<&SkillRuntimeData>,
    ) -> Option<IdentityGaugeSnapshot> {
        if let Some(runtime_data) = runtime_data
            && runtime_data.identity_gauge1_at_start.is_some()
        {
            return Some(IdentityGaugeSnapshot {
                identity_gauge1: runtime_data.identity_gauge1_at_start.unwrap_or_default(),
                identity_gauge2: runtime_data.identity_gauge2_at_start.unwrap_or_default(),
                identity_gauge3: runtime_data.identity_gauge3_at_start.unwrap_or_default(),
            });
        }
        self.runtime_state
            .skill_start_identity_gauge_snapshots
            .get(&skill_real_id)
            .copied()
            .or_else(|| {
                self.runtime_state
                    .skill_start_identity_gauge_snapshots
                    .get(&skill_id)
                    .copied()
            })
    }

    fn get_identity_element_value(
        &self,
        skill_id: u32,
        skill_real_id: u32,
        runtime_data: Option<&SkillRuntimeData>,
    ) -> Option<i32> {
        let gauge_1 = self
            .try_get_skill_start_identity_gauge_snapshot(skill_id, skill_real_id, runtime_data)
            .map(|snapshot| snapshot.identity_gauge1)
            .unwrap_or(self.runtime_state.identity_gauge1);
        let resolved_skill_id = if skill_real_id != 0 {
            skill_real_id
        } else {
            skill_id
        };
        match resolved_skill_id {
            27950 => Some(if gauge_1 >= 2000 { 1 } else { 0 }),
            34640 => Some(if gauge_1 >= 10_000 { 1 } else { 0 }),
            _ => None,
        }
    }

    fn npc_grade_rank(grade: &str) -> Option<i32> {
        match grade.to_ascii_lowercase().as_str() {
            "none" => Some(0),
            "underling" => Some(1),
            "normal" => Some(2),
            "elite" => Some(3),
            "named" => Some(4),
            "seed" => Some(5),
            "boss" => Some(6),
            "raid" => Some(7),
            "lucky" => Some(8),
            "epic_raid" => Some(9),
            "commander" => Some(10),
            _ => None,
        }
    }

    fn get_recent_destroyer_consumed_cores(&self, eval_tick_ms: i64) -> Option<i32> {
        if self.runtime_state.destroyer_recent_consumed_cores <= 0 {
            return None;
        }
        if eval_tick_ms < self.runtime_state.destroyer_recent_consumed_at_ms {
            return None;
        }
        if eval_tick_ms - self.runtime_state.destroyer_recent_consumed_at_ms
            > DESTROYER_RECENT_CONSUMED_CORE_WINDOW_MS
        {
            return None;
        }
        Some(self.runtime_state.destroyer_recent_consumed_cores)
    }

    fn try_is_destroyer_transform_mode(&self) -> Option<bool> {
        match self.runtime_state.identity_stance {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }

    fn has_reliable_identity_gauge_runtime_state(&self) -> bool {
        self.runtime_state.identity_runtime_reliable
    }

    fn add_self_to_stat_with_function<F>(
        add_to_stat: &mut StatData,
        source_stat: &StatData,
        source_stat_max_to_add: f64,
        add_to_stat_max_to_add: f64,
        mut calculate_stat_value_func: F,
        source: &str,
    ) where
        F: FnMut(f64) -> f64,
    {
        let mut to_add = source_stat.self_value().min(source_stat_max_to_add);
        let mut stat_left_to_add = source_stat_max_to_add - to_add;
        let mut added_value = calculate_stat_value_func(to_add).min(add_to_stat_max_to_add);
        let mut added_value_left_to_add = add_to_stat_max_to_add - added_value;
        add_to_stat.add_self(added_value, source);

        for modification in &source_stat.modified_values {
            to_add = stat_left_to_add.min(modification.value(source_stat.operation_type));
            stat_left_to_add -= to_add;
            added_value = calculate_stat_value_func(to_add).min(added_value_left_to_add);
            added_value_left_to_add -= added_value;
            add_to_stat.add_self(added_value, source);
        }
    }

    fn add_to_composite_stat_with_function(
        &mut self,
        source_stat: &StatData,
        source_stat_max_to_add: f64,
        add_to_stat_max_to_add: f64,
        rate: f64,
        source: &str,
        target: CompositeTarget,
        ownership: CompositeOwnership,
    ) {
        let mut stat_left_to_add = source_stat_max_to_add;
        let mut added_left = add_to_stat_max_to_add;

        let apply_chunk = |this: &mut Self,
                           value: f64,
                           owner_id: u64,
                           source_label: &str,
                           source_priority: i32,
                           added_left: &mut f64| {
            if value <= 0.0 || *added_left <= 0.0 {
                return;
            }
            let added_value = match target {
                CompositeTarget::CriticalRate => value * rate,
                CompositeTarget::CriticalDamage => value * rate,
                CompositeTarget::MoveSpeedDamage => {
                    calculate_move_speed_to_damage_bonus(value, rate, true)
                }
                CompositeTarget::CriticalHitToDamage => value * rate,
                CompositeTarget::CriticalRateCapOverToEvolution => value * rate,
            }
            .min(*added_left);
            *added_left -= added_value;
            let output_owner_id = match ownership {
                CompositeOwnership::PreserveSource => owner_id,
                CompositeOwnership::ForceSelf => this.owner_id,
            };
            let output_priority = match ownership {
                CompositeOwnership::PreserveSource => source_priority,
                CompositeOwnership::ForceSelf => STAT_PRIORITY_DEFAULT,
            };
            match target {
                CompositeTarget::CriticalRate => this.critical_hit_rate.add_with_priority(
                    added_value,
                    this.owner_id,
                    output_owner_id,
                    source_label,
                    output_priority,
                ),
                CompositeTarget::CriticalDamage => this.critical_damage_rate.add_with_priority(
                    added_value,
                    this.owner_id,
                    output_owner_id,
                    source_label,
                    output_priority,
                ),
                CompositeTarget::CriticalRateCapOverToEvolution => {
                    this.evolution_damage.add_with_priority(
                        added_value,
                        this.owner_id,
                        output_owner_id,
                        source_label,
                        output_priority,
                    );
                    this.evolution_damage_bonus_from_blunt_thorn
                        .add_with_priority(
                            added_value,
                            this.owner_id,
                            output_owner_id,
                            source_label,
                            output_priority,
                        );
                }
                CompositeTarget::MoveSpeedDamage => {
                    this.move_speed_to_damage_rate.add_with_priority(
                        added_value,
                        this.owner_id,
                        output_owner_id,
                        source_label,
                        output_priority,
                    )
                }
                CompositeTarget::CriticalHitToDamage => {
                    this.critical_hit_to_damage_rate.add_with_priority(
                        added_value,
                        this.owner_id,
                        output_owner_id,
                        source_label,
                        output_priority,
                    )
                }
            }
        };

        let self_value = source_stat.self_value().min(stat_left_to_add);
        stat_left_to_add -= self_value;
        apply_chunk(
            self,
            self_value,
            self.owner_id,
            source,
            STAT_PRIORITY_DEFAULT,
            &mut added_left,
        );

        for modification in &source_stat.modified_values {
            if stat_left_to_add <= 0.0 || added_left <= 0.0 {
                break;
            }
            let to_add = modification
                .value(source_stat.operation_type)
                .min(stat_left_to_add);
            stat_left_to_add -= to_add;
            apply_chunk(
                self,
                to_add,
                modification.source_entity_id,
                source,
                modification.source_priority,
                &mut added_left,
            );
        }
    }

    fn add_to_composite_stat_over_threshold(
        &mut self,
        source_stat: &StatData,
        threshold: f64,
        add_to_stat_max_to_add: f64,
        rate: f64,
        source: &str,
        target: CompositeTarget,
        ownership: CompositeOwnership,
    ) {
        let mut accumulator = 0.0;
        let mut added_left = add_to_stat_max_to_add;
        let apply_chunk = |this: &mut Self,
                           value: f64,
                           owner_id: u64,
                           source_label: &str,
                           source_priority: i32,
                           added_left: &mut f64,
                           accumulator: &mut f64| {
            if value <= 0.0 || *added_left <= 0.0 {
                return;
            }
            *accumulator += value;
            if *accumulator <= threshold {
                return;
            }
            let over_cap = (*accumulator - threshold).min(value);
            if over_cap <= 0.0 {
                return;
            }
            let added_value = (over_cap * rate).min(*added_left);
            *added_left -= added_value;
            let output_owner_id = match ownership {
                CompositeOwnership::PreserveSource => owner_id,
                CompositeOwnership::ForceSelf => this.owner_id,
            };
            let output_priority = match ownership {
                CompositeOwnership::PreserveSource => source_priority,
                CompositeOwnership::ForceSelf => STAT_PRIORITY_DEFAULT,
            };
            match target {
                CompositeTarget::CriticalRateCapOverToEvolution => {
                    this.evolution_damage.add_with_priority(
                        added_value,
                        this.owner_id,
                        output_owner_id,
                        source_label,
                        output_priority,
                    );
                    this.evolution_damage_bonus_from_blunt_thorn
                        .add_with_priority(
                            added_value,
                            this.owner_id,
                            output_owner_id,
                            source_label,
                            output_priority,
                        );
                }
                _ => {}
            }
        };

        apply_chunk(
            self,
            source_stat.self_value(),
            self.owner_id,
            source,
            STAT_PRIORITY_DEFAULT,
            &mut added_left,
            &mut accumulator,
        );
        for modification in &source_stat.modified_values {
            if added_left <= 0.0 {
                break;
            }
            apply_chunk(
                self,
                modification.value(source_stat.operation_type),
                modification.source_entity_id,
                source,
                modification.source_priority,
                &mut added_left,
                &mut accumulator,
            );
        }
    }

    pub fn calculate_base_attack_power(&self) -> StatData {
        let mut base_attack_power = StatData::default();
        let weapon_power = self.weapon_power.value();
        let main_stat = self.get_main_stat_calculated().value();
        if weapon_power <= 0.0 || main_stat <= 0.0 {
            return base_attack_power;
        }
        base_attack_power.add_self(
            (weapon_power * (1.0 + self.weapon_dam_x.value()) * (main_stat / 6.0)).sqrt(),
            "base_ap",
        );
        base_attack_power.add_self(
            base_attack_power.value() * self.attack_power_base_multiplier.value(),
            "attack_power_base_multiplier_",
        );
        base_attack_power
    }

    pub fn calculate_self_attack_power(&self) -> f64 {
        self.calculate_base_attack_power()
            .added(&self.attack_power_addend_2)
            .value()
    }

    pub fn calculate_stat_sheet_attack_power(&self) -> f64 {
        self.calculate_attack_power_pre_multipliers().value()
            * (1.0 + self.attack_power_rate.value())
    }

    pub fn calculate_move_speed_to_damage_bonus(
        move_speed_rate: f64,
        ms_to_damage_multiplier: f64,
        capped_ms: bool,
    ) -> f64 {
        calculate_move_speed_to_damage_bonus(move_speed_rate, ms_to_damage_multiplier, capped_ms)
    }

    pub fn calculate_attack_power_pre_multipliers(&self) -> StatData {
        self.calculate_base_attack_power()
            .added(&self.attack_power_addend_2)
            .added(&self.attack_power_addend)
    }

    pub fn calculate_final_attack_power(
        &self,
        hit_option: &HitOption,
        hit_flag: &HitFlag,
        damage_attr: Option<u8>,
        damage_type: u8,
        is_hyper_awakening: bool,
        is_affected_by_buffs: bool,
        can_crit: bool,
        include_average_crit: bool,
    ) -> StatData {
        if is_hyper_awakening {
            let mut attack_power = StatData::default();
            attack_power.set_self(1.0, "base");
            return attack_power.mad(&self.ultimate_awakening_damage_rate);
        }
        if !is_affected_by_buffs {
            let mut attack_power = StatData::default();
            attack_power.set_self(1.0, "base");
            return attack_power;
        }

        let mut attack_power = self
            .calculate_attack_power_pre_multipliers()
            .mad(&self.evolution_damage)
            .mad(&self.attack_power_rate)
            .mad(&self.skill_damage_sub_rate_1)
            .mad(&self.skill_damage_sub_rate_2)
            .mad(&self.attack_power_sub_rate_1)
            .mad(&self.attack_power_sub_rate_2)
            .mad(&self.outgoing_dmg_stat_amp)
            .mad(&self.skill_damage_amplify)
            .mad(&self.skill_damage_rate);
        if let Some(index) = damage_attr_to_index(damage_attr) {
            attack_power = attack_power.mad(&self.damage_attr_rates[index]);
        }
        attack_power = attack_power
            .mad(&self.modify_damage_combat_effect)
            .mad(&self.move_speed_to_damage_rate)
            .mad(&self.critical_hit_to_damage_rate);
        if let Some(index) = damage_attr_to_index(damage_attr) {
            attack_power = attack_power.mad(&self.damage_attr_amplifications[index]);
        }
        attack_power = if damage_type == 0 {
            attack_power.mad(&self.physical_defense_break)
        } else {
            attack_power.mad(&self.magical_defense_break)
        };

        match hit_option {
            HitOption::FRONTAL_ATTACK => {
                attack_power = attack_power.mad(&self.front_attack_amplify)
            }
            HitOption::BACK_ATTACK => attack_power = attack_power.mad(&self.back_attack_amplify),
            _ => {}
        }

        if can_crit {
            if include_average_crit {
                attack_power =
                    self.add_average_critical_damage_to_attack_power(&attack_power, damage_type);
            } else if is_critical(hit_flag) {
                attack_power = attack_power
                    .mad(&self.critical_damage_rate)
                    .mad(&self.critical_damage_rate_2);
                attack_power = if damage_type == 0 {
                    attack_power.mad(&self.physical_critical_damage_amplify)
                } else {
                    attack_power.mad(&self.magical_critical_damage_amplify)
                };
            }
        }

        attack_power
    }

    fn add_average_critical_damage_to_attack_power(
        &self,
        attack_power: &StatData,
        damage_type: u8,
    ) -> StatData {
        let mut crit_rate_capped = self.critical_hit_rate.clone();
        crit_rate_capped.clamp(0.0, self.critical_hit_rate_cap);

        let crit_damage_amp = if damage_type == 0 {
            &self.physical_critical_damage_amplify
        } else {
            &self.magical_critical_damage_amplify
        };
        let mut one = StatData::default();
        one.set_self(1.0, "base");
        let full_crit_multiplier = one
            .mad(&self.critical_damage_rate)
            .mad(&self.critical_damage_rate_2)
            .mad(crit_damage_amp);
        let crit_bonus_over_noncrit = full_crit_multiplier.subtracted(&one);

        attack_power.mad(&crit_rate_capped.multiplied_by_stat(&crit_bonus_over_noncrit))
    }

    pub fn resolve_damage_attr(&self, damage_attr: Option<u8>) -> Option<u8> {
        let resolved = self
            .damage_conversion_type
            .filter(|damage_attr| *damage_attr != 0)
            .or(damage_attr);
        Some(match resolved {
            Some(0..=7) => resolved.unwrap_or(0),
            _ => 0,
        })
    }

    pub fn get_damage_increase_contributed_from_entity_id(
        &mut self,
        entity_id: u64,
        total_attack_power_original: f64,
        hit_option: &HitOption,
        hit_flag: &HitFlag,
        damage_attr: Option<u8>,
        damage_type: u8,
        is_hyper_awakening: bool,
        is_affected_by_buffs: bool,
        can_crit: bool,
        include_average_crit: bool,
    ) -> f64 {
        let without_value = self.recompute_attack_power_without_entity(
            entity_id,
            hit_option,
            hit_flag,
            damage_attr,
            damage_type,
            is_hyper_awakening,
            is_affected_by_buffs,
            can_crit,
            include_average_crit,
        );
        let delta = total_attack_power_original - without_value;
        delta / without_value
    }

    pub fn get_damage_portion_contributed_from_entity_id(
        &mut self,
        entity_id: u64,
        total_attack_power_original: f64,
        hit_option: &HitOption,
        hit_flag: &HitFlag,
        damage_attr: Option<u8>,
        damage_type: u8,
        is_hyper_awakening: bool,
        is_affected_by_buffs: bool,
        can_crit: bool,
        include_average_crit: bool,
    ) -> f64 {
        let without_value = self.recompute_attack_power_without_entity(
            entity_id,
            hit_option,
            hit_flag,
            damage_attr,
            damage_type,
            is_hyper_awakening,
            is_affected_by_buffs,
            can_crit,
            include_average_crit,
        );
        let delta = total_attack_power_original - without_value;
        delta / total_attack_power_original
    }

    /// Recompute attack power as if `entity_id` had contributed nothing, then
    /// restore self to its original state. Avoids cloning PlayerStats by swapping
    /// each affected `values: Vec<StatDataValue>` out via mem::take and putting
    /// it back after the read-only computation.
    fn recompute_attack_power_without_entity(
        &mut self,
        entity_id: u64,
        hit_option: &HitOption,
        hit_flag: &HitFlag,
        damage_attr: Option<u8>,
        damage_type: u8,
        is_hyper_awakening: bool,
        is_affected_by_buffs: bool,
        can_crit: bool,
        include_average_crit: bool,
    ) -> f64 {
        let owner_id = self.owner_id;
        let mut snapshots: Vec<(usize, Vec<StatDataValue>)> = Vec::new();
        for stat_idx in self.iterate_stat_datas() {
            if let Some(taken) = self
                .get_stat_data_ref_mut(stat_idx)
                .take_values_for_entity(owner_id, entity_id)
            {
                snapshots.push((stat_idx, taken));
            }
        }

        let without_value = self
            .calculate_final_attack_power(
                hit_option,
                hit_flag,
                damage_attr,
                damage_type,
                is_hyper_awakening,
                is_affected_by_buffs,
                can_crit,
                include_average_crit,
            )
            .value();

        for (stat_idx, original) in snapshots {
            self.get_stat_data_ref_mut(stat_idx)
                .restore_values_for_entity(owner_id, entity_id, original);
        }

        without_value
    }

    pub fn get_damage_portions_contributed_from_all_entities(
        &mut self,
        total_attack_power_original: f64,
        hit_option: &HitOption,
        hit_flag: &HitFlag,
        damage_attr: Option<u8>,
        damage_type: u8,
        is_hyper_awakening: bool,
        is_affected_by_buffs: bool,
        can_crit: bool,
        include_average_crit: bool,
    ) -> Vec<(f64, u64)> {
        let entities = self.get_all_contributing_entity_ids();
        let mut increases = Vec::with_capacity(entities.len());
        for entity_id in &entities {
            increases.push(self.get_damage_increase_contributed_from_entity_id(
                *entity_id,
                total_attack_power_original,
                hit_option,
                hit_flag,
                damage_attr,
                damage_type,
                is_hyper_awakening,
                is_affected_by_buffs,
                can_crit,
                true,
            ));
        }
        let splits = get_damage_splits(1.0, &increases);
        let mut out = Vec::with_capacity(splits.len());
        out.push((splits.first().copied().unwrap_or_default(), self.owner_id));
        for (index, entity_id) in entities.into_iter().enumerate() {
            out.push((
                splits.get(index + 1).copied().unwrap_or_default(),
                entity_id,
            ));
        }
        out
    }

    pub fn debug_dump_value(&self) -> Value {
        let stat_datas = self
            .iterate_stat_datas()
            .into_iter()
            .map(|index| {
                let stat = self.get_stat_data_ref(index);
                (self.get_stat_data_name(index), debug_stat_data_value(stat))
            })
            .collect::<Vec<_>>();

        json!({
            "owner_id": self.owner_id,
            "summary": {
                "weapon_power": debug_float_value(self.weapon_power.value()),
                "main_stat": debug_float_value(self.get_main_stat_calculated().value()),
                "base_attack_power": debug_float_value(self.calculate_base_attack_power().value()),
                "attack_power_pre_multipliers": debug_float_value(self.calculate_attack_power_pre_multipliers().value()),
                "self_attack_power": debug_float_value(self.calculate_self_attack_power()),
                "stat_sheet_attack_power": debug_float_value(self.calculate_stat_sheet_attack_power()),
                "move_speed_to_damage_bonus_capped": debug_float_value(Self::calculate_move_speed_to_damage_bonus(
                    self.move_speed_rate.value(),
                    self.move_speed_to_damage_rate.value(),
                    true,
                )),
                "move_speed_to_damage_bonus_uncapped": debug_float_value(Self::calculate_move_speed_to_damage_bonus(
                    self.move_speed_rate.value(),
                    self.move_speed_to_damage_rate.value(),
                    false,
                )),
            },
            "runtime_state": {
                "identity_stance": self.runtime_state.identity_stance,
                "identity_gauge1": self.runtime_state.identity_gauge1,
                "identity_gauge2": self.runtime_state.identity_gauge2,
                "identity_gauge3": self.runtime_state.identity_gauge3,
                "identity_gauge1_prev": self.runtime_state.identity_gauge1_prev,
                "identity_gauge2_prev": self.runtime_state.identity_gauge2_prev,
                "identity_gauge3_prev": self.runtime_state.identity_gauge3_prev,
                "last_skill_start_id": self.runtime_state.last_skill_start_id,
                "last_skill_start_tick_ms": self.runtime_state.last_skill_start_at_ms,
                "destroyer_recent_consumed_cores": self.runtime_state.destroyer_recent_consumed_cores,
                "destroyer_recent_consumed_tick_ms": self.runtime_state.destroyer_recent_consumed_at_ms,
                "current_hp": self.runtime_state.current_hp,
                "max_hp": self.runtime_state.max_hp,
                "current_mp": self.runtime_state.current_mp,
                "max_mp": self.runtime_state.max_mp,
                "combat_mp_recovery": self.runtime_state.combat_mp_recovery,
                "identity_runtime_reliable": self.runtime_state.identity_runtime_reliable,
                "skill_start_identity_gauge_snapshots": self.runtime_state.skill_start_identity_gauge_snapshots.iter().map(|(skill_id, snapshot)| {
                    json!({
                        "skill_id": skill_id,
                        "identity_gauge1": snapshot.identity_gauge1,
                        "identity_gauge2": snapshot.identity_gauge2,
                        "identity_gauge3": snapshot.identity_gauge3,
                    })
                }).collect::<Vec<_>>(),
            },
            "critical_hit_rate_cap": self.critical_hit_rate_cap,
            "damage_conversion_type": self.damage_conversion_type,
            "skill_status_effect_multiplier": self.skill_status_effect_multiplier.iter().map(|(skill_id, value)| {
                json!({
                    "skill_id": skill_id,
                    "value": debug_float_value(*value),
                })
            }).collect::<Vec<_>>(),
            "skill_attack_power_multiplier": self.skill_attack_power_multiplier.iter().map(|(skill_id, value)| {
                json!({
                    "skill_id": skill_id,
                    "value": debug_float_value(*value),
                })
            }).collect::<Vec<_>>(),
            "skill_group_status_effect_multiplier": self.skill_group_status_effect_multiplier.iter().map(|(group_id, value)| {
                json!({
                    "group_id": group_id,
                    "value": debug_float_value(*value),
                })
            }).collect::<Vec<_>>(),
            "active_combat_effects": self.active_combat_effects.iter().map(|entry| {
                json!({
                    "owner_id": entry.owner_id,
                    "source": entry.source,
                    "combat_effect_id": entry.combat_effect_id,
                    "effect": format!("{:?}", entry.effect),
                })
            }).collect::<Vec<_>>(),
            "active_addon_skill_features": self.active_addon_skill_features.iter().map(|entry| {
                json!({
                    "owner_id": entry.owner_id,
                    "feature_id": entry.feature_id,
                })
            }).collect::<Vec<_>>(),
            "active_ability_features": self.active_ability_features.iter().map(|entry| {
                json!({
                    "owner_id": entry.owner_id,
                    "feature_type": entry.feature_type,
                    "level": entry.level,
                    "values": entry.values,
                })
            }).collect::<Vec<_>>(),
            "stat_datas": stat_datas,
        })
    }

    pub fn get_damage_splits(&self, damage: f64, factors: &[f64]) -> Vec<f64> {
        get_damage_splits(damage, factors)
    }

    pub fn get_damage_increase_contributed_from_all_entity_ids(
        &self,
        total_attack_power_original: f64,
        hit_option: &HitOption,
        hit_flag: &HitFlag,
        damage_attr: Option<u8>,
        damage_type: u8,
        is_hyper_awakening: bool,
        is_affected_by_buffs: bool,
        can_crit: bool,
        include_average_crit: bool,
    ) -> f64 {
        let mut copied = self.clone();
        let owner_id = copied.owner_id;
        copied.for_each_stat_mut(|stat| {
            let old_value = stat.get_value_for_entity_id(0);
            stat.clear();
            stat.set(old_value, owner_id, owner_id, "contribution");
        });
        let without_value = copied
            .calculate_final_attack_power(
                hit_option,
                hit_flag,
                damage_attr,
                damage_type,
                is_hyper_awakening,
                is_affected_by_buffs,
                can_crit,
                include_average_crit,
            )
            .value();
        let delta = total_attack_power_original - without_value;
        delta / without_value
    }

    pub fn get_stat_data_ref(&self, index: usize) -> &StatData {
        match index {
            0 => &self.weapon_power,
            1 => &self.weapon_dam_x,
            2 => &self.attack_power_base_multiplier,
            3 => &self.attack_power_rate,
            4 => &self.str_stat,
            5 => &self.dex_stat,
            6 => &self.int_stat,
            7 => &self.str_multiplier_stat,
            8 => &self.dex_multiplier_stat,
            9 => &self.int_multiplier_stat,
            10 => &self.critical_hit_stat,
            11 => &self.move_speed_rate,
            12 => &self.attack_speed_rate,
            13 => &self.ally_identity_damage_power,
            14 => &self.ally_attack_power_power,
            15 => &self.ally_brand_power,
            16 => &self.evolution_damage,
            17 => &self.modify_damage_combat_effect,
            18 => &self.spec_bonus_identity_1,
            19 => &self.spec_bonus_identity_2,
            20 => &self.spec_bonus_identity_3,
            21 => &self.critical_hit_rate,
            22 => &self.critical_damage_rate,
            23 => &self.critical_damage_rate_2,
            24 => &self.attack_power_addend,
            25 => &self.attack_power_addend_2,
            26 => &self.attack_power_sub_rate_1,
            27 => &self.attack_power_sub_rate_2,
            28 => &self.skill_damage_sub_rate_1,
            29 => &self.skill_damage_sub_rate_2,
            30 => &self.skill_damage_rate,
            31 => &self.ultimate_awakening_damage_rate,
            32 => &self.move_speed_to_damage_rate,
            33 => &self.physical_defense_break,
            34 => &self.magical_defense_break,
            35 => &self.outgoing_dmg_stat_amp,
            36 => &self.skill_damage_amplify,
            37 => &self.front_attack_amplify,
            38 => &self.back_attack_amplify,
            39 => &self.physical_critical_damage_amplify,
            40 => &self.magical_critical_damage_amplify,
            41 => &self.critical_hit_to_damage_rate,
            42 => &self.evolution_damage_bonus_from_blunt_thorn,
            43 => &self.evolution_damage_bonus_from_supersonic_breakthrough,
            _ => {
                let index_in_arrays = index.saturating_sub(FIXED_STAT_DATA_COUNT);
                if index_in_arrays < self.damage_attr_amplifications.len() {
                    &self.damage_attr_amplifications[index_in_arrays]
                } else {
                    let rate_index = index_in_arrays - self.damage_attr_amplifications.len();
                    if rate_index < self.damage_attr_rates.len() {
                        &self.damage_attr_rates[rate_index]
                    } else {
                        panic!("Invalid StatData index.");
                    }
                }
            }
        }
    }

    pub fn get_stat_data_ref_mut(&mut self, index: usize) -> &mut StatData {
        match index {
            0 => &mut self.weapon_power,
            1 => &mut self.weapon_dam_x,
            2 => &mut self.attack_power_base_multiplier,
            3 => &mut self.attack_power_rate,
            4 => &mut self.str_stat,
            5 => &mut self.dex_stat,
            6 => &mut self.int_stat,
            7 => &mut self.str_multiplier_stat,
            8 => &mut self.dex_multiplier_stat,
            9 => &mut self.int_multiplier_stat,
            10 => &mut self.critical_hit_stat,
            11 => &mut self.move_speed_rate,
            12 => &mut self.attack_speed_rate,
            13 => &mut self.ally_identity_damage_power,
            14 => &mut self.ally_attack_power_power,
            15 => &mut self.ally_brand_power,
            16 => &mut self.evolution_damage,
            17 => &mut self.modify_damage_combat_effect,
            18 => &mut self.spec_bonus_identity_1,
            19 => &mut self.spec_bonus_identity_2,
            20 => &mut self.spec_bonus_identity_3,
            21 => &mut self.critical_hit_rate,
            22 => &mut self.critical_damage_rate,
            23 => &mut self.critical_damage_rate_2,
            24 => &mut self.attack_power_addend,
            25 => &mut self.attack_power_addend_2,
            26 => &mut self.attack_power_sub_rate_1,
            27 => &mut self.attack_power_sub_rate_2,
            28 => &mut self.skill_damage_sub_rate_1,
            29 => &mut self.skill_damage_sub_rate_2,
            30 => &mut self.skill_damage_rate,
            31 => &mut self.ultimate_awakening_damage_rate,
            32 => &mut self.move_speed_to_damage_rate,
            33 => &mut self.physical_defense_break,
            34 => &mut self.magical_defense_break,
            35 => &mut self.outgoing_dmg_stat_amp,
            36 => &mut self.skill_damage_amplify,
            37 => &mut self.front_attack_amplify,
            38 => &mut self.back_attack_amplify,
            39 => &mut self.physical_critical_damage_amplify,
            40 => &mut self.magical_critical_damage_amplify,
            41 => &mut self.critical_hit_to_damage_rate,
            42 => &mut self.evolution_damage_bonus_from_blunt_thorn,
            43 => &mut self.evolution_damage_bonus_from_supersonic_breakthrough,
            _ => {
                let index_in_arrays = index.saturating_sub(FIXED_STAT_DATA_COUNT);
                if index_in_arrays < self.damage_attr_amplifications.len() {
                    &mut self.damage_attr_amplifications[index_in_arrays]
                } else {
                    let rate_index = index_in_arrays - self.damage_attr_amplifications.len();
                    if rate_index < self.damage_attr_rates.len() {
                        &mut self.damage_attr_rates[rate_index]
                    } else {
                        panic!("Invalid StatData index.");
                    }
                }
            }
        }
    }

    pub fn get_stat_data_name(&self, index: usize) -> String {
        match index {
            0 => "weapon_power_".to_string(),
            1 => "weapon_dam_x_".to_string(),
            2 => "attack_power_base_multiplier_".to_string(),
            3 => "attack_power_rate_".to_string(),
            4 => "str_stat_".to_string(),
            5 => "dex_stat_".to_string(),
            6 => "int_stat_".to_string(),
            7 => "str_multiplier_stat_".to_string(),
            8 => "dex_multiplier_stat_".to_string(),
            9 => "int_multiplier_stat_".to_string(),
            10 => "critical_hit_stat_".to_string(),
            11 => "move_speed_rate_".to_string(),
            12 => "attack_speed_rate_".to_string(),
            13 => "ally_identity_damage_power_".to_string(),
            14 => "ally_attack_power_power_".to_string(),
            15 => "ally_brand_power_".to_string(),
            16 => "evolution_damage_".to_string(),
            17 => "modify_damage_combat_effect_".to_string(),
            18 => "spec_bonus_identity_1_".to_string(),
            19 => "spec_bonus_identity_2_".to_string(),
            20 => "spec_bonus_identity_3_".to_string(),
            21 => "critical_hit_rate_".to_string(),
            22 => "critical_damage_rate_".to_string(),
            23 => "critical_damage_rate_2_".to_string(),
            24 => "attack_power_addend_".to_string(),
            25 => "attack_power_addend_2_".to_string(),
            26 => "attack_power_sub_rate_1_".to_string(),
            27 => "attack_power_sub_rate_2_".to_string(),
            28 => "skill_damage_sub_rate_1_".to_string(),
            29 => "skill_damage_sub_rate_2_".to_string(),
            30 => "skill_damage_rate_".to_string(),
            31 => "ultimate_awakening_damage_rate_".to_string(),
            32 => "move_speed_to_damage_rate_".to_string(),
            33 => "physical_defense_break_".to_string(),
            34 => "magical_defense_break_".to_string(),
            35 => "outgoing_dmg_stat_amp_".to_string(),
            36 => "skill_damage_amplify_".to_string(),
            37 => "front_attack_amplify_".to_string(),
            38 => "back_attack_amplify_".to_string(),
            39 => "physical_critical_damage_amplify_".to_string(),
            40 => "magical_critical_damage_amplify_".to_string(),
            41 => "critical_hit_to_damage_rate_".to_string(),
            42 => "evolution_damage_bonus_from_blunt_thorn_".to_string(),
            43 => "evolution_damage_bonus_from_supersonic_breakthrough_".to_string(),
            _ => {
                let index_in_arrays = index.saturating_sub(FIXED_STAT_DATA_COUNT);
                if index_in_arrays < self.damage_attr_amplifications.len() {
                    let damage_attr_name = damage_attr_debug_name(index_in_arrays)
                        .unwrap_or_else(|| panic!("Invalid StatData index."));
                    format!("damage_attr_amplifications_[{damage_attr_name}]")
                } else {
                    let rate_index = index_in_arrays - self.damage_attr_amplifications.len();
                    let damage_attr_name = damage_attr_debug_name(rate_index)
                        .unwrap_or_else(|| panic!("Invalid StatData index."));
                    format!("damage_attr_rates_[{damage_attr_name}]")
                }
            }
        }
    }

    pub fn iterate_stat_datas(&self) -> Vec<usize> {
        let mut indices = (0..FIXED_STAT_DATA_COUNT).collect::<Vec<_>>();
        for index in 0..self.damage_attr_amplifications.len() {
            indices.push(FIXED_STAT_DATA_COUNT + index);
            indices.push(FIXED_STAT_DATA_COUNT + self.damage_attr_amplifications.len() + index);
        }
        indices
    }

    pub fn get_all_contributing_entity_ids(&self) -> Vec<u64> {
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        for index in self.iterate_stat_datas() {
            let stat = self.get_stat_data_ref(index);
            for modification in &stat.modified_values {
                if modification.values.is_empty() || !seen.insert(modification.source_entity_id) {
                    continue;
                }
                out.push(modification.source_entity_id);
            }
        }
        out.sort_unstable();
        out
    }

    pub fn get_skill_status_effect_multiplier(&self, skill_id: u32, skill_groups: &[u32]) -> f64 {
        let mut multiplier = self
            .skill_status_effect_multiplier
            .get(&skill_id)
            .copied()
            .unwrap_or_default();
        for group_id in skill_groups {
            multiplier += self
                .skill_group_status_effect_multiplier
                .get(group_id)
                .copied()
                .unwrap_or_default();
        }
        multiplier
    }

    pub fn get_skill_attack_power_multiplier(&self, skill_id: u32) -> f64 {
        self.skill_attack_power_multiplier
            .get(&skill_id)
            .copied()
            .unwrap_or_default()
    }

    pub fn get_skill_identity_category(
        &self,
        skill_id: u32,
        runtime_data: Option<&SkillRuntimeData>,
    ) -> Option<String> {
        self.resolve_skill_identity_category(skill_id, runtime_data)
    }

    pub fn get_main_stat(&self) -> &StatData {
        let int_self = self.int_stat.self_value();
        let dex_self = self.dex_stat.self_value();
        let str_self = self.str_stat.self_value();
        if int_self >= dex_self && int_self >= str_self {
            return &self.int_stat;
        }
        if dex_self >= int_self && dex_self >= str_self {
            return &self.dex_stat;
        }
        &self.str_stat
    }

    pub fn get_main_stat_calculated(&self) -> StatData {
        let int_self = self.int_stat.self_value();
        let dex_self = self.dex_stat.self_value();
        let str_self = self.str_stat.self_value();
        if int_self >= dex_self && int_self >= str_self {
            return self.int_stat.mad(&self.int_multiplier_stat);
        }
        if dex_self >= int_self && dex_self >= str_self {
            return self.dex_stat.mad(&self.dex_multiplier_stat);
        }
        self.str_stat.mad(&self.str_multiplier_stat)
    }

    fn handle_external_addon(
        &mut self,
        addon: &crate::models::ExternalResourceAddon,
        owner_id: u64,
        class_id: u32,
        source: &str,
    ) {
        self.handle_external_addon_with_priority(
            addon,
            owner_id,
            class_id,
            source,
            STAT_PRIORITY_DEFAULT,
        );
    }

    fn handle_external_addon_with_priority(
        &mut self,
        addon: &crate::models::ExternalResourceAddon,
        owner_id: u64,
        class_id: u32,
        source: &str,
        source_priority: i32,
    ) {
        match addon.addon_type.as_str() {
            "stat" => self.add_stat_by_name_with_priority(
                &addon.stat_type,
                addon.key_value,
                owner_id,
                source,
                source_priority,
            ),
            "combat_effect" => {
                if let Some(combat_effect) = COMBAT_EFFECT_DATA.get(&(addon.key_index as i32)) {
                    for effect in &combat_effect.effects {
                        self.active_combat_effects.push(ActiveCombatEffect {
                            combat_effect_id: addon.key_index,
                            effect: effect.clone(),
                            owner_id,
                            source: source.to_string(),
                        });
                    }
                }
            }
            "ability_point" => {
                self.handle_ability_point(addon, owner_id, class_id, source, source_priority)
            }
            "ability_feature" => self.handle_ability_feature(addon, owner_id),
            "skill_feature" => {
                if EXTERNAL_ADDON_SKILL_FEATURE_DATA.contains_key(&addon.key_index) {
                    self.active_addon_skill_features
                        .push(ActiveAddonSkillFeature {
                            feature_id: addon.key_index,
                            owner_id,
                        });
                }
            }
            "attack_power_amplify_multiplier" => {
                self.ally_attack_power_power
                    .add_self(addon.key_value as f64 / 10000.0, source);
            }
            "skill_group_status_effect_stat_multiplier" => {
                *self
                    .skill_group_status_effect_multiplier
                    .entry(addon.key_index)
                    .or_default() += addon.key_value as f64 / 10000.0;
            }
            "skill_attack_power_amplify_multiplier" => {
                *self
                    .skill_attack_power_multiplier
                    .entry(addon.key_index)
                    .or_default() += addon.key_value as f64 / 10000.0;
            }
            "skill_status_effect_stat_multiplier" => {
                *self
                    .skill_status_effect_multiplier
                    .entry(addon.key_index)
                    .or_default() += addon.key_value as f64 / 10000.0;
            }
            "ark_passive_point"
            | "party_without_self_heal"
            | "party_without_self_shield"
            | "skill_group_cooldown_reduction"
            | "identity_gauge"
            | "mana_reduction" => {}
            "class_option" => self.handle_class_option(
                addon.key_index,
                owner_id,
                class_id,
                source,
                source_priority,
            ),
            _ => {}
        }
    }

    fn handle_ability_feature(
        &mut self,
        addon: &crate::models::ExternalResourceAddon,
        owner_id: u64,
    ) {
        if let Some(ability) = EXTERNAL_ABILITY_DATA.get(&addon.key_index)
            && let Some(level) = ability.levels.get(&(addon.key_value as u32))
        {
            self.add_ability_feature(
                &ability.feature_type,
                addon.key_value as u32,
                &level.values,
                owner_id,
            );
        }
    }

    fn handle_ability_point(
        &mut self,
        addon: &crate::models::ExternalResourceAddon,
        owner_id: u64,
        class_id: u32,
        source: &str,
        source_priority: i32,
    ) {
        let ability_id = addon
            .stat_type
            .parse::<u32>()
            .ok()
            .filter(|ability_id| *ability_id != 0)
            .or_else(|| {
                EXTERNAL_ABILITY_DATA
                    .contains_key(&addon.key_index)
                    .then_some(addon.key_index)
            });
        if let Some(ability_id) = ability_id
            && let Some(ability) = EXTERNAL_ABILITY_DATA.get(&ability_id)
            && let Some(level) = ability.levels.get(&1)
        {
            let nested_source = format!("{}_{}", source, ability.name);
            for nested in &level.addons {
                self.handle_external_addon_with_priority(
                    nested,
                    owner_id,
                    class_id,
                    &nested_source,
                    source_priority,
                );
            }
        }
    }

    fn handle_class_option(
        &mut self,
        key_index: u32,
        owner_id: u64,
        class_id: u32,
        source: &str,
        source_priority: i32,
    ) {
        if let Some(option) = EXTERNAL_ITEM_CLASS_OPTION_DATA.get(&key_index)
            && let Some(class_addon) = option.class_options.get(&class_id)
        {
            self.handle_external_addon_with_priority(
                class_addon,
                owner_id,
                class_id,
                source,
                source_priority,
            );
        }
    }

    fn add_stat_by_name(&mut self, name: &str, stat_value: i64, owner_id: u64, source: &str) {
        self.add_stat_by_name_with_priority(
            name,
            stat_value,
            owner_id,
            source,
            STAT_PRIORITY_DEFAULT,
        );
    }

    fn add_stat_by_name_with_priority(
        &mut self,
        name: &str,
        stat_value: i64,
        owner_id: u64,
        source: &str,
        source_priority: i32,
    ) {
        let value_as_multiplier = stat_value as f64 / 10000.0;
        match name {
            "int" => self.int_stat.add_with_priority(
                stat_value as f64,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "agi" => self.dex_stat.add_with_priority(
                stat_value as f64,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "str" => self.str_stat.add_with_priority(
                stat_value as f64,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "int_x" => self.int_multiplier_stat.add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "agi_x" => self.dex_multiplier_stat.add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "str_x" => self.str_multiplier_stat.add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "char_attack_dam" => {}
            "criticalhit" => self.critical_hit_stat.add_with_priority(
                stat_value as f64,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "hp" => self.runtime_state.current_hp = stat_value,
            "max_hp" => self.runtime_state.max_hp = stat_value,
            "mp" => self.runtime_state.current_mp = stat_value,
            "max_mp" => self.runtime_state.max_mp = stat_value,
            "combat_mp_recovery" => self.runtime_state.combat_mp_recovery = stat_value,
            "critical_hit_rate" => self.critical_hit_rate.add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "critical_dam_rate" => self.critical_damage_rate.add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "weapon_dam" => self.weapon_power.add_with_priority(
                stat_value as f64,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "weapon_dam_x" => self.weapon_dam_x.add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "attack_power_rate" => self.attack_power_rate.add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "evolution_dam_rate" => self.evolution_damage.add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "stigma_power_rate" => self.ally_brand_power.add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "move_speed_rate" => self.move_speed_rate.add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "base_damage_rate" => self.attack_power_base_multiplier.add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "attack_speed_rate" => self.attack_speed_rate.add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "attack_power_addend" => self.attack_power_addend.add_with_priority(
                stat_value as f64,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "attack_power_addend_2" => self.attack_power_addend_2.add_with_priority(
                stat_value as f64,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "attack_power_sub_rate_1" => self.attack_power_sub_rate_1.add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "attack_power_sub_rate_2" => self.attack_power_sub_rate_2.add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "skill_damage_sub_rate_1" => self.skill_damage_sub_rate_1.add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "skill_damage_sub_rate_2" => self.skill_damage_sub_rate_2.add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "skill_damage_rate" => self.skill_damage_rate.add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "ultimate_awakening_dam_rate" | "awakening_dam_rate" => {
                self.ultimate_awakening_damage_rate.add_with_priority(
                    value_as_multiplier,
                    self.owner_id,
                    owner_id,
                    source,
                    source_priority,
                )
            }
            "dark_dam_rate" => self.damage_attr_rates[6].add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "holy_dam_rate" => self.damage_attr_rates[7].add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "earth_dam_rate" => self.damage_attr_rates[5].add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "electricity_dam_rate" => self.damage_attr_rates[3].add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "fire_dam_rate" => self.damage_attr_rates[1].add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "ice_dam_rate" => self.damage_attr_rates[2].add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "elements_dam_rate" => {
                self.add_stat_by_name_with_priority(
                    "dark_dam_rate",
                    stat_value,
                    owner_id,
                    source,
                    source_priority,
                );
                self.add_stat_by_name_with_priority(
                    "holy_dam_rate",
                    stat_value,
                    owner_id,
                    source,
                    source_priority,
                );
                self.add_stat_by_name_with_priority(
                    "earth_dam_rate",
                    stat_value,
                    owner_id,
                    source,
                    source_priority,
                );
                self.add_stat_by_name_with_priority(
                    "electricity_dam_rate",
                    stat_value,
                    owner_id,
                    source,
                    source_priority,
                );
                self.add_stat_by_name_with_priority(
                    "fire_dam_rate",
                    stat_value,
                    owner_id,
                    source,
                    source_priority,
                );
                self.add_stat_by_name_with_priority(
                    "ice_dam_rate",
                    stat_value,
                    owner_id,
                    source,
                    source_priority,
                );
            }
            "identity_value1" => self.spec_bonus_identity_1.add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "identity_value2" => self.spec_bonus_identity_2.add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            "identity_value3" => self.spec_bonus_identity_3.add_with_priority(
                value_as_multiplier,
                self.owner_id,
                owner_id,
                source,
                source_priority,
            ),
            _ => {}
        }
    }

    fn apply_lal_base_inspect_bonuses(&mut self) {
        self.dex_stat.add_self(ROSTER_MAIN_STAT_BONUS, "roster");
        self.int_stat.add_self(ROSTER_MAIN_STAT_BONUS, "roster");
        self.str_stat.add_self(ROSTER_MAIN_STAT_BONUS, "roster");

        self.dex_multiplier_stat.set_self(
            self.dex_multiplier_stat
                .value()
                .min(SKIN_MAIN_STAT_MULTIPLIER_CAP),
            "skins",
        );
        self.int_multiplier_stat.set_self(
            self.int_multiplier_stat
                .value()
                .min(SKIN_MAIN_STAT_MULTIPLIER_CAP),
            "skins",
        );
        self.str_multiplier_stat.set_self(
            self.str_multiplier_stat
                .value()
                .min(SKIN_MAIN_STAT_MULTIPLIER_CAP),
            "skins",
        );

        self.dex_multiplier_stat
            .add_self(PET_MAIN_STAT_MULTIPLIER, "pets");
        self.int_multiplier_stat
            .add_self(PET_MAIN_STAT_MULTIPLIER, "pets");
        self.str_multiplier_stat
            .add_self(PET_MAIN_STAT_MULTIPLIER, "pets");
        self.skill_damage_rate
            .add_self(PET_SKILL_DAMAGE_RATE, "pets");
        self.critical_hit_stat
            .add_self(ROSTER_CRITICAL_HIT_BONUS, "roster");
    }

    fn apply_lal_stat_pair_fallback(&mut self, stat_pairs: &HashMap<u8, i64>) {
        for (stat_id, value) in stat_pairs {
            let Some(name) = stat_name_from_id(*stat_id) else {
                continue;
            };
            match name.as_str() {
                "criticalhit" => {
                    self.critical_hit_rate
                        .add_self((*value as f64 / 27.94) * 0.01, "StatType.CRITICALHIT");
                }
                "rapidity" => {
                    let rapidity_bonus = *value as f64 * 0.01717757 * 0.01;
                    self.move_speed_rate
                        .add_self(rapidity_bonus, "StatType.RAPIDITY");
                    self.attack_speed_rate
                        .add_self(rapidity_bonus, "StatType.RAPIDITY");
                }
                "base_damage_rate" => {
                    self.attack_power_base_multiplier
                        .set_self(*value as f64 / 10000.0, "StatType.BASE_DAMAGE_RATE");
                }
                "identity_value1" => {
                    self.spec_bonus_identity_1
                        .set_self(*value as f64 / 10000.0, "StatType.IDENTITY_VALUE1");
                }
                "identity_value2" => {
                    self.spec_bonus_identity_2
                        .set_self(*value as f64 / 10000.0, "StatType.IDENTITY_VALUE2");
                }
                "identity_value3" => {
                    self.spec_bonus_identity_3
                        .set_self(*value as f64 / 10000.0, "StatType.IDENTITY_VALUE3");
                }
                "hp" => self.runtime_state.current_hp = *value,
                "max_hp" => self.runtime_state.max_hp = *value,
                "mp" => self.runtime_state.current_mp = *value,
                "max_mp" => self.runtime_state.max_mp = *value,
                "combat_mp_recovery" => self.runtime_state.combat_mp_recovery = *value,
                _ => {}
            }
        }
    }

    fn for_each_stat<F: FnMut(&StatData)>(&self, mut f: F) {
        f(&self.weapon_power);
        f(&self.weapon_dam_x);
        f(&self.attack_power_base_multiplier);
        f(&self.attack_power_rate);
        f(&self.str_stat);
        f(&self.dex_stat);
        f(&self.int_stat);
        f(&self.str_multiplier_stat);
        f(&self.dex_multiplier_stat);
        f(&self.int_multiplier_stat);
        f(&self.critical_hit_stat);
        f(&self.move_speed_rate);
        f(&self.attack_speed_rate);
        f(&self.ally_identity_damage_power);
        f(&self.ally_attack_power_power);
        f(&self.ally_brand_power);
        f(&self.evolution_damage);
        f(&self.modify_damage_combat_effect);
        f(&self.spec_bonus_identity_1);
        f(&self.spec_bonus_identity_2);
        f(&self.spec_bonus_identity_3);
        f(&self.critical_hit_rate);
        f(&self.critical_damage_rate);
        f(&self.critical_damage_rate_2);
        f(&self.attack_power_addend);
        f(&self.attack_power_addend_2);
        f(&self.attack_power_sub_rate_1);
        f(&self.attack_power_sub_rate_2);
        f(&self.skill_damage_sub_rate_1);
        f(&self.skill_damage_sub_rate_2);
        f(&self.skill_damage_rate);
        f(&self.ultimate_awakening_damage_rate);
        f(&self.move_speed_to_damage_rate);
        f(&self.critical_hit_to_damage_rate);
        f(&self.physical_defense_break);
        f(&self.magical_defense_break);
        f(&self.outgoing_dmg_stat_amp);
        f(&self.skill_damage_amplify);
        f(&self.front_attack_amplify);
        f(&self.back_attack_amplify);
        f(&self.physical_critical_damage_amplify);
        f(&self.magical_critical_damage_amplify);
        for index in 0..self.damage_attr_amplifications.len() {
            f(&self.damage_attr_amplifications[index]);
            f(&self.damage_attr_rates[index]);
        }
    }

    fn for_each_stat_mut<F: FnMut(&mut StatData)>(&mut self, mut f: F) {
        f(&mut self.weapon_power);
        f(&mut self.weapon_dam_x);
        f(&mut self.attack_power_base_multiplier);
        f(&mut self.attack_power_rate);
        f(&mut self.str_stat);
        f(&mut self.dex_stat);
        f(&mut self.int_stat);
        f(&mut self.str_multiplier_stat);
        f(&mut self.dex_multiplier_stat);
        f(&mut self.int_multiplier_stat);
        f(&mut self.critical_hit_stat);
        f(&mut self.move_speed_rate);
        f(&mut self.attack_speed_rate);
        f(&mut self.ally_identity_damage_power);
        f(&mut self.ally_attack_power_power);
        f(&mut self.ally_brand_power);
        f(&mut self.evolution_damage);
        f(&mut self.modify_damage_combat_effect);
        f(&mut self.spec_bonus_identity_1);
        f(&mut self.spec_bonus_identity_2);
        f(&mut self.spec_bonus_identity_3);
        f(&mut self.critical_hit_rate);
        f(&mut self.critical_damage_rate);
        f(&mut self.critical_damage_rate_2);
        f(&mut self.attack_power_addend);
        f(&mut self.attack_power_addend_2);
        f(&mut self.attack_power_sub_rate_1);
        f(&mut self.attack_power_sub_rate_2);
        f(&mut self.skill_damage_sub_rate_1);
        f(&mut self.skill_damage_sub_rate_2);
        f(&mut self.skill_damage_rate);
        f(&mut self.ultimate_awakening_damage_rate);
        f(&mut self.move_speed_to_damage_rate);
        f(&mut self.critical_hit_to_damage_rate);
        f(&mut self.physical_defense_break);
        f(&mut self.magical_defense_break);
        f(&mut self.outgoing_dmg_stat_amp);
        f(&mut self.skill_damage_amplify);
        f(&mut self.front_attack_amplify);
        f(&mut self.back_attack_amplify);
        f(&mut self.physical_critical_damage_amplify);
        f(&mut self.magical_critical_damage_amplify);
        for index in 0..self.damage_attr_amplifications.len() {
            f(&mut self.damage_attr_amplifications[index]);
            f(&mut self.damage_attr_rates[index]);
        }
    }
}

fn debug_stat_data_value(stat: &StatData) -> Value {
    json!({
        "operation_type": format!("{:?}", stat.operation_type),
        "value": debug_float_value(stat.value()),
        "self_value": debug_float_value(stat.self_value()),
        "self_values": stat.self_values.iter().map(|value| {
            json!({
                "value": debug_float_value(value.value),
                "source": value.source,
            })
        }).collect::<Vec<_>>(),
        "modified_values": stat.modified_values.iter().map(|modification| {
            json!({
                "source_entity_id": modification.source_entity_id,
                "source_priority": modification.source_priority,
                "value": debug_float_value(modification.get_value(stat.operation_type)),
                "values": modification.values.iter().map(|value| {
                    json!({
                        "value": debug_float_value(value.value),
                        "source": value.source,
                    })
                }).collect::<Vec<_>>(),
            })
        }).collect::<Vec<_>>(),
    })
}

fn debug_float_value(value: f64) -> Value {
    if value.is_finite() {
        json!(value)
    } else {
        json!({
            "non_finite": format!("{value:?}"),
        })
    }
}

fn calculate_move_speed_to_damage_bonus(move_speed_rate: f64, rate: f64, capped_ms: bool) -> f64 {
    let move_speed_bonus = if capped_ms {
        move_speed_rate.min(MOVE_SPEED_ATTACK_SPEED_CAP)
    } else {
        move_speed_rate
    };
    move_speed_bonus * rate
}

fn stat_name_from_id(stat_id: u8) -> Option<String> {
    stat_type_name_from_id(stat_id as u32)
}

fn damage_attr_to_index(damage_attr: Option<u8>) -> Option<usize> {
    match damage_attr {
        Some(value @ 0..=7) => Some(value as usize),
        _ => None,
    }
}

fn is_critical(hit_flag: &HitFlag) -> bool {
    matches!(hit_flag, HitFlag::CRITICAL | HitFlag::DOT_CRITICAL)
}

fn normalize_feature_type(feature_type: &str) -> &str {
    feature_type.strip_prefix("ap_").unwrap_or(feature_type)
}

fn damage_attr_debug_name(index: usize) -> Option<&'static str> {
    match index {
        0 => Some("NONE"),
        1 => Some("FIRE"),
        2 => Some("ICE"),
        3 => Some("ELECTRICITY"),
        4 => Some("WIND"),
        5 => Some("EARTH"),
        6 => Some("DARK"),
        7 => Some("HOLY"),
        _ => None,
    }
}

fn get_damage_splits(damage: f64, factors: &[f64]) -> Vec<f64> {
    let count = factors.len();
    let mut pieces = vec![0.0; count + 1];
    if count == 0 {
        pieces[0] = damage;
        return pieces;
    }

    let compact_count = factors.iter().filter(|factor| **factor != 0.0).count();
    if compact_count == 0 {
        pieces[0] = damage;
        return pieces;
    }

    DAMAGE_SPLIT_SCRATCH.with(|scratch_cell| {
        let mut scratch = scratch_cell.borrow_mut();
        let DamageSplitScratch {
            compact_factors: scratch_compact_factors,
            compact_indices: scratch_compact_indices,
            subset_prod,
            subset_size,
            weights: scratch_weights,
            factorial: scratch_factorial,
        } = &mut *scratch;

        let compact_factors: &[f64];
        if compact_count == count {
            compact_factors = factors;
            scratch_compact_indices.clear();
        } else {
            scratch_compact_factors.clear();
            scratch_compact_indices.clear();
            scratch_compact_factors.reserve(compact_count);
            scratch_compact_indices.reserve(compact_count);
            for (index, factor) in factors.iter().copied().enumerate() {
                if factor == 0.0 {
                    continue;
                }
                scratch_compact_factors.push(factor);
                scratch_compact_indices.push(index);
            }
            compact_factors = scratch_compact_factors.as_slice();
        }

        let max_mask = 1usize << compact_count;
        subset_prod.resize(max_mask, 0.0);
        subset_size.resize(max_mask, 0);
        subset_prod[0] = 1.0;
        subset_size[0] = 0;
        for mask in 1..max_mask {
            let prev_mask = mask & (mask - 1);
            let bit_index = (mask ^ prev_mask).trailing_zeros() as usize;
            subset_prod[mask] = subset_prod[prev_mask] * (1.0 + compact_factors[bit_index]);
            subset_size[mask] = subset_size[prev_mask] + 1;
        }

        let base_damage = 1.0 / subset_prod[max_mask - 1];
        pieces[0] = base_damage * damage;

        let cached_weights = damage_split_weight_cache();
        let weights: &[f64] = if compact_count <= DAMAGE_SPLIT_CACHED_MAX_FACTORS {
            &cached_weights[compact_count]
        } else {
            scratch_factorial.resize(compact_count + 1, 1.0);
            scratch_factorial[0] = 1.0;
            for index in 1..=compact_count {
                scratch_factorial[index] = scratch_factorial[index - 1] * index as f64;
            }
            scratch_weights.resize(compact_count, 0.0);
            for subset_size in 0..compact_count {
                scratch_weights[subset_size] = scratch_factorial[subset_size]
                    * scratch_factorial[compact_count - subset_size - 1]
                    / scratch_factorial[compact_count];
            }
            &scratch_weights[..compact_count]
        };

        for index in 0..compact_count {
            let mut sum = 0.0;
            let skip_bit = 1usize << index;
            for mask in 0..max_mask {
                if (mask & skip_bit) != 0 {
                    continue;
                }
                sum += subset_prod[mask] * weights[subset_size[mask]];
            }

            let piece = base_damage * compact_factors[index] * sum * damage;
            let output_index = if compact_count == count {
                index + 1
            } else {
                scratch_compact_indices[index] + 1
            };
            pieces[output_index] = piece;
        }
    });

    pieces
}
