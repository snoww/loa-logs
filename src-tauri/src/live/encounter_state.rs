use crate::api::{GetCharacterInfoArgs, NtpClock, StatsApi};
use crate::constants::{DARK_GRENADE_ENTITY_ID, DARK_GRENADE_ENTITY_NAME};
use crate::data::*;
use crate::database::Repository;
use crate::database::models::InsertEncounterArgs;
use crate::live::entity_tracker::{Entity, EntityTracker, SkillOptionSnapshot};
use crate::live::rdps::{
    HitCritMetrics, HitRdpsOutcome, HitRdpsResult, HitStatDamageMetrics, RdpsInvalidReason,
    analyze_hit_rdps, filter_target_effects_for_attacker, resolve_skill_effect_flags,
};
use crate::live::skill_tracker::SkillTracker;
use crate::live::status_tracker::{StatusEffectDetails, StatusTracker};
use crate::live::utils::*;
use crate::live::{DEBUG_DUMP_DAMAGE_STATE_JSON, debug_print, write_debug_json_dump};
use crate::models::*;
use crate::utils::{
    get_class_from_id, get_player_spec, is_confirmed_player_entity, is_support_class,
    normalize_encounter_damage_totals,
};
use chrono::Utc;
use hashbrown::HashMap;
use log::{info, warn};
use meter_defs::defs::{CombatAnalyzerEntry, SkillCooldownStruct};
use meter_defs::types::SkillMoveOptionData;
use serde::Serialize;
use serde_json::json;
use std::cmp::max;
use std::collections::BTreeMap;
use std::default::Default;
use std::hash::Hash;
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager};
use tokio::task;

#[derive(Debug, Serialize, Clone, Default)]
struct StatDamageDump {
    damage_done_by_stat_: i64,
    damage_done_by_stat_plus_value_: i64,
}

impl From<StatDamageContribution> for StatDamageDump {
    fn from(value: StatDamageContribution) -> Self {
        Self {
            damage_done_by_stat_: value.damage_done_by_stat,
            damage_done_by_stat_plus_value_: value.damage_done_by_stat_plus_value,
        }
    }
}

#[derive(Debug, Serialize, Clone, Default)]
struct SkillStatsDump {
    damage_: i64,
    skill_casts_: i64,
    hits_: i64,
}

#[derive(Debug, Clone, Default)]
struct DamageDataAccumulator {
    damage_split_by_entity_id_: HashMap<u64, i64>,
    damage_done_by_entity_skill_group_: HashMap<u64, HashMap<String, i64>>,
    damage_increase_by_entity_skill_group_: HashMap<u64, HashMap<String, i64>>,
    damage_done_without_crits_: i64,
    damage_done_without_ultimate_awakening_: i64,
    damage_done_with_all_crits_: i64,
    damage_done_with_average_crits_: i64,
    critical_hit_rate_adjusted_damage_raw_: i64,
    critical_hit_rate_adjusted_damage_raw_capped_: i64,
    additional_damage_1percent_damage_: StatDamageContribution,
    critical_hit_rate_1percent_damage_: StatDamageContribution,
    critical_damage_rate_1percent_damage_: StatDamageContribution,
    evo_damage_1percent_damage_: StatDamageContribution,
    weapon_power_1000_damage_: StatDamageContribution,
    weapon_power_1percent_damage_: StatDamageContribution,
    attack_power_1000_damage_: StatDamageContribution,
    attack_power_1percent_damage_: StatDamageContribution,
    main_stat_1000_damage_: StatDamageContribution,
    raid_captain_efficiency_: StatDamageContribution,
    blunt_thorn_efficiency_: StatDamageContribution,
    supersonic_breakthrough_efficiency_: StatDamageContribution,
    standing_striker_efficiency_: StatDamageContribution,
    casts_: i64,
    skill_casts_: i64,
    skill_to_damage_map_: HashMap<u32, SkillStatsDump>,
}

#[derive(Debug, Serialize, Clone, Default)]
struct DamageDataDump {
    player_name_: String,
    #[serde(skip_serializing_if = "lal_party_number_unknown")]
    party_number_: i32,
    entity_id_: u64,
    damage_done_: i64,
    damage_done_without_ultimate_awakening_: i64,
    damage_done_without_crits_: i64,
    damage_done_with_all_crits_: i64,
    damage_done_with_average_crits_: i64,
    critical_hit_rate_adjusted_damage_raw_: i64,
    critical_hit_rate_adjusted_damage_raw_capped_: i64,
    damage_split_by_entity_id_: HashMap<u64, i64>,
    damage_done_by_entity_skill_group_: HashMap<u64, HashMap<String, i64>>,
    damage_increase_by_entity_skill_group_: HashMap<u64, HashMap<String, i64>>,
    additional_damage_1percent_damage_: StatDamageDump,
    critical_hit_rate_1percent_damage_: StatDamageDump,
    critical_damage_rate_1percent_damage_: StatDamageDump,
    evo_damage_1percent_damage_: StatDamageDump,
    weapon_power_1000_damage_: StatDamageDump,
    weapon_power_1percent_damage_: StatDamageDump,
    attack_power_1000_damage_: StatDamageDump,
    attack_power_1percent_damage_: StatDamageDump,
    main_stat_1000_damage_: StatDamageDump,
    raid_captain_efficiency_: StatDamageDump,
    blunt_thorn_efficiency_: StatDamageDump,
    supersonic_breakthrough_efficiency_: StatDamageDump,
    standing_striker_efficiency_: StatDamageDump,
    ally_identity_damage_power_1percent_damage_: StatDamageDump,
    ally_attack_power_power_1percent_damage_: StatDamageDump,
    ally_brand_power_1percent_damage_: StatDamageDump,
    support_spec_scaling_1percent_damage_: StatDamageDump,
    support_weapon_power_scaling_1000_damage_: StatDamageDump,
    support_main_stat_scaling_1000_damage_: StatDamageDump,
    support_base_attack_power_scaling_1percent_damage_: StatDamageDump,
    support_gem_attack_power_scaling_1percent_damage_: StatDamageDump,
    support_gem_identity_scaling_1percent_damage_: StatDamageDump,
    damage_done_under_atk_power_: i64,
    damage_done_under_brand_: i64,
    damage_done_under_identity_: i64,
    damage_done_under_hyper_: i64,
    casts_: i64,
    skill_casts_: i64,
    skill_to_damage_map_: HashMap<u32, SkillStatsDump>,
}

#[derive(Debug, Serialize, Clone, Default)]
struct DamageStateDump {
    start_time_: String,
    end_time_: String,
    last_damage_done_time_: String,
    silent_period_duration_seconds_: f64,
    zone_id_: u32,
    zone_level_: u32,
    #[serde(skip_serializing_if = "String::is_empty")]
    damage_key_base64_: String,
    player_id_to_damage_data_: HashMap<u64, DamageDataDump>,
    npc_to_skill_cast_data_: HashMap<u32, serde_json::Value>,
}

#[derive(Debug)]
struct PendingDamageEvent {
    packet_seq: i64,
    dmg_src_entity: Entity,
    proj_entity: Entity,
    dmg_target_entity: Entity,
    damage_data: DamageData,
    hit_context: DamageHitContext,
    se_on_source: Vec<StatusEffectDetails>,
    se_on_target: Vec<StatusEffectDetails>,
    target_count: i32,
    timestamp: i64,
    buffered_player_entities: HashMap<u64, Entity>,
    owner_self_effects_by_entity_id: HashMap<u64, Vec<StatusEffectDetails>>,
}

#[derive(Debug, Default, Clone)]
struct PendingSkillEvent {
    packet_seq: i64,
    source_entity: Entity,
    skill_id: u32,
    skill_level: u8,
    tripod_index: Option<TripodIndex>,
    skill_option_snapshot: Option<SkillOptionSnapshot>,
    timestamp: i64,
    create_skill_tracker_cast: bool,
}

#[derive(Debug, Default)]
struct StartupBarrierState {
    inspect_targets: HashMap<String, StartupInspectTarget>,
    pending_skill: Vec<PendingSkillEvent>,
    pending_damage: Vec<PendingDamageEvent>,
    freeze_registered_names: bool,
}

#[derive(Debug, Default)]
struct StartupInspectTarget {
    gate_required: bool,
    live_stats_required: bool,
    buffered_stats_required: bool,
}

#[derive(Debug)]
struct DamageHitContext {
    hit_flag: HitFlag,
    hit_option: HitOption,
}

#[derive(Debug)]
pub struct EncounterState {
    pub app: AppHandle,
    pub encounter: Encounter,
    pub resetting: bool,
    pub boss_dead_update: bool,
    pub saved: bool,
    pub disabled: bool,

    pub raid_clear: bool,

    damage_log: HashMap<String, Vec<(i64, i64)>>,
    cast_log: HashMap<String, HashMap<u32, Vec<i32>>>,

    boss_hp_log: HashMap<String, Vec<BossHpLog>>,

    pub intermission_start: Option<i64>,
    pub intermission_end: Option<i64>,

    pub party_info: Vec<Vec<String>>,
    pub raid_difficulty: String,
    pub raid_difficulty_id: u32,
    pub boss_only_damage: bool,
    pub region: Option<String>,

    ntp_clock: NtpClock,
    ntp_fight_start: i64,
    fight_start_instant: Option<Instant>,

    pub rdps_valid: bool,
    pub rdps_message: Option<String>,

    pub skill_tracker: SkillTracker,

    custom_id_map: HashMap<u32, u32>,
    source_owner_aliases: HashMap<u64, u64>,
    resolved_source_owner_aliases: HashMap<u64, u64>,
    startup_barrier: Option<StartupBarrierState>,
    rearm_startup_barrier_on_next_combat: bool,
    pending_phase_transition: Option<i32>,

    pub damage_is_valid: bool,
    player_contributions: HashMap<String, DamageDataAccumulator>,
    lal_debug_zone_id: u32,
    lal_debug_zone_level: u32,
    lal_debug_end_time_ms: Option<i64>,
    lal_debug_damage_key_base64: String,
}

impl EncounterState {
    pub fn new(window: AppHandle) -> EncounterState {
        EncounterState {
            app: window,
            encounter: Encounter::default(),
            resetting: false,
            raid_clear: false,
            boss_dead_update: false,
            saved: false,
            disabled: false,

            damage_log: HashMap::new(),
            boss_hp_log: HashMap::new(),
            cast_log: HashMap::new(),
            intermission_start: None,
            intermission_end: None,

            party_info: Vec::new(),
            raid_difficulty: "".to_string(),
            raid_difficulty_id: 0,
            boss_only_damage: false,
            region: None,

            ntp_clock: NtpClock::start(),
            ntp_fight_start: 0,
            fight_start_instant: None,

            // todo
            rdps_valid: false,
            rdps_message: None,

            skill_tracker: SkillTracker::new(),

            custom_id_map: HashMap::new(),
            source_owner_aliases: HashMap::new(),
            resolved_source_owner_aliases: HashMap::new(),
            startup_barrier: None,
            rearm_startup_barrier_on_next_combat: false,
            pending_phase_transition: None,

            damage_is_valid: true,
            player_contributions: HashMap::new(),
            lal_debug_zone_id: 0,
            lal_debug_zone_level: 0,
            lal_debug_end_time_ms: None,
            lal_debug_damage_key_base64: String::new(),
        }
    }

    // keep all player entities, reset all stats
    pub fn soft_reset(&mut self, keep_bosses: bool) {
        let clone = self.encounter.clone();

        self.encounter.fight_start = 0;
        self.encounter.boss_only_damage = self.boss_only_damage;
        self.encounter.entities = HashMap::new();
        self.encounter.current_boss_name = "".to_string();
        self.encounter.encounter_damage_stats = Default::default();
        self.raid_clear = false;

        self.damage_log = HashMap::new();
        self.cast_log = HashMap::new();
        self.boss_hp_log = HashMap::new();
        self.intermission_start = None;
        self.intermission_end = None;
        self.party_info = Vec::new();

        self.ntp_fight_start = 0;
        self.fight_start_instant = None;

        self.rdps_valid = false;
        self.rdps_message = None;

        self.skill_tracker = SkillTracker::new();

        self.custom_id_map = HashMap::new();
        self.source_owner_aliases.clear();
        self.resolved_source_owner_aliases.clear();
        self.startup_barrier = None;
        self.rearm_startup_barrier_on_next_combat = false;
        self.pending_phase_transition = None;
        self.player_contributions.clear();
        self.lal_debug_zone_id = 0;
        self.lal_debug_zone_level = 0;
        self.lal_debug_end_time_ms = None;
        self.lal_debug_damage_key_base64.clear();

        for (key, entity) in clone.entities.into_iter().filter(|(_, e)| {
            e.entity_type == EntityType::Player
                || (keep_bosses && e.entity_type == EntityType::Boss)
        }) {
            self.encounter.entities.insert(
                key,
                EncounterEntity {
                    name: entity.name,
                    id: entity.id,
                    character_id: entity.character_id,
                    npc_id: entity.npc_id,
                    hp_bars: entity.hp_bars,
                    class: entity.class,
                    class_id: entity.class_id,
                    entity_type: entity.entity_type,
                    gear_score: entity.gear_score,
                    max_hp: entity.max_hp,
                    current_hp: entity.max_hp,
                    is_dead: entity.is_dead,
                    ..Default::default()
                },
            );
        }
    }

    fn refresh_encounter_player_damage_totals(&mut self) {
        normalize_encounter_damage_totals(&mut self.encounter);
    }

    fn refresh_encounter_entity_metadata(
        encounter_entity: &mut EncounterEntity,
        entity: &Entity,
        current_hp: Option<i64>,
        max_hp: Option<i64>,
    ) {
        if entity.id != 0 {
            encounter_entity.id = entity.id;
        }
        if entity.character_id != 0 {
            encounter_entity.character_id = entity.character_id;
        }
        if entity.npc_id != 0 {
            encounter_entity.npc_id = entity.npc_id;
        }
        if entity.hp_bars.is_some() {
            encounter_entity.hp_bars = entity.hp_bars;
        }
        if entity.entity_type != EntityType::Unknown {
            encounter_entity.entity_type = entity.entity_type;
        }
        if entity.class_id != 0 {
            encounter_entity.class_id = entity.class_id;
            encounter_entity.class = get_class_from_id(&entity.class_id);
        }
        if entity.gear_level > 0.0 {
            encounter_entity.gear_score = entity.gear_level;
        }
        if let Some(current_hp) = current_hp {
            encounter_entity.current_hp = current_hp;
        }
        if let Some(max_hp) = max_hp.filter(|max_hp| *max_hp > 0) {
            encounter_entity.max_hp = max_hp;
        }
    }

    pub fn on_source_owner_resolved(
        &mut self,
        source_id: u64,
        owner_id: u64,
        entity_tracker: &EntityTracker,
    ) {
        if source_id == 0 || owner_id == 0 || source_id == owner_id {
            return;
        }

        self.source_owner_aliases.insert(source_id, owner_id);

        let Some(owner) = self.resolve_confirmed_owner_player(owner_id, entity_tracker) else {
            return;
        };

        self.merge_source_alias_into_player(source_id, owner);
        self.source_owner_aliases.remove(&source_id);
        self.resolve_source_aliases_for_player(owner, entity_tracker);
    }

