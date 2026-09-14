use crate::data::*;
use crate::live::entity_tracker::Entity;
use crate::live::entity_tracker::SkillRuntimeData;
use crate::live::party_tracker::PartyTracker;
use crate::live::player_stats::PlayerStats;
use crate::live::stat_type::StatType;
use crate::live::status_tracker::StatusEffectBuffCategory::{BattleItem, Bracelet, Elixir, Etc};
use crate::live::status_tracker::StatusEffectCategory::Debuff;
use crate::live::status_tracker::StatusEffectShowType::All;
use crate::live::utils::{get_new_id, get_status_effect_buff_type_flags};
use crate::models::{EncounterEntity, EntityType};
use chrono::{DateTime, Duration, Utc};
use hashbrown::{HashMap, HashSet};
use log::info;
use meter_defs::defs::StatusEffectData;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

pub const WORKSHOP_BUFF_ID: u32 = 9701;

pub type StatusEffectRegistry = HashMap<u32, StatusEffectDetails>;

const INFINITE_SERVER_TICK: u64 = u64::MAX;
/// Candidates from unextended instances of one zone agree to within a few milliseconds (9 ms
/// observed), while an instance whose endTick is not on the tick clock (a remaining-time value,
/// seen on one player's party-channel refreshes) yields a candidate tied to its own occurrence
/// time. A later epoch is therefore adopted only once two samples taken more than this far
/// apart in server time agree this closely: such a pair never agrees unless the tick relation
/// really holds.
const SERVER_TICK_EPOCH_CORROBORATION_MS: i64 = 50;
const SERVER_TICK_EPOCH_CANDIDATE_COUNT: usize = 8;
/// Leeway past a deadline after which an instance the server never removed is dropped for
/// every reader (LAL `BuffTracker.Update`, `seconds_leeway` 1000).
const FAIL_SAFE_EXPIRY_LEEWAY_SECONDS: i64 = 1000;

/// How a reader treats an instance past its deadline. LAL reads an attacker's own buffs from
/// its persistent collection, which only a remove packet or the fail-safe prunes, and every
/// other set (target effects, calibration receivers, buffered-hit owners) from its packet
/// collection, which hides an instance from its deadline on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeadlineMode {
    HideExpired,
    KeepUntilRemoved,
}
/// The receiver-side stat sheet stats whose buff sources are tracked for inspect calibration.
pub const IDENTITY_STAT_TYPES: [StatType; 2] = [
    StatType::SKILL_DAMAGE_SUB_RATE_1,
    StatType::SKILL_DAMAGE_SUB_RATE_2,
];

pub fn identity_stat_key(stat_type: StatType) -> &'static str {
    match stat_type {
        StatType::SKILL_DAMAGE_SUB_RATE_1 => "skill_damage_sub_rate_1",
        StatType::SKILL_DAMAGE_SUB_RATE_2 => "skill_damage_sub_rate_2",
        _ => "",
    }
}

/// Applied value of a buff's passive stat measured from a receiver's inspect stat sheet; it
/// replaces the value computed from the owner snapshot for the life of the instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeasuredStat {
    pub stat_type: StatType,
    pub value: i64,
}

pub struct StatusTracker {
    party_tracker: Rc<RefCell<PartyTracker>>,
    local_status_effect_registry: HashMap<u64, StatusEffectRegistry>,
    party_status_effect_registry: HashMap<u64, StatusEffectRegistry>,
    /// Entries dropped by a zone or object snapshot reset, keyed by status effect instance id.
    /// The server keeps those instances alive, so a re-notified instance resumes its
    /// creation-time owner snapshot and source skill.
    retained_entries_by_instance_id: HashMap<u32, StatusEffectDetails>,
    /// Objects seen through a direct status effect notify or an object snapshot since the last
    /// zone change; their party-channel status effects are on this zone's tick clock.
    zone_published_object_ids: HashSet<u64>,
    /// When a registry target last gained, refreshed or lost a source of an identity stat.
    stat_source_change_at: HashMap<(u64, StatType), DateTime<Utc>>,
    /// UTC time of status effect tick 0 on the current server. Every finite StatusEffectData
    /// satisfies occur_time + total_time == epoch + end_tick. An instance the server extended
    /// before re-sending it (a stacked Moonfall in a zone snapshot) only pushes its end_tick
    /// later, so the latest corroborated candidate seen since the zone change is the epoch;
    /// see `observe_server_tick_epoch`.
    server_tick_epoch: Option<DateTime<Utc>>,
    /// Recent candidates later than the adopted epoch, with the occurrence time of the status
    /// that produced each.
    server_tick_epoch_candidates:
        [Option<(DateTime<Utc>, DateTime<Utc>)>; SERVER_TICK_EPOCH_CANDIDATE_COUNT],
    server_tick_epoch_candidate_cursor: usize,
    /// Local clock minus server clock: the smallest local-arrival-minus-`occur_time` seen on a
    /// status notify since the zone change, the sample carrying the least network latency.
    /// Deadlines are server instants and meet the local clock through this offset. The previous
    /// zone's estimate stands in until the first notify of the new zone replaces it.
    server_clock_offset: Option<Duration>,
    server_clock_offset_inherited: bool,
}

impl StatusTracker {
    pub fn new(party_tracker: Rc<RefCell<PartyTracker>>) -> Self {
        Self {
            party_tracker,
            local_status_effect_registry: HashMap::new(),
            party_status_effect_registry: HashMap::new(),
            retained_entries_by_instance_id: HashMap::new(),
            zone_published_object_ids: HashSet::new(),
            stat_source_change_at: HashMap::new(),
            server_tick_epoch: None,
            server_tick_epoch_candidates: [None; SERVER_TICK_EPOCH_CANDIDATE_COUNT],
            server_tick_epoch_candidate_cursor: 0,
            server_clock_offset: None,
            server_clock_offset_inherited: false,
        }
    }

    fn registry_mut(
        &mut self,
        target_type: StatusEffectTargetType,
    ) -> &mut HashMap<u64, StatusEffectRegistry> {
        match target_type {
            StatusEffectTargetType::Local => &mut self.local_status_effect_registry,
            StatusEffectTargetType::Party => &mut self.party_status_effect_registry,
        }
    }

    fn registry(&self, target_type: StatusEffectTargetType) -> &HashMap<u64, StatusEffectRegistry> {
        match target_type {
            StatusEffectTargetType::Local => &self.local_status_effect_registry,
            StatusEffectTargetType::Party => &self.party_status_effect_registry,
        }
    }

    /// Registers a status effect. A re-notified instance (a refresh, or one re-sent after a
    /// zone or object snapshot reset) is the same server-side instance: its magnitude was fixed
    /// at creation, so the creation-time owner snapshot, source skill, first-tracked time and
    /// measured strength carry over.
    pub fn register_status_effect(&mut self, se: StatusEffectDetails) {
        self.register_status_effect_with(se, |_| None);
    }

    /// `register_status_effect` with a resolver for the current runtime cache of a carried-over
    /// source skill, used when the carried instance has no creation-time cache and the incoming
    /// notify resolved a different source skill.
    pub fn register_status_effect_with(
        &mut self,
        mut se: StatusEffectDetails,
        runtime_for_carried_skill: impl FnOnce(u32) -> Option<SkillRuntimeData>,
    ) {
        self.prune_retained_entries(se.timestamp);
        let existing = self
            .registry(se.target_type)
            .get(&se.target_id)
            .and_then(|effects| effects.get(&se.instance_id))
            .filter(|existing| existing.status_effect_id == se.status_effect_id)
            .cloned();
        let carried_over = existing.or_else(|| {
            let matches_retained = self
                .retained_entries_by_instance_id
                .get(&se.instance_id)
                .is_some_and(|retained| {
                    retained.status_effect_id == se.status_effect_id
                        && retained.source_id == se.source_id
                });
            matches_retained
                .then(|| self.retained_entries_by_instance_id.remove(&se.instance_id))
                .flatten()
        });
        if let Some(carried_over) = carried_over {
            if let Some(carried_skill_id) = carried_over.source_skill_id {
                // The creation-time runtime cache belongs with the creation-time source skill; a
                // cache the re-notify resolved for another skill would revalue the instance.
                let incoming_skill_id = se.source_skill_id;
                let incoming_runtime_snapshot = se.source_skill_runtime_snapshot.take();
                se.source_skill_id = Some(carried_skill_id);
                se.custom_id = carried_over.custom_id;
                se.source_skill_runtime_snapshot =
                    carried_over.source_skill_runtime_snapshot.or_else(|| {
                        if incoming_skill_id == Some(carried_skill_id) {
                            incoming_runtime_snapshot
                        } else {
                            runtime_for_carried_skill(carried_skill_id)
                        }
                    });
            }
            if carried_over.owner_player_stats_snapshot.is_some() {
                se.owner_player_stats_snapshot = carried_over.owner_player_stats_snapshot;
            }
            se.first_tracked = carried_over.first_tracked;
            se.created_before_zone = carried_over.created_before_zone;
            se.measured_stat = carried_over.measured_stat;
        }

        let now = se.timestamp;
        self.observe_server_clock_offset(&se);
        let epoch_moved_later = se.end_tick_on_zone_clock && self.observe_server_tick_epoch(&se);
        se.expire_at = self.expire_at_for(&se);
        se.packet_expired = false;
        self.note_stat_sources_changed(se.target_id, &se, now);

        self.registry_mut(se.target_type)
            .entry(se.target_id)
            .or_default()
            .insert(se.instance_id, se);
        if epoch_moved_later {
            self.refresh_expiry_from_server_end_ticks(now);
        }
    }

