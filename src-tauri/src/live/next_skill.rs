use crate::data::{EXTERNAL_ABILITY_DATA, SKILL_DATA};
use crate::models::{ExternalAbilityData, SkillData};
use hashbrown::HashMap;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, OnceLock};

// Correlates consumption notifications and skill starts delivered in either order.
const REORDER_WINDOW_MS: i64 = 1000;

pub type DefinitionKey = (u32, u32);
pub type MarkerKey = (u32, u32);

pub fn is_feature(feature_type: &str) -> bool {
    matches!(feature_type, "ag_alpha_next_skill_enhance" | "220")
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub key: DefinitionKey,
    pub skill_id: u32,
    pub skill_group: u32,
    pub marker_id: u32,
    pub damage_rate: i64,
    pub critical_rate: i64,
}

#[derive(Debug, Default)]
pub struct DefinitionIndex {
    pub definitions: BTreeMap<DefinitionKey, Definition>,
    by_skill: BTreeMap<u32, Vec<DefinitionKey>>,
    markers: BTreeSet<u32>,
}

impl DefinitionIndex {
    pub fn build(
        abilities: &HashMap<u32, ExternalAbilityData>,
        skills: &HashMap<u32, SkillData>,
    ) -> Self {
        let mut index = Self::default();
        let mut ordered = BTreeMap::new();
        for (&ability_id, ability) in abilities {
            if !is_feature(&ability.feature_type) {
                continue;
            }
            for (&level, data) in &ability.levels {
                let values = &data.values;
                if values.len() < 6 {
                    continue;
                }
                let (Ok(skill_id), Ok(skill_group), Ok(marker_id)) = (
                    u32::try_from(values[1]),
                    u32::try_from(values[2]),
                    u32::try_from(values[5]),
                ) else {
                    continue;
                };
                if marker_id == 0 || (skill_id == 0 && skill_group == 0) {
                    continue;
                }
                ordered.insert(
                    (ability_id, level),
                    Definition {
                        key: (ability_id, level),
                        skill_id,
                        skill_group,
                        marker_id,
                        damage_rate: values[3],
                        critical_rate: values[4],
                    },
                );
            }
        }
        for (&key, definition) in &ordered {
            index.markers.insert(definition.marker_id);
            let mut matching_skills = BTreeSet::new();
            if definition.skill_id != 0 {
                matching_skills.insert(definition.skill_id);
            }
            if definition.skill_group != 0 {
                for (&skill_id, skill) in skills {
                    if skill
                        .groups
                        .as_ref()
                        .is_some_and(|groups| groups.contains(&definition.skill_group))
                    {
                        matching_skills.insert(skill_id);
                    }
                }
            }
            for skill_id in matching_skills {
                index.by_skill.entry(skill_id).or_default().push(key);
            }
        }
        index.definitions = ordered;
        index
    }

    pub fn is_marker(&self, status_id: u32) -> bool {
        self.markers.contains(&status_id)
    }

    fn for_skill(&self, skill_id: u32) -> &[DefinitionKey] {
        self.by_skill
            .get(&skill_id)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    fn for_skill_and_marker(&self, skill_id: u32, marker_id: u32) -> Vec<DefinitionKey> {
        self.for_skill(skill_id)
            .iter()
            .copied()
            .filter(|key| self.definitions[key].marker_id == marker_id)
            .collect()
    }
}

pub fn definition_index() -> Option<&'static DefinitionIndex> {
    static INDEX: OnceLock<DefinitionIndex> = OnceLock::new();
    let abilities = EXTERNAL_ABILITY_DATA.try_get()?;
    let skills = SKILL_DATA.try_get()?;
    Some(INDEX.get_or_init(|| DefinitionIndex::build(abilities, skills)))
}

pub fn is_marker(status_id: u32) -> bool {
    definition_index().is_some_and(|index| index.is_marker(status_id))
}

#[derive(Debug, Clone)]
pub struct MarkerEvent {
    pub target_id: u64,
    pub party: bool,
    pub key: MarkerKey,
    pub time_ms: i64,
    pub kind: MarkerEventKind,
}

#[derive(Debug, Clone)]
pub enum MarkerEventKind {
    Upsert {
        stacks: u8,
        previous_stacks: Option<u8>,
    },
    Remove {
        consumption: bool,
    },
}

#[derive(Debug, Default, Clone)]
struct Grant {
    stacks: u8,
    reservations: usize,
    zero_stack_available: bool,
    active: bool,
    sequence: u64,
}

