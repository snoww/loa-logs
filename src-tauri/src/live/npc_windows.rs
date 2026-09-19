use crate::data::{NPC_WINDOW_DATA, SKILL_BUFF_DATA};
use crate::live::entity_tracker::Entity;
use crate::live::stat_type::StatType;
use crate::live::status_tracker::StatusEffectDetails;
use hashbrown::{HashMap, HashSet};
use serde::Deserialize;

bitflags::bitflags! {
    /// Enabled categories credit the target NPC; disabled categories retain player attribution.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct NpcDamageAttribution: u8 {
        const BROKEN_BONE = 1 << 0;
        const DOMINATION = 1 << 1;
        const DAMAGE_TAKEN = 1 << 2;
        const COMBAT_EFFECTS = 1 << 3;
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(try_from = "NpcWindowResource")]
pub struct NpcWindowData {
    pub schema_version: u32,
    pub paralyzation_coefficient: f64,
    pub oppression_coefficient_by_level: HashMap<u16, f64>,
    pub npc_groups: HashMap<u32, usize>,
    pub groups: Vec<ActionGroup>,
    stages: Vec<Option<WeaknessStage>>,
}

#[derive(Debug, Default)]
pub struct ActionGroup {
    inactive_actions: HashSet<u32>,
    actions: HashMap<u32, WeaknessAction>,
}