    fn resolve_confirmed_owner_player<'a>(
        &self,
        owner_id: u64,
        entity_tracker: &'a EntityTracker,
    ) -> Option<&'a Entity> {
        let mut current_id = owner_id;
        let mut seen_ids = Vec::new();

        for _ in 0..16 {
            if current_id == 0 || seen_ids.contains(&current_id) {
                return None;
            }
            seen_ids.push(current_id);

            if let Some(entity) = entity_tracker.get_entity_ref(current_id) {
                if entity.entity_type == EntityType::Player {
                    return Some(entity);
                }

                if let Some(alias_owner_id) = self
                    .source_owner_aliases
                    .get(&current_id)
                    .copied()
                    .filter(|alias_owner_id| *alias_owner_id != current_id)
                {
                    current_id = alias_owner_id;
                    continue;
                }

                if let Some(resolved_owner_id) = self
                    .resolved_source_owner_aliases
                    .get(&current_id)
                    .copied()
                    .filter(|resolved_owner_id| *resolved_owner_id != current_id)
                {
                    current_id = resolved_owner_id;
                    continue;
                }

                if matches!(
                    entity.entity_type,
                    EntityType::Projectile | EntityType::Summon
                ) && entity.owner_id != 0
                {
                    current_id = entity.owner_id;
                    continue;
                }
            } else if let Some(alias_owner_id) = self
                .source_owner_aliases
                .get(&current_id)
                .copied()
                .filter(|alias_owner_id| *alias_owner_id != current_id)
            {
                current_id = alias_owner_id;
                continue;
            } else if let Some(resolved_owner_id) = self
                .resolved_source_owner_aliases
                .get(&current_id)
                .copied()
                .filter(|resolved_owner_id| *resolved_owner_id != current_id)
            {
                current_id = resolved_owner_id;
                continue;
            }

            return None;
        }

        None
    }

    fn resolve_source_aliases_for_player(
        &mut self,
        player: &Entity,
        entity_tracker: &EntityTracker,
    ) {
        if player.id == 0 || player.entity_type != EntityType::Player {
            return;
        }

        loop {
            let source_ids = self
                .source_owner_aliases
                .iter()
                .filter_map(|(source_id, owner_id)| {
                    self.resolve_confirmed_owner_player(*owner_id, entity_tracker)
                        .filter(|owner| owner.id == player.id)
                        .map(|_| *source_id)
                })
                .collect::<Vec<_>>();

            if source_ids.is_empty() {
                break;
            }

            for source_id in source_ids {
                self.merge_source_alias_into_player(source_id, player);
                self.source_owner_aliases.remove(&source_id);
            }
        }
    }

    fn merge_source_alias_into_player(&mut self, source_id: u64, player: &Entity) {
        if source_id == 0 || player.id == 0 || source_id == player.id {
            return;
        }

        self.encounter
            .entities
            .entry(player.name.clone())
            .or_insert_with(|| encounter_entity_from_entity(player));

        let old_names = self
            .encounter
            .entities
            .iter()
            .filter(|(name, entity)| {
                name.as_str() != player.name
                    && Self::is_mergeable_owned_source(entity)
                    && (entity.id == source_id || Self::is_unresolved_entity_name(name, source_id))
            })
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();

        for old_name in old_names {
            self.merge_owned_source_entity_name_into_player(&old_name, player);
        }

        self.skill_tracker.rebind_entity_id(source_id, player.id);
        self.rebind_startup_player_entity_ids(source_id, player.id);
        self.rebind_contribution_entity_id(source_id, player.id);
        self.resolved_source_owner_aliases
            .insert(source_id, player.id);
        for owner_id in self.source_owner_aliases.values_mut() {
            if *owner_id == source_id {
                *owner_id = player.id;
            }
        }
        for owner_id in self.resolved_source_owner_aliases.values_mut() {
            if *owner_id == source_id {
                *owner_id = player.id;
            }
        }
        self.refresh_encounter_player_damage_totals();
    }

    fn is_mergeable_owned_source(entity: &EncounterEntity) -> bool {
        !matches!(
            entity.entity_type,
            EntityType::Player | EntityType::Boss | EntityType::Esther | EntityType::DarkGrenade
        )
    }

    fn rebind_contribution_entity_id(&mut self, old_entity_id: u64, new_entity_id: u64) {
        if old_entity_id == 0 || old_entity_id == new_entity_id {
            return;
        }

        for accumulator in self.player_contributions.values_mut() {
            Self::rebind_numeric_map_entity_id(
                &mut accumulator.damage_split_by_entity_id_,
                old_entity_id,
                new_entity_id,
            );
            Self::rebind_nested_numeric_map_entity_id(
                &mut accumulator.damage_done_by_entity_skill_group_,
                old_entity_id,
                new_entity_id,
            );
            Self::rebind_nested_numeric_map_entity_id(
                &mut accumulator.damage_increase_by_entity_skill_group_,
                old_entity_id,
                new_entity_id,
            );
        }
    }

    fn rebind_numeric_map_entity_id(
        map: &mut HashMap<u64, i64>,
        old_entity_id: u64,
        new_entity_id: u64,
    ) {
        if let Some(value) = map.remove(&old_entity_id) {
            *map.entry(new_entity_id).or_default() += value;
        }
    }

    fn rebind_nested_numeric_map_entity_id(
        map: &mut HashMap<u64, HashMap<String, i64>>,
        old_entity_id: u64,
        new_entity_id: u64,
    ) {
        if let Some(old_values) = map.remove(&old_entity_id) {
            let new_values = map.entry(new_entity_id).or_default();
            for (key, value) in old_values {
                *new_values.entry(key).or_default() += value;
            }
        }
    }

    // update local player as we get more info
    pub fn update_local_player(&mut self, entity: &Entity, entity_tracker: &EntityTracker) {
        // we replace the existing local player if it exists, since its name might have changed (from hex or "You" to character name)
        if let Some(mut local) = self.encounter.entities.remove(&self.encounter.local_player) {
            // update local player name, insert back into encounter
            self.encounter.local_player.clone_from(&entity.name);
            update_player_entity(&mut local, entity);
            self.encounter
                .entities
                .insert(self.encounter.local_player.clone(), local);
        } else {
            // cannot find old local player by name, so we look by local player's entity id
            // this can happen when the user started meter late
            let old_local = self
                .encounter
                .entities
                .iter()
                .find(|(_, e)| e.id == entity.id)
                .map(|(key, _)| key.clone());

            // if we find the old local player, we update its name and insert back into encounter
            if let Some(old_local) = old_local {
                let mut new_local = self.encounter.entities[&old_local].clone();
                update_player_entity(&mut new_local, entity);
                self.encounter.entities.remove(&old_local);
                self.encounter.local_player.clone_from(&entity.name);
                self.encounter
                    .entities
                    .insert(self.encounter.local_player.clone(), new_local);
            }
        }
        self.resolve_source_aliases_for_player(entity, entity_tracker);
        self.refresh_encounter_player_damage_totals();
    }

    pub fn on_init_env(&mut self, entity: Entity) {
        // if not already saved to db, we save again
        if !self.saved && !self.encounter.current_boss_name.is_empty() {
            self.save_to_db(false);
        }

        // replace or insert local player
        if let Some(mut local_player) = self.encounter.entities.remove(&self.encounter.local_player)
        {
            update_player_entity(&mut local_player, &entity);
            self.encounter
                .entities
                .insert(entity.name.clone(), local_player);
        } else {
            let entity = encounter_entity_from_entity(&entity);
            self.encounter.entities.insert(entity.name.clone(), entity);
        }
        self.encounter.local_player = entity.name;

        // remove unrelated entities
        self.encounter.entities.retain(|_, e| {
            e.name == self.encounter.local_player || e.damage_stats.damage_dealt > 0
        });

        self.app
            .emit("zone-change", "")
            .expect("failed to emit zone-change");

        self.soft_reset(false);
    }

    pub fn on_transit(&mut self, zone_id: u32) {
        if zone_id == 37545 {
            // do not reset on kazeros g2-2 for nm/hm
            if self.raid_difficulty != "The First" {
                let now = Utc::now().timestamp_millis();
                self.intermission_start = Some(now);
                self.rearm_startup_barrier_on_next_combat = true;
                info!("starting intermission");
                for entity in self
                    .encounter
                    .entities
                    .values_mut()
                    .filter(|e| e.entity_type == EntityType::Player)
                {
                    if let Some(death) = entity
                        .damage_stats
                        .death_info
                        .as_mut()
                        .and_then(|info| info.last_mut())
                    {
                        death.dead_for = Some(now - death.death_time);
                    }
                }
            } else {
                self.on_phase_transition(2);
            }
            return;
        }

        self.app
            .emit("zone-change", "no-toast")
            .expect("failed to emit zone-change");

        self.soft_reset(false);
    }

    pub fn on_phase_transition(&mut self, phase_code: i32) {
        self.app
            .emit("phase-transition", phase_code)
            .expect("failed to emit phase-transition");

        match phase_code {
            0 | 2 | 3 | 4 => {
                if !self.encounter.current_boss_name.is_empty() {
                    self.lal_debug_end_time_ms = Some(Utc::now().timestamp_millis());
                    self.save_to_db(false);
                    self.saved = true;
                }
                self.resetting = true;
            }
            _ => (),
        }
    }

    // replace local player
    pub fn on_init_pc(
        &mut self,
        entity: Entity,
        hp: i64,
        max_hp: i64,
        entity_tracker: &EntityTracker,
    ) {
        self.encounter.entities.remove(&self.encounter.local_player);
        self.encounter.local_player.clone_from(&entity.name);
        let mut player = encounter_entity_from_entity(&entity);
        player.current_hp = hp;
        player.max_hp = max_hp;
        self.encounter.entities.insert(player.name.clone(), player);
        self.resolve_source_aliases_for_player(&entity, entity_tracker);
        self.refresh_encounter_player_damage_totals();
    }

    // add or update player to encounter
    pub fn on_new_pc(
        &mut self,
        entity: Entity,
        hp: i64,
        max_hp: i64,
        entity_tracker: &EntityTracker,
    ) {
        self.merge_unresolved_player_entity(&entity);
        self.encounter
            .entities
            .entry(entity.name.clone())
            .and_modify(|player| {
                player.id = entity.id;
                player.name.clone_from(&entity.name);
                player.entity_type = entity.entity_type;
                player.class_id = entity.class_id;
                player.class = get_class_from_id(&entity.class_id);
                player.gear_score = entity.gear_level;
                player.current_hp = hp;
                if max_hp > 0 {
                    player.max_hp = max_hp;
                }
                if entity.character_id > 0 {
                    player.character_id = entity.character_id;
                }
                if hp > 0 {
                    Self::mark_entity_alive(player, Utc::now().timestamp_millis());
                }
            })
            .or_insert_with(|| {
                let mut player = encounter_entity_from_entity(&entity);
                player.current_hp = hp;
                player.max_hp = max_hp;
                player
            });
        self.resolve_source_aliases_for_player(&entity, entity_tracker);
        self.refresh_encounter_player_damage_totals();
    }

    fn mark_entity_alive(entity: &mut EncounterEntity, timestamp: i64) {
        if !entity.is_dead {
            return;
        }

        entity.is_dead = false;
        if let Some(death) = entity
            .damage_stats
            .death_info
            .as_mut()
            .and_then(|info| info.last_mut())
            .filter(|death| death.dead_for.is_none())
        {
            death.dead_for = Some((timestamp - death.death_time).max(0));
        }
    }

    fn merge_unresolved_player_entity(&mut self, entity: &Entity) {
        if entity.id == 0 || Self::is_unresolved_entity_name(&entity.name, entity.id) {
            return;
        }

        let new_name = entity.name.clone();
        let old_names = self
            .encounter
            .entities
            .iter()
            .filter(|(name, existing)| {
                existing.id == entity.id
                    && name.as_str() != new_name
                    && Self::is_unresolved_entity_name(name, entity.id)
            })
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();

        for old_name in old_names {
            self.merge_player_entity_name(&old_name, &new_name);
        }
    }

    fn merge_player_entity_name(&mut self, old_name: &str, new_name: &str) {
        if old_name == new_name {
            return;
        }

        if let Some(mut old_entity) = self.encounter.entities.remove(old_name) {
            old_entity.name = new_name.to_string();
            match self.encounter.entities.entry(new_name.to_string()) {
                hashbrown::hash_map::Entry::Occupied(mut entry) => {
                    Self::merge_encounter_entity(entry.get_mut(), old_entity);
                }
                hashbrown::hash_map::Entry::Vacant(entry) => {
                    entry.insert(old_entity);
                }
            }
        }

        Self::merge_name_keyed_vec_map(&mut self.damage_log, old_name, new_name);
        if let Some(log) = self.damage_log.get_mut(new_name) {
            log.sort_unstable_by_key(|(timestamp, _)| *timestamp);
        }

        Self::merge_name_keyed_cast_log(&mut self.cast_log, old_name, new_name);

        if let Some(old_accumulator) = self.player_contributions.remove(old_name) {
            let accumulator = self
                .player_contributions
                .entry(new_name.to_string())
                .or_default();
            Self::merge_damage_data_accumulator(accumulator, old_accumulator);
        }

        for party in &mut self.party_info {
            for member in party {
                if member == old_name {
                    member.clear();
                    member.push_str(new_name);
                }
            }
        }

        if self.encounter.local_player == old_name {
            self.encounter.local_player = new_name.to_string();
        }

        self.refresh_encounter_player_damage_totals();
    }

    fn merge_owned_source_entity_name_into_player(&mut self, old_name: &str, player: &Entity) {
        if old_name == player.name {
            return;
        }

        self.encounter
            .entities
            .entry(player.name.clone())
            .or_insert_with(|| encounter_entity_from_entity(player));

        if let Some(old_entity) = self.encounter.entities.remove(old_name)
            && let Some(player_entity) = self.encounter.entities.get_mut(&player.name)
        {
            Self::merge_owned_source_entity(player_entity, old_entity);
        }

        Self::merge_name_keyed_vec_map(&mut self.damage_log, old_name, &player.name);
        if let Some(log) = self.damage_log.get_mut(&player.name) {
            log.sort_unstable_by_key(|(timestamp, _)| *timestamp);
        }

        Self::merge_name_keyed_cast_log(&mut self.cast_log, old_name, &player.name);

        if let Some(old_accumulator) = self.player_contributions.remove(old_name) {
            let accumulator = self
                .player_contributions
                .entry(player.name.clone())
                .or_default();
            Self::merge_damage_data_accumulator(accumulator, old_accumulator);
        }

        self.refresh_encounter_player_damage_totals();
    }

    fn merge_owned_source_entity(target: &mut EncounterEntity, source: EncounterEntity) {
        Self::merge_skills(&mut target.skills, source.skills);
        Self::merge_owned_source_damage_stats(&mut target.damage_stats, source.damage_stats);
        Self::merge_skill_stats(&mut target.skill_stats, source.skill_stats);
    }

    fn merge_owned_source_damage_stats(target: &mut DamageStats, source: DamageStats) {
        target.damage_dealt += source.damage_dealt;
        target.hyper_awakening_damage += source.hyper_awakening_damage;
        Self::merge_numeric_map(&mut target.buffed_by, source.buffed_by);
        Self::merge_numeric_map(&mut target.debuffed_by, source.debuffed_by);
        target.buffed_by_support += source.buffed_by_support;
        target.buffed_by_identity += source.buffed_by_identity;
        target.debuffed_by_support += source.debuffed_by_support;
        target.buffed_by_hat += source.buffed_by_hat;
        target.crit_damage += source.crit_damage;
        target.back_attack_damage += source.back_attack_damage;
        target.front_attack_damage += source.front_attack_damage;
        target.dps += source.dps;
        target.rdps_damage_received += source.rdps_damage_received;
        target.rdps_damage_received_support += source.rdps_damage_received_support;
        target.rdps_damage_given += source.rdps_damage_given;
        target.stagger += source.stagger;
        target.buffed_damage += source.buffed_damage;
        target.unbuffed_damage += source.unbuffed_damage;
        target.unbuffed_dps += source.unbuffed_dps;
        target.rdps += source.rdps;
        target.ndps += source.ndps;
    }

    fn merge_encounter_entity(target: &mut EncounterEntity, source: EncounterEntity) {
        if target.id == 0 {
            target.id = source.id;
        }
        if target.character_id == 0 {
            target.character_id = source.character_id;
        }
        if target.npc_id == 0 {
            target.npc_id = source.npc_id;
        }
        if target.hp_bars.is_none() {
            target.hp_bars = source.hp_bars;
        }
        if target.entity_type == EntityType::Unknown {
            target.entity_type = source.entity_type;
        }
        if target.class_id == 0 {
            target.class_id = source.class_id;
            target.class = source.class;
        }
        if target.gear_score == 0.0 {
            target.gear_score = source.gear_score;
        }
        if target.current_hp == 0 {
            target.current_hp = source.current_hp;
        }
        if target.max_hp == 0 {
            target.max_hp = source.max_hp;
        }
        target.current_shield = target.current_shield.max(source.current_shield);
        target.is_dead |= source.is_dead;

        Self::merge_skills(&mut target.skills, source.skills);
        Self::merge_damage_stats(&mut target.damage_stats, source.damage_stats);
        Self::merge_skill_stats(&mut target.skill_stats, source.skill_stats);

        if target.engraving_data.is_none() {
            target.engraving_data = source.engraving_data;
        }
        if target.ark_passive_active.is_none() {
            target.ark_passive_active = source.ark_passive_active;
        }
        if target.ark_passive_data.is_none() {
            target.ark_passive_data = source.ark_passive_data;
        }
        if target.spec.is_none() {
            target.spec = source.spec;
        }
        if target.loadout_hash.is_none() {
            target.loadout_hash = source.loadout_hash;
        }
        if target.combat_power.is_none() {
            target.combat_power = source.combat_power;
        }
    }

    fn merge_damage_stats(target: &mut DamageStats, source: DamageStats) {
        target.damage_dealt += source.damage_dealt;
        target.hyper_awakening_damage += source.hyper_awakening_damage;
        target.damage_taken += source.damage_taken;
        Self::merge_numeric_map(&mut target.buffed_by, source.buffed_by);
        Self::merge_numeric_map(&mut target.debuffed_by, source.debuffed_by);
        target.buffed_by_support += source.buffed_by_support;
        target.buffed_by_identity += source.buffed_by_identity;
        target.debuffed_by_support += source.debuffed_by_support;
        target.buffed_by_hat += source.buffed_by_hat;
        target.crit_damage += source.crit_damage;
        target.back_attack_damage += source.back_attack_damage;
        target.front_attack_damage += source.front_attack_damage;
        target.shields_given += source.shields_given;
        target.shields_received += source.shields_received;
        target.damage_absorbed += source.damage_absorbed;
        target.damage_absorbed_on_others += source.damage_absorbed_on_others;
        Self::merge_numeric_map(&mut target.shields_given_by, source.shields_given_by);
        Self::merge_numeric_map(&mut target.shields_received_by, source.shields_received_by);
        Self::merge_numeric_map(&mut target.damage_absorbed_by, source.damage_absorbed_by);
        Self::merge_numeric_map(
            &mut target.damage_absorbed_on_others_by,
            source.damage_absorbed_on_others_by,
        );
        target.deaths += source.deaths;
        target.death_time = target.death_time.max(source.death_time);
        match (&mut target.death_info, source.death_info) {
            (Some(target_deaths), Some(mut source_deaths)) => {
                target_deaths.append(&mut source_deaths);
                target_deaths.sort_unstable_by_key(|death| death.death_time);
            }
            (None, source_deaths @ Some(_)) => target.death_info = source_deaths,
            _ => {}
        }
        if target.boss_hp_at_death.is_none() {
            target.boss_hp_at_death = source.boss_hp_at_death;
        }
        target.dps += source.dps;
        target.dps_average.extend(source.dps_average);
        target
            .dps_rolling_10s_avg
            .extend(source.dps_rolling_10s_avg);
        target.rdps_damage_received += source.rdps_damage_received;
        target.rdps_damage_received_support += source.rdps_damage_received_support;
        target.rdps_damage_given += source.rdps_damage_given;
        target.incapacitations.extend(source.incapacitations);
        target
            .incapacitations
            .sort_unstable_by_key(|event| event.timestamp);
        target.incapacitations.dedup_by(|a, b| {
            a.timestamp == b.timestamp && a.duration == b.duration && a.event_type == b.event_type
        });
        target.stagger += source.stagger;
        target.buffed_damage += source.buffed_damage;
        target.unbuffed_damage += source.unbuffed_damage;
        target.unbuffed_dps += source.unbuffed_dps;
        target.rdps += source.rdps;
        target.ndps += source.ndps;
    }

    fn merge_skill_stats(target: &mut SkillStats, source: SkillStats) {
        target.casts += source.casts;
        target.hits += source.hits;
        target.crits += source.crits;
        target.back_attacks += source.back_attacks;
        target.front_attacks += source.front_attacks;
        target.counters += source.counters;
        if target.identity_stats.is_none() {
            target.identity_stats = source.identity_stats;
        }
    }

    fn merge_skills(target: &mut HashMap<u32, Skill>, source: HashMap<u32, Skill>) {
        for (skill_id, source_skill) in source {
            match target.entry(skill_id) {
                hashbrown::hash_map::Entry::Occupied(mut entry) => {
                    Self::merge_skill(entry.get_mut(), source_skill);
                }
                hashbrown::hash_map::Entry::Vacant(entry) => {
                    entry.insert(source_skill);
                }
            }
        }
    }

    fn merge_skill(target: &mut Skill, source: Skill) {
        if target.name.is_empty() {
            target.name = source.name;
        }
        if target.icon.is_empty() {
            target.icon = source.icon;
        }
        target.total_damage += source.total_damage;
        target.max_damage = target.max_damage.max(source.max_damage);
        target.max_damage_cast = target.max_damage_cast.max(source.max_damage_cast);
        Self::merge_numeric_map(&mut target.buffed_by, source.buffed_by);
        Self::merge_numeric_map(&mut target.debuffed_by, source.debuffed_by);
        target.buffed_by_support += source.buffed_by_support;
        target.buffed_by_identity += source.buffed_by_identity;
        target.buffed_by_hat += source.buffed_by_hat;
        target.debuffed_by_support += source.debuffed_by_support;
        target.casts += source.casts;
        target.hits += source.hits;
        target.crits += source.crits;
        if target.adjusted_crit.is_none() {
            target.adjusted_crit = source.adjusted_crit;
        }
        target.crit_damage += source.crit_damage;
        target.back_attacks += source.back_attacks;
        target.front_attacks += source.front_attacks;
        target.back_attack_damage += source.back_attack_damage;
        target.front_attack_damage += source.front_attack_damage;
        target.dps += source.dps;
        Self::merge_cast_log(&mut target.cast_log, source.cast_log);
        if target.tripod_index.is_none() {
            target.tripod_index = source.tripod_index;
        }
        if target.tripod_level.is_none() {
            target.tripod_level = source.tripod_level;
        }
        if target.gem_cooldown.is_none() {
            target.gem_cooldown = source.gem_cooldown;
        }
        if target.gem_tier.is_none() {
            target.gem_tier = source.gem_tier;
        }
        if target.gem_damage.is_none() {
            target.gem_damage = source.gem_damage;
        }
        if target.gem_tier_dmg.is_none() {
            target.gem_tier_dmg = source.gem_tier_dmg;
        }
        Self::merge_skill_cast_log(&mut target.skill_cast_log, source.skill_cast_log);
        target.stagger += source.stagger;
        target.is_hyper_awakening |= source.is_hyper_awakening;
        if target.special.is_none() {
            target.special = source.special;
        }
        target.last_timestamp = target.last_timestamp.max(source.last_timestamp);
        if target.time_available.is_none() {
            target.time_available = source.time_available;
        }
        Self::merge_nested_numeric_map(&mut target.rdps_received, source.rdps_received);
        Self::merge_numeric_map(&mut target.rdps_contributed, source.rdps_contributed);
        target.rdps_damage_received += source.rdps_damage_received;
        target.rdps_damage_received_support += source.rdps_damage_received_support;
    }

    fn merge_cast_log(target: &mut Vec<i32>, source: Vec<i32>) {
        target.extend(source);
        target.sort_unstable();
        target.dedup();
    }

    fn merge_skill_cast_log(target: &mut Vec<SkillCast>, source: Vec<SkillCast>) {
        let mut merged = target
            .drain(..)
            .map(|cast| (cast.timestamp, cast))
            .collect::<BTreeMap<_, _>>();

        for source_cast in source {
            match merged.entry(source_cast.timestamp) {
                std::collections::btree_map::Entry::Occupied(mut entry) => {
                    let cast = entry.get_mut();
                    cast.last = cast.last.max(source_cast.last);
                    Self::append_missing_skill_hits(&mut cast.hits, source_cast.hits);
                }
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(source_cast);
                }
            }
        }

        target.extend(merged.into_values());
    }

    fn append_missing_skill_hits(target: &mut Vec<SkillHit>, source: Vec<SkillHit>) {
        for hit in source {
            if !target
                .iter()
                .any(|existing| Self::skill_hits_match(existing, &hit))
            {
                target.push(hit);
            }
        }
        target.sort_unstable_by_key(|hit| hit.timestamp);
    }

    fn skill_hits_match(a: &SkillHit, b: &SkillHit) -> bool {
        a.timestamp == b.timestamp
            && a.damage == b.damage
            && a.unbuffed_damage == b.unbuffed_damage
            && a.rdps_damage_received == b.rdps_damage_received
            && a.rdps_damage_received_support == b.rdps_damage_received_support
            && a.crit == b.crit
            && a.back_attack == b.back_attack
            && a.front_attack == b.front_attack
            && a.buffed_by == b.buffed_by
            && a.debuffed_by == b.debuffed_by
            && a.stagger == b.stagger
    }

    fn merge_damage_data_accumulator(
        target: &mut DamageDataAccumulator,
        source: DamageDataAccumulator,
    ) {
        Self::merge_numeric_map(
            &mut target.damage_split_by_entity_id_,
            source.damage_split_by_entity_id_,
        );
        Self::merge_nested_numeric_map(
            &mut target.damage_done_by_entity_skill_group_,
            source.damage_done_by_entity_skill_group_,
        );
        Self::merge_nested_numeric_map(
            &mut target.damage_increase_by_entity_skill_group_,
            source.damage_increase_by_entity_skill_group_,
        );
        target.damage_done_without_crits_ += source.damage_done_without_crits_;
        target.damage_done_without_ultimate_awakening_ +=
            source.damage_done_without_ultimate_awakening_;
        target.damage_done_with_all_crits_ += source.damage_done_with_all_crits_;
        target.damage_done_with_average_crits_ += source.damage_done_with_average_crits_;
        target.critical_hit_rate_adjusted_damage_raw_ +=
            source.critical_hit_rate_adjusted_damage_raw_;
        target.critical_hit_rate_adjusted_damage_raw_capped_ +=
            source.critical_hit_rate_adjusted_damage_raw_capped_;
        target
            .additional_damage_1percent_damage_
            .merge(source.additional_damage_1percent_damage_);
        target
            .critical_hit_rate_1percent_damage_
            .merge(source.critical_hit_rate_1percent_damage_);
        target
            .critical_damage_rate_1percent_damage_
            .merge(source.critical_damage_rate_1percent_damage_);
        target
            .evo_damage_1percent_damage_
            .merge(source.evo_damage_1percent_damage_);
        target
            .weapon_power_1000_damage_
            .merge(source.weapon_power_1000_damage_);
        target
            .weapon_power_1percent_damage_
            .merge(source.weapon_power_1percent_damage_);
        target
            .attack_power_1000_damage_
            .merge(source.attack_power_1000_damage_);
        target
            .attack_power_1percent_damage_
            .merge(source.attack_power_1percent_damage_);
        target
            .main_stat_1000_damage_
            .merge(source.main_stat_1000_damage_);
        target
            .raid_captain_efficiency_
            .merge(source.raid_captain_efficiency_);
        target
            .blunt_thorn_efficiency_
            .merge(source.blunt_thorn_efficiency_);
        target
            .supersonic_breakthrough_efficiency_
            .merge(source.supersonic_breakthrough_efficiency_);
        target
            .standing_striker_efficiency_
            .merge(source.standing_striker_efficiency_);
        target.casts_ += source.casts_;
        target.skill_casts_ += source.skill_casts_;
        for (skill_id, source_skill) in source.skill_to_damage_map_ {
            let target_skill = target.skill_to_damage_map_.entry(skill_id).or_default();
            target_skill.damage_ += source_skill.damage_;
            target_skill.skill_casts_ += source_skill.skill_casts_;
            target_skill.hits_ += source_skill.hits_;
        }
    }

    fn merge_name_keyed_vec_map<T>(
        map: &mut HashMap<String, Vec<T>>,
        old_name: &str,
        new_name: &str,
    ) {
        if let Some(mut source) = map.remove(old_name) {
            map.entry(new_name.to_string())
                .or_default()
                .append(&mut source);
        }
    }

    fn merge_name_keyed_cast_log(
        map: &mut HashMap<String, HashMap<u32, Vec<i32>>>,
        old_name: &str,
        new_name: &str,
    ) {
        if let Some(source) = map.remove(old_name) {
            let target = map.entry(new_name.to_string()).or_default();
            for (skill_id, mut values) in source {
                let log = target.entry(skill_id).or_default();
                log.append(&mut values);
                log.sort_unstable();
                log.dedup();
            }
        }
    }

    fn merge_numeric_map<K, V>(target: &mut HashMap<K, V>, source: HashMap<K, V>)
    where
        K: Eq + Hash,
        V: Default + std::ops::AddAssign,
    {
        for (key, value) in source {
            *target.entry(key).or_default() += value;
        }
    }

    fn merge_nested_numeric_map<K1, K2, V>(
        target: &mut HashMap<K1, HashMap<K2, V>>,
        source: HashMap<K1, HashMap<K2, V>>,
    ) where
        K1: Eq + Hash,
        K2: Eq + Hash,
        V: Default + std::ops::AddAssign,
    {
        for (key, inner) in source {
            Self::merge_numeric_map(target.entry(key).or_default(), inner);
        }
    }

    fn is_unresolved_entity_name(name: &str, entity_id: u64) -> bool {
        if name.is_empty() || entity_id == 0 {
            return false;
        }

        match u64::from_str_radix(name, 16) {
            Ok(parsed_id) => parsed_id == entity_id,
            Err(_) => false,
        }
    }

    // add or update npc to encounter
    // we set current boss if npc matches criteria
    pub fn on_new_npc(&mut self, entity: Entity, hp: i64, max_hp: i64) {
        let entity_name = entity.name.clone();
        self.encounter
            .entities
            .entry(entity_name.clone())
            .and_modify(|e| {
                if entity.entity_type != EntityType::Boss && e.entity_type != EntityType::Boss {
                    e.npc_id = entity.npc_id;
                    e.id = entity.id;
                    e.current_hp = hp;
                    e.max_hp = max_hp;
                    e.hp_bars = entity.hp_bars;
                } else if entity.entity_type == EntityType::Boss && e.entity_type == EntityType::Npc
                {
                    e.entity_type = EntityType::Boss;
                    e.npc_id = entity.npc_id;
                    e.id = entity.id;
                    e.current_hp = hp;
                    e.max_hp = max_hp;
                    e.hp_bars = entity.hp_bars;
                }
            })
            .or_insert_with(|| {
                let mut npc = encounter_entity_from_entity(&entity);
                npc.current_hp = hp;
                npc.max_hp = max_hp;
                npc.hp_bars = entity.hp_bars;
                npc
            });

        if let Some(npc) = self.encounter.entities.get(&entity_name)
            && npc.entity_type == EntityType::Boss
        {
            // if current encounter has no boss, we set the boss
            // if current encounter has a boss, we check if new boss has more max hp, or if current boss is dead
            self.encounter.current_boss_name = if self
                .encounter
                .entities
                .get(&self.encounter.current_boss_name)
                .is_none_or(|boss| npc.max_hp >= boss.max_hp || boss.is_dead)
            {
                entity_name
            } else {
                self.encounter.current_boss_name.clone()
            };

            // set intermission end if boss is kazeros g2
            if self.encounter.current_boss_name == "Death Incarnate Kazeros"
                && self.intermission_start.is_some()
                && self.intermission_end.is_none()
            {
                self.intermission_end = Some(Utc::now().timestamp_millis());
                info!("ending intermission");
            }
        }
    }

    pub fn on_death(&mut self, dead_entity: &Entity) {
        // get current boss hp
        let boss_hp = self
            .encounter
            .entities
            .get(&self.encounter.current_boss_name)
            .map(|b| b.current_hp)
            .unwrap_or_default();

        let entity = self
            .encounter
            .entities
            .entry(dead_entity.name.clone())
            .or_insert_with(|| encounter_entity_from_entity(dead_entity));

        if (dead_entity.entity_type != EntityType::Player
            && dead_entity.entity_type != EntityType::Boss)
            || entity.id != dead_entity.id
            || (entity.entity_type == EntityType::Boss && entity.npc_id != dead_entity.npc_id)
        {
            return;
        }

        if entity.entity_type == EntityType::Boss
            && dead_entity.entity_type == EntityType::Boss
            && entity.name == self.encounter.current_boss_name
            && !entity.is_dead
        {
            self.boss_dead_update = true;
        }

        let now = Utc::now().timestamp_millis();
        entity.current_hp = 0;
        entity.is_dead = true;
        entity.damage_stats.deaths += 1;
        entity.damage_stats.death_time = now;
        entity
            .damage_stats
            .death_info
            .get_or_insert_default()
            .push(DeathInfo {
                death_time: now,
                dead_for: None,
            });
        // record boss hp at time of death
        entity.damage_stats.boss_hp_at_death = Some(boss_hp);

        entity
            .damage_stats
            .incapacitations
            .iter_mut()
            .rev()
            .take_while(|x| x.timestamp + x.duration > entity.damage_stats.death_time)
            .for_each(|x| {
                // cap duration to death time if it exceeds it
                x.duration = x.timestamp - entity.damage_stats.death_time;
            });
    }

    pub fn on_skill_cooldown(&mut self, cooldown_struct: SkillCooldownStruct) {
        let now = Utc::now().timestamp_millis();

        let cooldown_duration = if cooldown_struct.skill_cooldown_stack_data.has_stacks > 0 {
            (cooldown_struct
                .skill_cooldown_stack_data
                .current_stack_cooldown
                .unwrap_or_default()
                * 1000.0) as i64
        } else {
            (cooldown_struct.current_cooldown * 1000.0) as i64
        };

        let cooldowns = self
            .skill_tracker
            .skill_cooldowns
            .entry(cooldown_struct.skill_id)
            .or_default();

        // check if this is a cooldown reduction event (e.g. quick recharge, instant cooldown reduction)
        if let Some(last_event) = cooldowns.last_mut() {
            let last_cooldown_end = last_event.timestamp + last_event.cooldown_duration_ms;

            // if skill is still on cooldown, this is a cooldown reduction
            if now < last_cooldown_end {
                // update the cooldown to end at: current_time + new_duration
                // this means the total cooldown duration from cast time is:
                // (timestamp - last_event.timestamp) + cooldown_duration
                last_event.cooldown_duration_ms = (now - last_event.timestamp) + cooldown_duration;
                return;
            }
        }
        cooldowns.push(CastEvent {
            timestamp: now,
            cooldown_duration_ms: cooldown_duration,
        });

        // info!("skill cooldowns: {cooldowns:#?}");
        // info!(
        //     "total available time for {}: {}ms",
        //     cooldown_struct.skill_id,
        //     self.skill_tracker.calculate_total_available_time(
        //         cooldown_struct.skill_id,
        //         self.encounter.fight_start,
        //         now
        //     )
        // );
    }

    pub fn on_skill_start(
        &mut self,
        source_entity: &Entity,
        skill_id: u32,
        tripod_index: Option<TripodIndex>,
        timestamp: i64,
    ) -> (u32, Option<Vec<u32>>) {
        // do not track skills if encounter not started
        if self.encounter.fight_start == 0 || skill_id == 0 {
            return (0, None);
        }

        let (skill_name, skill_icon, summons, _, is_hyper_awakening) =
            get_skill_name_and_icon(skill_id, 0, &self.skill_tracker, source_entity.id);

        let entity = self
            .encounter
            .entities
            .entry(source_entity.name.clone())
            .or_insert_with(|| {
                let mut entity = encounter_entity_from_entity(source_entity);
                entity.skills = HashMap::from([(
                    skill_id,
                    Skill {
                        id: skill_id,
                        name: skill_name.clone(),
                        icon: skill_icon.clone(),
                        tripod_index,
                        casts: 0,
                        is_hyper_awakening,
                        ..Default::default()
                    },
                )]);
                entity
            });

        if entity.class_id == 0
            && source_entity.entity_type == EntityType::Player
            && source_entity.class_id > 0
        {
            entity.class_id = source_entity.class_id;
            entity.class = get_class_from_id(&source_entity.class_id);
        }

        Self::mark_entity_alive(entity, timestamp);
        entity.skill_stats.casts += 1;

        // if skills have different ids but the same name, we group them together
        let mut skill_id = skill_id;
        if let Some(skill) = entity.skills.get_mut(&skill_id) {
            skill.casts += 1;
            if tripod_index.is_some() {
                skill.tripod_index = tripod_index;
            }
        } else if let Some(skill) = entity
            .skills
            .values_mut()
            .find(|s| s.name == skill_name.clone())
        {
            // no id match found, search skills by name
            skill.casts += 1;
            skill_id = skill.id;
            if tripod_index.is_some() {
                skill.tripod_index = tripod_index;
            }
        } else {
            // no match for id or name
            entity.skills.insert(
                skill_id,
                Skill {
                    id: skill_id,
                    name: skill_name,
                    icon: skill_icon,
                    tripod_index,
                    casts: 1,
                    ..Default::default()
                },
            );
        }

        let relative_timestamp = if self.encounter.fight_start == 0 {
            0
        } else {
            (timestamp - self.encounter.fight_start) as i32
        };

        self.cast_log
            .entry(entity.name.clone())
            .or_default()
            .entry(skill_id)
            .or_default()
            .push(relative_timestamp);

        // if this is a getup skill and we have an ongoing abnormal move incapacitation, this will end it
        if let Some(skill_data) = SKILL_DATA.get(&skill_id)
            && skill_data.skill_type == "getup"
        {
            for ongoing_event in entity
                .damage_stats
                .incapacitations
                .iter_mut()
                .rev()
                .take_while(|x| x.timestamp + x.duration > timestamp)
                .filter(|x| x.event_type == IncapacitationEventType::FALL_DOWN)
            {
                info!(
                    "Shortening down duration from {} to {} because of getup skill",
                    ongoing_event.duration,
                    timestamp - ongoing_event.timestamp
                );
                ongoing_event.duration = timestamp - ongoing_event.timestamp;
            }
        }

        // set spec for supports to determine buff source
        if is_support_class(&entity.class_id) && entity.spec.is_none() {
            let spec = get_player_spec(entity, &self.encounter.encounter_damage_stats.buffs, true);
            if spec != "Unknown" {
                entity.spec = Some(spec);
            }
        }

        (skill_id, summons)
    }

    fn start_fight(
        &mut self,
        timestamp: i64,
        target_type: EntityType,
        skill_key: u32,
        source_entity_id: u64,
    ) {
        self.encounter.fight_start = timestamp;
        self.skill_tracker.fight_start = timestamp;
        if target_type == EntityType::Player && skill_key > 0 {
            self.skill_tracker
                .new_cast(source_entity_id, skill_key, None, timestamp);
        }

        self.fight_start_instant = Some(Instant::now());
        self.set_ntp_fight_start_from_cache();

        self.encounter.boss_only_damage = self.boss_only_damage;
        self.app
            .emit("raid-start", timestamp)
            .expect("failed to emit raid-start");
    }

    fn set_ntp_fight_start_from_cache(&mut self) {
        if self.ntp_fight_start != 0 || self.encounter.fight_start == 0 {
            return;
        }

        let Some(fight_start_instant) = self.fight_start_instant else {
            return;
        };

        if let Some(ntp_fight_start) = self
            .ntp_clock
            .timestamp_for_event(self.encounter.fight_start, fight_start_instant)
        {
            self.ntp_fight_start = ntp_fight_start;
        }
    }

    fn open_startup_barrier(
        &mut self,
        entity_tracker: &mut EntityTracker,
        required_inspect_names: Vec<String>,
        missing_inspects: &[String],
    ) {
        entity_tracker.reset_bootstrap_inspect_throttle();
        self.rdps_valid = false;
        self.rdps_message = if missing_inspects.is_empty() {
            None
        } else {
            Some("inspect_pending".into())
        };
        let inspect_targets = required_inspect_names
            .into_iter()
            .filter(|name| crate::live::entity_tracker::is_resolved_player_name(name))
            .map(|name| {
                (
                    name,
                    StartupInspectTarget {
                        gate_required: true,
                        live_stats_required: true,
                        buffered_stats_required: false,
                    },
                )
            })
            .collect();
        self.startup_barrier = Some(StartupBarrierState {
            inspect_targets,
            pending_skill: Vec::new(),
            pending_damage: Vec::new(),
            freeze_registered_names: false,
        });
    }

    fn record_contribution_data(
        &mut self,
        player_name: &str,
        player_entity_id: u64,
        skill_id: u32,
        damage: i64,
        can_crit: bool,
        is_critical_hit: bool,
        crit_metrics: Option<&HitCritMetrics>,
        stat_damage_metrics: Option<&HitStatDamageMetrics>,
        rdps_result: Option<&HitRdpsResult>,
        rdps_valid: bool,
    ) {
        let entry = self
            .player_contributions
            .entry(player_name.to_string())
            .or_default();
        let skill_entry = entry.skill_to_damage_map_.entry(skill_id).or_default();
        skill_entry.damage_ += damage;
        skill_entry.hits_ += 1;
        let self_damage = if rdps_valid {
            rdps_result.map_or(damage, |result| damage - result.rdps_damage_received)
        } else {
            damage
        };
        *entry
            .damage_split_by_entity_id_
            .entry(player_entity_id)
            .or_default() += self_damage;

        if can_crit {
            entry.damage_done_without_ultimate_awakening_ += damage;
        }

        if let Some(crit_metrics) = crit_metrics
            && crit_metrics.crit_damage_multiplier > 0.0
        {
            let damage_done_without_crits = if is_critical_hit {
                (damage as f64 / crit_metrics.crit_damage_multiplier) as i64
            } else {
                damage
            };
            let damage_done_with_all_crits = if is_critical_hit {
                damage
            } else {
                (damage as f64 * crit_metrics.crit_damage_multiplier) as i64
            };
            let damage_done_with_average_crits = (damage_done_without_crits as f64
                + (damage_done_without_crits as f64
                    * crit_metrics.crit_rate_capped
                    * (crit_metrics.crit_damage_multiplier - 1.0)))
                as i64;
            entry.damage_done_without_crits_ += damage_done_without_crits;
            entry.damage_done_with_all_crits_ += damage_done_with_all_crits;
            entry.damage_done_with_average_crits_ += damage_done_with_average_crits;
            entry.critical_hit_rate_adjusted_damage_raw_ +=
                (damage_done_without_crits as f64 * crit_metrics.crit_rate_raw) as i64;
            entry.critical_hit_rate_adjusted_damage_raw_capped_ +=
                (damage_done_without_crits as f64 * crit_metrics.crit_rate_capped) as i64;
        }

        if let Some(stat_damage_metrics) = stat_damage_metrics {
            entry
                .additional_damage_1percent_damage_
                .merge(stat_damage_metrics.additional_damage_1percent_damage);
            entry
                .critical_hit_rate_1percent_damage_
                .merge(stat_damage_metrics.critical_hit_rate_1percent_damage);
            entry
                .critical_damage_rate_1percent_damage_
                .merge(stat_damage_metrics.critical_damage_rate_1percent_damage);
            entry
                .evo_damage_1percent_damage_
                .merge(stat_damage_metrics.evo_damage_1percent_damage);
            entry
                .weapon_power_1000_damage_
                .merge(stat_damage_metrics.weapon_power_1000_damage);
            entry
                .weapon_power_1percent_damage_
                .merge(stat_damage_metrics.weapon_power_1percent_damage);
            entry
                .attack_power_1000_damage_
                .merge(stat_damage_metrics.attack_power_1000_damage);
            entry
                .attack_power_1percent_damage_
                .merge(stat_damage_metrics.attack_power_1percent_damage);
            entry
                .main_stat_1000_damage_
                .merge(stat_damage_metrics.main_stat_1000_damage);
            entry
                .raid_captain_efficiency_
                .merge(stat_damage_metrics.raid_captain_efficiency);
            entry
                .blunt_thorn_efficiency_
                .merge(stat_damage_metrics.blunt_thorn_efficiency);
            entry
                .supersonic_breakthrough_efficiency_
                .merge(stat_damage_metrics.supersonic_breakthrough_efficiency);
            entry
                .standing_striker_efficiency_
                .merge(stat_damage_metrics.standing_striker_efficiency);
        }

        let Some(result) = rdps_result else {
            return;
        };

        for attribution in &result.entity_attributions {
            if attribution.source_entity_id == 0 || attribution.damage <= 0 {
                continue;
            }
            *entry
                .damage_split_by_entity_id_
                .entry(attribution.source_entity_id)
                .or_default() += attribution.damage;
        }

        for attribution in &result.skill_group_attributions {
            if attribution.source_entity_id == 0
                || (attribution.damage <= 0 && attribution.damage_increase <= 0)
            {
                continue;
            }
            *entry
                .damage_done_by_entity_skill_group_
                .entry(attribution.source_entity_id)
                .or_default()
                .entry(attribution.group_name.clone())
                .or_default() += attribution.damage;
            *entry
                .damage_increase_by_entity_skill_group_
                .entry(attribution.source_entity_id)
                .or_default()
                .entry(attribution.group_name.clone())
                .or_default() += attribution.damage_increase;
        }
    }

    fn invalidate_rdps(&mut self, reason: RdpsInvalidReason) {
        if !self.rdps_valid {
            return;
        }

        let reason_key = reason.message_key();
        let reason_context = reason.diagnostic_context();
        self.rdps_valid = false;
        warn!(
            "rDPS invalidated: reason={} context=\"{}\" boss=\"{}\" local_player=\"{}\" fight_start={} last_combat_packet={}",
            reason_key,
            reason_context,
            self.encounter.current_boss_name,
            self.encounter.local_player,
            self.encounter.fight_start,
            self.encounter.last_combat_packet
        );
        self.rdps_message = Some(reason_key.to_string());
        self.scrub_rdps_derived_state();
    }

    fn scrub_rdps_derived_state(&mut self) {
        for entity in self.encounter.entities.values_mut() {
            entity.damage_stats.rdps_damage_received = 0;
            entity.damage_stats.rdps_damage_received_support = 0;
            entity.damage_stats.rdps_damage_given = 0;
            entity.damage_stats.rdps = 0;
            entity.damage_stats.ndps = 0;

            for skill in entity.skills.values_mut() {
                skill.rdps_damage_received = 0;
                skill.rdps_damage_received_support = 0;
                for cast in &mut skill.skill_cast_log {
                    for hit in &mut cast.hits {
                        hit.rdps_damage_received = 0;
                        hit.rdps_damage_received_support = 0;
                    }
                }
            }
        }

        for (name, accumulator) in self.player_contributions.iter_mut() {
            if let Some(entity) = self.encounter.entities.get(name) {
                accumulator.damage_split_by_entity_id_.clear();
                accumulator
                    .damage_split_by_entity_id_
                    .insert(entity.id, entity.damage_stats.damage_dealt);
            } else {
                accumulator.damage_split_by_entity_id_.clear();
            }
            accumulator.damage_done_by_entity_skill_group_.clear();
            accumulator.damage_increase_by_entity_skill_group_.clear();
        }
    }

    fn recompute_entity_udps_unbuffed(entity: &mut EncounterEntity) {
        entity.damage_stats.unbuffed_damage =
            entity.damage_stats.damage_dealt - entity.damage_stats.buffed_damage;
    }

    fn include_in_lal_damage_dump(entity: &EncounterEntity, local_player: &str) -> bool {
        match entity.entity_type {
            EntityType::Player => {
                is_confirmed_player_entity(entity, local_player)
                    && entity.damage_stats.damage_dealt > 0
            }
            EntityType::DarkGrenade => entity.damage_stats.rdps_damage_given > 0,
            _ => false,
        }
    }

    fn ensure_dark_grenade_entity(&mut self) -> &mut EncounterEntity {
        self.encounter
            .entities
            .entry(DARK_GRENADE_ENTITY_NAME.to_string())
            .or_insert_with(|| EncounterEntity {
                id: DARK_GRENADE_ENTITY_ID,
                name: DARK_GRENADE_ENTITY_NAME.to_string(),
                entity_type: EntityType::DarkGrenade,
                class: DARK_GRENADE_ENTITY_NAME.to_string(),
                ..Default::default()
            })
    }

    pub fn record_lal_skill_event_debug(
        &mut self,
        source_entity: &Entity,
        skill_id: u32,
        is_skill_cast_notify: bool,
    ) {
        if !DEBUG_DUMP_DAMAGE_STATE_JSON
            || source_entity.entity_type != EntityType::Player
            || source_entity.id == 0
        {
            return;
        }

        let entry = self
            .player_contributions
            .entry(source_entity.name.clone())
            .or_default();
        if is_skill_cast_notify {
            entry.casts_ += 1;
        }
        entry.skill_casts_ += 1;
        entry
            .skill_to_damage_map_
            .entry(skill_id)
            .or_default()
            .skill_casts_ += 1;
    }

    pub fn set_lal_debug_zone(&mut self, zone_id: u32, zone_level: Option<u32>) {
        if zone_id != 0 {
            self.lal_debug_zone_id = zone_id;
        }
        if let Some(zone_level) = zone_level {
            self.lal_debug_zone_level = zone_level;
        }
    }

    pub fn set_lal_debug_damage_key_base64(&mut self, damage_key_base64: Option<String>) {
        self.lal_debug_damage_key_base64 = damage_key_base64.unwrap_or_default();
    }

    fn build_contribution_splits(&mut self) -> Vec<ContributionSplit> {
        let id_to_name: HashMap<u64, &str> = self
            .encounter
            .entities
            .values()
            .map(|e| (e.id, e.name.as_str()))
            .collect();

        let resolve_ids = |map: HashMap<u64, i64>| -> HashMap<String, i64> {
            map.into_iter()
                .filter_map(|(id, v)| id_to_name.get(&id).map(|name| (name.to_string(), v)))
                .collect()
        };

        let resolve_ids_nested =
            |map: HashMap<u64, HashMap<String, i64>>| -> HashMap<String, HashMap<String, i64>> {
                map.into_iter()
                    .filter_map(|(id, inner)| {
                        id_to_name.get(&id).map(|name| (name.to_string(), inner))
                    })
                    .collect()
            };

        let mut accumulators = std::mem::take(&mut self.player_contributions);

        self.encounter
            .entities
            .values()
            .filter(|e| Self::include_in_lal_damage_dump(e, &self.encounter.local_player))
            .map(|entity| {
                let (
                    damage_split_by_name,
                    damage_done_by_entity_skill_group,
                    damage_increase_by_entity_skill_group,
                    damage_done_without_ultimate_awakening,
                    damage_done_without_crits,
                    damage_done_with_all_crits,
                    damage_done_with_average_crits,
                    critical_hit_rate_adjusted_damage_raw,
                    critical_hit_rate_adjusted_damage_raw_capped,
                    additional_damage_1percent_damage,
                    critical_hit_rate_1percent_damage,
                    critical_damage_rate_1percent_damage,
                    evo_damage_1percent_damage,
                    weapon_power_1000_damage,
                    weapon_power_1percent_damage,
                    attack_power_1000_damage,
                    attack_power_1percent_damage,
                    main_stat_1000_damage,
                    raid_captain_efficiency,
                    blunt_thorn_efficiency,
                    supersonic_breakthrough_efficiency,
                    standing_striker_efficiency,
                ) = if let Some(acc) = accumulators.remove(&entity.name) {
                    (
                        resolve_ids(acc.damage_split_by_entity_id_),
                        resolve_ids_nested(acc.damage_done_by_entity_skill_group_),
                        resolve_ids_nested(acc.damage_increase_by_entity_skill_group_),
                        acc.damage_done_without_ultimate_awakening_,
                        acc.damage_done_without_crits_,
                        acc.damage_done_with_all_crits_,
                        acc.damage_done_with_average_crits_,
                        acc.critical_hit_rate_adjusted_damage_raw_,
                        acc.critical_hit_rate_adjusted_damage_raw_capped_,
                        acc.additional_damage_1percent_damage_,
                        acc.critical_hit_rate_1percent_damage_,
                        acc.critical_damage_rate_1percent_damage_,
                        acc.evo_damage_1percent_damage_,
                        acc.weapon_power_1000_damage_,
                        acc.weapon_power_1percent_damage_,
                        acc.attack_power_1000_damage_,
                        acc.attack_power_1percent_damage_,
                        acc.main_stat_1000_damage_,
                        acc.raid_captain_efficiency_,
                        acc.blunt_thorn_efficiency_,
                        acc.supersonic_breakthrough_efficiency_,
                        acc.standing_striker_efficiency_,
                    )
                } else {
                    let self_damage = if self.rdps_valid {
                        entity.damage_stats.damage_dealt - entity.damage_stats.rdps_damage_received
                    } else {
                        entity.damage_stats.damage_dealt
                    };
                    (
                        [(entity.name.clone(), self_damage)].into(),
                        HashMap::new(),
                        HashMap::new(),
                        entity
                            .damage_stats
                            .damage_dealt
                            .saturating_sub(entity.damage_stats.hyper_awakening_damage),
                        0,
                        0,
                        0,
                        0,
                        0,
                        StatDamageContribution::default(),
                        StatDamageContribution::default(),
                        StatDamageContribution::default(),
                        StatDamageContribution::default(),
                        StatDamageContribution::default(),
                        StatDamageContribution::default(),
                        StatDamageContribution::default(),
                        StatDamageContribution::default(),
                        StatDamageContribution::default(),
                        StatDamageContribution::default(),
                        StatDamageContribution::default(),
                        StatDamageContribution::default(),
                        StatDamageContribution::default(),
                    )
                };
                let hyper_awakening_damage = entity.damage_stats.hyper_awakening_damage;
                ContributionSplit {
                    name: entity.name.clone(),
                    party_number: if entity.entity_type == EntityType::DarkGrenade {
                        Some(-1)
                    } else {
                        self.party_info
                            .iter()
                            .position(|party| party.iter().any(|n| n == &entity.name))
                            .map(|i| i as i32)
                    },
                    damage_split_by_name,
                    damage_done_by_entity_skill_group,
                    damage_increase_by_entity_skill_group,
                    damage_done_without_ultimate_awakening,
                    hyper_awakening_damage,
                    damage_done_without_crits,
                    damage_done_with_all_crits,
                    damage_done_with_average_crits,
                    critical_hit_rate_adjusted_damage_raw,
                    critical_hit_rate_adjusted_damage_raw_capped,
                    additional_damage_1percent_damage,
                    critical_hit_rate_1percent_damage,
                    critical_damage_rate_1percent_damage,
                    evo_damage_1percent_damage,
                    weapon_power_1000_damage,
                    weapon_power_1percent_damage,
                    attack_power_1000_damage,
                    attack_power_1percent_damage,
                    main_stat_1000_damage,
                    raid_captain_efficiency,
                    blunt_thorn_efficiency,
                    supersonic_breakthrough_efficiency,
                    standing_striker_efficiency,
                }
            })
            .collect()
    }

    fn build_damage_state_dump(&self) -> DamageStateDump {
        let mut player_id_to_damage_data_ = HashMap::new();
        for entity in
            self.encounter.entities.values().filter(|entity| {
                Self::include_in_lal_damage_dump(entity, &self.encounter.local_player)
            })
        {
            let accumulator = self.player_contributions.get(&entity.name);
            let entity_id_ = entity.id;
            let damage_done_ = entity.damage_stats.damage_dealt;
            let damage_done_without_ultimate_awakening_ = accumulator
                .map(|value| value.damage_done_without_ultimate_awakening_)
                .unwrap_or_else(|| {
                    damage_done_.saturating_sub(entity.damage_stats.hyper_awakening_damage)
                });
            let damage_done_without_crits_ = accumulator
                .map(|value| value.damage_done_without_crits_)
                .unwrap_or_default();
            let damage_done_with_all_crits_ = accumulator
                .map(|value| value.damage_done_with_all_crits_)
                .unwrap_or_default();
            let damage_done_with_average_crits_ = accumulator
                .map(|value| value.damage_done_with_average_crits_)
                .unwrap_or_default();
            let critical_hit_rate_adjusted_damage_raw_ = accumulator
                .map(|value| value.critical_hit_rate_adjusted_damage_raw_)
                .unwrap_or_default();
            let critical_hit_rate_adjusted_damage_raw_capped_ = accumulator
                .map(|value| value.critical_hit_rate_adjusted_damage_raw_capped_)
                .unwrap_or_default();
            let additional_damage_1percent_damage_ = accumulator
                .map(|value| StatDamageDump::from(value.additional_damage_1percent_damage_))
                .unwrap_or_default();
            let critical_hit_rate_1percent_damage_ = accumulator
                .map(|value| StatDamageDump::from(value.critical_hit_rate_1percent_damage_))
                .unwrap_or_default();
            let critical_damage_rate_1percent_damage_ = accumulator
                .map(|value| StatDamageDump::from(value.critical_damage_rate_1percent_damage_))
                .unwrap_or_default();
            let evo_damage_1percent_damage_ = accumulator
                .map(|value| StatDamageDump::from(value.evo_damage_1percent_damage_))
                .unwrap_or_default();
            let weapon_power_1000_damage_ = accumulator
                .map(|value| StatDamageDump::from(value.weapon_power_1000_damage_))
                .unwrap_or_default();
            let weapon_power_1percent_damage_ = accumulator
                .map(|value| StatDamageDump::from(value.weapon_power_1percent_damage_))
                .unwrap_or_default();
            let attack_power_1000_damage_ = accumulator
                .map(|value| StatDamageDump::from(value.attack_power_1000_damage_))
                .unwrap_or_default();
            let attack_power_1percent_damage_ = accumulator
                .map(|value| StatDamageDump::from(value.attack_power_1percent_damage_))
                .unwrap_or_default();
            let main_stat_1000_damage_ = accumulator
                .map(|value| StatDamageDump::from(value.main_stat_1000_damage_))
                .unwrap_or_default();
            let raid_captain_efficiency_ = accumulator
                .map(|value| StatDamageDump::from(value.raid_captain_efficiency_))
                .unwrap_or_default();
            let blunt_thorn_efficiency_ = accumulator
                .map(|value| StatDamageDump::from(value.blunt_thorn_efficiency_))
                .unwrap_or_default();
            let supersonic_breakthrough_efficiency_ = accumulator
                .map(|value| StatDamageDump::from(value.supersonic_breakthrough_efficiency_))
                .unwrap_or_default();
            let standing_striker_efficiency_ = accumulator
                .map(|value| StatDamageDump::from(value.standing_striker_efficiency_))
                .unwrap_or_default();
            let skill_to_damage_map_ = if let Some(accumulator) = accumulator {
                accumulator.skill_to_damage_map_.clone()
            } else {
                let mut skill_to_damage_map_ = HashMap::<u32, SkillStatsDump>::new();
                for skill in entity.skills.values() {
                    let skill_dump = skill_to_damage_map_.entry(skill.id).or_default();
                    skill_dump.damage_ = skill.total_damage;
                    skill_dump.hits_ = skill.hits;
                    skill_dump.skill_casts_ = skill.casts;
                }
                skill_to_damage_map_
            };
            let casts_ = accumulator.map(|value| value.casts_).unwrap_or_default();
            let skill_casts_ = accumulator
                .map(|value| value.skill_casts_)
                .unwrap_or(entity.skill_stats.casts);

            player_id_to_damage_data_.insert(
                entity_id_,
                DamageDataDump {
                    player_name_: entity.name.clone(),
                    party_number_: if entity.entity_type == EntityType::DarkGrenade {
                        -1
                    } else {
                        self.party_info
                            .iter()
                            .position(|party| party.iter().any(|name| name == &entity.name))
                            .map(|index| index as i32)
                            .unwrap_or(-2)
                    },
                    entity_id_,
                    damage_done_,
                    damage_done_without_ultimate_awakening_,
                    damage_done_without_crits_,
                    damage_done_with_all_crits_,
                    damage_done_with_average_crits_,
                    critical_hit_rate_adjusted_damage_raw_,
                    critical_hit_rate_adjusted_damage_raw_capped_,
                    damage_split_by_entity_id_: accumulator
                        .map(|value| value.damage_split_by_entity_id_.clone())
                        .unwrap_or_else(|| {
                            let mut map = HashMap::new();
                            map.insert(
                                entity_id_,
                                if self.rdps_valid {
                                    damage_done_ - entity.damage_stats.rdps_damage_received
                                } else {
                                    damage_done_
                                },
                            );
                            map
                        }),
                    damage_done_by_entity_skill_group_: accumulator
                        .map(|value| value.damage_done_by_entity_skill_group_.clone())
                        .unwrap_or_default(),
                    damage_increase_by_entity_skill_group_: accumulator
                        .map(|value| value.damage_increase_by_entity_skill_group_.clone())
                        .unwrap_or_default(),
                    additional_damage_1percent_damage_,
                    critical_hit_rate_1percent_damage_,
                    critical_damage_rate_1percent_damage_,
                    evo_damage_1percent_damage_,
                    weapon_power_1000_damage_,
                    weapon_power_1percent_damage_,
                    attack_power_1000_damage_,
                    attack_power_1percent_damage_,
                    main_stat_1000_damage_,
                    raid_captain_efficiency_,
                    blunt_thorn_efficiency_,
                    supersonic_breakthrough_efficiency_,
                    standing_striker_efficiency_,
                    damage_done_under_atk_power_: 0,
                    damage_done_under_brand_: 0,
                    damage_done_under_identity_: 0,
                    damage_done_under_hyper_: 0,
                    casts_,
                    skill_casts_,
                    skill_to_damage_map_,
                    ..Default::default()
                },
            );
        }

        DamageStateDump {
            start_time_: timestamp_ms_to_lal_datetime(self.encounter.fight_start),
            end_time_: self
                .lal_debug_end_time_ms
                .map(timestamp_ms_to_lal_datetime)
                .unwrap_or_else(lal_default_datetime),
            last_damage_done_time_: timestamp_ms_to_lal_datetime(self.encounter.last_combat_packet),
            silent_period_duration_seconds_: 0.0,
            zone_id_: self.lal_debug_zone_id,
            zone_level_: self.lal_debug_zone_level,
            damage_key_base64_: self.lal_debug_damage_key_base64.clone(),
            player_id_to_damage_data_,
            npc_to_skill_cast_data_: HashMap::new(),
        }
    }

    fn enqueue_pending_damage(
        &mut self,
        packet_seq: i64,
        dmg_src_entity: &Entity,
        proj_entity: &Entity,
        dmg_target_entity: &Entity,
        damage_data: DamageData,
        hit_context: DamageHitContext,
        se_on_source: Vec<StatusEffectDetails>,
        se_on_target: Vec<StatusEffectDetails>,
        target_count: i32,
        timestamp: i64,
        entity_tracker: &mut EntityTracker,
        status_tracker: &mut StatusTracker,
    ) {
        let buffered_player_entities = Self::collect_buffered_player_entities(
            dmg_src_entity,
            &se_on_source,
            &se_on_target,
            entity_tracker,
        );
        let owner_self_effects_by_entity_id =
            Self::collect_buffered_owner_self_effects(&buffered_player_entities, status_tracker);
        if let Some(barrier) = self.startup_barrier.as_mut() {
            let pending_damage = PendingDamageEvent {
                packet_seq,
                dmg_src_entity: dmg_src_entity.clone(),
                proj_entity: proj_entity.clone(),
                dmg_target_entity: dmg_target_entity.clone(),
                damage_data,
                hit_context,
                se_on_source,
                se_on_target,
                target_count,
                timestamp,
                buffered_player_entities,
                owner_self_effects_by_entity_id,
            };
            Self::mark_startup_buffered_inspect_targets(
                barrier,
                entity_tracker,
                pending_damage.buffered_player_entities.values(),
            );
            barrier.pending_damage.push(pending_damage);
        }
    }

    pub fn queue_pending_skill_event(
        &mut self,
        packet_seq: i64,
        source_entity: &Entity,
        skill_id: u32,
        skill_level: u8,
        tripod_index: Option<TripodIndex>,
        skill_option_snapshot: Option<SkillOptionSnapshot>,
        timestamp: i64,
        create_skill_tracker_cast: bool,
    ) {
        if let Some(barrier) = self.startup_barrier.as_mut() {
            barrier.pending_skill.push(PendingSkillEvent {
                packet_seq,
                source_entity: source_entity.clone(),
                skill_id,
                skill_level,
                tripod_index,
                skill_option_snapshot,
                timestamp,
                create_skill_tracker_cast,
            });
        }
    }

    pub fn startup_barrier_active(&self) -> bool {
        self.startup_barrier.is_some()
    }

    pub fn remove_startup_required_player(&mut self, player: &Entity) -> bool {
        let Some(barrier) = self.startup_barrier.as_mut() else {
            return false;
        };

        if !crate::live::entity_tracker::is_resolved_player_name(&player.name) {
            return false;
        }

        let keep_stats_required = barrier
            .inspect_targets
            .get(&player.name)
            .is_some_and(|target| target.buffered_stats_required);
        if let Some(target) = barrier.inspect_targets.get_mut(&player.name) {
            target.gate_required = false;
            target.live_stats_required = false;
        }
        barrier
            .inspect_targets
            .retain(|_, target| target.gate_required || Self::target_stats_required(target));

        keep_stats_required
    }

    fn collect_buffered_player_entities(
        dmg_src_entity: &Entity,
        se_on_source: &[StatusEffectDetails],
        se_on_target: &[StatusEffectDetails],
        entity_tracker: &mut EntityTracker,
    ) -> HashMap<u64, Entity> {
        let mut buffered_player_entities = HashMap::new();
        if dmg_src_entity.entity_type == EntityType::Player {
            buffered_player_entities.insert(dmg_src_entity.id, dmg_src_entity.clone());
        }

        for effect in se_on_source.iter().chain(se_on_target.iter()) {
            let Some(source_entity) = entity_tracker.get_entity_ref(effect.source_id) else {
                continue;
            };

            let owner_entity_id = if matches!(
                source_entity.entity_type,
                EntityType::Projectile | EntityType::Summon
            ) && source_entity.owner_id != 0
            {
                source_entity.owner_id
            } else {
                source_entity.id
            };
            let Some(owner_entity) = entity_tracker
                .get_entity_ref(owner_entity_id)
                .filter(|entity| entity.entity_type == EntityType::Player)
            else {
                continue;
            };
            buffered_player_entities
                .entry(owner_entity.id)
                .or_insert_with(|| owner_entity.clone());
        }

        buffered_player_entities
    }

    fn collect_buffered_owner_self_effects(
        buffered_player_entities: &HashMap<u64, Entity>,
        status_tracker: &mut StatusTracker,
    ) -> HashMap<u64, Vec<StatusEffectDetails>> {
        let timestamp = Utc::now();
        let mut owner_self_effects_by_entity_id = HashMap::new();

        for entity in buffered_player_entities.values() {
            if entity.entity_type != EntityType::Player {
                continue;
            }

            owner_self_effects_by_entity_id.insert(
                entity.id,
                status_tracker.get_source_status_effects(entity, timestamp),
            );
        }

        owner_self_effects_by_entity_id
    }

    fn reconcile_startup_live_inspect_targets(
        barrier: &mut StartupBarrierState,
        entity_tracker: &mut EntityTracker,
    ) {
        if !barrier.freeze_registered_names {
            for target in barrier.inspect_targets.values_mut() {
                target.gate_required = false;
                target.live_stats_required = false;
            }
            for name in entity_tracker.get_required_bootstrap_player_names() {
                Self::mark_startup_live_inspect_target(barrier, name);
            }
        }
        for name in entity_tracker.get_bootstrap_visible_fallback_required_names() {
            Self::mark_startup_live_inspect_target(barrier, name);
        }
        barrier
            .inspect_targets
            .retain(|_, target| target.gate_required || Self::target_stats_required(target));
    }

    fn mark_startup_buffered_inspect_targets<'a>(
        barrier: &mut StartupBarrierState,
        entity_tracker: &EntityTracker,
        entities: impl IntoIterator<Item = &'a Entity>,
    ) {
        for entity in entities {
            if entity.entity_type != EntityType::Player {
                continue;
            }
            let is_gate_eligible =
                entity_tracker.is_bootstrap_party_inspect_eligible_player_entity(entity);
            if crate::live::entity_tracker::is_resolved_player_name(&entity.name) {
                let target = barrier
                    .inspect_targets
                    .entry(entity.name.clone())
                    .or_default();
                target.buffered_stats_required = true;
                if is_gate_eligible {
                    target.gate_required = true;
                }
            }
        }
        barrier
            .inspect_targets
            .retain(|_, target| target.gate_required || Self::target_stats_required(target));
    }

    fn mark_startup_live_inspect_target(barrier: &mut StartupBarrierState, name: String) {
        if !crate::live::entity_tracker::is_resolved_player_name(&name) {
            return;
        }

        let target = barrier.inspect_targets.entry(name).or_default();
        target.gate_required = true;
        target.live_stats_required = true;
    }

    fn target_stats_required(target: &StartupInspectTarget) -> bool {
        target.live_stats_required || target.buffered_stats_required
    }

    fn startup_gate_required_names(barrier: &StartupBarrierState) -> Vec<String> {
        Self::startup_required_names(barrier, |target| target.gate_required)
    }

    fn startup_stats_required_names(barrier: &StartupBarrierState) -> Vec<String> {
        Self::startup_required_names(barrier, Self::target_stats_required)
    }

    fn startup_required_names(
        barrier: &StartupBarrierState,
        mut is_required: impl FnMut(&StartupInspectTarget) -> bool,
    ) -> Vec<String> {
        let mut names = barrier
            .inspect_targets
            .iter()
            .filter_map(|(name, target)| is_required(target).then_some(name.clone()))
            .collect::<Vec<_>>();
        names.sort();
        names
    }

    pub fn try_flush_startup_barrier(&mut self, entity_tracker: &mut EntityTracker) {
        let Some(barrier) = self.startup_barrier.as_mut() else {
            return;
        };

        Self::reconcile_startup_live_inspect_targets(barrier, entity_tracker);
        let gate_names = Self::startup_gate_required_names(barrier);
        let stats_names = Self::startup_stats_required_names(barrier);
        let gate_ready = entity_tracker.is_startup_barrier_gate_ready(&gate_names);
        let stats_ready = entity_tracker.is_startup_barrier_stats_ready(&stats_names);
        if barrier.pending_damage.is_empty() && !barrier.pending_skill.is_empty() && !gate_ready {
            let pending_skill = std::mem::take(&mut barrier.pending_skill);
            for pending_skill in pending_skill {
                self.replay_pending_skill_event(pending_skill, entity_tracker);
            }
            if let Some(phase_code) = self.pending_phase_transition.take() {
                self.on_phase_transition(phase_code);
            }
            return;
        }
        if !gate_ready {
            return;
        }

        let (pending_skill, pending_damage) = self
            .startup_barrier
            .take()
            .map(|barrier| (barrier.pending_skill, barrier.pending_damage))
            .unwrap_or_default();
        entity_tracker.reset_bootstrap_inspect_throttle();

        if stats_ready {
            self.rdps_valid = true;
            self.rdps_message = None;
        } else {
            self.rdps_valid = false;
            self.rdps_message = Some("inspect_timeout".into());
        }

        let mut ordered_events = Vec::with_capacity(pending_skill.len() + pending_damage.len());
        for pending_skill in pending_skill {
            ordered_events.push((pending_skill.packet_seq, Some(pending_skill), None));
        }
        for pending_damage in pending_damage {
            ordered_events.push((pending_damage.packet_seq, None, Some(pending_damage)));
        }
        ordered_events.sort_by_key(|(packet_seq, _, _)| *packet_seq);

        for (_, pending_skill, pending_damage) in ordered_events {
            if let Some(pending_skill) = pending_skill {
                self.replay_pending_skill_event(pending_skill, entity_tracker);
            } else if let Some(pending_damage) = pending_damage {
                self.apply_damage(
                    &pending_damage.dmg_src_entity,
                    &pending_damage.proj_entity,
                    &pending_damage.dmg_target_entity,
                    pending_damage.damage_data,
                    pending_damage.hit_context,
                    pending_damage.se_on_source,
                    pending_damage.se_on_target,
                    pending_damage.target_count,
                    entity_tracker,
                    pending_damage.timestamp,
                    Some(&pending_damage.buffered_player_entities),
                    Some(&pending_damage.owner_self_effects_by_entity_id),
                );
            }
        }

        if let Some(phase_code) = self.pending_phase_transition.take() {
            self.on_phase_transition(phase_code);
        }
    }

    pub fn force_release_startup_barrier(
        &mut self,
        entity_tracker: &mut EntityTracker,
        invalid_reason: &str,
    ) {
        let Some(barrier) = self.startup_barrier.as_mut() else {
            return;
        };

        Self::reconcile_startup_live_inspect_targets(barrier, entity_tracker);
        let stats_names = Self::startup_stats_required_names(barrier);
        let stats_ready = entity_tracker.is_startup_barrier_stats_ready(&stats_names);
        let has_inspect_failures = entity_tracker.has_failed_startup_barrier_inspects(&stats_names);
        let (pending_skill, pending_damage) = self
            .startup_barrier
            .take()
            .map(|barrier| (barrier.pending_skill, barrier.pending_damage))
            .unwrap_or_default();
        entity_tracker.reset_bootstrap_inspect_throttle();

        if stats_ready {
            self.rdps_valid = true;
            self.rdps_message = None;
        } else if has_inspect_failures {
            self.rdps_valid = false;
            self.rdps_message = Some("inspect_timeout".into());
        } else {
            self.rdps_valid = false;
            self.rdps_message = Some(invalid_reason.into());
        }

        let mut ordered_events = Vec::with_capacity(pending_skill.len() + pending_damage.len());
        for pending_skill in pending_skill {
            ordered_events.push((pending_skill.packet_seq, Some(pending_skill), None));
        }
        for pending_damage in pending_damage {
            ordered_events.push((pending_damage.packet_seq, None, Some(pending_damage)));
        }
        ordered_events.sort_by_key(|(packet_seq, _, _)| *packet_seq);

        for (_, pending_skill, pending_damage) in ordered_events {
            if let Some(pending_skill) = pending_skill {
                self.replay_pending_skill_event(pending_skill, entity_tracker);
            } else if let Some(pending_damage) = pending_damage {
                self.apply_damage(
                    &pending_damage.dmg_src_entity,
                    &pending_damage.proj_entity,
                    &pending_damage.dmg_target_entity,
                    pending_damage.damage_data,
                    pending_damage.hit_context,
                    pending_damage.se_on_source,
                    pending_damage.se_on_target,
                    pending_damage.target_count,
                    entity_tracker,
                    pending_damage.timestamp,
                    Some(&pending_damage.buffered_player_entities),
                    Some(&pending_damage.owner_self_effects_by_entity_id),
                );
            }
        }

        if let Some(phase_code) = self.pending_phase_transition.take() {
            self.on_phase_transition(phase_code);
        }
    }

    pub fn request_phase_transition(
        &mut self,
        phase_code: i32,
        entity_tracker: &mut EntityTracker,
    ) -> bool {
        let Some(barrier) = self.startup_barrier.as_mut() else {
            return false;
        };

        Self::reconcile_startup_live_inspect_targets(barrier, entity_tracker);
        barrier.freeze_registered_names = true;
        self.pending_phase_transition = Some(phase_code);
        true
    }

    pub fn rebind_startup_player_entity_ids(&mut self, old_entity_id: u64, new_entity_id: u64) {
        if old_entity_id == 0 || old_entity_id == new_entity_id {
            return;
        }

        for owner_id in self.source_owner_aliases.values_mut() {
            if *owner_id == old_entity_id {
                *owner_id = new_entity_id;
            }
        }
        for owner_id in self.resolved_source_owner_aliases.values_mut() {
            if *owner_id == old_entity_id {
                *owner_id = new_entity_id;
            }
        }

        let Some(barrier) = self.startup_barrier.as_mut() else {
            return;
        };

        for pending_skill in &mut barrier.pending_skill {
            if pending_skill.source_entity.id == old_entity_id {
                pending_skill.source_entity.id = new_entity_id;
            }
            if pending_skill.source_entity.owner_id == old_entity_id {
                pending_skill.source_entity.owner_id = new_entity_id;
            }
        }

        for pending_damage in &mut barrier.pending_damage {
            if pending_damage.dmg_src_entity.id == old_entity_id {
                pending_damage.dmg_src_entity.id = new_entity_id;
            }
            if pending_damage.dmg_src_entity.owner_id == old_entity_id {
                pending_damage.dmg_src_entity.owner_id = new_entity_id;
            }
            if pending_damage.proj_entity.id == old_entity_id {
                pending_damage.proj_entity.id = new_entity_id;
            }
            if pending_damage.proj_entity.owner_id == old_entity_id {
                pending_damage.proj_entity.owner_id = new_entity_id;
            }
            for effect in pending_damage
                .se_on_source
                .iter_mut()
                .chain(pending_damage.se_on_target.iter_mut())
            {
                if effect.source_id == old_entity_id {
                    effect.source_id = new_entity_id;
                }
            }

            if let Some(mut buffered_entity) = pending_damage
                .buffered_player_entities
                .remove(&old_entity_id)
            {
                buffered_entity.id = new_entity_id;
                buffered_entity.owner_id = if buffered_entity.owner_id == old_entity_id {
                    new_entity_id
                } else {
                    buffered_entity.owner_id
                };
                pending_damage
                    .buffered_player_entities
                    .entry(new_entity_id)
                    .or_insert(buffered_entity);
            }

            if let Some(owner_self_effects) = pending_damage
                .owner_self_effects_by_entity_id
                .remove(&old_entity_id)
            {
                let owner_self_effects = owner_self_effects
                    .into_iter()
                    .map(|mut effect| {
                        if effect.source_id == old_entity_id {
                            effect.source_id = new_entity_id;
                        }
                        effect
                    })
                    .collect();
                pending_damage
                    .owner_self_effects_by_entity_id
                    .entry(new_entity_id)
                    .or_insert(owner_self_effects);
            }
        }
    }

    fn replay_pending_skill_event(
        &mut self,
        pending_skill: PendingSkillEvent,
        entity_tracker: &mut EntityTracker,
    ) {
        if pending_skill.create_skill_tracker_cast {
            entity_tracker.record_skill_start_snapshot(
                pending_skill.source_entity.id,
                pending_skill.skill_id,
                pending_skill.skill_level,
                pending_skill.skill_option_snapshot,
                pending_skill.timestamp,
            );
        } else {
            entity_tracker.record_skill_cast(
                pending_skill.source_entity.id,
                pending_skill.skill_id,
                pending_skill.skill_level,
                pending_skill.timestamp,
            );
        }

        let (skill_id, summon_source) = self.on_skill_start(
            &pending_skill.source_entity,
            pending_skill.skill_id,
            pending_skill.tripod_index,
            pending_skill.timestamp,
        );
        self.record_lal_skill_event_debug(
            &pending_skill.source_entity,
            pending_skill.skill_id,
            !pending_skill.create_skill_tracker_cast,
        );
        if pending_skill.create_skill_tracker_cast
            && (pending_skill.source_entity.entity_type == EntityType::Player
                || pending_skill.source_entity.entity_type == EntityType::Unknown)
            && pending_skill.source_entity.class_id > 0
            && skill_id > 0
        {
            self.skill_tracker.new_cast(
                pending_skill.source_entity.id,
                skill_id,
                summon_source,
                pending_skill.timestamp,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn on_damage(
        &mut self,
        packet_seq: i64,
        dmg_src_entity: &Entity,
        proj_entity: &Entity,
        dmg_target_entity: &Entity,
        damage_data: DamageData,
        se_on_source: Vec<StatusEffectDetails>,
        se_on_target: Vec<StatusEffectDetails>,
        _target_count: i32,
        entity_tracker: &mut EntityTracker,
        status_tracker: &mut StatusTracker,
        inspect_transport_available: bool,
        timestamp: i64,
    ) {
        if self.disabled {
            return;
        }

        let Some(hit_context) =
            self.get_valid_damage_hit(dmg_src_entity, dmg_target_entity, &damage_data)
        else {
            return;
        };

        if self.startup_barrier.is_some() {
            self.enqueue_pending_damage(
                packet_seq,
                dmg_src_entity,
                proj_entity,
                dmg_target_entity,
                damage_data,
                hit_context,
                se_on_source,
                se_on_target,
                _target_count,
                timestamp,
                entity_tracker,
                status_tracker,
            );
            return;
        }

        let required_inspect_names = entity_tracker.get_required_bootstrap_player_names();
        let missing_inspects = required_inspect_names
            .iter()
            .filter(|name| !entity_tracker.has_inspect_snapshot_for_name(name))
            .cloned()
            .collect::<Vec<_>>();

        self.apply_or_gate_damage(
            dmg_src_entity,
            proj_entity,
            dmg_target_entity,
            damage_data,
            se_on_source,
            se_on_target,
            _target_count,
            hit_context,
            entity_tracker,
            status_tracker,
            inspect_transport_available,
            timestamp,
            required_inspect_names,
            missing_inspects,
            packet_seq,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_or_gate_damage(
        &mut self,
        dmg_src_entity: &Entity,
        proj_entity: &Entity,
        dmg_target_entity: &Entity,
        damage_data: DamageData,
        se_on_source: Vec<StatusEffectDetails>,
        se_on_target: Vec<StatusEffectDetails>,
        target_count: i32,
        hit_context: DamageHitContext,
        entity_tracker: &mut EntityTracker,
        status_tracker: &mut StatusTracker,
        inspect_transport_available: bool,
        timestamp: i64,
        required_inspect_names: Vec<String>,
        missing_inspects: Vec<String>,
        packet_seq: i64,
    ) {
        let should_open_startup_barrier =
            self.encounter.fight_start == 0 || self.rearm_startup_barrier_on_next_combat;
        if should_open_startup_barrier {
            let startup_barrier_eligible = self.should_open_startup_barrier_for_hit(
                dmg_src_entity.entity_type,
                dmg_target_entity.entity_type,
            );
            if startup_barrier_eligible {
                if inspect_transport_available {
                    if self.encounter.fight_start == 0 {
                        self.start_fight(
                            timestamp,
                            dmg_target_entity.entity_type,
                            damage_data.skill_id,
                            dmg_src_entity.id,
                        );
                    }
                    self.rearm_startup_barrier_on_next_combat = false;
                    self.open_startup_barrier(
                        entity_tracker,
                        required_inspect_names,
                        &missing_inspects,
                    );
                    self.enqueue_pending_damage(
                        packet_seq,
                        dmg_src_entity,
                        proj_entity,
                        dmg_target_entity,
                        damage_data,
                        hit_context,
                        se_on_source,
                        se_on_target,
                        target_count,
                        timestamp,
                        entity_tracker,
                        status_tracker,
                    );
                    return;
                }
                if !missing_inspects.is_empty() {
                    self.rdps_valid = false;
                    self.rdps_message = Some("inspect_unavailable".into());
                } else {
                    self.rdps_valid = true;
                    self.rdps_message = None;
                }
                self.rearm_startup_barrier_on_next_combat = false;
            }
        }

        self.apply_damage(
            dmg_src_entity,
            proj_entity,
            dmg_target_entity,
            damage_data,
            hit_context,
            se_on_source,
            se_on_target,
            target_count,
            entity_tracker,
            timestamp,
            None,
            None,
        );
    }

    fn get_valid_damage_hit(
        &self,
        dmg_src_entity: &Entity,
        dmg_target_entity: &Entity,
        damage_data: &DamageData,
    ) -> Option<DamageHitContext> {
        let hit_flag = match damage_data.modifier & 0xf {
            0 => HitFlag::NORMAL,
            1 => HitFlag::CRITICAL,
            2 => HitFlag::MISS,
            3 => HitFlag::INVINCIBLE,
            4 => HitFlag::DOT,
            5 => HitFlag::IMMUNE,
            6 => HitFlag::IMMUNE_SILENCED,
            7 => HitFlag::FONT_SILENCED,
            8 => HitFlag::DOT_CRITICAL,
            9 => HitFlag::DODGE,
            10 => HitFlag::REFLECT,
            11 => HitFlag::DAMAGE_SHARE,
            12 => HitFlag::DODGE_HIT,
            13 => HitFlag::MAX,
            _ => return None,
        };
        let hit_option_raw = ((damage_data.modifier >> 4) & 0x7) - 1;
        let hit_option = match hit_option_raw {
            -1 => HitOption::NONE,
            0 => HitOption::BACK_ATTACK,
            1 => HitOption::FRONTAL_ATTACK,
            2 => HitOption::FLANK_ATTACK,
            3 => HitOption::MAX,
            _ => return None,
        };

        if hit_flag == HitFlag::INVINCIBLE {
            return None;
        }
        if hit_flag == HitFlag::DAMAGE_SHARE
            && damage_data.skill_id == 0
            && damage_data.skill_effect_id == 0
        {
            return None;
        }
        if dmg_src_entity.name == dmg_target_entity.name {
            info!("ignoring self damage from {}", dmg_src_entity.name);
            return None;
        }

        Some(DamageHitContext {
            hit_flag,
            hit_option,
        })
    }

    fn should_open_startup_barrier_for_hit(
        &self,
        source_type: EntityType,
        target_type: EntityType,
    ) -> bool {
        !self.boss_only_damage
            || ((target_type == EntityType::Boss || target_type == EntityType::Player)
                && (target_type != EntityType::Player || source_type == EntityType::Boss))
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_damage(
        &mut self,
        dmg_src_entity: &Entity,
        proj_entity: &Entity,
        dmg_target_entity: &Entity,
        damage_data: DamageData,
        hit_context: DamageHitContext,
        se_on_source: Vec<StatusEffectDetails>,
        se_on_target: Vec<StatusEffectDetails>,
        _target_count: i32,
        entity_tracker: &EntityTracker,
        timestamp: i64,
        buffered_player_entities: Option<&HashMap<u64, Entity>>,
        buffered_owner_self_effects: Option<&HashMap<u64, Vec<StatusEffectDetails>>>,
    ) {
        let mut skill_effect_id = damage_data.skill_effect_id;
        let is_battle_item = is_battle_item(&proj_entity.skill_effect_id, "attack");
        if proj_entity.entity_type == EntityType::Projectile && is_battle_item {
            skill_effect_id = proj_entity.skill_effect_id;
        }

        // ensure source entity exists in encounter
        let source_entity = self
            .encounter
            .entities
            .entry(dmg_src_entity.name.clone())
            .or_insert_with(|| encounter_entity_from_entity(dmg_src_entity));
        let source_was_player = source_entity.entity_type == EntityType::Player;
        let previous_source_damage_dealt = source_entity.damage_stats.damage_dealt;
        Self::refresh_encounter_entity_metadata(source_entity, dmg_src_entity, None, None);
        let source_type = source_entity.entity_type;
        let source_promoted_to_player = !source_was_player
            && source_entity.entity_type == EntityType::Player
            && previous_source_damage_dealt > 0;

        let resolved_skill_id = resolve_skill_id(damage_data.skill_id, skill_effect_id);

        // get skill info here early for stagger tracking
        // since we can stagger mobs that are not bosses that would otherwise be ignored
        let mut skill_key = if is_battle_item {
            // pad battle item skill effect id to avoid overlap with skill ids
            skill_effect_id + 1_000_000
        } else if damage_data.skill_id == 0 {
            if resolved_skill_id == 0 {
                skill_effect_id
            } else {
                resolved_skill_id
            }
        } else {
            damage_data.skill_id
        };

        let (skill_name, skill_icon, skill_summon_sources, special, is_hyper_awakening) =
            get_skill_name_and_icon(
                damage_data.skill_id,
                skill_effect_id,
                &self.skill_tracker,
                source_entity.id,
            );

        let battle_item_name = skill_name.clone();

        if !source_entity.skills.contains_key(&skill_key) {
            if let Some(skill) = source_entity
                .skills
                .values()
                .find(|&s| s.name == skill_name)
            {
                skill_key = skill.id;
            } else {
                source_entity.skills.insert(
                    skill_key,
                    Skill {
                        id: skill_key,
                        name: skill_name,
                        icon: skill_icon,
                        casts: 1,
                        ..Default::default()
                    },
                );
                source_entity.skill_stats.casts += 1;
            }
        }

        // add stagger damage here
        source_entity
            .skills
            .entry(skill_key)
            .and_modify(|s| s.stagger += damage_data.stagger as i64);
        source_entity.damage_stats.stagger += damage_data.stagger as i64;

        let mut player_totals_stale = source_promoted_to_player;

        // ensure target entity exists in encounter
        let (target_type, target_promoted_to_player) = {
            let target_entity = self
                .encounter
                .entities
                .entry(dmg_target_entity.name.clone())
                .or_insert_with(|| {
                    let mut target_entity = encounter_entity_from_entity(dmg_target_entity);
                    target_entity.current_hp = damage_data.target_current_hp;
                    target_entity.max_hp = damage_data.target_max_hp;
                    target_entity
                });
            let was_player = target_entity.entity_type == EntityType::Player;
            let previous_damage_taken = target_entity.damage_stats.damage_taken;
            Self::refresh_encounter_entity_metadata(
                target_entity,
                dmg_target_entity,
                Some(damage_data.target_current_hp),
                Some(damage_data.target_max_hp),
            );
            (
                target_entity.entity_type,
                !was_player
                    && target_entity.entity_type == EntityType::Player
                    && previous_damage_taken > 0,
            )
        };
        player_totals_stale |= target_promoted_to_player;

        if player_totals_stale {
            self.refresh_encounter_player_damage_totals();
        }

        let DamageHitContext {
            hit_flag,
            hit_option,
        } = hit_context;

        // if boss only damage is enabled
        // check if target is boss and not player
        // check if target is player and source is boss
        if self.boss_only_damage
            && ((target_type != EntityType::Boss && target_type != EntityType::Player)
                || (target_type == EntityType::Player && source_type != EntityType::Boss))
        {
            return;
        }

        if self.encounter.fight_start == 0 {
            self.start_fight(timestamp, target_type, skill_key, dmg_src_entity.id);
        }

        self.encounter.last_combat_packet = timestamp;

        let mut damage = damage_data.damage + damage_data.shield_damage.unwrap_or(0);
        if target_type != EntityType::Player && damage_data.target_current_hp < 0 {
            damage += damage_data.target_current_hp;
        }

        let (can_crit, _) =
            resolve_skill_effect_flags(damage_data.skill_effect_id, is_hyper_awakening);
        let mut crit_metrics = None;
        let mut stat_damage_metrics = None;
        let mut rdps_result = None;
        if self.rdps_valid {
            let hit_analysis = analyze_hit_rdps(
                dmg_src_entity,
                dmg_target_entity,
                damage.max(0),
                damage_data.skill_id,
                resolved_skill_id,
                damage_data.skill_effect_id,
                &hit_option,
                &hit_flag,
                damage_data.damage_attribute,
                damage_data.damage_type,
                is_hyper_awakening,
                special,
                &se_on_source,
                &se_on_target,
                timestamp,
                entity_tracker,
                buffered_player_entities,
                buffered_owner_self_effects,
            );
            crit_metrics = hit_analysis.crit_metrics;
            stat_damage_metrics = hit_analysis.stat_damage_metrics;
            match hit_analysis.rdps {
                HitRdpsOutcome::Computed(result) => {
                    rdps_result = Some(result);
                }
                HitRdpsOutcome::NotApplicable(reason) => {
                    let _ = reason;
                }
                HitRdpsOutcome::Invalid(reason) => {
                    self.invalidate_rdps(reason);
                }
            }
        }

        let [Some(source_entity), Some(target_entity)] = self
            .encounter
            .entities
            .get_disjoint_mut([&dmg_src_entity.name, &dmg_target_entity.name])
        else {
            warn!(
                "{}, {} not found in encounter entities",
                dmg_src_entity.name, dmg_target_entity.name
            );
            return;
        };

        if is_battle_item && battle_item_name.contains("Dark") {
            debug_print(format_args!(
                "from: {}, hit: {}",
                source_entity.name, battle_item_name
            ))
        }

        source_entity.id = dmg_src_entity.id;

        if target_entity.id == dmg_target_entity.id {
            target_entity.current_hp = damage_data.target_current_hp;
            target_entity.max_hp = damage_data.target_max_hp;
        }

        let damage_apply_debug_before = if DEBUG_DUMP_DAMAGE_STATE_JSON
            && source_entity.entity_type == EntityType::Player
        {
            Some(json!({
                "source_entity": {
                    "name": source_entity.name,
                    "id": source_entity.id,
                    "character_id": source_entity.character_id,
                    "damage_stats": {
                        "damage_dealt": source_entity.damage_stats.damage_dealt,
                        "buffed_damage": source_entity.damage_stats.buffed_damage,
                        "unbuffed_damage": source_entity.damage_stats.unbuffed_damage,
                        "rdps_damage_received": source_entity.damage_stats.rdps_damage_received,
                        "rdps_damage_received_support": source_entity.damage_stats.rdps_damage_received_support,
                        "rdps_damage_given": source_entity.damage_stats.rdps_damage_given,
                    },
                },
                "skill": source_entity.skills.get(&skill_key).map(|skill| json!({
                    "skill_key": skill_key,
                    "id": skill.id,
                    "name": skill.name,
                    "total_damage": skill.total_damage,
                    "max_damage": skill.max_damage,
                    "hits": skill.hits,
                    "crits": skill.crits,
                    "rdps_received": skill.rdps_received,
                    "rdps_contributed": skill.rdps_contributed,
                })),
                "rdps_result": rdps_result.as_ref().map(|result| json!({
                    "rdps_damage_received": result.rdps_damage_received,
                    "rdps_damage_received_support": result.rdps_damage_received_support,
                    "entity_attributions": result.entity_attributions.iter().map(|attribution| json!({
                        "source_entity_id": attribution.source_entity_id,
                        "damage": attribution.damage,
                        "is_support": attribution.is_support,
                    })).collect::<Vec<_>>(),
                    "attributions": result.attributions.iter().map(|attribution| json!({
                        "rdps_type": attribution.rdps_type,
                        "source_entity_id": attribution.source_entity_id,
                        "source_skill_id": attribution.source_skill_id,
                        "damage": attribution.damage,
                        "is_support": attribution.is_support,
                    })).collect::<Vec<_>>(),
                })),
            }))
        } else {
            None
        };

        source_entity.damage_stats.damage_dealt += damage;
        if let Some(rdps_result) = rdps_result.as_ref() {
            source_entity.damage_stats.rdps_damage_received += rdps_result.rdps_damage_received;
            source_entity.damage_stats.rdps_damage_received_support +=
                rdps_result.rdps_damage_received_support;
        }
        Self::recompute_entity_udps_unbuffed(source_entity);

        let skill = source_entity.skills.get_mut(&skill_key).unwrap();
        skill.is_hyper_awakening = is_hyper_awakening;
        if special {
            skill.special = Some(true);
        }

        let relative_timestamp = (timestamp - self.encounter.fight_start) as i32;
        let mut skill_hit = SkillHit {
            damage,
            stagger: damage_data.stagger as i64,
            timestamp: relative_timestamp as i64,
            ..Default::default()
        };
        if let Some(rdps_result) = rdps_result.as_ref() {
            skill.rdps_damage_received += rdps_result.rdps_damage_received;
            skill.rdps_damage_received_support += rdps_result.rdps_damage_received_support;

            skill_hit.rdps_damage_received = rdps_result.rdps_damage_received;
            skill_hit.rdps_damage_received_support = rdps_result.rdps_damage_received_support;
        }

        skill.total_damage += damage;
        if damage > skill.max_damage {
            skill.max_damage = damage;
        }
        skill.last_timestamp = timestamp;

        if is_hyper_awakening {
            source_entity.damage_stats.hyper_awakening_damage += damage;
        }

        let contribution_data = if source_entity.entity_type == EntityType::Player {
            Some((
                source_entity.name.clone(),
                dmg_src_entity.id,
                resolved_skill_id,
                damage,
                can_crit,
                hit_flag == HitFlag::CRITICAL || hit_flag == HitFlag::DOT_CRITICAL,
                crit_metrics,
                stat_damage_metrics,
                rdps_result.clone(),
                self.rdps_valid,
            ))
        } else {
            None
        };

        target_entity.damage_stats.damage_taken += damage;

        source_entity.skill_stats.hits += 1;
        skill.hits += 1;
        if hit_flag == HitFlag::CRITICAL || hit_flag == HitFlag::DOT_CRITICAL {
            source_entity.skill_stats.crits += 1;
            source_entity.damage_stats.crit_damage += damage;
            skill.crits += 1;
            skill.crit_damage += damage;
            skill_hit.crit = true;
        }
        if hit_option == HitOption::BACK_ATTACK {
            source_entity.skill_stats.back_attacks += 1;
            source_entity.damage_stats.back_attack_damage += damage;
            skill.back_attacks += 1;
            skill.back_attack_damage += damage;
            skill_hit.back_attack = true;
        }
        if hit_option == HitOption::FRONTAL_ATTACK {
            source_entity.skill_stats.front_attacks += 1;
            source_entity.damage_stats.front_attack_damage += damage;
            skill.front_attacks += 1;
            skill.front_attack_damage += damage;
            skill_hit.front_attack = true;
        }

        if let Some(debug_before) = damage_apply_debug_before.as_ref() {
            write_debug_json_dump(
                "damage-apply-hit",
                &format!("{}-{}-{}", source_entity.name, timestamp, skill_key),
                &json!({
                    "context": {
                        "timestamp": timestamp,
                        "damage": damage,
                        "skill_key": skill_key,
                        "resolved_skill_id": resolved_skill_id,
                        "rdps_valid": self.rdps_valid,
                        "hit_option": format!("{hit_option:?}"),
                        "hit_flag": format!("{hit_flag:?}"),
                    },
                    "before": debug_before,
                    "after": {
                        "source_entity": {
                            "name": source_entity.name,
                            "id": source_entity.id,
                            "character_id": source_entity.character_id,
                            "damage_stats": {
                                "damage_dealt": source_entity.damage_stats.damage_dealt,
                                "buffed_damage": source_entity.damage_stats.buffed_damage,
                                "unbuffed_damage": source_entity.damage_stats.unbuffed_damage,
                                "rdps_damage_received": source_entity.damage_stats.rdps_damage_received,
                                "rdps_damage_received_support": source_entity.damage_stats.rdps_damage_received_support,
                                "rdps_damage_given": source_entity.damage_stats.rdps_damage_given,
                                "crit_damage": source_entity.damage_stats.crit_damage,
                                "back_attack_damage": source_entity.damage_stats.back_attack_damage,
                                "front_attack_damage": source_entity.damage_stats.front_attack_damage,
                            },
                        },
                        "skill": {
                            "skill_key": skill_key,
                            "id": skill.id,
                            "name": skill.name,
                            "total_damage": skill.total_damage,
                            "max_damage": skill.max_damage,
                            "hits": skill.hits,
                            "crits": skill.crits,
                            "back_attacks": skill.back_attacks,
                            "front_attacks": skill.front_attacks,
                            "rdps_received": skill.rdps_received,
                            "rdps_contributed": skill.rdps_contributed,
                        },
                        "skill_hit": {
                            "damage": skill_hit.damage,
                            "rdps_damage_received": skill_hit.rdps_damage_received,
                            "rdps_damage_received_support": skill_hit.rdps_damage_received_support,
                            "crit": skill_hit.crit,
                            "back_attack": skill_hit.back_attack,
                            "front_attack": skill_hit.front_attack,
                        },
                    },
                }),
            );
        }

        let unresolved_damage_log_source = (source_entity.entity_type == EntityType::Unknown
            && (source_entity.class_id > 0
                || self.source_owner_aliases.contains_key(&source_entity.id)
                || self
                    .resolved_source_owner_aliases
                    .contains_key(&source_entity.id)))
        .then(|| source_entity.name.clone());
        if let Some(source_name) = unresolved_damage_log_source {
            self.damage_log
                .entry(source_name)
                .or_default()
                .push((timestamp, damage));
        }

        if source_entity.entity_type == EntityType::Player {
            self.encounter.encounter_damage_stats.total_damage_dealt += damage;
            self.encounter.encounter_damage_stats.top_damage_dealt = max(
                self.encounter.encounter_damage_stats.top_damage_dealt,
                source_entity.damage_stats.damage_dealt,
            );

            self.damage_log
                .entry(source_entity.name.clone())
                .or_default()
                .push((timestamp, damage));

            let mut is_buffed_by_support = false;
            let mut is_buffed_by_identity = false;
            let mut is_debuffed_by_support = false;
            let mut is_buffed_by_hat = false;

            if !special {
                let se_on_source_ids = se_on_source
                    .iter()
                    .map(|se| map_status_effect(se, &mut self.custom_id_map))
                    .collect::<Vec<_>>();
                for buff_id in se_on_source_ids.iter() {
                    if !self
                        .encounter
                        .encounter_damage_stats
                        .unknown_buffs
                        .contains(buff_id)
                        && !self
                            .encounter
                            .encounter_damage_stats
                            .buffs
                            .contains_key(buff_id)
                    {
                        let mut source_id: Option<u32> = None;
                        let original_buff_id =
                            if let Some(deref_id) = self.custom_id_map.get(buff_id) {
                                source_id = Some(get_skill_id(*buff_id, *deref_id));
                                *deref_id
                            } else {
                                *buff_id
                            };

                        if let Some(status_effect) =
                            get_status_effect_data(original_buff_id, source_id)
                        {
                            self.encounter
                                .encounter_damage_stats
                                .buffs
                                .insert(*buff_id, status_effect);
                        } else {
                            self.encounter
                                .encounter_damage_stats
                                .unknown_buffs
                                .insert(*buff_id);
                        }
                    }

                    // will count dps spec of supports as support buffs until proper spec is determined
                    let hat = is_hat_buff(buff_id) || is_hyper_hat_buff(buff_id);
                    if ((!is_buffed_by_support && !hat) || !is_buffed_by_identity)
                        && let Some(buff) = self.encounter.encounter_damage_stats.buffs.get(buff_id)
                    {
                        if !is_buffed_by_support
                            && !hat
                            && buff.buff_type & StatusEffectBuffTypeFlags::DMG.bits() != 0
                            && buff.buff_category == "supportbuff"
                            && SUPPORT_AP_GROUP.contains(&buff.unique_group)
                        {
                            is_buffed_by_support = true;
                        }

                        if !is_buffed_by_identity
                            && buff.buff_type & StatusEffectBuffTypeFlags::DMG.bits() != 0
                            && buff.buff_category == "supportbuff"
                            && SUPPORT_IDENTITY_GROUP.contains(&buff.unique_group)
                        {
                            is_buffed_by_identity = true;
                        }
                    }

                    // T skill has two buffs, one buffs hyper awakening damage, one buffs all other skill damage
                    // if normal skill, check if normal buff
                    // if hyper awakening, check if hyper buff
                    if !is_buffed_by_hat
                        && ((is_hat_buff(buff_id) && !is_hyper_awakening)
                            || (is_hyper_hat_buff(buff_id) && is_hyper_awakening))
                    {
                        is_buffed_by_hat = true;
                    }
                }
                let filtered_se_on_target = filter_target_effects_for_attacker(
                    dmg_src_entity,
                    &se_on_target,
                    entity_tracker,
                    buffered_player_entities,
                );
                let se_on_target_ids = filtered_se_on_target
                    .iter()
                    .map(|se| map_status_effect(se, &mut self.custom_id_map))
                    .collect::<Vec<_>>();
                for debuff_id in se_on_target_ids.iter() {
                    if !self
                        .encounter
                        .encounter_damage_stats
                        .unknown_buffs
                        .contains(debuff_id)
                        && !self
                            .encounter
                            .encounter_damage_stats
                            .debuffs
                            .contains_key(debuff_id)
                    {
                        let mut source_id: Option<u32> = None;
                        let original_debuff_id =
                            if let Some(deref_id) = self.custom_id_map.get(debuff_id) {
                                source_id = Some(get_skill_id(*debuff_id, *deref_id));
                                *deref_id
                            } else {
                                *debuff_id
                            };

                        if let Some(status_effect) =
                            get_status_effect_data(original_debuff_id, source_id)
                        {
                            self.encounter
                                .encounter_damage_stats
                                .debuffs
                                .insert(*debuff_id, status_effect);
                        } else {
                            self.encounter
                                .encounter_damage_stats
                                .unknown_buffs
                                .insert(*debuff_id);
                        }
                    }
                    if !is_debuffed_by_support
                        && let Some(debuff) =
                            self.encounter.encounter_damage_stats.debuffs.get(debuff_id)
                    {
                        is_debuffed_by_support = debuff.unique_group == 210230 // brand group
                                && debuff.buff_type & StatusEffectBuffTypeFlags::DMG.bits() != 0
                                && debuff.target == StatusEffectTarget::PARTY;
                    }
                }

                if is_buffed_by_support && !is_hyper_awakening {
                    skill.buffed_by_support += damage;
                    source_entity.damage_stats.buffed_by_support += damage;
                }
                if is_buffed_by_identity && !is_hyper_awakening {
                    skill.buffed_by_identity += damage;
                    source_entity.damage_stats.buffed_by_identity += damage;
                }
                if is_debuffed_by_support && !is_hyper_awakening {
                    skill.debuffed_by_support += damage;
                    source_entity.damage_stats.debuffed_by_support += damage;
                }
                if is_buffed_by_hat {
                    skill.buffed_by_hat += damage;
                    source_entity.damage_stats.buffed_by_hat += damage;
                }

                let stabilized_status_active =
                    (source_entity.current_hp as f64 / source_entity.max_hp as f64) > 0.65;
                let mut filtered_se_on_source_ids: Vec<u32> = vec![];

                for buff_id in se_on_source_ids.iter() {
                    // hyper only affected by hat buff
                    if is_hyper_awakening && !is_hyper_hat_buff(buff_id) {
                        continue;
                    } else if let Some(buff) =
                        self.encounter.encounter_damage_stats.buffs.get(buff_id)
                        && !stabilized_status_active
                        && buff.source.name.contains("Stabilized Status")
                    {
                        continue;
                    }

                    filtered_se_on_source_ids.push(*buff_id);

                    skill
                        .buffed_by
                        .entry(*buff_id)
                        .and_modify(|e| *e += damage)
                        .or_insert(damage);
                    source_entity
                        .damage_stats
                        .buffed_by
                        .entry(*buff_id)
                        .and_modify(|e| *e += damage)
                        .or_insert(damage);
                }
                for debuff_id in se_on_target_ids.iter() {
                    if is_hyper_awakening {
                        break;
                    }

                    skill
                        .debuffed_by
                        .entry(*debuff_id)
                        .and_modify(|e| *e += damage)
                        .or_insert(damage);
                    source_entity
                        .damage_stats
                        .debuffed_by
                        .entry(*debuff_id)
                        .and_modify(|e| *e += damage)
                        .or_insert(damage);
                }

                skill_hit.buffed_by = filtered_se_on_source_ids;
                // no debuffs affect hyper
                if !is_hyper_awakening {
                    skill_hit.debuffed_by = se_on_target_ids;
                }
            }
        }

        if target_entity.entity_type == EntityType::Player {
            self.encounter.encounter_damage_stats.total_damage_taken += damage;
            self.encounter.encounter_damage_stats.top_damage_taken = max(
                self.encounter.encounter_damage_stats.top_damage_taken,
                target_entity.damage_stats.damage_taken,
            );
        }
        // update current_boss
        else if target_entity.entity_type == EntityType::Boss {
            self.encounter
                .current_boss_name
                .clone_from(&target_entity.name);
            target_entity.id = dmg_target_entity.id;
            target_entity.npc_id = dmg_target_entity.npc_id;
            target_entity.hp_bars = dmg_target_entity.hp_bars;

            let log = self
                .boss_hp_log
                .entry(target_entity.name.clone())
                .or_default();

            let current_hp = if target_entity.current_hp >= 0 {
                target_entity.current_hp + target_entity.current_shield as i64
            } else {
                0
            };
            let hp_percent = if target_entity.max_hp != 0 {
                current_hp as f32 / target_entity.max_hp as f32
            } else {
                0.0
            };

            let relative_timestamp_s = relative_timestamp / 1000;

            if log.is_empty() || log.last().unwrap().time != relative_timestamp_s {
                log.push(BossHpLog::new(relative_timestamp_s, current_hp, hp_percent));
            } else {
                let last = log.last_mut().unwrap();
                last.hp = current_hp;
                last.p = hp_percent;
            }
        }

        if skill_key > 0 {
            self.skill_tracker.on_hit(
                source_entity.id,
                proj_entity.id,
                skill_key,
                skill_hit,
                skill_summon_sources,
            );
        }

        if let Some(rdps_result) = rdps_result {
            for attribution in rdps_result.entity_attributions {
                if attribution.source_entity_id == DARK_GRENADE_ENTITY_ID {
                    let contributor_entity = self.ensure_dark_grenade_entity();
                    contributor_entity.damage_stats.rdps_damage_given += attribution.damage;
                    continue;
                }
                let Some(contributor_name) = entity_tracker
                    .get_entity_ref(attribution.source_entity_id)
                    .map(|entity| entity.name.clone())
                else {
                    continue;
                };
                if contributor_name == dmg_src_entity.name {
                    continue;
                }
                if let Some(contributor_entity) = self.encounter.entities.get_mut(&contributor_name)
                {
                    contributor_entity.damage_stats.rdps_damage_given += attribution.damage;
                }
            }
        }

        if let Some((
            player_name,
            player_entity_id,
            skill_id,
            damage,
            can_crit,
            hit_flag,
            crit_metrics,
            stat_damage_metrics,
            rdps_result,
            rdps_valid,
        )) = contribution_data
        {
            self.record_contribution_data(
                &player_name,
                player_entity_id,
                skill_id,
                damage,
                can_crit,
                hit_flag,
                crit_metrics.as_ref(),
                stat_damage_metrics.as_ref(),
                rdps_result.as_ref(),
                rdps_valid,
            );
        }
    }

    pub fn on_support_combat_analyzer_data(
        &mut self,
        events: Vec<CombatAnalyzerEntry>,
        entity_tracker: &EntityTracker,
    ) {
        for event in events {
            let mut debug_dump = if DEBUG_DUMP_DAMAGE_STATE_JSON {
                Some(json!({
                    "event": {
                        "support_character_id": event.support_character_id,
                        "skill_id": event.skill_id,
                        "source_id": event.source_id,
                        "target_id": event.target_id,
                        "value": event.value,
                        "event_type": event.event_type,
                    },
                    "rdps_valid": self.rdps_valid,
                }))
            } else {
                None
            };
            // find the source entity by source_id
            let source_name = if let Some(entity) = entity_tracker.entities.get(&event.source_id) {
                entity.name.clone()
            } else {
                if let Some(debug_dump) = debug_dump.as_mut() {
                    debug_dump["source_lookup"] = json!({
                        "resolved": false,
                    });
                    write_debug_json_dump(
                        "support-combat-event",
                        &format!("missing-source-{}-{}", event.source_id, event.skill_id),
                        debug_dump,
                    );
                }
                continue;
            };

            // find the support (contributor) entity by support_character_id
            let contributor_name = if let Some(name) = entity_tracker
                .character_id_to_name
                .get(&event.support_character_id)
            {
                name.clone()
            } else if let Some(entity) = self
                .encounter
                .entities
                .values()
                .find(|e| e.character_id == event.support_character_id)
            {
                entity.name.clone()
            } else {
                if let Some(debug_dump) = debug_dump.as_mut() {
                    debug_dump["source_lookup"] = json!({
                        "resolved": true,
                        "source_name": source_name,
                    });
                    debug_dump["contributor_lookup"] = json!({
                        "resolved": false,
                    });
                    write_debug_json_dump(
                        "support-combat-event",
                        &format!(
                            "missing-contributor-{}-{}-{}",
                            source_name, event.skill_id, event.support_character_id
                        ),
                        debug_dump,
                    );
                }
                continue;
            };

            if let Some(debug_dump) = debug_dump.as_mut() {
                debug_dump["source_lookup"] = json!({
                    "resolved": true,
                    "source_name": source_name,
                    "source_entity": self.encounter.entities.get(&source_name).map(|entity| json!({
                        "id": entity.id,
                        "character_id": entity.character_id,
                        "damage_stats": {
                            "damage_dealt": entity.damage_stats.damage_dealt,
                            "buffed_damage": entity.damage_stats.buffed_damage,
                            "unbuffed_damage": entity.damage_stats.unbuffed_damage,
                            "rdps_damage_received": entity.damage_stats.rdps_damage_received,
                            "rdps_damage_given": entity.damage_stats.rdps_damage_given,
                        },
                    })),
                });
                debug_dump["contributor_lookup"] = json!({
                    "resolved": true,
                    "contributor_name": contributor_name,
                    "contributor_entity": self.encounter.entities.get(&contributor_name).map(|entity| json!({
                        "id": entity.id,
                        "character_id": entity.character_id,
                        "skill_has_direct_id": entity.skills.contains_key(&event.skill_id),
                        "skills_matching_name": SKILL_DATA.get(&event.skill_id)
                            .and_then(|skill_data| skill_data.name.clone())
                            .map(|skill_name| {
                                entity.skills
                                    .iter()
                                    .filter(|(_, skill)| skill.name == skill_name)
                                    .map(|(skill_id, skill)| json!({
                                        "skill_id": skill_id,
                                        "name": skill.name,
                                        "rdps_contributed": skill.rdps_contributed,
                                    }))
                                    .collect::<Vec<_>>()
                            }),
                    })),
                });
            }

            // add rdps_contributed to the support's skill
            if let Some(contributor_entity) = self.encounter.entities.get_mut(&contributor_name) {
                if let Some(contributor_skill) = contributor_entity.skills.get_mut(&event.skill_id)
                {
                    *contributor_skill
                        .rdps_contributed
                        .entry(event.event_type)
                        .or_default() += event.value;
                } else if let Some(skill_data) = SKILL_DATA.get(&event.skill_id)
                    && let Some(skill_name) = skill_data.name.clone()
                    && let Some(contributor_skill) = contributor_entity
                        .skills
                        .values_mut()
                        .find(|s| s.name == skill_name)
                {
                    *contributor_skill
                        .rdps_contributed
                        .entry(event.event_type)
                        .or_default() += event.value;
                }
            }

            // only track at entity level, can't reliably attribute to a specific skill
            if let Some(source_entity) = self.encounter.entities.get_mut(&source_name) {
                if matches!(event.event_type, 1 | 3 | 5) {
                    source_entity.damage_stats.buffed_damage += event.value;
                }
                source_entity.damage_stats.unbuffed_damage =
                    source_entity.damage_stats.damage_dealt
                        - source_entity.damage_stats.buffed_damage;
            }

            if let Some(debug_dump) = debug_dump.as_mut() {
                debug_dump["udps_resolution"] = json!({
                    "event_counts_as_damage_given": matches!(event.event_type, 1 | 3 | 5),
                });
                debug_dump["source_after"] = json!({
                    "source_name": source_name,
                    "source_entity": self.encounter.entities.get(&source_name).map(|entity| json!({
                        "id": entity.id,
                        "character_id": entity.character_id,
                        "damage_stats": {
                            "damage_dealt": entity.damage_stats.damage_dealt,
                            "buffed_damage": entity.damage_stats.buffed_damage,
                            "unbuffed_damage": entity.damage_stats.unbuffed_damage,
                            "rdps_damage_received": entity.damage_stats.rdps_damage_received,
                            "rdps_damage_given": entity.damage_stats.rdps_damage_given,
                        },
                    })),
                });
                debug_dump["contributor_after"] = json!({
                    "contributor_name": contributor_name,
                    "contributor_entity": self.encounter.entities.get(&contributor_name).map(|entity| json!({
                        "id": entity.id,
                        "character_id": entity.character_id,
                        "skills": entity.skills.iter()
                            .filter(|(skill_id, skill)| {
                                **skill_id == event.skill_id
                                    || SKILL_DATA
                                        .get(&event.skill_id)
                                        .and_then(|skill_data| skill_data.name.clone())
                                        .is_some_and(|skill_name| skill.name == skill_name)
                            })
                            .map(|(skill_id, skill)| json!({
                                "skill_id": skill_id,
                                "name": skill.name,
                                "rdps_contributed": skill.rdps_contributed,
                            }))
                            .collect::<Vec<_>>(),
                    })),
                });
                write_debug_json_dump(
                    "support-combat-event",
                    &format!(
                        "{}-{}-{}-{}",
                        source_name, contributor_name, event.skill_id, event.event_type
                    ),
                    debug_dump,
                );
            }
        }
    }

    pub fn on_counterattack(&mut self, source_entity: &Entity) {
        let entity = self
            .encounter
            .entities
            .entry(source_entity.name.clone())
            .or_insert_with(|| {
                let mut entity = encounter_entity_from_entity(source_entity);
                entity.skill_stats = SkillStats {
                    counters: 0,
                    ..Default::default()
                };
                entity
            });
        entity.skill_stats.counters += 1;
    }

    pub fn on_abnormal_move(
        &mut self,
        victim_entity: &Entity,
        movement: &SkillMoveOptionData,
        timestamp: i64,
    ) {
        if victim_entity.entity_type != EntityType::Player {
            // we don't care about npc knockups
            return;
        }

        // only count movement events that would result in a knockup
        let Some(down_time) = movement.down_time else {
            return;
        };

        // todo: unclear if this is fully correct. It's hard to debug this, but it seems roughly accurate
        // if this is not accurate, we should probably factor out the stand_up_time and instead add in the
        // animation duration of the standup action for each class (seems to be 0.9s)
        let total_incapacitated_time = down_time
            + movement.move_time.unwrap_or_default()
            + movement.stand_up_time.unwrap_or_default();
        let incapacitated_time_ms = (total_incapacitated_time * 1000.0) as i64;

        let victim_entity_state = self
            .encounter
            .entities
            .entry(victim_entity.name.clone())
            .or_insert_with(|| encounter_entity_from_entity(victim_entity));

        // see if we have a previous incapacitation event that is still in effect (i.e. the player was knocked up again before
        // they could stand up), in which case we should shorten the previous event duration to the current timestamp
        let prev_incapacitation = victim_entity_state
            .damage_stats
            .incapacitations
            .iter_mut()
            .rev()
            .take_while(|e| e.timestamp + e.duration > timestamp) // stop as soon as we only hit expired events
            .find(|x| x.event_type == IncapacitationEventType::FALL_DOWN); // find an unexpired one that was caused by an abnormal move
        if let Some(prev_incapacitation) = prev_incapacitation {
            info!(
                "Shortening down duration from {} to {} because of new abnormal move",
                prev_incapacitation.duration,
                timestamp - prev_incapacitation.timestamp
            );
            prev_incapacitation.duration = timestamp - prev_incapacitation.timestamp;
        }

        let new_event = IncapacitatedEvent {
            timestamp,
            duration: incapacitated_time_ms,
            event_type: IncapacitationEventType::FALL_DOWN,
        };
        victim_entity_state
            .damage_stats
            .incapacitations
            .push(new_event);
        info!(
            "Player {} will be incapacitated for {}ms",
            victim_entity_state.name, incapacitated_time_ms
        );
    }

    pub fn on_cc_applied(&mut self, victim_entity: &Entity, status_effect: &StatusEffectDetails) {
        let victim_entity_state = self
            .encounter
            .entities
            .entry(victim_entity.name.clone())
            .or_insert_with(|| encounter_entity_from_entity(victim_entity));

        // expiration delay is zero or negative for infinite effects. Instead of applying them now,
        // only apply them after they've been removed (this avoids an issue where if we miss the removal
        // we end up applying a very long incapacitation)
        if status_effect_is_infinite(status_effect) {
            return;
        }

        let duration_ms = status_effect.expiration_delay * 1000.0;
        let new_event = IncapacitatedEvent {
            timestamp: status_effect.timestamp.timestamp_millis(),
            duration: duration_ms as i64,
            event_type: IncapacitationEventType::CROWD_CONTROL,
        };
        info!(
            "Player {} will be status-effect incapacitated for {}ms by buff {}",
            victim_entity_state.name, duration_ms, status_effect.status_effect_id
        );
        victim_entity_state
            .damage_stats
            .incapacitations
            .push(new_event);
    }

    pub fn on_cc_removed(
        &mut self,
        victim_entity: &Entity,
        status_effect: &StatusEffectDetails,
        timestamp: i64,
    ) {
        let victim_entity_state = self
            .encounter
            .entities
            .entry(victim_entity.name.clone())
            .or_insert_with(|| encounter_entity_from_entity(victim_entity));

        if status_effect_is_infinite(status_effect) {
            // this status effect was infinite, meaning we didn't apply it on_cc_applied
            // apply it now retroactively, then sort the events to ensure that our sorted
            // invariant does not get violated
            let duration_ms = timestamp - status_effect.timestamp.timestamp_millis();
            let new_event = IncapacitatedEvent {
                timestamp: status_effect.timestamp.timestamp_millis(),
                duration: duration_ms,
                event_type: IncapacitationEventType::CROWD_CONTROL,
            };
            info!(
                "Player {} was incapacitated by an infinite status effect buff for {}ms",
                victim_entity_state.name, duration_ms
            );
            victim_entity_state
                .damage_stats
                .incapacitations
                .push(new_event);
            victim_entity_state
                .damage_stats
                .incapacitations
                .sort_by_key(|x| x.timestamp);
            return;
        }

        // we use the application timestamp as the key. Attempt to find all buff instances that started
        // at this time and cap their duration to the current timestamp
        for event in victim_entity_state
            .damage_stats
            .incapacitations
            .iter_mut()
            .rev()
            .take_while(|e| e.timestamp + e.duration > timestamp)
        {
            if event.event_type == IncapacitationEventType::CROWD_CONTROL
                && event.timestamp == status_effect.timestamp.timestamp_millis()
            {
                info!(
                    "Removing status-effect {} incapacitation for player {} (shortened {}ms to {}ms)",
                    status_effect.status_effect_id,
                    victim_entity_state.name,
                    event.duration,
                    timestamp - event.timestamp
                );
                event.duration = timestamp - event.timestamp;
            }
        }
    }

    // pub fn on_identity_gain(&mut self, pkt: &PKTIdentityGaugeChangeNotify) {
    //     if self.encounter.fight_start == 0 {
    //         return;
    //     }
    //
    //     if self.encounter.local_player.is_empty() {
    //         if let Some((_, entity)) = self
    //             .encounter
    //             .entities
    //             .iter()
    //             .find(|(_, e)| e.id == pkt.player_id)
    //         {
    //             self.encounter.local_player.clone_from(&entity.name);
    //         } else {
    //             return;
    //         }
    //     }
    //
    //     if let Some(entity) = self
    //         .encounter
    //         .entities
    //         .get_mut(&self.encounter.local_player)
    //     {
    //         self.identity_log
    //             .entry(entity.name.clone())
    //             .or_default()
    //             .push((
    //                 Utc::now().timestamp_millis(),
    //                 (
    //                     pkt.identity_gauge1,
    //                     pkt.identity_gauge2,
    //                     pkt.identity_gauge3,
    //                 ),
    //             ));
    //     }
    // }

    // pub fn on_stagger_change(&mut self, pkt: &PKTParalyzationStateNotify) {
    //     if self.encounter.current_boss_name.is_empty() || self.encounter.fight_start == 0 {
    //         return;
    //     }

    //     if let Some(boss) = self
    //         .encounter
    //         .entities
    //         .get_mut(&self.encounter.current_boss_name)
    //     {
    //         let timestamp = Utc::now().timestamp_millis();
    //         let current_stagger = pkt.paralyzation_point as i32;
    //         let max_stagger = pkt.paralyzation_max_point as i32;
    //         if boss.id == pkt.object_id {
    //             if current_stagger == max_stagger {
    //                 let staggered_in =
    //                     (timestamp - self.encounter.encounter_damage_stats.stagger_start) / 1000;
    //                 self.stagger_intervals
    //                     .push((staggered_in as i32, max_stagger))
    //             } else if current_stagger != 0 && self.prev_stagger == 0 {
    //                 self.encounter.encounter_damage_stats.stagger_start = timestamp;
    //             }

    //             self.prev_stagger = current_stagger;

    //             let relative_timestamp_s = ((timestamp - self.encounter.fight_start) / 1000) as i32;
    //             let stagger_percent = (1.0 - (current_stagger as f32 / max_stagger as f32)) * 100.0;
    //             if let Some(last) = self.stagger_log.last_mut() {
    //                 if last.0 == relative_timestamp_s {
    //                     last.1 = stagger_percent;
    //                 } else {
    //                     self.stagger_log
    //                         .push((relative_timestamp_s, stagger_percent));
    //                 }
    //             } else {
    //                 self.stagger_log
    //                     .push((relative_timestamp_s, stagger_percent));
    //             }

    //             if max_stagger > self.encounter.encounter_damage_stats.max_stagger {
    //                 self.encounter.encounter_damage_stats.max_stagger = max_stagger;
    //             }
    //         }
    //     }
    // }

    pub fn on_boss_shield(&mut self, target_entity: &Entity, shield: u64) {
        if target_entity.entity_type == EntityType::Boss
            && target_entity.name == self.encounter.current_boss_name
        {
            self.encounter
                .entities
                .entry(target_entity.name.clone())
                .and_modify(|e| {
                    e.current_shield = shield;
                });
        }
    }

    pub fn on_shield_applied(
        &mut self,
        source_entity: &Entity,
        target_entity: &Entity,
        buff_id: u32,
        shield: u64,
    ) {
        if source_entity.entity_type == EntityType::Player
            && target_entity.entity_type == EntityType::Player
        {
            if !self
                .encounter
                .encounter_damage_stats
                .applied_shield_buffs
                .contains_key(&buff_id)
            {
                let mut source_id: Option<u32> = None;
                let original_buff_id = if let Some(deref_id) = self.custom_id_map.get(&buff_id) {
                    source_id = Some(get_skill_id(buff_id, *deref_id));
                    *deref_id
                } else {
                    buff_id
                };

                if let Some(status_effect) = get_status_effect_data(original_buff_id, source_id) {
                    self.encounter
                        .encounter_damage_stats
                        .applied_shield_buffs
                        .insert(buff_id, status_effect);
                }
            }

            self.encounter.encounter_damage_stats.total_shielding += shield;

            let source_entity_state = self
                .encounter
                .entities
                .entry(source_entity.name.clone())
                .or_insert_with(|| encounter_entity_from_entity(source_entity));

            // shields on self
            if source_entity.id == target_entity.id || source_entity.name == target_entity.name {
                source_entity_state.damage_stats.shields_received += shield;
                source_entity_state.damage_stats.shields_given += shield;
                source_entity_state
                    .damage_stats
                    .shields_given_by
                    .entry(buff_id)
                    .and_modify(|e| *e += shield)
                    .or_insert(shield);
                source_entity_state
                    .damage_stats
                    .shields_received_by
                    .entry(buff_id)
                    .and_modify(|e| *e += shield)
                    .or_insert(shield);

                return;
            }

            // shields on others
            self.encounter
                .entities
                .entry(target_entity.name.clone())
                .or_insert_with(|| encounter_entity_from_entity(target_entity));

            let [Some(source_entity_state), Some(target_entity_state)] = self
                .encounter
                .entities
                .get_disjoint_mut([&source_entity.name, &target_entity.name])
            else {
                warn!(
                    "{}, {} not found in encounter entities",
                    source_entity.name, target_entity.name
                );
                return;
            };

            target_entity_state.damage_stats.shields_received += shield;
            source_entity_state.damage_stats.shields_given += shield;
            source_entity_state
                .damage_stats
                .shields_given_by
                .entry(buff_id)
                .and_modify(|e| *e += shield)
                .or_insert(shield);
            target_entity_state
                .damage_stats
                .shields_received_by
                .entry(buff_id)
                .and_modify(|e| *e += shield)
                .or_insert(shield);
        }
    }

    pub fn on_shield_used(
        &mut self,
        source_entity: &Entity,
        target_entity: &Entity,
        buff_id: u32,
        shield_removed: u64,
    ) {
        if source_entity.entity_type == EntityType::Player
            && target_entity.entity_type == EntityType::Player
        {
            self.encounter
                .encounter_damage_stats
                .total_effective_shielding += shield_removed;

            let source_entity_state = self
                .encounter
                .entities
                .entry(source_entity.name.clone())
                .or_insert_with(|| encounter_entity_from_entity(source_entity));

            // shields on self
            if source_entity.id == target_entity.id || source_entity.name == target_entity.name {
                source_entity_state.damage_stats.damage_absorbed += shield_removed;
                source_entity_state.damage_stats.damage_absorbed_on_others += shield_removed;
                source_entity_state
                    .damage_stats
                    .damage_absorbed_by
                    .entry(buff_id)
                    .and_modify(|e| *e += shield_removed)
                    .or_insert(shield_removed);
                source_entity_state
                    .damage_stats
                    .damage_absorbed_on_others_by
                    .entry(buff_id)
                    .and_modify(|e| *e += shield_removed)
                    .or_insert(shield_removed);

                return;
            }

            // shields on others
            self.encounter
                .entities
                .entry(target_entity.name.clone())
                .or_insert_with(|| encounter_entity_from_entity(target_entity));

            let [Some(source_entity_state), Some(target_entity_state)] = self
                .encounter
                .entities
                .get_disjoint_mut([&source_entity.name, &target_entity.name])
            else {
                warn!(
                    "{}, {} not found in encounter entities",
                    source_entity.name, target_entity.name
                );
                return;
            };

            target_entity_state.damage_stats.damage_absorbed += shield_removed;
            source_entity_state.damage_stats.damage_absorbed_on_others += shield_removed;
            target_entity_state
                .damage_stats
                .damage_absorbed_by
                .entry(buff_id)
                .and_modify(|e| *e += shield_removed)
                .or_insert(shield_removed);
            source_entity_state
                .damage_stats
                .damage_absorbed_on_others_by
                .entry(buff_id)
                .and_modify(|e| *e += shield_removed)
                .or_insert(shield_removed);
        }
    }

    // track battle items used in an encounter
    pub fn on_battle_item_use(&mut self, battle_item_id: &u32) {
        if self.encounter.fight_start == 0 {
            return;
        }
    }

    pub fn save_to_db(&mut self, manual: bool) {
        if self.disabled {
            return;
        }
        if !manual
            && (self.encounter.fight_start == 0
                || self.encounter.current_boss_name.is_empty()
                || !self
                    .encounter
                    .entities
                    .contains_key(&self.encounter.current_boss_name)
                || !self.encounter.entities.values().any(|e| {
                    is_confirmed_player_entity(e, &self.encounter.local_player)
                        && e.damage_stats.damage_dealt > 0
                }))
        {
            info!("not saving to db, no players with damage dealt");
            return;
        }

        if !self.damage_is_valid {
            warn!("damage decryption is invalid, not saving to db");
        }

        self.refresh_encounter_player_damage_totals();
        let mut encounter = self.encounter.clone();

        let damage_log = self.damage_log.clone();
        let cast_log = self.cast_log.clone();
        let boss_hp_log = self.boss_hp_log.clone();
        let raid_clear = self.raid_clear;
        encounter.cleared = raid_clear;
        let party_info = self.party_info.clone();
        let raid_difficulty = self.raid_difficulty.clone();
        encounter.difficulty = raid_difficulty.clone().into();
        let region = self.region.clone();
        let meter_version = self.app.app_handle().package_info().version.to_string();

        self.set_ntp_fight_start_from_cache();
        let ntp_fight_start = self.ntp_fight_start;

        let rdps_valid = self.rdps_valid;
        let rdps_message = self.rdps_message.clone();

        let skill_cast_log = self.skill_tracker.get_cast_log();
        let skill_cooldowns = self.skill_tracker.skill_cooldowns.clone();
        let intermission_start = self.intermission_start;
        let intermission_end = self.intermission_end;

        // debug_print(format_args!("skill cast log:\n{}", serde_json::to_string(&skill_cast_log).unwrap()));

        // debug_print(format_args!("rdps_data valid: [{}]", rdps_valid));
        info!(
            "saving to db - cleared: [{}], difficulty: [{}] {}",
            raid_clear, self.raid_difficulty, encounter.current_boss_name
        );

        encounter.current_boss_name = update_current_boss_name(&encounter.current_boss_name);

        if DEBUG_DUMP_DAMAGE_STATE_JSON {
            let dump = self.build_damage_state_dump();
            let dump_label = format!(
                "{}-{}-{}",
                encounter.local_player,
                encounter.current_boss_name,
                if raid_clear { "clear" } else { "wipe" }
            );
            write_debug_json_dump("damage-state", &dump_label, &dump);
        }

        let contribution_splits = self.build_contribution_splits();

        let app = self.app.clone();
        task::spawn(async move {
            let stats_api = app.state::<StatsApi>();
            let player_info =
                if let Some(args) = GetCharacterInfoArgs::new(&encounter, &raid_difficulty) {
                    info!("fetching player info");

                    stats_api.get_character_info(args).await
                } else {
                    None
                };

            let _ = task::spawn_blocking(move || {
                let repository = app.state::<Repository>();

                let args = InsertEncounterArgs {
                    encounter,
                    damage_log,
                    cast_log,
                    boss_hp_log,
                    raid_clear,
                    party_info,
                    raid_difficulty,
                    region,
                    player_info,
                    meter_version,
                    ntp_fight_start,
                    rdps_valid,
                    rdps_message,
                    manual,
                    skill_cast_log,
                    skill_cooldowns,
                    intermission_start,
                    intermission_end,
                    contribution_splits,
                };

                let encounter_id = repository
                    .insert_data(args)
                    .expect("could not save encounter");

                info!("saved to db");

                if raid_clear {
                    app.emit("clear-encounter", encounter_id)
                        .expect("failed to emit clear-encounter");
                }
            })
            .await;
        });
    }
}

fn lal_party_number_unknown(party_number: &i32) -> bool {
    *party_number == -2
}

fn lal_default_datetime() -> String {
    "0001-01-01T00:00:00Z".to_string()
}

fn timestamp_ms_to_lal_datetime(timestamp_ms: i64) -> String {
    chrono::DateTime::<Utc>::from_timestamp_millis(timestamp_ms)
        .map(|datetime| {
            let fractional_100ns = datetime.timestamp_subsec_nanos() / 100;
            format!(
                "{}.{fractional_100ns:07}Z",
                datetime.format("%Y-%m-%dT%H:%M:%S")
            )
        })
        .unwrap_or_else(lal_default_datetime)
}

fn status_effect_is_infinite(status_effect: &StatusEffectDetails) -> bool {
    // infinite if duration is (sub-)zero or longer than an hour
    status_effect.expiration_delay <= 0.0 || status_effect.expiration_delay > 3600.0
}