impl Grant {
    fn available(&self) -> usize {
        let uses = if self.stacks > 0 {
            usize::from(self.stacks)
        } else {
            usize::from(self.zero_stack_available)
        };
        uses.saturating_sub(self.reservations)
    }
}

#[derive(Debug, Clone)]
struct SkillStart {
    skill_id: u32,
    sequence: u64,
    time_ms: i64,
    consumed_markers: BTreeSet<u32>,
}

#[derive(Debug, Clone)]
struct Consumption {
    key: MarkerKey,
    sequence: u64,
    time_ms: i64,
    consumed: bool,
}

#[derive(Debug, Clone)]
pub struct CastEvidence {
    pub skill_id: u32,
    pub marker: MarkerKey,
    pub sequence: u64,
    pub generation: u64,
}

#[derive(Debug, Default, Clone)]
pub struct NextSkillSnapshot {
    generation: u64,
    fight_open: bool,
    casts: Arc<BTreeMap<DefinitionKey, CastEvidence>>,
}

impl NextSkillSnapshot {
    pub fn debug_value(&self) -> serde_json::Value {
        serde_json::json!({
            "generation": self.generation, "fight_open": self.fight_open,
            "casts": self.casts.iter().map(|(key, cast)| serde_json::json!({
                "ability_id": key.0, "level": key.1, "skill_id": cast.skill_id,
                "marker_id": cast.marker.0, "instance_id": cast.marker.1,
                "sequence": cast.sequence, "generation": cast.generation,
            })).collect::<Vec<_>>()
        })
    }

    pub fn has_cast(&self, key: DefinitionKey, skill_id: u32, real_skill_id: u32) -> bool {
        self.fight_open
            && self.casts.get(&key).is_some_and(|cast| {
                cast.generation == self.generation
                    && (cast.skill_id == skill_id || cast.skill_id == real_skill_id)
            })
    }
}

#[derive(Debug, Default, Clone)]
pub struct NextSkillState {
    sequence: u64,
    generation: u64,
    fight_open: bool,
    grants: Arc<BTreeMap<MarkerKey, Grant>>,
    starts: Arc<Vec<SkillStart>>,
    signals: Arc<Vec<Consumption>>,
    reservations: Arc<Vec<MarkerKey>>,
    pub casts: Arc<BTreeMap<DefinitionKey, CastEvidence>>,
}

impl NextSkillState {
    pub fn snapshot(&self) -> NextSkillSnapshot {
        NextSkillSnapshot {
            generation: self.generation,
            fight_open: self.fight_open,
            casts: self.casts.clone(),
        }
    }

    pub fn evaluation_state(&self) -> Self {
        Self {
            generation: self.generation,
            fight_open: self.fight_open,
            casts: self.casts.clone(),
            ..Default::default()
        }
    }

    pub fn begin_fight(&mut self, generation: u64) {
        if self.fight_open && self.generation == generation {
            return;
        }
        if self.fight_open && self.generation != generation {
            self.clear_functional(false);
        }
        self.generation = generation;
        self.fight_open = true;
        for cast in Arc::make_mut(&mut self.casts).values_mut() {
            cast.generation = generation;
        }
    }

    pub fn close_fight(&mut self, preserve: bool) {
        self.fight_open = false;
        if preserve {
            for cast in Arc::make_mut(&mut self.casts).values_mut() {
                cast.generation = 0;
            }
        } else {
            self.clear_functional(true);
        }
    }

    fn clear_functional(&mut self, clear_markers: bool) {
        if clear_markers {
            self.grants = Default::default();
        } else {
            for grant in Arc::make_mut(&mut self.grants).values_mut() {
                grant.reservations = 0;
            }
        }
        self.starts = Default::default();
        self.signals = Default::default();
        self.reservations = Default::default();
        self.casts = Default::default();
    }

    #[cfg(test)]
    pub fn has_cast(&self, key: DefinitionKey, skill_id: u32, real_skill_id: u32) -> bool {
        self.snapshot().has_cast(key, skill_id, real_skill_id)
    }

    fn confirm(&mut self, key: MarkerKey, count: usize) -> usize {
        let mut confirmed = 0;
        Arc::make_mut(&mut self.reservations).retain(|candidate| {
            if *candidate == key && confirmed < count {
                confirmed += 1;
                false
            } else {
                true
            }
        });
        if let Some(grant) = Arc::make_mut(&mut self.grants).get_mut(&key) {
            grant.reservations = grant.reservations.saturating_sub(confirmed);
        }
        confirmed
    }