    /// The instance's deadline on the server clock: the server end tick is exact for every
    /// instance, including one the server extended before re-sending it in a snapshot, and
    /// PKTStatusEffectDurationNotify moves it on each later extension. The nominal duration,
    /// counted from the arrival time read on the server clock, only covers a finite instance
    /// without a usable end tick.
    fn expire_at_for(&self, se: &StatusEffectDetails) -> Option<DateTime<Utc>> {
        if !has_finite_duration(se.expiration_delay) {
            return None;
        }
        if se.end_tick_on_zone_clock
            && let Some(server_end_at) = self.server_end_time(se.end_tick)
        {
            return Some(server_end_at);
        }
        add_duration_clamped(
            self.local_to_server_time(se.timestamp),
            f64::from(se.expiration_delay),
        )
    }

    /// Samples the clock offset from a status notify's local arrival and server occurrence. A
    /// snapshot re-sends instances created earlier, so its samples only ever lower the estimate;
    /// a direct notify's sample also replaces the estimate inherited from the previous zone.
    fn observe_server_clock_offset(&mut self, se: &StatusEffectDetails) {
        if se.occur_time == DateTime::<Utc>::default() {
            return;
        }
        let sample = se.timestamp - se.occur_time;
        if (self.server_clock_offset_inherited && !se.created_before_zone)
            || self
                .server_clock_offset
                .is_none_or(|offset| sample < offset)
        {
            self.server_clock_offset = Some(sample);
            self.server_clock_offset_inherited = false;
        }
    }

    fn local_to_server_time(&self, local: DateTime<Utc>) -> DateTime<Utc> {
        self.server_clock_offset.map_or(local, |offset| {
            local.checked_sub_signed(offset).unwrap_or(local)
        })
    }

    fn server_to_local_time(&self, server: DateTime<Utc>) -> DateTime<Utc> {
        self.server_clock_offset.map_or(server, |offset| {
            server.checked_add_signed(offset).unwrap_or(server)
        })
    }

    pub fn note_zone_published(&mut self, object_id: u64) {
        self.zone_published_object_ids.insert(object_id);
    }

    pub fn is_zone_published(&self, object_id: u64) -> bool {
        self.zone_published_object_ids.contains(&object_id)
    }

    /// Records the tick epoch implied by a finite status effect and reports whether a later
    /// epoch was adopted.
    fn observe_server_tick_epoch(&mut self, se: &StatusEffectDetails) -> bool {
        if se.end_tick == INFINITE_SERVER_TICK
            || !has_finite_duration(se.expiration_delay)
            || se.end_tick > i64::MAX as u64
        {
            return false;
        }
        let occur_time = se.occur_time;
        if occur_time == DateTime::<Utc>::default() {
            return false;
        }
        let Some(end_time) = add_duration_clamped(occur_time, f64::from(se.expiration_delay))
        else {
            return false;
        };
        let Some(candidate_epoch) =
            end_time.checked_sub_signed(Duration::milliseconds(se.end_tick as i64))
        else {
            return false;
        };
        if let Some(epoch) = self.server_tick_epoch
            && (candidate_epoch - epoch).num_milliseconds() <= SERVER_TICK_EPOCH_CORROBORATION_MS
        {
            return false;
        }
        for recorded in self.server_tick_epoch_candidates.iter().flatten() {
            let (recorded_candidate_epoch, recorded_occur_time) = *recorded;
            if (occur_time - recorded_occur_time).num_milliseconds().abs()
                <= SERVER_TICK_EPOCH_CORROBORATION_MS
                || (candidate_epoch - recorded_candidate_epoch)
                    .num_milliseconds()
                    .abs()
                    > SERVER_TICK_EPOCH_CORROBORATION_MS
            {
                continue;
            }
            self.server_tick_epoch = Some(candidate_epoch.max(recorded_candidate_epoch));
            self.server_tick_epoch_candidates = [None; SERVER_TICK_EPOCH_CANDIDATE_COUNT];
            info!(
                "Status effect tick epoch adopted at {} from instance {} ({}).",
                self.server_tick_epoch.unwrap(),
                se.instance_id,
                se.status_effect_id
            );
            return true;
        }
        self.server_tick_epoch_candidates[self.server_tick_epoch_candidate_cursor] =
            Some((candidate_epoch, occur_time));
        self.server_tick_epoch_candidate_cursor =
            (self.server_tick_epoch_candidate_cursor + 1) % SERVER_TICK_EPOCH_CANDIDATE_COUNT;
        false
    }

    fn server_end_time(&self, end_tick: u64) -> Option<DateTime<Utc>> {
        if end_tick == INFINITE_SERVER_TICK {
            return None;
        }
        let epoch = self.server_tick_epoch?;
        let end_tick = i64::try_from(end_tick).ok()?;
        Some(
            epoch
                .checked_add_signed(Duration::milliseconds(end_tick))
                .unwrap_or(DateTime::<Utc>::MAX_UTC),
        )
    }

    /// Re-derives every finite instance's deadline from its server end tick after the tick epoch
    /// moved later. An instance the new deadline hides or restores is a stat source change.
    fn refresh_expiry_from_server_end_ticks(&mut self, now: DateTime<Utc>) {
        let Some(epoch) = self.server_tick_epoch else {
            return;
        };
        let server_now = self.local_to_server_time(now);
        let mut transitions = Vec::new();
        for registry in [
            &mut self.local_status_effect_registry,
            &mut self.party_status_effect_registry,
        ] {
            for (target_id, effect) in registry.iter_mut().flat_map(|(target_id, effects)| {
                effects.values_mut().map(move |effect| (*target_id, effect))
            }) {
                if !effect.end_tick_on_zone_clock
                    || effect.end_tick == INFINITE_SERVER_TICK
                    || !has_finite_duration(effect.expiration_delay)
                {
                    continue;
                }
                let Ok(end_tick) = i64::try_from(effect.end_tick) else {
                    continue;
                };
                let server_end_at = epoch
                    .checked_add_signed(Duration::milliseconds(end_tick))
                    .unwrap_or(DateTime::<Utc>::MAX_UTC);
                if let Some(transition) = apply_deadline(effect, Some(server_end_at), server_now) {
                    transitions.push((target_id, transition));
                }
            }
        }
        for (target_id, effect) in transitions {
            self.note_stat_sources_changed(target_id, &effect, now);
        }
    }

    /// Records that a source of an identity stat on `target_id` was added, refreshed, removed,
    /// hidden by its deadline or restored at `at`. A later recorded change never moves back.
    fn note_stat_sources_changed(
        &mut self,
        target_id: u64,
        se: &StatusEffectDetails,
        at: DateTime<Utc>,
    ) {
        for stat_type in IDENTITY_STAT_TYPES {
            if crate::live::rdps::effect_provides_passive_stat(se, identity_stat_key(stat_type)) {
                self.stat_source_change_at
                    .entry((target_id, stat_type))
                    .and_modify(|changed_at| *changed_at = (*changed_at).max(at))
                    .or_insert(at);
            }
        }
    }

    /// Whether any source of `stat_type` on one of `target_ids` was added, refreshed, removed or
    /// reset at or after `since`.
    pub fn has_stat_source_changed_since(
        &self,
        target_ids: &[u64],
        stat_type: StatType,
        since: DateTime<Utc>,
    ) -> bool {
        target_ids.iter().any(|target_id| {
            self.stat_source_change_at
                .get(&(*target_id, stat_type))
                .is_some_and(|changed_at| *changed_at >= since)
        })
    }

    /// Pins a measured stat on the live instance matching `sampled`; see `MeasuredStat`.
    pub fn try_apply_measured_stat(
        &mut self,
        target_ids: &[u64],
        sampled: &StatusEffectDetails,
        measured: MeasuredStat,
    ) -> bool {
        let mut applied = false;
        for registry in [
            &mut self.local_status_effect_registry,
            &mut self.party_status_effect_registry,
        ] {
            for target_id in target_ids {
                if let Some(effect) = registry
                    .get_mut(target_id)
                    .and_then(|effects| effects.get_mut(&sampled.instance_id))
                {
                    applied |= apply_measured_stat(effect, sampled, measured);
                }
            }
        }
        applied
    }

    /// The live entry for an instance on one of `target_ids`, if tracked.
    pub fn tracked_status_effect(
        &self,
        target_ids: &[u64],
        instance_id: u32,
    ) -> Option<&StatusEffectDetails> {
        [
            &self.local_status_effect_registry,
            &self.party_status_effect_registry,
        ]
        .into_iter()
        .flat_map(|registry| {
            target_ids
                .iter()
                .filter_map(move |target_id| registry.get(target_id))
        })
        .find_map(|effects| effects.get(&instance_id))
    }