#[derive(Debug, Deserialize)]
struct WeaknessAction {
    #[serde(deserialize_with = "deserialize_layer_counts")]
    layers: Vec<[u32; 3]>,
    stages: HashMap<u32, usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WeaknessStage {
    #[serde(default = "normal_play_rate")]
    rate: f64,
    #[serde(default)]
    attack_speed: bool,
    #[serde(deserialize_with = "Option::deserialize")]
    length: Option<f64>,
    #[serde(default)]
    start: f64,
    #[serde(deserialize_with = "Option::deserialize")]
    end: Option<f64>,
    bonus: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NpcWindowResource {
    schema_version: u32,
    paralyzation_coefficient: f64,
    oppression_coefficient_by_level: HashMap<u16, f64>,
    common_inactive_actions: HashSet<u32>,
    groups: Vec<ActionGroupResource>,
    stages: Vec<Option<WeaknessStage>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActionGroupResource {
    npc_id_deltas: Vec<u32>,
    #[serde(default)]
    inactive_action_deltas: Vec<u32>,
    #[serde(default)]
    actions: HashMap<u32, WeaknessAction>,
}

fn normal_play_rate() -> f64 {
    1.0
}

fn deserialize_layer_counts<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<[u32; 3]>, D::Error> {
    let counts = Vec::<u32>::deserialize(deserializer)?;
    let mut start = 0u32;
    counts
        .into_iter()
        .enumerate()
        .map(|(index, count)| {
            let index = u32::try_from(index).map_err(serde::de::Error::custom)?;
            let layer = [index, start, count];
            start = start
                .checked_add(count)
                .ok_or_else(|| serde::de::Error::custom("NPC layer range overflow"))?;
            Ok(layer)
        })
        .collect()
}

fn decode_id_deltas(deltas: Vec<u32>) -> anyhow::Result<Vec<u32>> {
    let mut previous = 0u32;
    deltas
        .into_iter()
        .enumerate()
        .map(|(index, delta)| {
            anyhow::ensure!(index == 0 || delta > 0, "Duplicate NPC window ID");
            previous = previous
                .checked_add(delta)
                .ok_or_else(|| anyhow::anyhow!("NPC window ID overflow"))?;
            Ok(previous)
        })
        .collect()
}

impl TryFrom<NpcWindowResource> for NpcWindowData {
    type Error = anyhow::Error;

    fn try_from(resource: NpcWindowResource) -> Result<Self, Self::Error> {
        let mut data = Self {
            schema_version: resource.schema_version,
            paralyzation_coefficient: resource.paralyzation_coefficient,
            oppression_coefficient_by_level: resource.oppression_coefficient_by_level,
            stages: resource.stages,
            ..Default::default()
        };
        for group in resource.groups {
            for npc in decode_id_deltas(group.npc_id_deltas)? {
                anyhow::ensure!(
                    data.npc_groups.insert(npc, data.groups.len()).is_none(),
                    "NPC has multiple action groups"
                );
            }
            let mut inactive_actions = resource.common_inactive_actions.clone();
            inactive_actions.extend(decode_id_deltas(group.inactive_action_deltas)?);
            data.groups.push(ActionGroup {
                inactive_actions,
                actions: group.actions,
            });
        }
        data.validate()?;
        Ok(data)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Signal {
    #[default]
    Unresolved,
    Inactive,
    Active,
}

pub fn stagger_state(effects: &[StatusEffectDetails]) -> Signal {
    let mut state = Signal::Inactive;
    for effect in effects {
        match SKILL_BUFF_DATA.get(&effect.status_effect_id) {
            Some(buff) if buff.buff_type.eq_ignore_ascii_case("paralyzation") => {
                return Signal::Active;
            }
            None => state = Signal::Unresolved,
            _ => {}
        }
    }
    state
}

pub fn is_stagger_condition(kind: &str, arg: i32) -> bool {
    matches!(
        (kind, arg),
        ("abnormal_move", 4) | ("abnormal_move_all" | "abnormal_move_status_all", 0)
    )
}

/// Cloned with Entity when damage is queued. Later packets cannot change an earlier hit.
#[derive(Debug, Clone)]
pub struct ObservedAction {
    action_id: u32,
    layer: Option<u32>,
    stage: Option<u32>,
    observed_ms: i64,
    start_time: f64,
    /// ATTACK_SPEED sampled at the stage change, as the client does; later changes wait for the next stage.
    attack_speed: Option<i64>,
    clock_valid: bool,
}

impl NpcWindowData {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.schema_version == 3,
            "Unsupported NPC window resource schema"
        );
        anyhow::ensure!(
            self.paralyzation_coefficient.is_finite() && self.paralyzation_coefficient >= 0.0,
            "Invalid domination paralyzation coefficient"
        );
        anyhow::ensure!(
            !self.oppression_coefficient_by_level.is_empty()
                && self
                    .oppression_coefficient_by_level
                    .iter()
                    .all(|(level, coefficient)| *level > 0
                        && coefficient.is_finite()
                        && *coefficient > 0.0),
            "Invalid domination level coefficients"
        );
        anyhow::ensure!(
            self.npc_groups
                .values()
                .all(|index| *index < self.groups.len()),
            "Invalid NPC action group reference"
        );
        for group in &self.groups {
            for (action_id, action) in &group.actions {
                anyhow::ensure!(
                    !group.inactive_actions.contains(action_id),
                    "Conflicting action coverage"
                );
                anyhow::ensure!(
                    action
                        .stages
                        .values()
                        .all(|index| *index < self.stages.len()),
                    "Invalid NPC weakness stage reference"
                );
                anyhow::ensure!(
                    action
                        .layers
                        .iter()
                        .all(|layer| layer[1].checked_add(layer[2]).is_some()),
                    "Invalid NPC stage layer range"
                );
            }
        }
        anyhow::ensure!(
            self.stages.iter().flatten().all(|stage| {
                stage.rate.is_finite()
                    && stage.rate > 0.0
                    && stage
                        .length
                        .is_none_or(|length| length.is_finite() && length > 0.0)
                    && stage.start.is_finite()
                    && stage.start >= 0.0
                    && stage
                        .end
                        .is_none_or(|end| end.is_finite() && end >= stage.start)
                    && stage.bonus.is_finite()
            }),
            "Invalid NPC weakness timing or multiplier"
        );
        Ok(())
    }

    fn group(&self, npc_id: u32) -> Option<&ActionGroup> {
        self.npc_groups
            .get(&npc_id)
            .and_then(|index| self.groups.get(*index))
    }

    pub fn domination_bonus(&self, stat: Option<i64>, level: u16) -> Option<f64> {
        let stat = stat.filter(|value| *value >= 0)?;
        let coefficient = *self.oppression_coefficient_by_level.get(&level)?;
        (self.schema_version == 3
            && coefficient.is_finite()
            && coefficient > 0.0
            && self.paralyzation_coefficient.is_finite()
            && self.paralyzation_coefficient >= 0.0)
            .then_some(stat as f64 / coefficient * self.paralyzation_coefficient / 10000.0)
    }

    fn resolve_stage(
        &self,
        npc_id: u32,
        action_id: u32,
        layer: Option<u32>,
        local_stage: u8,
    ) -> Option<(u32, u32)> {
        let action = self.group(npc_id)?.actions.get(&action_id)?;
        let mut matches = action.layers.iter().filter(|entry| {
            layer.is_none_or(|layer| entry[0] == layer) && u32::from(local_stage) < entry[2]
        });
        let entry = matches.next()?;
        if matches.next().is_some() {
            return None;
        }
        Some((entry[0], entry[1] + u32::from(local_stage)))
    }

    pub fn weakness_bonus(&self, entity: &Entity, timestamp: i64) -> Option<f64> {
        let observed = entity.npc_action.as_ref()?;
        let group = self.group(entity.npc_id)?;
        if group.inactive_actions.contains(&observed.action_id) {
            return Some(0.0);
        }
        let action = group.actions.get(&observed.action_id)?;
        let stage_index = observed.stage?;
        // Omitted stages have no enabled weakness; pooled null stages are unresolved.
        let Some(index) = action.stages.get(&stage_index) else {
            return Some(0.0);
        };
        let stage = self.stages.get(*index)?.as_ref()?;
        if !observed.clock_valid || timestamp < observed.observed_ms {
            return None;
        }
        // Client (build 20260916): CEFActionStageAgent::ChangeStage samples ATTACK_SPEED * 0.01 and an
        // AtkSpeedStage stage runs at StagePlayRate times that factor. An unknown speed leaves the clock unresolved.
        let rate = if stage.attack_speed {
            match observed.attack_speed {
                Some(speed) if speed > 0 => stage.rate * speed as f64 / 100.0,
                _ => return None,
            }
        } else {
            stage.rate
        };
        let time = observed.start_time + (timestamp - observed.observed_ms) as f64 / 1000.0 * rate;
        if stage.length.is_some_and(|length| time > length) {
            return None;
        }
        Some(
            if stage.bonus > 0.0 && time >= stage.start && stage.end.is_none_or(|end| time <= end) {
                stage.bonus
            } else {
                0.0
            },
        )
    }
}

impl Entity {
    pub fn observe_npc_action(
        &mut self,
        action_id: u32,
        layer: Option<u32>,
        local_stage: u8,
        start_time: f64,
        timestamp: i64,
        stage_packet: bool,
    ) {
        if self.npc_id == 0 {
            return;
        }
        let known_layer = layer.or_else(|| {
            self.npc_action
                .as_ref()
                .filter(|state| state.action_id == action_id && state.stage.is_some())
                .and_then(|state| state.layer)
        });
        let resolved =
            NPC_WINDOW_DATA.resolve_stage(self.npc_id, action_id, known_layer, local_stage);
        if stage_packet
            && resolved.is_some()
            && self.npc_action.as_ref().is_some_and(|state| {
                state.action_id == action_id && state.stage == resolved.map(|value| value.1)
            })
        {
            return;
        }
        self.npc_action = Some(ObservedAction {
            action_id,
            layer: resolved.map(|value| value.0),
            stage: resolved.map(|value| value.1),
            observed_ms: timestamp,
            start_time,
            attack_speed: self.stats.get(&(StatType::ATTACK_SPEED as u8)).copied(),
            clock_valid: start_time.is_finite() && start_time >= 0.0,
        });
    }

    /// A mid-stage ATTACK_SPEED change keeps the running stage's sampled speed. The client only re-rates
    /// the stage immediately under a BULLET_TIME status or a pending time-dilation override, neither of
    /// which is modeled; the next stage packet samples the new value.
    pub fn observe_stat(&mut self, stat_type: u8, value: i64) {
        self.stats.insert(stat_type, value);
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct HitWindow {
    pub target_stagger: Signal,
    pub self_stagger: Signal,
    pub enabled: bool,
    pub npc_id: u64,
    pub attribution: NpcDamageAttribution,
    pub domination: Option<f64>,
    pub weakness: Option<f64>,
}

impl HitWindow {
    pub fn bonus_owner(&self, category: NpcDamageAttribution, original_owner: u64) -> u64 {
        if self.enabled && self.npc_id != 0 && self.attribution.contains(category) {
            self.npc_id
        } else {
            original_owner
        }
    }

    pub fn incomplete(&self) -> bool {
        self.enabled
            && (self.target_stagger == Signal::Unresolved
                || self.domination.is_none()
                || self.weakness.is_none())
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::data::*;
    use crate::live::player_stats::{PlayerStats, StatSource};
    use crate::live::test_data::initialize;
    use crate::models::{HitFlag, HitOption};

    pub fn drex() -> Entity {
        initialize();
        Entity {
            id: 2,
            npc_id: 620440,
            stats: HashMap::from([(StatType::ATTACK_SPEED as u8, 100)]),
            ..Default::default()
        }
    }

    pub fn effect(id: u32, stacks: u8) -> StatusEffectDetails {
        StatusEffectDetails {
            status_effect_id: id,
            target_id: 2,
            source_id: 2,
            skill_level: 1,
            stack_count: stacks,
            ..Default::default()
        }
    }

    pub fn base_stats() -> PlayerStats {
        let mut stats = PlayerStats::default();
        stats.owner_id = 1;
        stats.weapon_power.add_self(600.0, StatSource::Base);
        stats.str_stat.add_self(6000.0, StatSource::Base);
        stats
    }

    pub fn attack_power(stats: &PlayerStats, hyper: bool) -> f64 {
        stats
            .calculate_final_attack_power(
                &HitOption::NONE,
                &HitFlag::NORMAL,
                None,
                0,
                hyper,
                true,
                false,
                false,
                None,
            )
            .value()
    }

    pub fn apply_dynamic(stats: &mut PlayerStats, target: &Entity) {
        stats.apply_dynamic_effects(
            0,
            0,
            0,
            &[],
            &[],
            None,
            Some(target),
            Some(&HitOption::NONE),
            &[],
            &[],
            0,
        );
    }

    #[test]
    fn domination_uses_exported_level_data_without_tooltip_rounding() {
        initialize();
        assert!(
            (NPC_WINDOW_DATA.domination_bonus(Some(79), 70).unwrap() - 0.05641472392638036).abs()
                < 1e-12
        );
        assert_eq!(NPC_WINDOW_DATA.domination_bonus(Some(0), 70), Some(0.0));
        assert_eq!(NPC_WINDOW_DATA.domination_bonus(None, 70), None);
        assert_eq!(NPC_WINDOW_DATA.domination_bonus(Some(79), 0), None);
        assert_eq!(NPC_WINDOW_DATA.domination_bonus(Some(-1), 70), None);
    }

    #[test]
    fn stagger_is_status_type_based_and_missing_metadata_is_unresolved() {
        initialize();
        for id in [49000010, 49000020, 49000030, 425141704, 430560011] {
            assert_eq!(stagger_state(&[effect(id, 1)]), Signal::Active);
        }
        assert_eq!(stagger_state(&[effect(420676006, 1)]), Signal::Inactive);
        assert_eq!(stagger_state(&[effect(u32::MAX, 1)]), Signal::Unresolved);
        assert_eq!(
            stagger_state(&[effect(u32::MAX, 1), effect(49000010, 1)]),
            Signal::Active
        );
    }

    #[test]
    fn observed_stages_speed_changes_and_queued_hits_are_isolated() {
        let mut npc = drex();
        assert_eq!(NPC_WINDOW_DATA.weakness_bonus(&npc, 0), None);
        npc.observe_npc_action(4206760, Some(0), 1, 0.0, 1000, false);
        assert_eq!(NPC_WINDOW_DATA.weakness_bonus(&npc, 2000), Some(0.2));
        npc.observe_npc_action(4206760, None, 1, 0.0, 4000, true);
        assert_eq!(
            NPC_WINDOW_DATA.weakness_bonus(&npc, 6001),
            None,
            "duplicate stage must not restart the five-second clock"
        );
        let queued = npc.clone();
        npc.observe_stat(StatType::ATTACK_SPEED as u8, 110);
        npc.observe_stat(StatType::ATTACK_SPEED as u8, 100);
        assert_eq!(
            NPC_WINDOW_DATA.weakness_bonus(&npc, 2000),
            Some(0.2),
            "a mid-stage speed change keeps the sampled stage clock"
        );
        assert_eq!(NPC_WINDOW_DATA.weakness_bonus(&queued, 2000), Some(0.2));
        assert_eq!(NPC_WINDOW_DATA.weakness_bonus(&queued, 999), None);
        npc.npc_action = None;
        assert_eq!(NPC_WINDOW_DATA.weakness_bonus(&npc, 2000), None);
    }

    // Client rule (build 20260916): an AtkSpeedStage stage runs at StagePlayRate * ATTACK_SPEED / 100 with the
    // stat sampled at the stage change. Drextalas 4206760 stage 1 is a 5 s window, so at speed 120 it expires
    // at 5 / 1.2 = 4.1667 s; stage 2 is 1.3333 s and expires at 1.3333 s once speed 100 is sampled.
    #[test]
    fn attack_speed_scales_the_clock_sampled_at_each_stage() {
        let mut npc = drex();
        npc.observe_stat(StatType::ATTACK_SPEED as u8, 120);
        npc.observe_npc_action(4206760, Some(0), 1, 0.0, 1000, false);
        assert_eq!(NPC_WINDOW_DATA.weakness_bonus(&npc, 5100), Some(0.2));
        assert_eq!(NPC_WINDOW_DATA.weakness_bonus(&npc, 5200), None);
        npc.observe_stat(StatType::ATTACK_SPEED as u8, 100);
        assert_eq!(
            NPC_WINDOW_DATA.weakness_bonus(&npc, 5100),
            Some(0.2),
            "the running stage keeps the speed sampled at its stage change"
        );
        assert_eq!(NPC_WINDOW_DATA.weakness_bonus(&npc, 5200), None);
        npc.observe_npc_action(4206760, None, 2, 0.0, 6000, true);
        assert_eq!(NPC_WINDOW_DATA.weakness_bonus(&npc, 7300), Some(0.2));
        assert_eq!(NPC_WINDOW_DATA.weakness_bonus(&npc, 7400), None);
        npc.stats.clear();
        npc.observe_npc_action(4206760, Some(0), 1, 0.0, 8000, false);
        assert_eq!(
            NPC_WINDOW_DATA.weakness_bonus(&npc, 9000),
            None,
            "an attack-speed stage without a known ATTACK_SPEED is unresolved"
        );
        npc.observe_stat(StatType::ATTACK_SPEED as u8, 0);
        npc.observe_npc_action(4206760, Some(0), 1, 0.0, 10000, false);
        assert_eq!(
            NPC_WINDOW_DATA.weakness_bonus(&npc, 11000),
            None,
            "a non-positive ATTACK_SPEED is no more of a clock rate than an unknown one"
        );
    }

    #[test]
    fn rumble_uses_layer_local_stage_mapping() {
        initialize();
        let (npc_id, _) = NPC_WINDOW_DATA
            .npc_groups
            .iter()
            .find(|(_, index)| {
                NPC_WINDOW_DATA.groups[**index]
                    .actions
                    .contains_key(&4303238)
            })
            .unwrap();
        let action = &NPC_WINDOW_DATA.group(*npc_id).unwrap().actions[&4303238];
        assert!(action.layers.len() > 1);
        for &[layer, start, count] in &action.layers {
            for local in 0..count {
                assert_eq!(
                    NPC_WINDOW_DATA.resolve_stage(*npc_id, 4303238, Some(layer), local as u8),
                    Some((layer, start + local))
                );
            }
        }
        assert_eq!(
            NPC_WINDOW_DATA.resolve_stage(*npc_id, 4303238, None, 0),
            None
        );
    }

    #[test]
    fn compiled_unknown_stage_stays_unresolved() {
        let mut data: NpcWindowData =
            serde_json::from_str(include_str!("../../meter-data/NpcWindows.json")).unwrap();
        let mut npc = drex();
        npc.observe_npc_action(4206760, Some(0), 1, 0.0, 0, false);
        let group = *data.npc_groups.get(&npc.npc_id).unwrap();
        let stage = data.groups[group].actions[&4206760].stages[&1];
        data.stages[stage] = None;
        assert_eq!(data.weakness_bonus(&npc, 1000), None);
    }

    #[test]
    fn compact_resource_preserves_negative_coverage_and_rejects_corrupt_references() {
        let mut npc = drex();
        let group = NPC_WINDOW_DATA.group(npc.npc_id).unwrap();
        let inactive = *group.inactive_actions.iter().next().unwrap();
        npc.observe_npc_action(inactive, Some(0), 0, 0.0, 0, false);
        assert_eq!(NPC_WINDOW_DATA.weakness_bonus(&npc, 1000), Some(0.0));
        npc.observe_npc_action(u32::MAX, Some(0), 0, 0.0, 0, false);
        assert_eq!(NPC_WINDOW_DATA.weakness_bonus(&npc, 1000), None);
        npc.npc_id = u32::MAX;
        npc.observe_npc_action(inactive, Some(0), 0, 0.0, 0, false);
        assert_eq!(NPC_WINDOW_DATA.weakness_bonus(&npc, 1000), None);

        assert_eq!(decode_id_deltas(vec![0, 1, 2]).unwrap(), vec![0, 1, 3]);
        assert!(decode_id_deltas(vec![1, 0]).is_err());
        assert!(decode_id_deltas(vec![u32::MAX, 1]).is_err());
        let mut resource: serde_json::Value =
            serde_json::from_str(include_str!("../../meter-data/NpcWindows.json")).unwrap();
        for field in ["length", "end"] {
            let mut missing_bound = resource.clone();
            missing_bound["stages"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|stage| stage.is_object())
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(
                serde_json::from_value::<NpcWindowData>(missing_bound).is_err(),
                "missing {field} must not become unlimited coverage"
            );
        }
        let groups = resource["groups"].as_array_mut().unwrap();
        let group = groups
            .iter_mut()
            .find(|group| group.get("actions").is_some())
            .unwrap();
        let action = group["actions"]
            .as_object_mut()
            .unwrap()
            .values_mut()
            .next()
            .unwrap();
        action["stages"] = serde_json::json!({"0": usize::MAX});
        assert!(serde_json::from_value::<NpcWindowData>(resource).is_err());
    }

    #[test]
    fn ownership_changes_attribution_but_not_multiplication_or_hyper_awakening() {
        initialize();
        let base = attack_power(&base_stats(), false);
        let window = HitWindow {
            target_stagger: Signal::Active,
            enabled: true,
            npc_id: 2,
            domination: Some(0.0564),
            weakness: Some(0.2),
            ..Default::default()
        };
        for bits in 0..=15 {
            let attribution = NpcDamageAttribution::from_bits(bits).unwrap();
            let mut stats = base_stats();
            stats.apply_npc_window(HitWindow {
                attribution,
                ..window
            });
            stats.add_target_incoming_damage_stat(
                "physical_inc_sub_rate_1",
                0.2,
                StatSource::SkillBuff(420676006),
            );
            stats.add_ability_feature(
                "broken_bone",
                3,
                &EXTERNAL_ABILITY_DATA[&245].levels[&3].values,
                1,
            );
            // This bracelet has unconditional 3% and target-stagger 5% damage actions.
            stats.add_combat_effect(605100031, 1, StatSource::Ability(605100031));
            apply_dynamic(&mut stats, &drex());
            let ap = attack_power(&stats, false);
            assert!((ap / base - 1.0564 * 1.2 * 1.2 * 1.4 * 1.03 * 1.05).abs() < 1e-12);
            assert_eq!(attack_power(&stats, true), 1.0);
            assert!((stats.modify_damage_combat_effect.self_value() - 0.03).abs() < 1e-12);
            for (category, stat, bonus) in [
                (
                    NpcDamageAttribution::BROKEN_BONE,
                    &stats.broken_bone_damage_rate,
                    0.4,
                ),
                (
                    NpcDamageAttribution::DOMINATION,
                    &stats.domination_damage_rate,
                    0.0564,
                ),
                (
                    NpcDamageAttribution::DAMAGE_TAKEN,
                    &stats.npc_action_weakness_damage_rate,
                    0.2,
                ),
                (
                    NpcDamageAttribution::DAMAGE_TAKEN,
                    &stats.target_physical_inc_sub_rate_1,
                    0.2,
                ),
                (
                    NpcDamageAttribution::COMBAT_EFFECTS,
                    &stats.stagger_combat_effect_damage_rate,
                    0.05,
                ),
            ] {
                assert!((stat.value() - bonus).abs() < 1e-12);
                let expected_self = if attribution.contains(category) {
                    0.0
                } else {
                    bonus
                };
                assert!(
                    (stat.self_value() - expected_self).abs() < 1e-12,
                    "{attribution:?}: {category:?}"
                );
            }
            let portions = stats.get_damage_portions_contributed_from_all_entities(
                ap,
                &HitOption::NONE,
                &HitFlag::NORMAL,
                None,
                0,
                false,
                true,
                false,
                false,
            );
            assert_eq!(
                portions
                    .iter()
                    .any(|(portion, owner)| *owner == 2 && *portion > 0.0),
                !attribution.is_empty(),
            );
            assert!((portions.iter().map(|(portion, _)| portion).sum::<f64>() - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn missing_domination_still_enables_broken_bone_and_matching_npc_effect_only() {
        initialize();
        let mut stats = base_stats();
        stats.apply_npc_window(HitWindow {
            enabled: true,
            npc_id: 2,
            attribution: NpcDamageAttribution::all(),
            target_stagger: Signal::Active,
            domination: None,
            weakness: Some(0.0),
            ..Default::default()
        });
        stats.add_ability_feature(
            "broken_bone",
            3,
            &EXTERNAL_ABILITY_DATA[&245].levels[&3].values,
            1,
        );
        stats.add_combat_effect(426590025, 1, StatSource::SkillBuff(426590025));
        let target = Entity {
            npc_id: 480406,
            ..Default::default()
        };
        apply_dynamic(&mut stats, &target);
        assert!((stats.broken_bone_damage_rate.value() - 0.4).abs() < 1e-12);
        assert!((stats.stagger_combat_effect_damage_rate.value() - 0.1).abs() < 1e-12);
        assert!(stats.npc_window.incomplete());
        stats.apply_npc_window(HitWindow {
            enabled: false,
            ..stats.npc_window
        });
        apply_dynamic(&mut stats, &target);
        assert_eq!(stats.broken_bone_damage_rate.value(), 0.0);
        assert_eq!(stats.stagger_combat_effect_damage_rate.value(), 0.0);
    }

    #[test]
    fn npc_window_reductions_stay_in_base_damage_alongside_positive_npc_bonuses() {
        initialize();
        let mut stats = base_stats();
        let base = attack_power(&stats, false);
        stats.apply_npc_window(HitWindow {
            enabled: true,
            npc_id: 2,
            attribution: NpcDamageAttribution::DAMAGE_TAKEN,
            weakness: Some(-0.5),
            ..Default::default()
        });
        assert_eq!(stats.npc_action_weakness_damage_rate.self_value(), -0.5);
        assert_eq!(
            stats
                .npc_action_weakness_damage_rate
                .get_value_for_entity_id(2),
            0.0
        );
        stats.add_target_incoming_damage_stat(
            "physical_inc_sub_rate_1",
            0.03,
            StatSource::SkillBuff(429970096),
        );
        assert!((attack_power(&stats, false) / base - 0.515).abs() < 1e-12);
        assert!(
            (stats
                .target_physical_inc_sub_rate_1
                .get_value_for_entity_id(2)
                - 0.03)
                .abs()
                < 1e-12
        );
        assert!((stats.get_damage_window_multiplier(false, 0) - 0.515).abs() < 1e-12);
        let ap = attack_power(&stats, false);
        let portions = stats.get_damage_portions_contributed_from_all_entities(
            ap,
            &HitOption::NONE,
            &HitFlag::NORMAL,
            None,
            0,
            false,
            true,
            false,
            false,
        );
        assert!(
            (portions.iter().find(|(_, owner)| *owner == 2).unwrap().0 - 0.03 / 1.03).abs() < 1e-12
        );
        assert_eq!(attack_power(&stats, true), 1.0);
    }
}