    pub fn marker_event(&mut self, event: &MarkerEvent, index: &DefinitionIndex) {
        self.sequence += 1;
        match event.kind {
            MarkerEventKind::Upsert {
                stacks,
                previous_stacks,
            } => self.upsert(
                event.key,
                stacks,
                previous_stacks,
                event.time_ms,
                index,
                false,
            ),
            MarkerEventKind::Remove { consumption } => {
                self.remove(event.key, consumption, event.time_ms, index)
            }
        }
    }

    pub fn reconcile(
        &mut self,
        present: &[(MarkerKey, u8)],
        time_ms: i64,
        index: &DefinitionIndex,
    ) {
        self.sequence += 1;
        let keys: BTreeSet<_> = present.iter().map(|(key, _)| *key).collect();
        for (key, stacks) in present {
            // The other status channel may still hold a copy of a removed instance.
            // Only a new lifecycle notification can restore a known inactive grant.
            if self.grants.get(key).is_none_or(|grant| grant.active) {
                self.upsert(*key, *stacks, None, time_ms, index, true);
            }
        }
        let absent: Vec<_> = self
            .grants
            .iter()
            .filter(|(key, grant)| grant.active && !keys.contains(key))
            .map(|(key, _)| *key)
            .collect();
        for key in absent {
            self.remove(key, false, time_ms, index);
        }
    }

    fn upsert(
        &mut self,
        key: MarkerKey,
        stacks: u8,
        previous: Option<u8>,
        time_ms: i64,
        index: &DefinitionIndex,
        snapshot: bool,
    ) {
        let old = self.grants.get(&key).cloned();
        if old.as_ref().is_none_or(|grant| !grant.active) {
            self.confirm(key, usize::MAX);
            Arc::make_mut(&mut self.grants).insert(
                key,
                Grant {
                    stacks,
                    active: true,
                    zero_stack_available: stacks == 0,
                    sequence: self.sequence,
                    ..Default::default()
                },
            );
            for signal in Arc::make_mut(&mut self.signals) {
                if signal.key.0 == key.0 && signal.sequence < self.sequence {
                    signal.consumed = true;
                }
            }
            if old.is_none() && !snapshot && previous.is_some_and(|value| value > stacks) {
                if stacks == 0 {
                    Arc::make_mut(&mut self.grants)
                        .get_mut(&key)
                        .unwrap()
                        .zero_stack_available = false;
                }
                self.add_signals(key, usize::from(previous.unwrap() - stacks), time_ms, index);
            }
            return;
        }
        let old = old.unwrap();
        let consumed = old.stacks.saturating_sub(stacks);
        let confirmed = if stacks < old.stacks {
            self.confirm(key, usize::from(consumed))
        } else if stacks > old.stacks {
            self.confirm(key, usize::MAX)
        } else {
            0
        };
        let grant = Arc::make_mut(&mut self.grants).get_mut(&key).unwrap();
        grant.stacks = stacks;
        if stacks < old.stacks && stacks == 0 {
            grant.zero_stack_available = false;
        } else if stacks > old.stacks {
            grant.zero_stack_available = stacks == 0;
        }
        if !snapshot {
            grant.sequence = self.sequence;
        }
        if !snapshot && usize::from(consumed) > confirmed {
            self.add_signals(key, usize::from(consumed) - confirmed, time_ms, index);
        }
    }

    fn remove(&mut self, key: MarkerKey, consumption: bool, time_ms: i64, index: &DefinitionIndex) {
        let was_active = self.grants.get(&key).map(|grant| grant.active);
        let confirmed = self.confirm(key, usize::MAX);
        if let Some(grant) = Arc::make_mut(&mut self.grants).get_mut(&key) {
            grant.active = false;
            grant.reservations = 0;
            grant.sequence = self.sequence;
        }
        if consumption && confirmed == 0 && was_active != Some(false) {
            self.add_signals(key, 1, time_ms, index);
        }
    }