    /// Applies `update` to every tracked effect owned by `source_id`.
    pub fn update_effects_owned_by(
        &mut self,
        source_id: u64,
        mut update: impl FnMut(&mut StatusEffectDetails),
    ) {
        for registry in [
            &mut self.local_status_effect_registry,
            &mut self.party_status_effect_registry,
        ] {
            for effect in registry
                .values_mut()
                .flat_map(|effects| effects.values_mut())
                .filter(|effect| effect.source_id == source_id)
            {
                update(effect);
            }
        }
    }

    /// Drops an object's tracked effects while retaining player-owned instances for a later
    /// re-notify. The server did not end these instances; only player-owned buffs carry an owner
    /// snapshot worth resuming, and NPC-owned permanent effects would accumulate.
    fn drop_object_effects(&mut self, target_type: StatusEffectTargetType, object_id: u64) {
        let Some(effects) = self.registry_mut(target_type).remove(&object_id) else {
            return;
        };
        let now = Utc::now();
        for (instance_id, effect) in effects {
            self.note_stat_sources_changed(object_id, &effect, now);
            if effect.owner_is_player {
                self.retained_entries_by_instance_id
                    .insert(instance_id, effect);
            }
        }
    }

    fn prune_retained_entries(&mut self, now: DateTime<Utc>) {
        let server_now = self.local_to_server_time(now);
        self.retained_entries_by_instance_id.retain(|_, effect| {
            effect
                .expire_at
                .is_none_or(|expire_at| expire_at > server_now)
        });
    }

    pub fn remove_local_object(&mut self, object_id: u64) {
        self.drop_object_effects(StatusEffectTargetType::Local, object_id);
    }

    pub fn remove_party_object(&mut self, object_id: u64) {
        self.drop_object_effects(StatusEffectTargetType::Party, object_id);
    }

    pub fn remove_status_effects(
        &mut self,
        target_id: u64,
        instance_id: Vec<u32>,
        reason: u8,
        sett: StatusEffectTargetType,
    ) -> (
        bool,
        Vec<StatusEffectDetails>,
        Vec<StatusEffectDetails>,
        bool,
    ) {
        let registry = match sett {
            StatusEffectTargetType::Local => &mut self.local_status_effect_registry,
            StatusEffectTargetType::Party => &mut self.party_status_effect_registry,
        };

        let mut has_shield_buff = false;
        let mut shields_broken: Vec<StatusEffectDetails> = Vec::new();
        let mut left_workshop = false;
        let mut effects_removed = Vec::new();
        let mut removed_entries = Vec::new();

        if let Some(ser) = registry.get_mut(&target_id) {
            for id in instance_id {
                // The server ended this instance; a retained copy must not resume it.
                self.retained_entries_by_instance_id.remove(&id);
                if let Some(se) = ser.remove(&id) {
                    removed_entries.push(se.clone());
                    if se.status_effect_id == WORKSHOP_BUFF_ID {
                        left_workshop = true;
                    }
                    if se.status_effect_type == StatusEffectType::Shield {
                        has_shield_buff = true;
                        if reason == 4 {
                            shields_broken.push(se);
                            continue;
                        }
                    }
                    effects_removed.push(se);
                }
            }
        }
        let now = Utc::now();
        for se in &removed_entries {
            self.note_stat_sources_changed(target_id, se, now);
        }

        (
            has_shield_buff,
            shields_broken,
            effects_removed,
            left_workshop,
        )
    }

    /// Applies a PKTStatusEffectDurationNotify: the server moved the instance's end tick.
    /// Changing a continuously present instance's deadline leaves its stat contribution
    /// unchanged; a deadline that hides the instance or restores a hidden one is a stat source
    /// change.
    pub fn update_status_duration(
        &mut self,
        instance_id: u32,
        target_id: u64,
        end_tick: u64,
        sett: StatusEffectTargetType,
        now: DateTime<Utc>,
    ) -> bool {
        let server_end_at = self.server_end_time(end_tick);
        let server_now = self.local_to_server_time(now);
        let Some(effect) = self
            .registry_mut(sett)
            .get_mut(&target_id)
            .and_then(|effects| effects.get_mut(&instance_id))
        else {
            return false;
        };
        let previous_end_tick = effect.end_tick;
        effect.end_tick = end_tick;
        let new_deadline = if end_tick == INFINITE_SERVER_TICK {
            None
        } else if let Some(server_end_at) = server_end_at {
            Some(server_end_at)
        } else {
            info!(
                "Status effect duration for instance {instance_id} on {target_id:X} arrived before any tick epoch sample: end tick {end_tick}."
            );
            return true;
        };
        effect.end_tick_on_zone_clock = true;
        let transition = apply_deadline(effect, new_deadline, server_now);
        let delta_text =
            if previous_end_tick == INFINITE_SERVER_TICK || end_tick == INFINITE_SERVER_TICK {
                "n/a".to_string()
            } else {
                format!(
                    "{:+.1} s",
                    (end_tick as f64 - previous_end_tick as f64) / 1000.0
                )
            };
        info!(
            "Buff {} ({}) on {target_id:X} instance {instance_id}: server end tick {previous_end_tick} -> {end_tick} ({delta_text}).",
            effect.name, effect.status_effect_id
        );
        if let Some(effect) = transition {
            self.note_stat_sources_changed(target_id, &effect, now);
        }
        true
    }

    pub fn sync_status_effect(
        &mut self,
        instance_id: u32,
        character_id: u64,
        object_id: u64,
        value: u64,
        local_character_id: u64,
    ) -> (Option<StatusEffectDetails>, u64) {
        let use_party = self.should_use_party_status_effect(character_id, local_character_id);
        let (target_id, sett) = if use_party {
            (character_id, StatusEffectTargetType::Party)
        } else {
            (object_id, StatusEffectTargetType::Local)
        };
        if target_id == 0 {
            return (None, 0);
        }
        let registry = match sett {
            StatusEffectTargetType::Local => &mut self.local_status_effect_registry,
            StatusEffectTargetType::Party => &mut self.party_status_effect_registry,
        };

        let ser = match registry.get_mut(&target_id) {
            Some(ser) => ser,
            None => return (None, 0),
        };

        let se = match ser.get_mut(&instance_id) {
            Some(se) => se,
            None => return (None, 0),
        };

        let old_value = se.value;
        se.value = value;

        (Some(se.clone()), old_value)
    }

    pub fn get_status_effects(
        &mut self,
        source_entity: &Entity,
        target_entity: &Entity,
        local_character_id: u64,
    ) -> (Vec<StatusEffectDetails>, Vec<StatusEffectDetails>) {
        let timestamp = Utc::now();

        let use_party_for_source = if source_entity.entity_type == EntityType::Player {
            self.should_use_party_status_effect(source_entity.character_id, local_character_id)
        } else {
            false
        };
        // println!("use_party_for_source: {:?}", use_party_for_source);
        let (source_id, source_type) = if use_party_for_source {
            (source_entity.character_id, StatusEffectTargetType::Party)
        } else {
            (source_entity.id, StatusEffectTargetType::Local)
        };
        // println!("source_id: {:?}, source_type: {:?}", source_id, source_type);

        let status_effects_on_source = self.actually_get_status_effects(
            source_id,
            source_type,
            timestamp,
            DeadlineMode::KeepUntilRemoved,
        );

        // Party filtering for cross-party effects on the target is done downstream
        // via rdps::filter_target_effects_for_attacker (and analyze_hit_rdps's own
        // should_apply_target_effect), so we just pull every effect from the right
        // registry here.
        let use_party_for_target = source_entity.entity_type == EntityType::Player
            && self.should_use_party_status_effect(target_entity.character_id, local_character_id);
        let (target_id, target_type) = if use_party_for_target {
            (target_entity.character_id, StatusEffectTargetType::Party)
        } else {
            (target_entity.id, StatusEffectTargetType::Local)
        };
        let mut status_effects_on_target = self.actually_get_status_effects(
            target_id,
            target_type,
            timestamp,
            DeadlineMode::HideExpired,
        );
        // println!("status_effects_on_target: {:?}", status_effects_on_target);
        // println!(
        //     "status_effects_on_source: {:?}, status_effects_on_target: {:?}",
        //     status_effects_on_source, status_effects_on_target);
        status_effects_on_target.retain(|se| {
            target_entity.npc_id != 0
                || !(se.target_type == StatusEffectTargetType::Local
                    && se.category == Debuff
                    && se.source_id != source_id
                    && se.db_target_type == "self")
        });
        (status_effects_on_source, status_effects_on_target)
    }

    pub fn get_source_status_effects(
        &mut self,
        source_entity: &Entity,
        timestamp: DateTime<Utc>,
        deadline_mode: DeadlineMode,
    ) -> Vec<StatusEffectDetails> {
        if source_entity.entity_type != EntityType::Player {
            return self.actually_get_status_effects(
                source_entity.id,
                StatusEffectTargetType::Local,
                timestamp,
                deadline_mode,
            );
        }

        let mut merged = HashMap::new();
        for effect in self.actually_get_status_effects(
            source_entity.id,
            StatusEffectTargetType::Local,
            timestamp,
            deadline_mode,
        ) {
            merged.insert(
                (
                    effect.instance_id,
                    effect.status_effect_id,
                    effect.source_id,
                ),
                effect,
            );
        }
        for effect in self.actually_get_status_effects(
            source_entity.character_id,
            StatusEffectTargetType::Party,
            timestamp,
            deadline_mode,
        ) {
            let key = (
                effect.instance_id,
                effect.status_effect_id,
                effect.source_id,
            );
            match merged.entry(key) {
                hashbrown::hash_map::Entry::Occupied(mut entry) => {
                    if effect.timestamp > entry.get().timestamp {
                        entry.insert(effect);
                    }
                }
                hashbrown::hash_map::Entry::Vacant(entry) => {
                    entry.insert(effect);
                }
            }
        }

        let mut effects = merged.into_values().collect::<Vec<_>>();
        effects.sort_by_key(|effect| {
            (
                effect.timestamp,
                effect.instance_id,
                effect.status_effect_id,
                effect.source_id,
            )
        });
        effects
    }

