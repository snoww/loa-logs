pub mod encounter_state;
mod entity_tracker;
mod id_tracker;
pub mod models;
mod party_tracker;
mod status_tracker;

use crate::parser::encounter_state::{get_class_from_id, EncounterState};
use crate::parser::entity_tracker::{get_current_and_max_hp, EntityTracker};
use crate::parser::id_tracker::IdTracker;
use crate::parser::models::{EntityType, Identity, Stagger};
use crate::parser::party_tracker::PartyTracker;
use crate::parser::status_tracker::{
    get_status_effect_value, StatusEffect, StatusEffectTargetType, StatusEffectType, StatusTracker,
};
use anyhow::Result;
use chrono::Utc;
use hashbrown::HashMap;
use log::{info, warn};
use meter_core::packets::definitions::*;
use meter_core::packets::opcodes::Pkt;
use meter_core::{start_capture, start_raw_capture};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{Manager, Window, Wry};

use self::models::{Settings, TripodIndex, TripodLevel};

pub fn start(
    window: Window<Wry>,
    ip: String,
    port: u16,
    raw_socket: bool,
    settings: Option<Settings>,
) -> Result<()> {
    let id_tracker = Rc::new(RefCell::new(IdTracker::new()));
    let party_tracker = Rc::new(RefCell::new(PartyTracker::new(id_tracker.clone())));
    let status_tracker = Rc::new(RefCell::new(StatusTracker::new(party_tracker.clone())));
    let mut entity_tracker = EntityTracker::new(
        status_tracker.clone(),
        id_tracker.clone(),
        party_tracker.clone(),
    );
    let mut state = EncounterState::new(window.clone());
    let rx = if raw_socket {
        if !meter_core::check_is_admin() {
            warn!("Not running as admin, cannot use raw socket");
            loop {
                window.emit("admin", "")?;
                thread::sleep(Duration::from_millis(5000));
            }
        }
        meter_core::add_firewall()?;
        match start_raw_capture(ip, port) {
            Ok(rx) => rx,
            Err(e) => {
                warn!("Error starting capture: {}", e);
                return Ok(());
            }
        }
    } else {
        match start_capture(ip, port) {
            Ok(rx) => rx,
            Err(e) => {
                warn!("Error starting capture: {}", e);
                return Ok(());
            }
        }
    };

    let mut last_update = Instant::now();
    let duration = Duration::from_millis(100);
    let mut last_party_update = Instant::now();
    let party_duration = Duration::from_millis(1000);
    let mut raid_end_cd: Instant = Instant::now();

    let reset = Arc::new(AtomicBool::new(false));
    let pause = Arc::new(AtomicBool::new(false));
    let save = Arc::new(AtomicBool::new(false));
    let boss_only_damage = Arc::new(AtomicBool::new(false));
    if let Some(settings) = settings {
        if settings.general.boss_only_damage {
            boss_only_damage.store(true, Ordering::Relaxed);
            info!("boss only damage enabled")
        }
    }

    // read saved local players
    // this info is used in case meter was opened late
    let mut local_players: HashMap<u64, String> = HashMap::new();
    let mut local_player_path = window.app_handle().path_resolver().resource_dir().unwrap();
    local_player_path.push("local_players.json");

    if local_player_path.exists() {
        let local_players_file = std::fs::read_to_string(local_player_path.clone())?;
        local_players = serde_json::from_str(&local_players_file).unwrap_or_default();
    }

    let emit_details = Arc::new(AtomicBool::new(false));

    let meter_window_clone = window.clone();
    window.listen_global("reset-request", {
        let reset_clone = reset.clone();
        let meter_window_clone = meter_window_clone.clone();
        move |_event| {
            reset_clone.store(true, Ordering::Relaxed);
            info!("resetting meter");
            meter_window_clone.emit("reset-encounter", "").ok();
        }
    });

    window.listen_global("save-request", {
        let save_clone = save.clone();
        let meter_window_clone = meter_window_clone.clone();
        move |_event| {
            save_clone.store(true, Ordering::Relaxed);
            info!("manual saving encounter");
            meter_window_clone.emit("save-encounter", "").ok();
        }
    });

    window.listen_global("pause-request", {
        let pause_clone = pause.clone();
        let meter_window_clone = meter_window_clone.clone();
        move |_event| {
            let prev = pause_clone.fetch_xor(true, Ordering::Relaxed);
            if prev {
                info!("unpausing meter");
            } else {
                info!("pausing meter");
            }
            meter_window_clone.emit("pause-encounter", "").ok();
        }
    });

    window.listen_global("boss-only-damage-request", {
        let boss_only_damage = boss_only_damage.clone();
        move |event| {
            if let Some(bod) = event.payload() {
                if bod == "true" {
                    boss_only_damage.store(true, Ordering::Relaxed);
                    info!("boss only damage enabled")
                } else {
                    boss_only_damage.store(false, Ordering::Relaxed);
                    info!("boss only damage disabled")
                }
            }
        }
    });

    window.listen_global("emit-details-request", {
        let emit_clone = emit_details.clone();
        move |_event| {
            let prev = emit_clone.fetch_xor(true, Ordering::Relaxed);
            if prev {
                info!("stopped sending details");
            } else {
                info!("sending details");
            }
        }
    });

    let mut party_freeze = false;
    let mut party_cache: Option<Vec<Vec<String>>> = None;
    let mut party_map_cache: HashMap<i32, Vec<String>> = HashMap::new();

    while let Ok((op, data)) = rx.recv() {
        if reset.load(Ordering::Relaxed) {
            state.soft_reset(true);
            reset.store(false, Ordering::Relaxed);
        }
        if pause.load(Ordering::Relaxed) {
            continue;
        }
        if save.load(Ordering::Relaxed) {
            save.store(false, Ordering::Relaxed);
            state.party_info = update_party(&party_tracker, &entity_tracker);
            state.save_to_db(true);
            state.saved = true;
            state.resetting = true;
        }

        if boss_only_damage.load(Ordering::Relaxed) {
            state.boss_only_damage = true;
        } else {
            state.boss_only_damage = false;
            state.encounter.boss_only_damage = false;
        }

        match op {
            Pkt::CounterAttackNotify => {
                if let Some(pkt) =
                    parse_pkt(&data, PKTCounterAttackNotify::new, "PKTCounterAttackNotify")
                {
                    if let Some(entity) = entity_tracker.entities.get(&pkt.source_id) {
                        state.on_counterattack(entity);
                    }
                }
            }
            Pkt::DeathNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTDeathNotify::new, "PKTDeathNotify") {
                    if let Some(entity) = entity_tracker.entities.get(&pkt.target_id) {
                        debug_print(format_args!(
                            "death: {}, {}, {}",
                            entity.name, entity.entity_type, entity.id
                        ));
                        state.on_death(entity);
                    }
                }
            }
            Pkt::IdentityGaugeChangeNotify => {
                if let Some(pkt) = parse_pkt(
                    &data,
                    PKTIdentityGaugeChangeNotify::new,
                    "PKTIdentityGaugeChangeNotify",
                ) {
                    state.on_identity_gain(&pkt);
                    if emit_details.load(Ordering::Relaxed) {
                        window.emit(
                            "identity-update",
                            Identity {
                                gauge1: pkt.identity_gauge1,
                                gauge2: pkt.identity_gauge2,
                                gauge3: pkt.identity_gauge3,
                            },
                        )?;
                    }
                }
            }
            Pkt::InitEnv => {
                // three methods of getting local player info
                // 1. MigrationExecute    + InitEnv      + PartyInfo
                // 2. Cached Local Player + InitEnv      + PartyInfo
                //    > character_id        > entity_id    > player_info
                // 3. InitPC

                if let Some(pkt) = parse_pkt(&data, PKTInitEnv::new, "PKTInitEnv") {
                    party_tracker.borrow_mut().reset_party_mappings();
                    state.raid_difficulty = "".to_string();
                    party_cache = None;
                    party_map_cache = HashMap::new();
                    let entity = entity_tracker.init_env(pkt);
                    state.on_init_env(entity);
                }
            }
            Pkt::InitPC => {
                if let Some(pkt) = parse_pkt(&data, PKTInitPC::new, "PKTInitPC") {
                    let (hp, max_hp) = get_current_and_max_hp(&pkt.stat_pair);
                    let entity = entity_tracker.init_pc(pkt);
                    info!(
                        "local player: {}, {}, {}, eid: {}, id: {}",
                        entity.name,
                        get_class_from_id(&entity.class_id),
                        entity.gear_level,
                        entity.id,
                        entity.character_id
                    );
                    if !local_players.contains_key(&entity.character_id) {
                        local_players.insert(entity.character_id, entity.name.clone());
                        write_local_players(&local_players, &local_player_path)?;
                    }
                    state.on_init_pc(entity, hp, max_hp)
                }
            }
            Pkt::MigrationExecute => {
                if let Some(pkt) = parse_pkt(&data, PKTMigrationExecute::new, "PKTMigrationExecute")
                {
                    entity_tracker.migration_execute(pkt);
                }
            }
            Pkt::NewPC => {
                if let Some(pkt) = parse_pkt(&data, PKTNewPC::new, "PKTNewPC") {
                    let (hp, max_hp) = get_current_and_max_hp(&pkt.pc_struct.stat_pair);
                    let entity = entity_tracker.new_pc(pkt);
                    debug_print(format_args!(
                        "new PC: {}, {}, {}, eid: {}, cid: {}",
                        entity.name,
                        get_class_from_id(&entity.class_id),
                        entity.gear_level,
                        entity.id,
                        entity.character_id
                    ));
                    state.on_new_pc(entity, hp, max_hp);
                }
            }
            Pkt::NewNpc => {
                if let Some(pkt) = parse_pkt(&data, PKTNewNpc::new, "PKTNewNpc") {
                    let (hp, max_hp) = get_current_and_max_hp(&pkt.npc_struct.stat_pair);
                    let entity = entity_tracker.new_npc(pkt, max_hp);
                    debug_print(format_args!(
                        "new {}: {}, eid: {}, id: {}, hp: {}",
                        entity.entity_type, entity.name, entity.id, entity.npc_id, max_hp
                    ));
                    state.on_new_npc(entity, hp, max_hp);
                }
            }
            Pkt::NewNpcSummon => {
                if let Some(pkt) = parse_pkt(&data, PKTNewNpcSummon::new, "PKTNewNpcSummon") {
                    let (hp, max_hp) = get_current_and_max_hp(&pkt.npc_data.stat_pair);
                    let entity = entity_tracker.new_npc_summon(pkt, max_hp);
                    debug_print(format_args!(
                        "new {}: {}, eid: {}, id: {}, hp: {}",
                        entity.entity_type, entity.name, entity.id, entity.npc_id, max_hp
                    ));
                    state.on_new_npc(entity, hp, max_hp);
                }
            }
            Pkt::NewProjectile => {
                if let Some(pkt) = parse_pkt(&data, PKTNewProjectile::new, "PKTNewProjectile") {
                    entity_tracker.new_projectile(pkt);
                }
            }
            Pkt::NewTrap => {
                if let Some(pkt) = parse_pkt(&data, PKTNewTrap::new, "PKTNewTrap") {
                    entity_tracker.new_trap(pkt);
                }
            }
            Pkt::ParalyzationStateNotify => {
                if let Some(pkt) = parse_pkt(
                    &data,
                    PKTParalyzationStateNotify::new,
                    "PKTParalyzationStateNotify",
                ) {
                    state.on_stagger_change(&pkt);
                    if emit_details.load(Ordering::Relaxed) {
                        window.emit(
                            "stagger-update",
                            Stagger {
                                current: pkt.paralyzation_point,
                                max: pkt.paralyzation_max_point,
                            },
                        )?;
                    }
                }
            }
            Pkt::PartyInfo => {
                if let Some(pkt) = parse_pkt(&data, PKTPartyInfo::new, "PKTPartyInfo") {
                    entity_tracker.party_info(pkt, &local_players);
                    let local_player_id = entity_tracker.local_entity_id;
                    if let Some(entity) = entity_tracker.entities.get(&local_player_id) {
                        state.update_local_player(entity);
                    }
                    party_cache = None;
                    party_map_cache = HashMap::new();
                }
            }
            Pkt::PartyLeaveResult => {
                if let Some(pkt) = parse_pkt(&data, PKTPartyLeaveResult::new, "PKTPartyLeaveResult")
                {
                    party_tracker
                        .borrow_mut()
                        .remove(pkt.party_instance_id, pkt.name);
                    party_cache = None;
                    party_map_cache = HashMap::new();
                }
            }
            Pkt::PartyStatusEffectAddNotify => {
                if let Some(pkt) = parse_pkt(
                    &data,
                    PKTPartyStatusEffectAddNotify::new,
                    "PKTPartyStatusEffectAddNotify",
                ) {
                    let shields = entity_tracker.party_status_effect_add(pkt);
                    for status_effect in shields {
                        let source = entity_tracker.get_source_entity(status_effect.source_id);
                        let target_id =
                            if status_effect.target_type == StatusEffectTargetType::Party {
                                id_tracker
                                    .borrow()
                                    .get_entity_id(status_effect.target_id)
                                    .unwrap_or_default()
                            } else {
                                status_effect.target_id
                            };
                        let target = entity_tracker.get_source_entity(target_id);
                        state.on_boss_shield(&target, status_effect.value);
                        state.on_shield_applied(
                            &source,
                            &target,
                            status_effect.status_effect_id,
                            status_effect.value,
                        );
                    }
                }
            }
            Pkt::PartyStatusEffectRemoveNotify => {
                if let Some(pkt) = parse_pkt(
                    &data,
                    PKTPartyStatusEffectRemoveNotify::new,
                    "PKTPartyStatusEffectRemoveNotify",
                ) {
                    let (is_shield, shields_broken) =
                        entity_tracker.party_status_effect_remove(pkt);
                    if is_shield {
                        for status_effect in shields_broken {
                            let change = status_effect.value;
                            on_shield_change(
                                &mut entity_tracker,
                                &id_tracker,
                                &mut state,
                                status_effect,
                                change,
                            );
                        }
                    }
                }
            }
            Pkt::PartyStatusEffectResultNotify => {
                if let Some(pkt) = parse_pkt(
                    &data,
                    PKTPartyStatusEffectResultNotify::new,
                    "PKTPartyStatusEffectResultNotify",
                ) {
                    party_tracker.borrow_mut().add(
                        pkt.raid_instance_id,
                        pkt.party_instance_id,
                        pkt.character_id,
                        0,
                        None,
                    );
                }
            }
            Pkt::RaidBegin => {
                if let Some(pkt) = parse_pkt(&data, PKTRaidBegin::new, "PKTRaidBegin") {
                    debug_print(format_args!("raid begin: {}", pkt.raid_id));
                    match pkt.raid_id {
                        308226 | 308227 => {
                            state.raid_difficulty = "Trial".to_string();
                        }
                        308428 | 308429 | 308420 | 308410 | 308411 | 308414 | 308422 | 308424
                        | 308421 | 308412 | 308423 | 308426 | 308416 | 308419 | 308415 | 308437
                        | 308417 | 308418 | 308425 | 308430 => {
                            state.raid_difficulty = "Challenge".to_string();
                        }
                        _ => {
                            state.raid_difficulty = "".to_string();
                        }
                    }
                }
            }
            Pkt::RaidBossKillNotify => {
                state.on_phase_transition(1);
                state.raid_clear = true;
                debug_print(format_args!("phase: 1 - RaidBossKillNotify"));
            }
            Pkt::RaidResult => {
                party_freeze = true;
                state.party_info = if let Some(party) = party_cache.take() {
                    party
                } else {
                    update_party(&party_tracker, &entity_tracker)
                };
                state.on_phase_transition(0);
                raid_end_cd = Instant::now();
                debug_print(format_args!("phase: 0 - RaidResult"));
            }
            Pkt::RemoveObject => {
                if let Some(pkt) = parse_pkt(&data, PKTRemoveObject::new, "PKTRemoveObject") {
                    for upo in pkt.unpublished_objects {
                        entity_tracker.entities.remove(&upo.object_id);
                        status_tracker
                            .borrow_mut()
                            .remove_local_object(upo.object_id);
                    }
                }
            }
            Pkt::SkillCastNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTSkillCastNotify::new, "PKTSkillCastNotify") {
                    let mut entity = entity_tracker.get_source_entity(pkt.caster);
                    entity_tracker.guess_is_player(&mut entity, pkt.skill_id);
                    if entity.class_id == 202 {
                        state.on_skill_start(
                            entity,
                            pkt.skill_id,
                            None,
                            None,
                            Utc::now().timestamp_millis(),
                        );
                    }
                }
            }
            Pkt::SkillStartNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTSkillStartNotify::new, "PKTSkillStartNotify")
                {
                    let mut entity = entity_tracker.get_source_entity(pkt.source_id);
                    entity_tracker.guess_is_player(&mut entity, pkt.skill_id);
                    let tripod_index =
                        pkt.skill_option_data
                            .tripod_index
                            .map(|tripod_index| TripodIndex {
                                first: tripod_index.first,
                                second: tripod_index.second,
                                third: tripod_index.third,
                            });
                    let tripod_level =
                        pkt.skill_option_data
                            .tripod_level
                            .map(|tripod_level| TripodLevel {
                                first: tripod_level.first,
                                second: tripod_level.second,
                                third: tripod_level.third,
                            });
                    state.on_skill_start(
                        entity,
                        pkt.skill_id,
                        tripod_index,
                        tripod_level,
                        Utc::now().timestamp_millis(),
                    );
                }
            }
            // Pkt::SkillStageNotify => {
            //     let pkt = PKTSkillStageNotify::new(&data);
            // }
            Pkt::SkillDamageAbnormalMoveNotify => {
                if Instant::now() - raid_end_cd < Duration::from_secs(10) {
                    debug_print(format_args!(
                        "ignoring damage - SkillDamageAbnormalMoveNotify"
                    ));
                    continue;
                }
                if let Some(pkt) = parse_pkt(
                    &data,
                    PKTSkillDamageAbnormalMoveNotify::new,
                    "PKTSkillDamageAbnormalMoveNotify",
                ) {
                    let owner = entity_tracker.get_source_entity(pkt.source_id);
                    let local_character_id = id_tracker
                        .borrow()
                        .get_local_character_id(entity_tracker.local_entity_id);
                    for event in pkt.skill_damage_abnormal_move_events.iter() {
                        let target_entity =
                            entity_tracker.get_or_create_entity(event.skill_damage_event.target_id);
                        let source_entity = entity_tracker.get_or_create_entity(pkt.source_id);
                        let (se_on_source, se_on_target) = status_tracker
                            .borrow_mut()
                            .get_status_effects(&owner, &target_entity, local_character_id);
                        state.on_damage(
                            &owner,
                            &source_entity,
                            &target_entity,
                            event.skill_damage_event.damage,
                            pkt.skill_id,
                            pkt.skill_effect_id,
                            event.skill_damage_event.modifier as i32,
                            event.skill_damage_event.cur_hp,
                            event.skill_damage_event.max_hp,
                            se_on_source,
                            se_on_target,
                            Utc::now().timestamp_millis(),
                        );
                    }
                }
            }
            Pkt::SkillDamageNotify => {
                // use this to make sure damage packets are not tracked after a raid just wiped
                if Instant::now() - raid_end_cd < Duration::from_secs(10) {
                    debug_print(format_args!("ignoring damage - SkillDamageNotify"));
                    continue;
                }
                if let Some(pkt) =
                    parse_pkt(&data, PKTSkillDamageNotify::new, "PktSkillDamageNotify")
                {
                    let owner = entity_tracker.get_source_entity(pkt.source_id);
                    let local_character_id = id_tracker
                        .borrow()
                        .get_local_character_id(entity_tracker.local_entity_id);
                    for event in pkt.skill_damage_events.iter() {
                        let target_entity = entity_tracker.get_or_create_entity(event.target_id);
                        // source_entity is to determine battle item
                        let source_entity = entity_tracker.get_or_create_entity(pkt.source_id);
                        let (se_on_source, se_on_target) = status_tracker
                            .borrow_mut()
                            .get_status_effects(&owner, &target_entity, local_character_id);
                        state.on_damage(
                            &owner,
                            &source_entity,
                            &target_entity,
                            event.damage,
                            pkt.skill_id,
                            pkt.skill_effect_id.unwrap_or_default(),
                            event.modifier as i32,
                            event.cur_hp,
                            event.max_hp,
                            se_on_source,
                            se_on_target,
                            Utc::now().timestamp_millis(),
                        );
                    }
                }
            }
            Pkt::StatusEffectAddNotify => {
                if let Some(pkt) = parse_pkt(
                    &data,
                    PKTStatusEffectAddNotify::new,
                    "PKTStatusEffectAddNotify",
                ) {
                    let status_effect = entity_tracker.build_and_register_status_effect(
                        &pkt.status_effect_data,
                        pkt.object_id,
                        Utc::now(),
                    );
                    if status_effect.status_effect_type == StatusEffectType::Shield {
                        let source = entity_tracker.get_source_entity(status_effect.source_id);
                        let target_id =
                            if status_effect.target_type == StatusEffectTargetType::Party {
                                id_tracker
                                    .borrow()
                                    .get_entity_id(status_effect.target_id)
                                    .unwrap_or_default()
                            } else {
                                status_effect.target_id
                            };
                        let target = entity_tracker.get_source_entity(target_id);
                        state.on_boss_shield(&target, status_effect.value);
                        state.on_shield_applied(
                            &source,
                            &target,
                            status_effect.status_effect_id,
                            status_effect.value,
                        );
                    }
                }
            }
            Pkt::StatusEffectDurationNotify => {
                if let Some(pkt) = parse_pkt(
                    &data,
                    PKTStatusEffectDurationNotify::new,
                    "PKTStatusEffectDurationNotify",
                ) {
                    // debug_print(format_args!("status effect duration: {:?}", pkt));
                    status_tracker.borrow_mut().update_status_duration(
                        pkt.effect_instance_id,
                        pkt.target_id,
                        pkt.expiration_tick,
                        StatusEffectTargetType::Local,
                    );
                }
            }
            Pkt::StatusEffectRemoveNotify => {
                if let Some(pkt) = parse_pkt(
                    &data,
                    PKTStatusEffectRemoveNotify::new,
                    "PKTStatusEffectRemoveNotify",
                ) {
                    let (is_shield, shields_broken) =
                        status_tracker.borrow_mut().remove_status_effects(
                            pkt.object_id,
                            pkt.status_effect_ids,
                            pkt.reason,
                            StatusEffectTargetType::Local,
                        );

                    if is_shield {
                        if shields_broken.is_empty() {
                            let target = entity_tracker.get_source_entity(pkt.object_id);
                            state.on_boss_shield(&target, 0);
                        } else {
                            for status_effect in shields_broken {
                                let change = status_effect.value;
                                on_shield_change(
                                    &mut entity_tracker,
                                    &id_tracker,
                                    &mut state,
                                    status_effect,
                                    change,
                                );
                            }
                        }
                    }
                }
            }
            Pkt::TriggerBossBattleStatus => {
                // need to hard code clown because it spawns before the trigger is sent???
                if state.encounter.current_boss_name.is_empty()
                    || state.encounter.fight_start == 0
                    || state.encounter.current_boss_name == "Saydon"
                {
                    state.on_phase_transition(3);
                    debug_print(format_args!(
                        "phase: 3 - resetting encounter - TriggerBossBattleStatus"
                    ));
                }
            }
            Pkt::TriggerStartNotify => {
                if let Some(pkt) =
                    parse_pkt(&data, PKTTriggerStartNotify::new, "PKTTriggerStartNotify")
                {
                    match pkt.trigger_signal_type {
                        57 | 59 | 61 | 63 | 74 | 76 => {
                            party_freeze = true;
                            state.party_info = if let Some(party) = party_cache.take() {
                                party
                            } else {
                                update_party(&party_tracker, &entity_tracker)
                            };
                            state.raid_clear = true;
                            state.on_phase_transition(2);
                            raid_end_cd = Instant::now();
                            debug_print(format_args!("phase: 2 - clear - TriggerStartNotify"));
                        }
                        58 | 60 | 62 | 64 | 75 | 77 => {
                            party_freeze = true;
                            state.party_info = if let Some(party) = party_cache.take() {
                                party
                            } else {
                                update_party(&party_tracker, &entity_tracker)
                            };
                            state.raid_clear = false;
                            state.on_phase_transition(4);
                            raid_end_cd = Instant::now();
                            debug_print(format_args!("phase: 4 - wipe - TriggerStartNotify"));
                        }
                        _ => {}
                    }
                }
            }
            Pkt::ZoneMemberLoadStatusNotify => {
                if let Some(pkt) = parse_pkt(
                    &data,
                    PKTZoneMemberLoadStatusNotify::new,
                    "PKTZoneMemberLoadStatusNotify",
                ) {
                    if !state.raid_difficulty.is_empty() {
                        continue;
                    }

                    match pkt.zone_level {
                        0 => {
                            state.raid_difficulty = "Normal".to_string();
                        }
                        1 => {
                            state.raid_difficulty = "Hard".to_string();
                        }
                        2 => {
                            state.raid_difficulty = "Inferno".to_string();
                        }
                        3 => {
                            state.raid_difficulty = "Challenge".to_string();
                        }
                        4 => {
                            state.raid_difficulty = "Special".to_string();
                        }
                        _ => {}
                    }
                }
            }
            Pkt::ZoneObjectUnpublishNotify => {
                if let Some(pkt) = parse_pkt(
                    &data,
                    PKTZoneObjectUnpublishNotify::new,
                    "PKTZoneObjectUnpublishNotify",
                ) {
                    status_tracker
                        .borrow_mut()
                        .remove_local_object(pkt.object_id);
                }
            }
            Pkt::StatusEffectSyncDataNotify => {
                if let Some(pkt) = parse_pkt(
                    &data,
                    PKTStatusEffectSyncDataNotify::new,
                    "PKTStatusEffectSyncDataNotify",
                ) {
                    let (status_effect, old_value) =
                        status_tracker.borrow_mut().sync_status_effect(
                            pkt.effect_instance_id,
                            pkt.character_id,
                            pkt.object_id,
                            pkt.value,
                            entity_tracker.local_character_id,
                        );
                    if let Some(status_effect) = status_effect {
                        if status_effect.status_effect_type == StatusEffectType::Shield {
                            let change = old_value - status_effect.value;
                            on_shield_change(
                                &mut entity_tracker,
                                &id_tracker,
                                &mut state,
                                status_effect,
                                change,
                            );
                        }
                    }
                }
            }
            Pkt::TroopMemberUpdateMinNotify => {
                if let Some(pkt) = parse_pkt(
                    &data,
                    PKTTroopMemberUpdateMinNotify::new,
                    "PKTTroopMemberUpdateMinNotify",
                ) {
                    for se in pkt.status_effect_datas.iter() {
                        if let Some(object_id) = id_tracker.borrow().get_entity_id(pkt.character_id)
                        {
                            let val = get_status_effect_value(&se.value);
                            let (status_effect, old_value) =
                                status_tracker.borrow_mut().sync_status_effect(
                                    se.effect_instance_id,
                                    pkt.character_id,
                                    object_id,
                                    val,
                                    entity_tracker.local_character_id,
                                );
                            if let Some(status_effect) = status_effect {
                                if status_effect.status_effect_type == StatusEffectType::Shield {
                                    let change = old_value - status_effect.value;
                                    on_shield_change(
                                        &mut entity_tracker,
                                        &id_tracker,
                                        &mut state,
                                        status_effect,
                                        change,
                                    );
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        if last_update.elapsed() >= duration || state.resetting || state.boss_dead_update {
            let boss_dead = state.boss_dead_update;
            if state.boss_dead_update {
                state.boss_dead_update = false;
            }
            let mut clone = state.encounter.clone();
            let window = window.clone();

            let party_info: Option<HashMap<i32, Vec<String>>> =
                if last_party_update.elapsed() >= party_duration && !party_freeze {
                    last_party_update = Instant::now();
                    // we used cached party if it exists
                    if party_cache.is_some() {
                        Some(party_map_cache.clone())
                    } else {
                        let party = update_party(&party_tracker, &entity_tracker);
                        // check if both parties are resolved
                        // if they are we then cache it
                        if party.len() == 2 && party[0].len() == 4 && party[1].len() == 4 {
                            party_cache = Some(party.clone());
                            party_map_cache = party
                                .into_iter()
                                .enumerate()
                                .map(|(index, party)| (index as i32, party))
                                .collect();

                            Some(party_map_cache.clone())
                        } else if party.len() > 1 {
                            Some(
                                party
                                    .into_iter()
                                    .enumerate()
                                    .map(|(index, party)| (index as i32, party))
                                    .collect(),
                            )
                        } else {
                            None
                        }
                    }
                } else {
                    None
                };

            tokio::task::spawn(async move {
                if !clone.current_boss_name.is_empty() {
                    let current_boss = clone.entities.get(&clone.current_boss_name).cloned();
                    if let Some(mut current_boss) = current_boss {
                        if boss_dead {
                            current_boss.is_dead = true;
                            current_boss.current_hp = 0;
                        }
                        clone.current_boss = Some(current_boss);
                    } else {
                        clone.current_boss_name = String::new();
                    }
                }
                clone.entities.retain(|_, e| {
                    ((e.entity_type == EntityType::PLAYER && e.class_id > 0)
                        || e.entity_type == EntityType::ESTHER
                        || e.entity_type == EntityType::BOSS)
                        && e.damage_stats.damage_dealt > 0
                });

                if !clone.entities.is_empty() {
                    window
                        .emit("encounter-update", Some(clone))
                        .expect("failed to emit encounter-update");

                    if party_info.is_some() {
                        window
                            .emit("party-update", party_info)
                            .expect("failed to emit party-update");
                    }
                }
            });

            last_update = Instant::now();
        }

        if state.resetting {
            state.soft_reset(true);
            state.resetting = false;
            state.saved = false;
            party_freeze = false;
            party_cache = None;
            party_map_cache = HashMap::new();
        }
    }

    Ok(())
}

fn update_party(
    party_tracker: &Rc<RefCell<PartyTracker>>,
    entity_tracker: &EntityTracker,
) -> Vec<Vec<String>> {
    let mut party_info: HashMap<u32, Vec<String>> = HashMap::new();

    for (entity_id, party_id) in party_tracker.borrow().entity_id_to_party_id.iter() {
        party_info.entry(*party_id).or_insert_with(Vec::new).extend(
            entity_tracker
                .entities
                .get(entity_id)
                .map(|entity| entity.name.clone()),
        );
    }
    let mut sorted_parties = party_info.into_iter().collect::<Vec<(u32, Vec<String>)>>();
    sorted_parties.sort_by_key(|&(party_id, _)| party_id);
    sorted_parties
        .into_iter()
        .map(|(_, members)| members)
        .collect()
}

fn on_shield_change(
    entity_tracker: &mut EntityTracker,
    id_tracker: &Rc<RefCell<IdTracker>>,
    state: &mut EncounterState,
    status_effect: StatusEffect,
    change: u64,
) {
    let source = entity_tracker.get_source_entity(status_effect.source_id);
    let target_id = if status_effect.target_type == StatusEffectTargetType::Party {
        id_tracker
            .borrow()
            .get_entity_id(status_effect.target_id)
            .unwrap_or_default()
    } else {
        status_effect.target_id
    };
    let target = entity_tracker.get_source_entity(target_id);
    state.on_boss_shield(&target, status_effect.value);
    state.on_shield_used(&source, &target, status_effect.status_effect_id, change);
}

fn write_local_players(local_players: &HashMap<u64, String>, path: &PathBuf) -> Result<()> {
    let ordered: BTreeMap<_, _> = local_players.iter().collect();
    let local_players_file = serde_json::to_string(&ordered)?;
    std::fs::write(path, local_players_file)?;
    Ok(())
}

fn parse_pkt<T, F>(data: &[u8], new_fn: F, pkt_name: &str) -> Option<T>
where
    F: FnOnce(&[u8]) -> Result<T, anyhow::Error>,
{
    match new_fn(data) {
        Ok(packet) => Some(packet),
        Err(e) => {
            warn!("Error parsing {}: {}", pkt_name, e);
            None
        }
    }
}

fn debug_print(args: std::fmt::Arguments<'_>) {
    #[cfg(debug_assertions)]
    {
        info!("{}", args);
    }
}