    fn add_signals(&mut self, key: MarkerKey, count: usize, time_ms: i64, index: &DefinitionIndex) {
        let first = self.signals.len();
        Arc::make_mut(&mut self.signals).extend((0..count).map(|_| Consumption {
            key,
            sequence: self.sequence,
            time_ms,
            consumed: false,
        }));
        let candidates: Vec<_> = self
            .starts
            .iter()
            .enumerate()
            .filter(|(_, start)| {
                start.sequence < self.sequence
                    && !start.consumed_markers.contains(&key.0)
                    && within_window(start.time_ms, time_ms)
                    && !index.for_skill_and_marker(start.skill_id, key.0).is_empty()
            })
            .map(|(position, _)| position)
            .collect();
        if candidates.len() > count {
            for signal in &mut Arc::make_mut(&mut self.signals)[first..] {
                signal.consumed = true;
            }
            return;
        }
        for (offset, position) in candidates.into_iter().enumerate() {
            Arc::make_mut(&mut self.signals)[first + offset].consumed = true;
            Arc::make_mut(&mut self.starts)[position]
                .consumed_markers
                .insert(key.0);
            let start = self.starts[position].clone();
            self.arm(
                &index.for_skill_and_marker(start.skill_id, key.0),
                &start,
                key,
            );
        }
    }

    pub fn skill_start(&mut self, skill_id: u32, time_ms: i64, index: &DefinitionIndex) {
        if skill_id == 0 || index.for_skill(skill_id).is_empty() {
            return;
        }
        self.sequence += 1;
        for key in index.for_skill(skill_id) {
            Arc::make_mut(&mut self.casts).remove(key);
        }
        let mut start = SkillStart {
            skill_id,
            sequence: self.sequence,
            time_ms,
            consumed_markers: BTreeSet::new(),
        };
        let marker_ids: BTreeSet<_> = index
            .for_skill(skill_id)
            .iter()
            .map(|key| index.definitions[key].marker_id)
            .collect();
        for marker_id in marker_ids {
            let definitions = index.for_skill_and_marker(skill_id, marker_id);
            let pending_reverse = self.signals.iter().any(|signal| {
                !signal.consumed
                    && signal.key.0 == marker_id
                    && signal.sequence < start.sequence
                    && within_window(signal.time_ms, time_ms)
            });
            let mut grants: Vec<_> = self
                .grants
                .iter()
                .filter(|(key, grant)| {
                    key.0 == marker_id && grant.active && grant.available() > 0 && !pending_reverse
                })
                .map(|(key, grant)| (*key, grant.sequence))
                .collect();
            grants
                .sort_by(|left, right| right.1.cmp(&left.1).then_with(|| right.0.1.cmp(&left.0.1)));
            if !grants.is_empty() && (grants.len() == 1 || grants[0].1 != grants[1].1) {
                let key = grants[0].0;
                let grant = Arc::make_mut(&mut self.grants).get_mut(&key).unwrap();
                grant.reservations += 1;
                if grant.stacks == 0 {
                    grant.zero_stack_available = false;
                }
                Arc::make_mut(&mut self.reservations).push(key);
                start.consumed_markers.insert(marker_id);
                self.arm(&definitions, &start, key);
                continue;
            }
            let signals: Vec<_> = self
                .signals
                .iter()
                .enumerate()
                .filter(|(_, signal)| {
                    !signal.consumed
                        && signal.key.0 == marker_id
                        && signal.sequence < start.sequence
                        && within_window(signal.time_ms, time_ms)
                })
                .map(|(position, _)| position)
                .collect();
            if let Some(&position) = signals.first() {
                let key = self.signals[position].key;
                if signals
                    .iter()
                    .all(|&candidate| self.signals[candidate].key.1 == key.1)
                {
                    Arc::make_mut(&mut self.signals)[position].consumed = true;
                    start.consumed_markers.insert(marker_id);
                    self.arm(&definitions, &start, key);
                }
            }
        }
        Arc::make_mut(&mut self.starts).push(start);
    }

    fn arm(&mut self, definitions: &[DefinitionKey], start: &SkillStart, marker: MarkerKey) {
        for &key in definitions {
            Arc::make_mut(&mut self.casts).insert(
                key,
                CastEvidence {
                    skill_id: start.skill_id,
                    marker,
                    sequence: start.sequence,
                    generation: if self.fight_open { self.generation } else { 0 },
                },
            );
        }
    }
}