    /// The instances visible on `target_id` at local time `timestamp` under `deadline_mode`.
    pub fn actually_get_status_effects(
        &mut self,
        target_id: u64,
        sett: StatusEffectTargetType,
        timestamp: DateTime<Utc>,
        deadline_mode: DeadlineMode,
    ) -> Vec<StatusEffectDetails> {
        let server_now = self.local_to_server_time(timestamp);
        let fail_safe_cutoff = server_now
            .checked_sub_signed(Duration::seconds(FAIL_SAFE_EXPIRY_LEEWAY_SECONDS))
            .unwrap_or(DateTime::<Utc>::MIN_UTC);
        let registry = match sett {
            StatusEffectTargetType::Local => &mut self.local_status_effect_registry,
            StatusEffectTargetType::Party => &mut self.party_status_effect_registry,
        };

        let ser = match registry.get_mut(&target_id) {
            Some(ser) => ser,
            None => return Vec::new(),
        };

        // Expired instances stay tracked until the server removes them or the fail-safe leeway
        // passes: a later duration correction can restore an instance whose nominal deadline
        // was too early. Reaching the deadline removes the instance's stat contribution from
        // deadline readers, which is recorded once, at the deadline itself.
        ser.retain(|_, se| {
            se.expire_at
                .is_none_or(|expire_at| expire_at > fail_safe_cutoff)
        });
        let mut visible = Vec::with_capacity(ser.len());
        let mut newly_expired = Vec::new();
        for se in ser.values_mut() {
            match se.expire_at {
                Some(expire_at) if expire_at <= server_now => {
                    if !se.packet_expired {
                        se.packet_expired = true;
                        newly_expired.push((se.clone(), expire_at));
                    }
                    if deadline_mode == DeadlineMode::KeepUntilRemoved {
                        visible.push(se.clone());
                    }
                }
                _ => visible.push(se.clone()),
            }
        }
        for (se, expire_at) in newly_expired {
            let expired_at = self.server_to_local_time(expire_at);
            self.note_stat_sources_changed(target_id, &se, expired_at);
        }
        visible
    }

    fn should_use_party_status_effect(&self, character_id: u64, local_character_id: u64) -> bool {
        let party_tracker = self.party_tracker.borrow();
        let local_player_party_id = party_tracker
            .character_id_to_party_id
            .get(&local_character_id);
        let affected_player_party_id = party_tracker.character_id_to_party_id.get(&character_id);
        // println!("party character_id_to_party_id: {:?}", party_tracker.character_id_to_party_id);
        // println!("character_id: {}, local_character_id: {}", character_id, local_character_id);
        // println!(
        //     "local_player_party_id: {:?}, affected_player_party_id: {:?}",
        //     local_player_party_id, affected_player_party_id);

        match (
            local_player_party_id,
            affected_player_party_id,
            character_id == local_character_id,
        ) {
            (Some(local_party), Some(affected_party), false) => local_party == affected_party,
            _ => false,
        }
    }

    /// Zone change: every tracked instance predates the new zone. Player-owned instances are
    /// retained for a re-notify, and their owner may have changed build before entering.
    pub fn clear(&mut self) {
        let targets = self
            .local_status_effect_registry
            .keys()
            .map(|target_id| (StatusEffectTargetType::Local, *target_id))
            .chain(
                self.party_status_effect_registry
                    .keys()
                    .map(|target_id| (StatusEffectTargetType::Party, *target_id)),
            )
            .collect::<Vec<_>>();
        for (target_type, target_id) in targets {
            self.drop_object_effects(target_type, target_id);
        }
        for retained in self.retained_entries_by_instance_id.values_mut() {
            retained.created_before_zone = true;
        }
        self.stat_source_change_at.clear();
        self.zone_published_object_ids.clear();
        self.server_tick_epoch = None;
        self.server_tick_epoch_candidates = [None; SERVER_TICK_EPOCH_CANDIDATE_COUNT];
        self.server_tick_epoch_candidate_cursor = 0;
        self.server_clock_offset_inherited = self.server_clock_offset.is_some();
        self.local_status_effect_registry.clear();
        self.party_status_effect_registry.clear();
    }

    pub fn has_status_effect(
        &self,
        character_id: u64,
        object_id: u64,
        local_character_id: u64,
        status_effect_id: u32,
    ) -> bool {
        let use_party = self.should_use_party_status_effect(character_id, local_character_id);
        let registry = if use_party {
            &self.party_status_effect_registry
        } else {
            &self.local_status_effect_registry
        };
        let target_id = if use_party { character_id } else { object_id };
        if target_id == 0 {
            return false;
        }

        registry.get(&target_id).is_some_and(|effects| {
            effects
                .values()
                .any(|effect| effect.status_effect_id == status_effect_id)
        })
    }
}

pub fn build_status_effect(
    se_data: &StatusEffectData,
    target_id: u64,
    source_id: u64,
    target_type: StatusEffectTargetType,
    timestamp: DateTime<Utc>,
    source_entity: Option<&EncounterEntity>,
) -> StatusEffectDetails {
    let value = get_status_effect_value(&se_data.value.bytearray_0);
    let mut status_effect_category = StatusEffectCategory::Other;
    let mut buff_category = StatusEffectBuffCategory::Other;
    let mut show_type = StatusEffectShowType::Other;
    let mut status_effect_type = StatusEffectType::Other;
    let mut name = "Unknown".to_string();
    let mut db_target_type = "".to_string();
    let mut custom_id = 0;
    let mut unique_group = 0;
    let mut source_skill_id = None;
    let mut buff_type_flags = 0;
    if let Some(effect) = SKILL_BUFF_DATA.get(&se_data.status_effect_id) {
        name = effect.name.clone().unwrap_or_default();
        unique_group = remap_effective_unique_group(se_data.status_effect_id, effect.unique_group);
        buff_type_flags = get_status_effect_buff_type_flags(effect);
        if effect.category.as_str() == "debuff" {
            status_effect_category = Debuff
        }
        match effect.buff_category.clone().unwrap_or_default().as_str() {
            "bracelet" => buff_category = Bracelet,
            "etc" => buff_category = Etc,
            "battleitem" => buff_category = BattleItem,
            "elixir" => buff_category = Elixir,
            _ => {}
        }
        if effect.icon_show_type.clone().unwrap_or_default() == "all" {
            show_type = All
        }
        status_effect_type = match effect.buff_type.as_str() {
            "shield" => StatusEffectType::Shield,
            "freeze" | "fear" | "stun" | "sleep" | "earthquake" | "electrocution"
            | "polymorph_pc" | "forced_move" | "mind_control" | "paralyzation"
            | "psychokinesis" => StatusEffectType::HardCrowdControl,
            _ => StatusEffectType::Other,
        };
        db_target_type = effect.target.to_string();

        if let Some(source_skills) = effect.source_skills.as_ref() {
            if source_skills.len() == 1 {
                source_skill_id = source_skills.first().copied();
            }
            // if skill has multiple source skills, we need to find the one that was last used
            // e.g. bard brands have same buff id, but have different source skills (sound shock, harp)
            // if skills only have one source skill, we dont care about it here and it gets handled later
            if source_skills.len() > 1
                && let Some(source_entity) = source_entity
            {
                let mut last_time = i64::MIN;
                let mut last_skill = 0_u32;
                for source_skill in source_skills {
                    if let Some(skill) = source_entity.skills.get(source_skill) {
                        if skill.name.is_empty() {
                            continue;
                        }
                        // hard code check for stigma brand tripod
                        // maybe set up a map of tripods for other skills in future idk??
                        if skill.id == 21090 {
                            if let Some(tripods) = skill.tripod_index {
                                if tripods.second != 2 {
                                    continue;
                                }
                            } else {
                                continue;
                            }
                        }
                        if skill.last_timestamp > last_time {
                            last_skill = *source_skill;
                            last_time = skill.last_timestamp;
                        }
                    }
                }

                // if such a skill exists, we assign a new custom buff id to distinguish it.
                // we encode the buff id as well too because there are multiple buffs that have
                // the same source skill, that also have multiple source skills.
                // without it, it leads to customids that are different but end up sharing the same id
                if last_skill > 0 {
                    source_skill_id = Some(last_skill);
                    custom_id = get_new_id(last_skill + (effect.id as u32));
                }
            }
        } else {
            source_skill_id = Some((unique_group / 10).max((effect.id as u32) / 10));
        }
    }

    StatusEffectDetails {
        instance_id: se_data.status_effect_instance_id,
        source_id,
        source_skill_id,
        target_id,
        status_effect_id: se_data.status_effect_id,
        custom_id,
        target_type,
        db_target_type,
        skill_level: se_data.skill_level,
        buff_type_flags,
        value,
        stack_count: se_data.stack_count,
        buff_category,
        category: status_effect_category,
        status_effect_type,
        show_type,
        expiration_delay: se_data.total_time,
        expire_at: None,
        end_tick: se_data.end_tick,
        occur_time: se_data.occur_time,
        name,
        timestamp,
        unique_group,
        owner_player_stats_snapshot: None,
        source_skill_runtime_snapshot: None,
        owner_is_player: false,
        first_tracked: timestamp,
        created_before_zone: false,
        measured_stat: None,
        end_tick_on_zone_clock: true,
        packet_expired: false,
    }
}