fn within_window(left: i64, right: i64) -> bool {
    left.abs_diff(right) <= REORDER_WINDOW_MS as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ExternalAbilityLevelData;

    fn index() -> DefinitionIndex {
        let ability = |id, feature: &str, values| ExternalAbilityData {
            id,
            feature_type: feature.to_string(),
            levels: HashMap::from([(
                1,
                ExternalAbilityLevelData {
                    values,
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        DefinitionIndex::build(
            &HashMap::from([
                (
                    11,
                    ability(
                        0,
                        "ag_alpha_next_skill_enhance",
                        vec![9, 101, 0, 1250, 750, 201, 18000, 4, 99, 1, 42],
                    ),
                ),
                (12, ability(0, "220", vec![1, 0, 301, 500, 250, 202])),
                (
                    13,
                    ability(
                        0,
                        "ag_alpha_next_skill_enhance",
                        vec![1, 101, 0, 0, 500, 201],
                    ),
                ),
                (
                    14,
                    ability(
                        0,
                        "ag_alpha_next_skill_enhance",
                        vec![1, 0, 0, 1000, 0, 203],
                    ),
                ),
                (
                    15,
                    ability(
                        0,
                        "ag_alpha_next_skill_enhance",
                        vec![1, -1, 0, 1000, 0, 203],
                    ),
                ),
            ]),
            &HashMap::from([
                (
                    101,
                    SkillData {
                        groups: Some(vec![301]),
                        ..Default::default()
                    },
                ),
                (
                    102,
                    SkillData {
                        groups: Some(vec![301]),
                        ..Default::default()
                    },
                ),
            ]),
        )
    }

    fn event(key: MarkerKey, stacks: Option<u8>, consumption: bool, time_ms: i64) -> MarkerEvent {
        MarkerEvent {
            target_id: 1,
            party: false,
            key,
            time_ms,
            kind: stacks
                .map(|stacks| MarkerEventKind::Upsert {
                    stacks,
                    previous_stacks: None,
                })
                .unwrap_or(MarkerEventKind::Remove { consumption }),
        }
    }

    #[test]
    fn definitions_use_dictionary_ids_and_admit_marker_plus_skill_or_group() {
        let index = index();
        assert_eq!(index.definitions.len(), 3);
        assert_eq!(index.for_skill(101), &[(11, 1), (12, 1), (13, 1)]);
        assert_eq!(index.for_skill(102), &[(12, 1)]);
        assert!(!index.is_marker(203));
        assert_eq!(index.definitions[&(11, 1)].damage_rate, 1250);
        assert_eq!(index.definitions[&(11, 1)].critical_rate, 750);
    }

    #[test]
    fn active_grants_survive_delays_and_arm_all_matching_definitions_and_markers() {
        let index = index();
        let mut state = NextSkillState::default();
        state.marker_event(&event((201, 1), Some(1), false, 1000), &index);
        state.marker_event(&event((202, 2), Some(1), false, 1000), &index);
        state.skill_start(999, 2000, &index);
        state.skill_start(101, 11000, &index);
        assert!(!state.has_cast((11, 1), 101, 101));
        state.begin_fight(7);
        assert_eq!(state.casts.len(), 3);
        assert!(state.has_cast((11, 1), 101, 0));
        assert!(state.has_cast((13, 1), 0, 101));
        assert!(!state.has_cast((11, 1), 102, 102));
        for _ in 0..3 {
            assert!(state.has_cast((11, 1), 101, 101));
        }
        state.marker_event(&event((201, 1), None, false, 12000), &index);
        assert!(state.has_cast((11, 1), 101, 101));
        state.skill_start(101, 20000, &index);
        assert!(state.casts.is_empty());
    }

    #[test]
    fn server_decrements_confirm_reservations_before_creating_reverse_evidence() {
        let index = index();
        let mut state = NextSkillState::default();
        state.begin_fight(1);
        state.marker_event(&event((201, 1), Some(2), false, 1000), &index);
        state.skill_start(101, 2000, &index);
        state.marker_event(&event((201, 1), Some(1), false, 2100), &index);
        assert!(state.signals.is_empty());
        state.skill_start(101, 3000, &index);
        assert!(state.has_cast((11, 1), 101, 101));
        state.skill_start(101, 4000, &index);
        assert!(!state.has_cast((11, 1), 101, 101));
        state.marker_event(&event((201, 1), None, true, 4100), &index);
        assert!(state.signals.is_empty());
    }

    #[test]
    fn zero_stack_grants_allow_one_use_and_same_stack_refresh_does_not_refill() {
        let index = index();
        let mut state = NextSkillState::default();
        state.begin_fight(1);
        state.marker_event(&event((201, 1), Some(0), false, 1000), &index);
        state.skill_start(101, 2000, &index);
        assert!(state.has_cast((11, 1), 101, 101));
        state.marker_event(&event((201, 1), Some(0), false, 2100), &index);
        state.skill_start(101, 2200, &index);
        assert!(!state.has_cast((11, 1), 101, 101));
    }

    #[test]
    fn consumption_and_start_correlate_in_both_orders_at_the_window_boundary() {
        let index = index();
        for (reverse, delay, expected) in [
            (false, 1000, true),
            (true, 1000, true),
            (false, 1001, false),
            (true, 1001, false),
        ] {
            let mut state = NextSkillState::default();
            state.begin_fight(1);
            if reverse {
                state.marker_event(&event((201, 1), None, true, 1000), &index);
                state.skill_start(101, 1000 + delay, &index);
            } else {
                state.skill_start(101, 1000, &index);
                state.marker_event(&event((201, 1), None, true, 1000 + delay), &index);
            }
            assert_eq!(
                state.has_cast((11, 1), 101, 101),
                expected,
                "reverse={reverse} delay={delay}"
            );
        }
    }

    #[test]
    fn expiry_and_snapshot_absence_create_no_consumption_evidence() {
        let index = index();
        for snapshot_absence in [false, true] {
            let mut state = NextSkillState::default();
            state.begin_fight(1);
            state.marker_event(&event((201, 1), Some(1), false, 1000), &index);
            if snapshot_absence {
                state.reconcile(&[], 2000, &index);
            } else {
                state.marker_event(&event((201, 1), None, false, 2000), &index);
            }
            state.marker_event(&event((201, 1), None, true, 2100), &index);
            state.skill_start(101, 2200, &index);
            assert!(state.casts.is_empty());
        }
    }

    #[test]
    fn missing_state_decrement_matches_before_reserving_the_remaining_stack() {
        let index = index();
        let mut state = NextSkillState::default();
        state.begin_fight(1);
        state.marker_event(
            &MarkerEvent {
                kind: MarkerEventKind::Upsert {
                    stacks: 1,
                    previous_stacks: Some(2),
                },
                ..event((201, 1), Some(1), false, 1000)
            },
            &index,
        );
        state.skill_start(101, 1100, &index);
        assert!(state.has_cast((11, 1), 101, 101));
        assert!(state.reservations.is_empty());
        state.skill_start(101, 1200, &index);
        assert!(state.has_cast((11, 1), 101, 101));
        assert_eq!(state.reservations.len(), 1);
        state.skill_start(101, 1300, &index);
        assert!(state.casts.is_empty());
    }

    #[test]
    fn ambiguous_consumptions_and_equal_age_marker_instances_do_not_guess() {
        let index = index();
        let mut state = NextSkillState::default();
        state.begin_fight(1);
        state.skill_start(101, 1000, &index);
        state.skill_start(101, 1100, &index);
        state.marker_event(&event((201, 1), None, true, 1200), &index);
        assert!(state.casts.is_empty());
        assert!(state.signals.iter().all(|signal| signal.consumed));
        state.reconcile(&[((201, 1), 1), ((201, 2), 1)], 2000, &index);
        state.skill_start(101, 2100, &index);
        assert!(state.casts.is_empty());
    }

    #[test]
    fn snapshots_do_not_invent_consumptions_and_new_grants_invalidate_old_signals() {
        let index = index();
        let mut state = NextSkillState::default();
        state.begin_fight(1);
        state.marker_event(&event((201, 1), None, true, 1000), &index);
        state.marker_event(&event((201, 2), Some(2), false, 1100), &index);
        state.reconcile(&[((201, 2), 1)], 1200, &index);
        assert!(state.signals.iter().all(|signal| signal.consumed));
        state.skill_start(101, 1300, &index);
        assert!(state.has_cast((11, 1), 101, 101));
        assert_eq!(state.casts[&(11, 1)].marker, (201, 2));
    }

    #[test]
    fn fight_resets_and_clones_keep_evidence_with_the_correct_hit_generation() {
        let index = index();
        let mut state = NextSkillState::default();
        state.marker_event(&event((201, 1), Some(1), false, 1000), &index);
        state.skill_start(101, 2000, &index);
        state.begin_fight(7);
        let frozen = state.clone();
        state.skill_start(101, 3000, &index);
        assert!(state.casts.is_empty());
        assert!(frozen.has_cast((11, 1), 101, 101));
        let mut preserved = frozen.clone();
        preserved.close_fight(true);
        assert!(!preserved.has_cast((11, 1), 101, 101));
        preserved.begin_fight(8);
        assert!(preserved.has_cast((11, 1), 101, 101));
        preserved.begin_fight(9);
        assert!(preserved.casts.is_empty());
        preserved.close_fight(false);
        assert!(preserved.grants.is_empty());
    }

    #[test]
    fn hit_snapshots_share_cast_evidence_without_correlation_history() {
        let index = index();
        let mut state = NextSkillState::default();
        state.begin_fight(1);
        for time in 0..256 {
            state.skill_start(101, time, &index);
        }
        state.marker_event(&event((201, 1), Some(1), false, 1000), &index);
        state.skill_start(101, 1100, &index);
        let copy = state.clone();
        assert!(Arc::ptr_eq(&state.starts, &copy.starts));
        assert!(Arc::ptr_eq(&state.grants, &copy.grants));
        assert!(Arc::ptr_eq(&state.signals, &copy.signals));
        assert!(Arc::ptr_eq(&state.reservations, &copy.reservations));
        let snapshot = state.snapshot();
        let hit = state.evaluation_state();
        assert!(Arc::ptr_eq(&snapshot.casts, &snapshot.clone().casts));
        assert!(Arc::ptr_eq(&snapshot.casts, &hit.casts));
        assert!(hit.starts.is_empty());
        assert!(hit.grants.is_empty());
        assert!(hit.signals.is_empty());
        assert!(hit.reservations.is_empty());
        state.skill_start(101, 1200, &index);
        state.close_fight(false);
        assert!(!state.has_cast((11, 1), 101, 101));
        assert!(snapshot.has_cast((11, 1), 101, 101));
        assert!(hit.has_cast((11, 1), 101, 101));
        assert_eq!(copy.starts.len(), 257);
        assert!(copy.has_cast((11, 1), 101, 101));
    }

    #[test]
    fn packet_lifecycle_skill_start_and_runtime_evaluation_preserve_distinct_abilities() {
        use crate::live::entity_tracker::{Entity, EntityTracker};
        use crate::live::id_tracker::IdTracker;
        use crate::live::party_tracker::PartyTracker;
        use crate::live::player_stats::PlayerStats;
        use crate::live::status_tracker::{
            StatusEffectDetails, StatusEffectTargetType, StatusTracker,
        };
        use crate::live::test_data::*;
        use std::{cell::RefCell, rc::Rc};
        crate::live::test_data::initialize();
        for party_channel in [false, true] {
            let ids = Rc::new(RefCell::new(IdTracker::new()));
            let parties = Rc::new(RefCell::new(PartyTracker::new(ids.clone())));
            let statuses = Rc::new(RefCell::new(StatusTracker::new(parties.clone())));
            let mut tracker = EntityTracker::new(statuses.clone(), ids, parties);
            tracker.entities.insert(
                1,
                Entity {
                    id: 1,
                    character_id: 11,
                    entity_type: crate::models::EntityType::Player,
                    ..Default::default()
                },
            );
            statuses
                .borrow_mut()
                .register_status_effect(StatusEffectDetails {
                    status_effect_id: NEXT_MARKER_ID,
                    instance_id: 17,
                    target_id: if party_channel { 11 } else { 1 },
                    target_type: if party_channel {
                        StatusEffectTargetType::Party
                    } else {
                        StatusEffectTargetType::Local
                    },
                    stack_count: 1,
                    timestamp: chrono::DateTime::from_timestamp_millis(1000).unwrap(),
                    ..Default::default()
                });
            tracker.record_skill_start(1, NEXT_SKILL_ID, 1, &Default::default(), 11000);
            tracker.begin_next_skill_fight();
            let frozen_entity = tracker.entities[&1].clone();
            let hit = tracker.snapshot_entity_for_hit(&frozen_entity);
            assert!(hit.next_skill.starts.is_empty());
            assert!(hit.next_skill.grants.is_empty());
            assert!(Arc::ptr_eq(
                &frozen_entity.next_skill.casts,
                &hit.next_skill.casts
            ));
            assert!(frozen_entity.next_skill.has_cast(
                (NEXT_ABILITY_ID, 1),
                NEXT_SKILL_ID,
                NEXT_SKILL_ID
            ));
            let mut base = PlayerStats {
                owner_id: 1,
                ..Default::default()
            };
            for ability_id in [NEXT_ABILITY_ID, SECOND_NEXT_ABILITY_ID] {
                let ability = &EXTERNAL_ABILITY_DATA[&ability_id];
                base.add_identified_ability_feature(
                    ability_id,
                    &ability.feature_type,
                    1,
                    &ability.levels[&1].values,
                    1,
                );
            }
            assert_eq!(base.active_ability_features.len(), 2);
            base.apply_runtime_state_snapshot(frozen_entity.runtime_state());
            for damage_scope in [false, true] {
                let mut stats = base.clone();
                stats.apply_dynamic_effects(
                    NEXT_SKILL_ID,
                    NEXT_SKILL_ID,
                    0,
                    &[],
                    &[],
                    None,
                    None,
                    None,
                    &[],
                    &[],
                    12000,
                    damage_scope,
                );
                assert!(
                    (stats.skill_damage_rate.value() - if damage_scope { 0.3 } else { 0.0 }).abs()
                        < 1e-12
                );
                assert!(
                    (stats.critical_hit_rate.value() - if damage_scope { 0.08 } else { 0.0 }).abs()
                        < 1e-12
                );
            }
            tracker.record_skill_start_snapshot(1, NEXT_SKILL_ID, 1, None, 11000);
            assert!(tracker.entities[&1].next_skill.has_cast(
                (NEXT_ABILITY_ID, 1),
                NEXT_SKILL_ID,
                NEXT_SKILL_ID
            ));
            tracker.record_skill_start(1, NEXT_SKILL_ID, 1, &Default::default(), 13000);
            assert!(tracker.entities[&1].next_skill.casts.is_empty());
            assert!(frozen_entity.next_skill.has_cast(
                (NEXT_ABILITY_ID, 1),
                NEXT_SKILL_ID,
                NEXT_SKILL_ID
            ));
            tracker.retire_fight_inspect_requests();
            assert!(tracker.entities[&1].next_skill.grants.is_empty());
        }
    }

    #[test]
    fn a_consumed_marker_is_not_restored_by_the_other_status_channel() {
        use crate::live::entity_tracker::{Entity, EntityTracker};
        use crate::live::id_tracker::IdTracker;
        use crate::live::party_tracker::PartyTracker;
        use crate::live::status_tracker::{
            StatusEffectDetails, StatusEffectTargetType, StatusTracker,
        };
        use crate::live::test_data::*;
        use std::{cell::RefCell, rc::Rc};
        crate::live::test_data::initialize();
        for party_first in [false, true] {
            let ids = Rc::new(RefCell::new(IdTracker::new()));
            let parties = Rc::new(RefCell::new(PartyTracker::new(ids.clone())));
            let statuses = Rc::new(RefCell::new(StatusTracker::new(parties.clone())));
            let mut tracker = EntityTracker::new(statuses.clone(), ids, parties);
            tracker.entities.insert(
                1,
                Entity {
                    id: 1,
                    character_id: 11,
                    entity_type: crate::models::EntityType::Player,
                    ..Default::default()
                },
            );
            let now = chrono::Utc::now().timestamp_millis();
            for (target_id, target_type) in [
                (1, StatusEffectTargetType::Local),
                (11, StatusEffectTargetType::Party),
            ] {
                statuses
                    .borrow_mut()
                    .register_status_effect(StatusEffectDetails {
                        status_effect_id: NEXT_MARKER_ID,
                        instance_id: 17,
                        target_id,
                        target_type,
                        stack_count: 1,
                        timestamp: chrono::DateTime::from_timestamp_millis(now).unwrap(),
                        ..Default::default()
                    });
            }
            tracker.begin_next_skill_fight();
            tracker.record_next_skill_start(1, NEXT_SKILL_ID, now + 100);
            assert!(tracker.entities[&1].next_skill.has_cast(
                (NEXT_ABILITY_ID, 1),
                NEXT_SKILL_ID,
                NEXT_SKILL_ID
            ));
            let channels = if party_first {
                [
                    (11, StatusEffectTargetType::Party),
                    (1, StatusEffectTargetType::Local),
                ]
            } else {
                [
                    (1, StatusEffectTargetType::Local),
                    (11, StatusEffectTargetType::Party),
                ]
            };
            for (offset, (target_id, target_type)) in channels.into_iter().enumerate() {
                statuses
                    .borrow_mut()
                    .remove_status_effects(target_id, vec![17], 0, target_type);
                tracker.record_next_skill_start(1, NEXT_SKILL_ID, now + 200 + offset as i64 * 100);
                assert!(
                    !tracker.entities[&1].next_skill.has_cast(
                        (NEXT_ABILITY_ID, 1),
                        NEXT_SKILL_ID,
                        NEXT_SKILL_ID
                    ),
                    "party_first={party_first}, removal={offset}"
                );
            }
        }
    }
}