/// The `HideExpired` view of a `KeepUntilRemoved` read taken at the same moment. LAL freezes a
/// queued hit's attacker buffs from its packet snapshot, so instances past their deadline drop
/// out of the queued copy while the live hit keeps them.
pub fn hide_expired(effects: Vec<StatusEffectDetails>) -> Vec<StatusEffectDetails> {
    effects
        .into_iter()
        .filter(|effect| !effect.packet_expired)
        .collect()
}

/// Moves an instance's deadline and reports the instance when its visibility at `now` changed:
/// a continuously present instance keeps its stat contribution, while one the new deadline hides
/// or restores is a stat source change.
fn apply_deadline(
    effect: &mut StatusEffectDetails,
    deadline: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> Option<StatusEffectDetails> {
    let was_visible =
        !effect.packet_expired && effect.expire_at.is_none_or(|expire_at| expire_at > now);
    effect.expire_at = deadline;
    let is_visible = deadline.is_none_or(|expire_at| expire_at > now);
    effect.packet_expired = !is_visible;
    (was_visible != is_visible).then(|| effect.clone())
}

fn has_finite_duration(total_time: f32) -> bool {
    total_time > 0.0 && total_time.is_finite() && total_time != f32::MAX
}

fn add_duration_clamped(start: DateTime<Utc>, duration_seconds: f64) -> Option<DateTime<Utc>> {
    if !duration_seconds.is_finite() || duration_seconds <= 0.0 {
        return None;
    }
    let duration_ms = (duration_seconds * 1000.0).min(i64::MAX as f64) as i64;
    Some(
        start
            .checked_add_signed(Duration::milliseconds(duration_ms))
            .unwrap_or(DateTime::<Utc>::MAX_UTC),
    )
}

/// Pins `measured` on `effect` when it is the same server-side instance as `sampled`.
pub fn apply_measured_stat(
    effect: &mut StatusEffectDetails,
    sampled: &StatusEffectDetails,
    measured: MeasuredStat,
) -> bool {
    if effect.instance_id != sampled.instance_id
        || effect.status_effect_id != sampled.status_effect_id
        || effect.source_id != sampled.source_id
        || effect.first_tracked != sampled.first_tracked
        || effect.measured_stat.is_some()
    {
        return false;
    }
    effect.measured_stat = Some(measured);
    true
}

pub fn get_status_effect_value(value: &Option<Vec<u8>>) -> u64 {
    value.as_ref().map_or(0, |v| {
        let c1 = v
            .get(0..8)
            .map_or(0, |bytes| u64::from_le_bytes(bytes.try_into().unwrap()));
        let c2 = v
            .get(8..16)
            .map_or(0, |bytes| u64::from_le_bytes(bytes.try_into().unwrap()));
        c1.min(c2)
    })
}

fn remap_effective_unique_group(status_effect_id: u32, unique_group: u32) -> u32 {
    match status_effect_id {
        // LAL keeps the Paladin self aura pieces independent from the party aura group.
        2360068 | 2360124 => 0,
        2360125 => 360102,
        _ => unique_group,
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum StatusEffectTargetType {
    #[default]
    Party = 0,
    Local = 1,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum StatusEffectCategory {
    #[default]
    Other = 0,
    Debuff = 1,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum StatusEffectBuffCategory {
    #[default]
    Other = 0,
    Bracelet = 1,
    Etc = 2,
    BattleItem = 3,
    Elixir = 4,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum StatusEffectShowType {
    #[default]
    Other = 0,
    All = 1,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum StatusEffectType {
    #[default]
    Shield = 0,
    Other = 1,
    HardCrowdControl = 2, // stun, root, MC, etc
}

#[derive(Debug, Default, Clone)]
pub struct StatusEffectDetails {
    pub instance_id: u32,
    pub status_effect_id: u32,
    pub custom_id: u32,
    pub target_id: u64,
    pub source_id: u64,
    pub source_skill_id: Option<u32>,
    pub target_type: StatusEffectTargetType,
    pub db_target_type: String,
    pub skill_level: u8,
    pub buff_type_flags: u32,
    pub value: u64,
    pub stack_count: u8,
    pub category: StatusEffectCategory,
    pub buff_category: StatusEffectBuffCategory,
    pub show_type: StatusEffectShowType,
    pub status_effect_type: StatusEffectType,
    pub expiration_delay: f32,
    /// Deadline on the server clock, see `StatusTracker::server_clock_offset`; None for a
    /// permanent instance.
    pub expire_at: Option<DateTime<Utc>>,
    pub end_tick: u64,
    /// Server-reported occurrence time of the instance, from the status effect data.
    pub occur_time: DateTime<Utc>,
    pub timestamp: DateTime<Utc>,
    pub name: String,
    pub unique_group: u32,
    pub owner_player_stats_snapshot: Option<Arc<PlayerStats>>,
    pub source_skill_runtime_snapshot: Option<SkillRuntimeData>,
    /// The resolved source is a player, so the instance carries an owner snapshot worth
    /// resuming across zone and object snapshot resets.
    pub owner_is_player: bool,
    /// When this status effect instance was first tracked; kept across same-instance
    /// re-notifies.
    pub first_tracked: DateTime<Utc>,
    /// The instance already existed when the current zone was entered, or was first seen
    /// through an object snapshot, so the owner's build at its creation may differ from the
    /// build the meter knows.
    pub created_before_zone: bool,
    /// See `MeasuredStat`.
    pub measured_stat: Option<MeasuredStat>,
    /// The instance's end tick is on this zone's server clock (seen through a direct notify, a
    /// snapshot, a duration notify, or a party notify for a player published in this zone), so
    /// it converts through the tick epoch. Party notifies for players never published here may
    /// come from another server's clock.
    pub end_tick_on_zone_clock: bool,
    /// The deadline passed and the instance's removal from the visible set was recorded as a stat
    /// source change; cleared when a correction restores it.
    pub packet_expired: bool,
}

#[cfg(test)]
mod buff_instance_tests {
    use super::*;
    use crate::live::id_tracker::IdTracker;
    use chrono::TimeZone;

    pub(super) const RECEIVER: u64 = 99;
    const OWNER: u64 = 61;

    pub(super) fn tracker() -> StatusTracker {
        let ids = Rc::new(RefCell::new(IdTracker::new()));
        StatusTracker::new(Rc::new(RefCell::new(PartyTracker::new(ids))))
    }

    /// Fixture time `seconds` after a base near now: retained entries are pruned against the
    /// wall clock, so a base in the past would drop them before a re-notify resumes them.
    pub(super) fn at(seconds: f64) -> DateTime<Utc> {
        static BASE_MS: std::sync::OnceLock<i64> = std::sync::OnceLock::new();
        let base_ms = *BASE_MS.get_or_init(|| Utc::now().timestamp_millis() / 1000 * 1000);
        Utc.timestamp_millis_opt(base_ms + (seconds * 1000.0) as i64)
            .unwrap()
    }

    /// A finite status whose end tick is on the fixture clock: tick 0 is 100 s before `at(0)`.
    pub(super) fn finite_status(
        target_id: u64,
        instance_id: u32,
        duration: f32,
        occurrence: f64,
    ) -> StatusEffectDetails {
        StatusEffectDetails {
            instance_id,
            status_effect_id: 1000 + instance_id,
            target_id,
            source_id: OWNER,
            target_type: StatusEffectTargetType::Local,
            expiration_delay: duration,
            end_tick: ((100.0 + occurrence + f64::from(duration)) * 1000.0) as u64,
            occur_time: at(occurrence),
            timestamp: at(occurrence),
            first_tracked: at(occurrence),
            end_tick_on_zone_clock: true,
            owner_is_player: true,
            ..Default::default()
        }
    }

    pub(super) fn has(
        tracker: &mut StatusTracker,
        target_id: u64,
        instance_id: u32,
        time: DateTime<Utc>,
    ) -> bool {
        tracker
            .actually_get_status_effects(
                target_id,
                StatusEffectTargetType::Local,
                time,
                DeadlineMode::HideExpired,
            )
            .iter()
            .any(|effect| effect.instance_id == instance_id)
    }

    /// The tick epoch is adopted only once two statuses from different server moments agree.
    pub(super) fn seed_server_tick_epoch(tracker: &mut StatusTracker) {
        tracker.register_status_effect(finite_status(RECEIVER, 9001, 1.0, 0.0));
        tracker.register_status_effect(finite_status(RECEIVER, 9002, 1.0, 0.3));
        assert_eq!(tracker.server_tick_epoch, Some(at(-100.0)));
    }

    #[test]
    fn nominal_deadline_expires_a_finite_status_without_an_epoch() {
        let mut tracker = tracker();
        tracker.register_status_effect(finite_status(RECEIVER, 1, 30.0, 0.0));
        assert_eq!(tracker.server_tick_epoch, None);
        assert!(has(&mut tracker, RECEIVER, 1, at(29.0)));
        assert!(!has(&mut tracker, RECEIVER, 1, at(30.0)));
        // Infinite statuses never expire by time.
        let mut permanent = finite_status(RECEIVER, 2, 0.0, 0.0);
        permanent.end_tick = INFINITE_SERVER_TICK;
        tracker.register_status_effect(permanent);
        assert!(has(&mut tracker, RECEIVER, 2, at(100_000.0)));
    }

    #[test]
    fn reliable_epoch_sample_restores_an_extended_snapshot_until_its_server_end() {
        let mut tracker = tracker();
        let mut extended = finite_status(RECEIVER, 1, 12.0, 100.0);
        extended.end_tick = 220_000;
        extended.created_before_zone = true;
        tracker.register_status_effect(extended);
        tracker.register_status_effect(finite_status(RECEIVER, 2, 5.0, 100.0));
        tracker.register_status_effect(finite_status(RECEIVER, 3, 5.0, 100.5));
        assert!(has(&mut tracker, RECEIVER, 1, at(119.0)));
        assert!(!has(&mut tracker, RECEIVER, 1, at(120.0)));
    }

    #[test]
    fn duration_correction_restores_packet_membership() {
        for received_at in [9.0, 10.0, 11.0] {
            let mut tracker = tracker();
            seed_server_tick_epoch(&mut tracker);
            tracker.register_status_effect(finite_status(RECEIVER, 1, 10.0, 0.0));
            assert_eq!(
                has(&mut tracker, RECEIVER, 1, at(received_at)),
                received_at < 10.0
            );
            assert!(tracker.update_status_duration(
                1,
                RECEIVER,
                120_000,
                StatusEffectTargetType::Local,
                at(received_at)
            ));
            assert!(
                has(&mut tracker, RECEIVER, 1, at(19.0)),
                "received at {received_at}"
            );
            assert!(
                !has(&mut tracker, RECEIVER, 1, at(20.0)),
                "received at {received_at}"
            );
        }
    }

    #[test]
    fn duration_correction_does_not_undo_explicit_removal() {
        let mut tracker = tracker();
        seed_server_tick_epoch(&mut tracker);
        tracker.register_status_effect(finite_status(RECEIVER, 1, 10.0, 0.0));
        tracker.remove_status_effects(RECEIVER, vec![1], 0, StatusEffectTargetType::Local);
        assert!(!tracker.update_status_duration(
            1,
            RECEIVER,
            130_000,
            StatusEffectTargetType::Local,
            at(1.0)
        ));
        assert!(!has(&mut tracker, RECEIVER, 1, at(1.0)));
        assert!(!tracker.retained_entries_by_instance_id.contains_key(&1));
    }

    #[test]
    fn epoch_adoption_needs_two_agreeing_samples_from_different_moments() {
        let mut tracker = tracker();
        // Same server moment (a bundle applied by one skill) does not corroborate; the
        // extension cannot convert yet.
        tracker.register_status_effect(finite_status(RECEIVER, 1, 10.0, 0.0));
        tracker.register_status_effect(finite_status(RECEIVER, 2, 10.0, 0.0));
        assert!(tracker.update_status_duration(
            1,
            RECEIVER,
            120_000,
            StatusEffectTargetType::Local,
            at(0.5)
        ));
        assert!(!has(&mut tracker, RECEIVER, 1, at(10.0)));
        // A sample from a later moment corroborates the epoch, after which extensions convert.
        tracker.register_status_effect(finite_status(RECEIVER, 3, 10.0, 10.0));
        tracker.register_status_effect(finite_status(RECEIVER, 4, 10.0, 10.5));
        assert!(tracker.update_status_duration(
            3,
            RECEIVER,
            130_000,
            StatusEffectTargetType::Local,
            at(11.0)
        ));
        assert!(has(&mut tracker, RECEIVER, 3, at(29.0)));
        assert!(!has(&mut tracker, RECEIVER, 3, at(30.0)));
    }

    #[test]
    fn relative_tick_refresh_stream_never_moves_the_epoch() {
        let mut tracker = tracker();
        seed_server_tick_epoch(&mut tracker);
        tracker.register_status_effect(finite_status(RECEIVER, 1, 30.0, 0.0));
        // One player's party refreshes carried the remaining duration in the end tick: each
        // sample implies an epoch equal to its own occurrence time, so a stream of them must
        // never be adopted.
        for second in 1..=10 {
            let mut relative = finite_status(98, 2, 10.0, f64::from(second));
            relative.end_tick = 10_000;
            tracker.register_status_effect(relative);
        }
        assert_eq!(tracker.server_tick_epoch, Some(at(-100.0)));
        assert!(has(&mut tracker, RECEIVER, 1, at(29.0)));
        assert!(!has(&mut tracker, RECEIVER, 1, at(30.0)));
    }

    #[test]
    fn renotification_keeps_known_source_and_owner_snapshot() {
        for snapshot_reset in [false, true] {
            let mut tracker = tracker();
            let owner_stats = Arc::new(PlayerStats::default());
            let mut original = finite_status(RECEIVER, 1, 30.0, 0.0);
            original.source_skill_id = Some(31050);
            original.owner_player_stats_snapshot = Some(owner_stats.clone());
            tracker.register_status_effect(original);

            if snapshot_reset {
                tracker.remove_local_object(RECEIVER);
                assert!(!has(&mut tracker, RECEIVER, 1, at(1.0)));
            }
            let mut renotified = finite_status(RECEIVER, 1, 30.0, 5.0);
            renotified.owner_player_stats_snapshot = Some(Arc::new(PlayerStats::default()));
            tracker.register_status_effect(renotified);

            let refreshed = tracker
                .actually_get_status_effects(
                    RECEIVER,
                    StatusEffectTargetType::Local,
                    at(6.0),
                    DeadlineMode::HideExpired,
                )
                .into_iter()
                .find(|effect| effect.instance_id == 1)
                .unwrap();
            assert_eq!(refreshed.source_skill_id, Some(31050));
            assert!(Arc::ptr_eq(
                refreshed.owner_player_stats_snapshot.as_ref().unwrap(),
                &owner_stats
            ));
            assert_eq!(refreshed.first_tracked, at(0.0));
            assert!(!refreshed.created_before_zone);
        }
    }

    #[test]
    fn zone_change_retains_player_owned_instances_as_pre_zone_and_drops_npc_owned_ones() {
        let mut tracker = tracker();
        let owner_stats = Arc::new(PlayerStats::default());
        let mut player_owned = finite_status(RECEIVER, 1, 300.0, 0.0);
        player_owned.owner_player_stats_snapshot = Some(owner_stats.clone());
        tracker.register_status_effect(player_owned);
        let mut npc_owned = finite_status(RECEIVER, 2, 300.0, 0.0);
        npc_owned.owner_is_player = false;
        npc_owned.source_id = 777;
        tracker.register_status_effect(npc_owned);
        tracker.clear();
        assert!(!has(&mut tracker, RECEIVER, 1, at(1.0)));

        tracker.register_status_effect(finite_status(RECEIVER, 1, 300.0, 5.0));
        let mut npc_renotified = finite_status(RECEIVER, 2, 300.0, 5.0);
        npc_renotified.owner_is_player = false;
        npc_renotified.source_id = 777;
        tracker.register_status_effect(npc_renotified);

        let effects = tracker.actually_get_status_effects(
            RECEIVER,
            StatusEffectTargetType::Local,
            at(6.0),
            DeadlineMode::HideExpired,
        );
        let resumed = effects
            .iter()
            .find(|effect| effect.instance_id == 1)
            .unwrap();
        assert!(resumed.created_before_zone);
        assert!(Arc::ptr_eq(
            resumed.owner_player_stats_snapshot.as_ref().unwrap(),
            &owner_stats
        ));
        let npc = effects
            .iter()
            .find(|effect| effect.instance_id == 2)
            .unwrap();
        assert!(!npc.created_before_zone);
        assert_eq!(npc.first_tracked, at(5.0));
    }

    #[test]
    fn measured_stat_pins_only_the_same_instance_once() {
        let sampled = finite_status(RECEIVER, 1, 30.0, 0.0);
        let measured = MeasuredStat {
            stat_type: StatType::SKILL_DAMAGE_SUB_RATE_2,
            value: 2500,
        };
        let mut same = sampled.clone();
        assert!(apply_measured_stat(&mut same, &sampled, measured));
        assert_eq!(same.measured_stat, Some(measured));
        assert!(!apply_measured_stat(&mut same, &sampled, measured));

        let mut reused_instance = finite_status(RECEIVER, 1, 30.0, 40.0);
        assert!(!apply_measured_stat(
            &mut reused_instance,
            &sampled,
            measured
        ));
        assert_eq!(reused_instance.measured_stat, None);

        let mut tracker = tracker();
        tracker.register_status_effect(sampled.clone());
        assert!(tracker.try_apply_measured_stat(&[RECEIVER], &sampled, measured));
        // A same-instance re-notify keeps the pinned measurement.
        tracker.register_status_effect(finite_status(RECEIVER, 1, 30.0, 5.0));
        assert_eq!(
            tracker
                .tracked_status_effect(&[RECEIVER], 1)
                .and_then(|effect| effect.measured_stat),
            Some(measured)
        );
    }
}

#[cfg(test)]
mod identity_source_history_tests {
    use super::buff_instance_tests::{
        RECEIVER, at, finite_status, has, seed_server_tick_epoch, tracker,
    };
    use super::*;
    use crate::live::test_data::initialize;

    const STAT: StatType = StatType::SKILL_DAMAGE_SUB_RATE_2;

    /// A tracked copy of a real identity buff that provides SKILL_DAMAGE_SUB_RATE_2.
    fn identity_status(
        instance_id: u32,
        status_effect_id: u32,
        source_skill_id: u32,
        duration: f32,
        occurrence: f64,
    ) -> StatusEffectDetails {
        let mut status = finite_status(RECEIVER, instance_id, duration, occurrence);
        status.status_effect_id = status_effect_id;
        status.source_skill_id = Some(source_skill_id);
        status
    }

    #[test]
    fn deadline_expiry_and_restoration_are_identity_stat_source_changes() {
        initialize();
        let mut tracker = tracker();
        seed_server_tick_epoch(&mut tracker);
        tracker.register_status_effect(identity_status(1, 310501, 31050, 30.0, 0.0));
        tracker.register_status_effect(identity_status(2, 211410, 21141, 8.0, 0.0));
        assert!(!tracker.has_stat_source_changed_since(&[RECEIVER], STAT, at(0.5)));

        // Serenade reaches its deadline while a request sent at 5 s is in flight: the sheet
        // still includes it, the tracked set no longer does.
        let visible = tracker.actually_get_status_effects(
            RECEIVER,
            StatusEffectTargetType::Local,
            at(10.0),
            DeadlineMode::HideExpired,
        );
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].instance_id, 1);
        assert!(tracker.has_stat_source_changed_since(&[RECEIVER], STAT, at(5.0)));
        // A request sent after the deadline saw the same set the sheet describes.
        assert!(!tracker.has_stat_source_changed_since(&[RECEIVER], STAT, at(9.0)));

        // A duration correction that restores the hidden instance is a change as well.
        assert!(tracker.update_status_duration(
            2,
            RECEIVER,
            120_000,
            StatusEffectTargetType::Local,
            at(12.0)
        ));
        assert!(has(&mut tracker, RECEIVER, 2, at(13.0)));
        assert!(tracker.has_stat_source_changed_since(&[RECEIVER], STAT, at(11.0)));

        // Shortening a present instance into the past hides it: a change.
        assert!(tracker.update_status_duration(
            1,
            RECEIVER,
            110_000,
            StatusEffectTargetType::Local,
            at(15.0)
        ));
        assert!(!has(&mut tracker, RECEIVER, 1, at(15.5)));
        assert!(tracker.has_stat_source_changed_since(&[RECEIVER], STAT, at(14.0)));
    }

    #[test]
    fn continuous_future_extension_is_not_a_stat_source_change() {
        initialize();
        let mut tracker = tracker();
        seed_server_tick_epoch(&mut tracker);
        tracker.register_status_effect(identity_status(1, 310501, 31050, 30.0, 0.0));
        assert!(tracker.update_status_duration(
            1,
            RECEIVER,
            140_000,
            StatusEffectTargetType::Local,
            at(5.0)
        ));
        assert!(has(&mut tracker, RECEIVER, 1, at(35.0)));
        assert!(!tracker.has_stat_source_changed_since(&[RECEIVER], STAT, at(1.0)));
    }

    #[test]
    fn epoch_correction_restoring_an_instance_is_a_stat_source_change() {
        initialize();
        let mut tracker = tracker();
        let mut extended = identity_status(1, 310501, 31050, 12.0, 100.0);
        extended.end_tick = 220_000;
        tracker.register_status_effect(extended);
        // Without an epoch the nominal deadline hides the extended instance.
        assert!(!has(&mut tracker, RECEIVER, 1, at(115.0)));
        // Two agreeing samples adopt the epoch; the refreshed server deadline restores it.
        tracker.register_status_effect(finite_status(RECEIVER, 2, 5.0, 116.0));
        tracker.register_status_effect(finite_status(RECEIVER, 3, 5.0, 116.5));
        assert!(has(&mut tracker, RECEIVER, 1, at(117.0)));
        assert!(!has(&mut tracker, RECEIVER, 1, at(120.0)));
        assert!(tracker.has_stat_source_changed_since(&[RECEIVER], STAT, at(116.2)));
    }

    fn runtime_snapshot(skill_level: u8) -> SkillRuntimeData {
        SkillRuntimeData {
            skill_level,
            ..Default::default()
        }
    }

    fn tracked_source(tracker: &StatusTracker, instance_id: u32) -> (Option<u32>, Option<u8>) {
        let effect = tracker
            .tracked_status_effect(&[RECEIVER], instance_id)
            .unwrap();
        (
            effect.source_skill_id,
            effect
                .source_skill_runtime_snapshot
                .as_ref()
                .map(|runtime| runtime.skill_level),
        )
    }

    #[test]
    fn renotification_keeps_the_creation_source_skill_with_its_own_cache() {
        let mut tracker = tracker();
        // A re-notify resolved through another source skill keeps the creation skill and cache.
        let mut original = finite_status(RECEIVER, 1, 30.0, 0.0);
        original.source_skill_id = Some(48041);
        original.source_skill_runtime_snapshot = Some(runtime_snapshot(2));
        tracker.register_status_effect(original);
        let mut renotified = finite_status(RECEIVER, 1, 30.0, 5.0);
        renotified.source_skill_id = Some(48042);
        renotified.source_skill_runtime_snapshot = Some(runtime_snapshot(5));
        tracker.register_status_effect_with(renotified, |_| {
            panic!("the creation cache must not be resolved again")
        });
        assert_eq!(tracked_source(&tracker, 1), (Some(48041), Some(2)));

        // The same source skill refreshed with a later cache also keeps the creation cache.
        let mut refreshed = finite_status(RECEIVER, 1, 30.0, 10.0);
        refreshed.source_skill_id = Some(48041);
        refreshed.source_skill_runtime_snapshot = Some(runtime_snapshot(7));
        tracker.register_status_effect(refreshed);
        assert_eq!(tracked_source(&tracker, 1), (Some(48041), Some(2)));

        // A creation skill without a cache resolves that skill's current cache, not the cache
        // the re-notify resolved for the other skill.
        let mut uncached = finite_status(RECEIVER, 2, 30.0, 0.0);
        uncached.source_skill_id = Some(48041);
        tracker.register_status_effect(uncached);
        let mut renotified = finite_status(RECEIVER, 2, 30.0, 5.0);
        renotified.source_skill_id = Some(48042);
        renotified.source_skill_runtime_snapshot = Some(runtime_snapshot(5));
        tracker.register_status_effect_with(renotified, |carried_skill_id| {
            assert_eq!(carried_skill_id, 48041);
            Some(runtime_snapshot(9))
        });
        assert_eq!(tracked_source(&tracker, 2), (Some(48041), Some(9)));

        // The same skill without a creation cache keeps the re-notify's cache for that skill.
        let mut uncached = finite_status(RECEIVER, 3, 30.0, 0.0);
        uncached.source_skill_id = Some(48041);
        tracker.register_status_effect(uncached);
        let mut refreshed = finite_status(RECEIVER, 3, 30.0, 5.0);
        refreshed.source_skill_id = Some(48041);
        refreshed.source_skill_runtime_snapshot = Some(runtime_snapshot(4));
        tracker.register_status_effect_with(refreshed, |_| panic!("same skill needs no resolver"));
        assert_eq!(tracked_source(&tracker, 3), (Some(48041), Some(4)));
    }
}

#[cfg(test)]
mod clock_domain_tests {
    use super::buff_instance_tests::{
        RECEIVER, at, finite_status, has, seed_server_tick_epoch, tracker,
    };
    use super::*;
    use crate::live::test_data::initialize;

    const STAT: StatType = StatType::SKILL_DAMAGE_SUB_RATE_2;

    /// A finite status whose notify arrived `skew` seconds of local clock after its server
    /// occurrence: the local clock runs `skew` ahead of the server (behind when negative).
    fn skewed_status(
        instance_id: u32,
        duration: f32,
        occurrence: f64,
        skew: f64,
    ) -> StatusEffectDetails {
        let mut status = finite_status(RECEIVER, instance_id, duration, occurrence);
        status.timestamp = at(occurrence + skew);
        status.first_tracked = status.timestamp;
        status
    }

    fn seed_skewed_epoch(tracker: &mut StatusTracker, skew: f64) {
        tracker.register_status_effect(skewed_status(9001, 1.0, 0.0, skew));
        tracker.register_status_effect(skewed_status(9002, 1.0, 0.3, skew));
        assert_eq!(tracker.server_tick_epoch, Some(at(-100.0)));
    }

    fn read(
        tracker: &mut StatusTracker,
        instance_id: u32,
        time: DateTime<Utc>,
        mode: DeadlineMode,
    ) -> bool {
        tracker
            .actually_get_status_effects(RECEIVER, StatusEffectTargetType::Local, time, mode)
            .iter()
            .any(|effect| effect.instance_id == instance_id)
    }

    #[test]
    fn server_deadlines_meet_the_local_clock_through_the_sampled_offset() {
        for skew in [3.5, -1.2] {
            let mut tracker = tracker();
            seed_skewed_epoch(&mut tracker, skew);
            // A 3 s status occurring at server second 5 ends at server second 8, which the local
            // clock reads as 8 + skew.
            tracker.register_status_effect(skewed_status(1, 3.0, 5.0, skew));
            assert!(
                has(&mut tracker, RECEIVER, 1, at(7.95 + skew)),
                "skew {skew}"
            );
            assert!(
                !has(&mut tracker, RECEIVER, 1, at(8.05 + skew)),
                "skew {skew}"
            );
        }
    }

    #[test]
    fn nominal_deadline_without_an_epoch_counts_from_arrival() {
        let skew = 3.5;
        let mut tracker = tracker();
        // A single sample adopts no epoch; the nominal 3 s run from the local arrival.
        tracker.register_status_effect(skewed_status(1, 3.0, 0.0, skew));
        assert_eq!(tracker.server_tick_epoch, None);
        assert!(has(&mut tracker, RECEIVER, 1, at(skew + 2.95)));
        assert!(!has(&mut tracker, RECEIVER, 1, at(skew + 3.05)));
    }

    #[test]
    fn duration_notify_deadline_meets_the_local_clock_through_the_offset() {
        let skew = 3.5;
        let mut tracker = tracker();
        seed_skewed_epoch(&mut tracker, skew);
        tracker.register_status_effect(skewed_status(1, 10.0, 0.0, skew));
        // The server moves the end to tick 120 000: server second 20, local 20 + skew.
        assert!(tracker.update_status_duration(
            1,
            RECEIVER,
            120_000,
            StatusEffectTargetType::Local,
            at(5.0 + skew)
        ));
        assert!(has(&mut tracker, RECEIVER, 1, at(19.95 + skew)));
        assert!(!has(&mut tracker, RECEIVER, 1, at(20.05 + skew)));
    }

    #[test]
    fn deadline_history_is_recorded_on_the_local_clock() {
        initialize();
        let skew = 3.5;
        let mut tracker = tracker();
        seed_skewed_epoch(&mut tracker, skew);
        let mut serenade = skewed_status(1, 8.0, 0.0, skew);
        serenade.status_effect_id = 211410;
        serenade.source_skill_id = Some(21141);
        tracker.register_status_effect(serenade);
        // The server deadline at second 8 is local 8 + skew; a later read records the change there.
        assert!(!has(&mut tracker, RECEIVER, 1, at(12.0 + skew)));
        assert!(tracker.has_stat_source_changed_since(&[RECEIVER], STAT, at(7.9 + skew)));
        assert!(!tracker.has_stat_source_changed_since(&[RECEIVER], STAT, at(8.1 + skew)));
    }

    #[test]
    fn a_stale_snapshot_sample_never_raises_the_offset() {
        let mut tracker = tracker();
        seed_skewed_epoch(&mut tracker, 3.5);
        // A snapshot re-sends an instance created 100 s ago: its arrival minus occurrence is far
        // above the real offset. Its own deadline still converts with the sampled 3.5 s.
        tracker.register_status_effect(skewed_status(1, 200.0, -100.0, 103.5));
        assert!(has(&mut tracker, RECEIVER, 1, at(103.45)));
        assert!(!has(&mut tracker, RECEIVER, 1, at(103.55)));
    }

    #[test]
    fn a_zone_change_keeps_the_offset_until_the_first_notify_replaces_it() {
        let mut tracker = tracker();
        seed_skewed_epoch(&mut tracker, 3.5);
        tracker.clear();
        // The clock was stepped between zones: the first samples of the new zone read 5 s and
        // replace the inherited 3.5 s even though they are larger.
        seed_skewed_epoch(&mut tracker, 5.0);
        tracker.register_status_effect(skewed_status(1, 3.0, 5.0, 5.0));
        assert!(has(&mut tracker, RECEIVER, 1, at(12.95)));
        assert!(!has(&mut tracker, RECEIVER, 1, at(13.05)));
    }

    #[test]
    fn a_zone_snapshot_never_replaces_the_inherited_offset() {
        let mut tracker = tracker();
        seed_skewed_epoch(&mut tracker, 3.5);
        tracker.clear();
        // The new zone's snapshot re-sends two instances created 100 s and 50 s ago; their
        // occurrence times adopt the epoch, and their arrival-minus-occurrence samples stay
        // above the inherited 3.5 s, so the deadlines at server seconds 100 and 150 convert
        // with it.
        for (instance_id, occurrence) in [(1, -100.0), (2, -50.0)] {
            let mut snapshot = skewed_status(instance_id, 200.0, occurrence, 3.5 - occurrence);
            snapshot.created_before_zone = true;
            tracker.register_status_effect(snapshot);
        }
        assert_eq!(tracker.server_tick_epoch, Some(at(-100.0)));
        assert!(has(&mut tracker, RECEIVER, 1, at(103.45)));
        assert!(!has(&mut tracker, RECEIVER, 1, at(103.55)));
        assert!(has(&mut tracker, RECEIVER, 2, at(153.45)));
        assert!(!has(&mut tracker, RECEIVER, 2, at(153.55)));
    }

    #[test]
    fn source_reads_keep_an_expired_instance_until_the_fail_safe_leeway_passes() {
        let mut tracker = tracker();
        seed_server_tick_epoch(&mut tracker);
        tracker.register_status_effect(finite_status(RECEIVER, 1, 3.0, 0.0));
        assert!(read(&mut tracker, 1, at(2.9), DeadlineMode::HideExpired));
        assert!(!read(&mut tracker, 1, at(3.1), DeadlineMode::HideExpired));
        assert!(read(
            &mut tracker,
            1,
            at(3.1),
            DeadlineMode::KeepUntilRemoved
        ));
        assert!(read(
            &mut tracker,
            1,
            at(3.0 + 999.9),
            DeadlineMode::KeepUntilRemoved
        ));
        assert!(!read(
            &mut tracker,
            1,
            at(3.0 + 1000.1),
            DeadlineMode::KeepUntilRemoved
        ));
        assert!(!read(
            &mut tracker,
            1,
            at(3.0 + 999.9),
            DeadlineMode::KeepUntilRemoved
        ));
    }

    #[test]
    fn hiding_expired_instances_of_a_source_read_yields_the_deadline_view() {
        let mut tracker = tracker();
        seed_server_tick_epoch(&mut tracker);
        tracker.register_status_effect(finite_status(RECEIVER, 1, 3.0, 0.0));
        tracker.register_status_effect(finite_status(RECEIVER, 2, 30.0, 0.0));
        let ids = |effects: &[StatusEffectDetails]| {
            let mut ids: Vec<u32> = effects.iter().map(|effect| effect.instance_id).collect();
            ids.sort_unstable();
            ids
        };
        let kept = tracker.actually_get_status_effects(
            RECEIVER,
            StatusEffectTargetType::Local,
            at(5.0),
            DeadlineMode::KeepUntilRemoved,
        );
        assert_eq!(ids(&kept), vec![1, 2, 9001, 9002]);
        let hidden = tracker.actually_get_status_effects(
            RECEIVER,
            StatusEffectTargetType::Local,
            at(5.0),
            DeadlineMode::HideExpired,
        );
        assert_eq!(ids(&hidden), vec![2]);
        assert_eq!(ids(&hide_expired(kept)), ids(&hidden));
    }

    #[test]
    fn a_remove_packet_ends_an_expired_instance_for_source_reads() {
        let mut tracker = tracker();
        seed_server_tick_epoch(&mut tracker);
        tracker.register_status_effect(finite_status(RECEIVER, 1, 3.0, 0.0));
        assert!(read(
            &mut tracker,
            1,
            at(5.0),
            DeadlineMode::KeepUntilRemoved
        ));
        tracker.remove_status_effects(RECEIVER, vec![1], 0, StatusEffectTargetType::Local);
        assert!(!read(
            &mut tracker,
            1,
            at(5.0),
            DeadlineMode::KeepUntilRemoved
        ));
    }

    #[test]
    fn hit_reads_keep_expired_attacker_buffs_and_hide_expired_target_effects() {
        let mut tracker = tracker();
        // 1 s statuses occurring 30 s ago are past their deadline at the wall-clock read.
        tracker.register_status_effect(finite_status(RECEIVER, 1, 1.0, -30.0));
        tracker.register_status_effect(finite_status(777, 2, 1.0, -30.0));
        let attacker = Entity {
            id: RECEIVER,
            entity_type: EntityType::Player,
            ..Default::default()
        };
        let target = Entity {
            id: 777,
            entity_type: EntityType::Boss,
            npc_id: 1,
            ..Default::default()
        };
        let (on_attacker, on_target) = tracker.get_status_effects(&attacker, &target, 0);
        assert!(on_attacker.iter().any(|effect| effect.instance_id == 1));
        assert!(!on_target.iter().any(|effect| effect.instance_id == 2));
    }
}
