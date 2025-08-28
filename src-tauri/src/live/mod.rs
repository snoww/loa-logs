mod encounter_state;
mod entity_tracker;
mod id_tracker;
mod party_tracker;
pub mod skill_tracker;
pub mod stats_api;
mod status_tracker;
pub mod utils;
pub mod capture;
pub mod local;
pub mod region;
pub mod heartbeat;

pub use capture::*;

use crate::live::capture::DamageEncryptionHandler;
use crate::live::encounter_state::EncounterState;
use crate::live::entity_tracker::{get_current_and_max_hp, EntityTracker};
use crate::live::heartbeat::{HeartbeatApi, HeartbeatSendArgs};
use crate::live::id_tracker::IdTracker;
use crate::live::local::LocalPlayerRepository;
use crate::live::party_tracker::PartyTracker;
use crate::live::region::RegionAcessor;
use crate::live::stats_api::StatsApi;
use crate::live::status_tracker::{
    get_status_effect_value, StatusEffectDetails, StatusEffectTargetType, StatusEffectType,
    StatusTracker,
};
use crate::live::utils::get_class_from_id;
use crate::parser::models::*;
use crate::parser::models::TripodIndex;
use anyhow::Result;
use chrono::Utc;
use hashbrown::HashMap;
use log::{info, warn};
use meter_core::packets::definitions::*;
use meter_core::packets::opcodes::Pkt;
use std::any::type_name;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

pub fn start<
    HC: HeartbeatApi,
    DE: DamageEncryptionHandler,
    PC: PacketCapture,
    RC: RegionAcessor>(
    mut heartbeat_api: HC,
    region_accessor: RC,
    packet_capture: PC,
    mut damage_handler: DE,
    app: AppHandle,
    port: u16,
    version: String,
    settings: Settings) -> Result<()> {
    let id_tracker = Rc::new(RefCell::new(IdTracker::new()));
    let party_tracker = Rc::new(RefCell::new(PartyTracker::new(id_tracker.clone())));
    let status_tracker = Rc::new(RefCell::new(StatusTracker::new(party_tracker.clone())));
    let mut entity_tracker = EntityTracker::new(
        status_tracker.clone(),
        id_tracker.clone(),
        party_tracker.clone(),
    );
    let mut state = EncounterState::new();

    let local_player_repository = app.state::<LocalPlayerRepository>();

    let mut stats_api = StatsApi::new(app.app_handle());
    let rx = match packet_capture.start(port) {
        Ok(rx) => rx,
        Err(e) => {
            warn!("Error starting capture: {}", e);
            return Ok(());
        }
    };

    damage_handler.start()?;

    let mut last_update = Instant::now();
    let mut duration = Duration::from_millis(200);
    let mut last_party_update = Instant::now();
    let party_duration = Duration::from_millis(2000);
    let mut raid_end_cd = Instant::now();

    // let client = Client::new();
    // let mut last_heartbeat = Instant::now();
    // let heartbeat_duration = Duration::from_secs(60 * 15);

    let reset = Arc::new(AtomicBool::new(false));
    let pause = Arc::new(AtomicBool::new(false));
    let save = Arc::new(AtomicBool::new(false));
    let boss_only_damage = Arc::new(AtomicBool::new(false));
    
    if settings.general.boss_only_damage {
        boss_only_damage.store(true, Ordering::Relaxed);
        info!("boss only damage enabled")
    }

    if settings.general.low_performance_mode {
        duration = Duration::from_millis(1500);
        info!("low performance mode enabled")
    }

    let mut local_info = local_player_repository.read()?;
    stats_api.client_id = local_info.client_id.clone();
    let client_id = local_info.client_id.clone();

    let region = region_accessor.get();
    state.region = region.clone();
    state.encounter.region = region.clone();

    let emit_details = Arc::new(AtomicBool::new(false));

    let cloned = app.app_handle();
    app.listen_global("reset-request", {
        let reset_clone = reset.clone();
        let app_clone = cloned.app_handle();
        move |_event| {
            reset_clone.store(true, Ordering::Relaxed);
            info!("resetting meter");
            app_clone.emit_all("reset-encounter", "").ok();
        }
    });

    app.listen_global("save-request", {
        let save_clone = save.clone();
        let app_clone = cloned.app_handle();
        move |_event| {
            save_clone.store(true, Ordering::Relaxed);
            info!("manual saving encounter");
            app_clone.emit_all("save-encounter", "").ok();
        }
    });

    app.listen_global("pause-request", {
        let pause_clone = pause.clone();
        let app_clone = cloned.app_handle();
        move |_event| {
            let prev = pause_clone.fetch_xor(true, Ordering::Relaxed);
            if prev {
                info!("unpausing meter");
            } else {
                info!("pausing meter");
            }
            app_clone.emit_all("pause-encounter", "").ok();
        }
    });

    app.listen_global("boss-only-damage-request", {
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

    app.listen_global("emit-details-request", {
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
            state.save_to_db(app.clone(), &stats_api, true);
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
                    parse_pkt(&data, PKTCounterAttackNotify::new, )
                {
                    if let Some(entity) = entity_tracker.entities.get(&pkt.source_id) {
                        state.on_counterattack(entity);
                    }
                }
            }
            Pkt::DeathNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTDeathNotify::new, ) {
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
                if let Some(pkt) = parse_pkt(&data, PKTIdentityGaugeChangeNotify::new) {
                    if emit_details.load(Ordering::Relaxed) {
                        app.emit_all(
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
            // Pkt::IdentityStanceChangeNotify => {
            //     if let Some(pkt) = parse_pkt(
            //         &data,
            //         PKTIdentityStanceChangeNotif:new,
            //         "PKTIdentityStanceChangeNotify",
            //     ) {
            //         if let Some(entity) = entity_tracker.entities.get_mut(&pkt.object_id) {
            //             if entity.entity_type == EntityType::PLAYER {
            //                 entity.stance = pkt.stance;
            //             }
            //         }
            //     }
            // }
            Pkt::InitEnv => {
                // three methods of getting local player info
                // 1. MigrationExecute    + InitEnv      + PartyInfo
                // 2. Cached Local Player + InitEnv      + PartyInfo
                //    > character_id        > entity_id    > player_info
                // 3. InitPC

                if let Some(pkt) = parse_pkt(&data, PKTInitEnv::new, ) {
                    party_tracker.borrow_mut().reset_party_mappings();
                    state.raid_difficulty = "".to_string();
                    state.raid_difficulty_id = 0;
                    state.damage_is_valid = true;
                    party_cache = None;
                    let entity = entity_tracker.init_env(pkt);
                    state.on_init_env(app.clone(), entity, &stats_api);
                    
                    let region = region_accessor.get();

                    if let Some(region) = region {
                        state.region = Some(region.clone());
                        state.encounter.region = Some(region);
                    }
                    
                    info!("region: {:?}", state.region);
                }
            }
            Pkt::InitPC => {
                if let Some(pkt) = parse_pkt(&data, PKTInitPC::new, ) {
                    let (hp, max_hp) = get_current_and_max_hp(&pkt.stat_pairs);
                    let entity = entity_tracker.init_pc(pkt);
                    info!(
                        "local player: {}, {}, {}, eid: {}, id: {}",
                        entity.name,
                        get_class_from_id(&entity.class_id),
                        entity.gear_level,
                        entity.id,
                        entity.character_id
                    );

                    local_info
                        .local_players
                        .entry(entity.character_id)
                        .and_modify(|e| {
                            e.name = entity.name.clone();
                            e.count += 1;
                        })
                        .or_insert(LocalPlayer {
                            name: entity.name.clone(),
                            count: 1,
                        });

                    local_player_repository.write(&local_info)?;

                    state.on_init_pc(entity, hp, max_hp)
                }
            }
            // Pkt::InitItem => {
            //     if let Some(pkt) = parse_pkt(&data, PKTInitItem::new, ) {
            //         if pkt.storage_type == 1 || pkt.storage_type == 20 {
            //             entity_tracker.get_local_player_set_options(pkt.item_data_list);
            //         }
            //     }
            // }
            // Pkt::MigrationExecute => {
            //     if let Some(pkt) = parse_pkt(&data, PKTMigrationExecute::new, )
            //     {
            //         entity_tracker.migration_execute(pkt);
            //         get_and_set_region(region_file_path.as_ref(), &mut state);
            //     }
            // }
            Pkt::NewPC => {
                if let Some(pkt) = parse_pkt(&data, PKTNewPC::new, ) {
                    let (hp, max_hp) = get_current_and_max_hp(&pkt.pc_struct.stat_pairs);
                    let entity = entity_tracker.new_pc(pkt.pc_struct);
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
            Pkt::NewVehicle => {
                if let Some(pkt) = parse_pkt(&data, PKTNewVehicle::new, ) {
                    if let Some(pc_struct) =
                        pkt.vehicle_struct.sub_p_k_t_new_vehicle_2_2_397.p_c_struct
                    {
                        let (hp, max_hp) = get_current_and_max_hp(&pc_struct.stat_pairs);
                        let entity = entity_tracker.new_pc(pc_struct);
                        debug_print(format_args!(
                            "new PC from vehicle: {}, {}, {}, eid: {}, cid: {}",
                            entity.name,
                            get_class_from_id(&entity.class_id),
                            entity.gear_level,
                            entity.id,
                            entity.character_id
                        ));
                        state.on_new_pc(entity, hp, max_hp);
                    }
                }
            }
            Pkt::NewNpc => {
                if let Some(pkt) = parse_pkt(&data, PKTNewNpc::new, ) {
                    let (hp, max_hp) = get_current_and_max_hp(&pkt.npc_struct.stat_pairs);
                    let entity = entity_tracker.new_npc(pkt, max_hp);
                    debug_print(format_args!(
                        "new {}: {}, eid: {}, id: {}, hp: {}",
                        entity.entity_type, entity.name, entity.id, entity.npc_id, max_hp
                    ));
                    state.on_new_npc(entity, hp, max_hp);
                }
            }
            Pkt::NewNpcSummon => {
                if let Some(pkt) = parse_pkt(&data, PKTNewNpcSummon::new, ) {
                    let (hp, max_hp) = get_current_and_max_hp(&pkt.npc_struct.stat_pairs);
                    let entity = entity_tracker.new_npc_summon(pkt, max_hp);
                    debug_print(format_args!(
                        "new {}: {}, eid: {}, id: {}, hp: {}",
                        entity.entity_type, entity.name, entity.id, entity.npc_id, max_hp
                    ));
                    state.on_new_npc(entity, hp, max_hp);
                }
            }
            Pkt::NewProjectile => {
                if let Some(pkt) = parse_pkt(&data, PKTNewProjectile::new, ) {
                    entity_tracker.new_projectile(&pkt);
                    if entity_tracker.id_is_player(pkt.projectile_info.owner_id)
                        && pkt.projectile_info.skill_id > 0
                    {
                        let key = (pkt.projectile_info.owner_id, pkt.projectile_info.skill_id);
                        if let Some(timestamp) = state.skill_tracker.skill_timestamp.get(&key) {
                            state
                                .skill_tracker
                                .projectile_id_to_timestamp
                                .insert(pkt.projectile_info.projectile_id, timestamp);
                        }
                    }
                }
            }
            Pkt::NewTrap => {
                if let Some(pkt) = parse_pkt(&data, PKTNewTrap::new, ) {
                    entity_tracker.new_trap(&pkt);
                    if entity_tracker.id_is_player(pkt.trap_struct.owner_id)
                        && pkt.trap_struct.skill_id > 0
                    {
                        let key = (pkt.trap_struct.owner_id, pkt.trap_struct.skill_id);
                        if let Some(timestamp) = state.skill_tracker.skill_timestamp.get(&key) {
                            state
                                .skill_tracker
                                .projectile_id_to_timestamp
                                .insert(pkt.trap_struct.object_id, timestamp);
                        }
                    }
                }
            }
            // Pkt::ParalyzationStateNotify => {
            //     if let Some(pkt) = parse_pkt(
            //         &data,
            //         PKTParalyzationStateNotif:new,
            //         "PKTParalyzationStateNotify",
            //     ) {
            //         state.on_stagger_change(&pkt);
            //         if emit_details.load(Ordering::Relaxed) {
            //             window.emit(
            //                 "stagger-update",
            //                 Stagger {
            //                     current: pkt.paralyzation_point,
            //                     max: pkt.paralyzation_max_point,
            //                 },
            //             )?;
            //         }
            //     }
            // }
            Pkt::RaidBegin => {
                if let Some(pkt) = parse_pkt(&data, PKTRaidBegin::new, ) {
                    debug_print(format_args!("raid begin: {}", pkt.raid_id));
                    match pkt.raid_id {
                        308226 | 308227 | 308239 | 308339 => {
                            state.raid_difficulty = "Trial".to_string();
                            state.raid_difficulty_id = 7;
                        }
                        308428 | 308429 | 308420 | 308410 | 308411 | 308414 | 308422 | 308424
                        | 308421 | 308412 | 308423 | 308426 | 308416 | 308419 | 308415 | 308437
                        | 308417 | 308418 | 308425 | 308430 => {
                            state.raid_difficulty = "Challenge".to_string();
                            state.raid_difficulty_id = 8;
                        }
                        _ => {
                            state.raid_difficulty = "".to_string();
                            state.raid_difficulty_id = 0;
                        }
                    }
                }
            }
            Pkt::RaidBossKillNotify => {
                state.on_phase_transition(app.clone(),1, &mut stats_api);
                state.raid_clear = true;
                info!("phase: 1 - RaidBossKillNotify");
            }
            Pkt::RaidResult => {
                party_freeze = true;
                state.party_info = if let Some(party) = party_cache.take() {
                    party
                } else {
                    update_party(&party_tracker, &entity_tracker)
                };
                state.on_phase_transition(app.clone(),0, &mut stats_api);
                raid_end_cd = Instant::now();
                info!("phase: 0 - RaidResult");
            }
            Pkt::RemoveObject => {
                if let Some(pkt) = parse_pkt(&data, PKTRemoveObject::new, ) {
                    for upo in pkt.unpublished_objects {
                        entity_tracker.entities.remove(&upo.object_id);
                        status_tracker
                            .borrow_mut()
                            .remove_local_object(upo.object_id);
                    }
                }
            }
            Pkt::SkillCastNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTSkillCastNotify::new, ) {
                    let mut entity = entity_tracker.get_source_entity(pkt.source_id);
                    entity_tracker.guess_is_player(&mut entity, pkt.skill_id);
                    // tracking arcana cards, bard major/minor chords
                    if entity.class_id == 202 || entity.class_id == 204 {
                        state.on_skill_start(
                            &entity,
                            pkt.skill_id,
                            None,
                            Utc::now().timestamp_millis(),
                        );
                    }
                }
            }
            Pkt::SkillCooldownNotify => {
                if let Some(pkt) =
                    parse_pkt(&data, PKTSkillCooldownNotify::new, )
                {
                    state.on_skill_cooldown(pkt.skill_cooldown_struct);
                }
            }
            Pkt::SkillStartNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTSkillStartNotify::new, )
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
                    let timestamp = Utc::now().timestamp_millis();
                    let (skill_id, summon_source) =
                        state.on_skill_start(&entity, pkt.skill_id, tripod_index, timestamp);

                    if entity.entity_type == EntityType::PLAYER && skill_id > 0 {
                        state
                            .skill_tracker
                            .new_cast(entity.id, skill_id, summon_source, timestamp);
                    }
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
                    PKTSkillDamageAbnormalMoveNotify::new
                ) {
                    let now = Utc::now().timestamp_millis();
                    let owner = entity_tracker.get_source_entity(pkt.source_id);
                    let local_character_id = id_tracker
                        .borrow()
                        .get_local_character_id(entity_tracker.local_entity_id);
                    let target_count = pkt.skill_damage_abnormal_move_events.len() as i32;
                    for mut event in pkt.skill_damage_abnormal_move_events.into_iter() {
                        if !damage_handler.decrypt_damage_event(&mut event.skill_damage_event) {
                            state.damage_is_valid = false;
                            continue;
                        }
                        let target_entity =
                            entity_tracker.get_or_create_entity(event.skill_damage_event.target_id);
                        let source_entity = entity_tracker.get_or_create_entity(pkt.source_id);

                        // track potential knockdown
                        state.on_abnormal_move(&target_entity, &event.skill_move_option_data, now);

                        let (se_on_source, se_on_target) = status_tracker
                            .borrow_mut()
                            .get_status_effects(&owner, &target_entity, local_character_id);
                        let damage_data = DamageData {
                            skill_id: pkt.skill_id,
                            skill_effect_id: pkt.skill_effect_id,
                            damage: event.skill_damage_event.damage,
                            shield_damage: event.skill_damage_event.shield_damage.p64_0,
                            modifier: event.skill_damage_event.modifier as i32,
                            target_current_hp: event.skill_damage_event.cur_hp,
                            target_max_hp: event.skill_damage_event.max_hp,
                            damage_attribute: event.skill_damage_event.damage_attr,
                            damage_type: event.skill_damage_event.damage_type,
                        };

                        state.on_damage(
                            app.clone(),
                            &owner,
                            &source_entity,
                            &target_entity,
                            damage_data,
                            se_on_source,
                            se_on_target,
                            target_count,
                            &entity_tracker,
                            now,
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
                    parse_pkt(&data, PKTSkillDamageNotify::new, )
                {
                    let now = Utc::now().timestamp_millis();
                    let owner = entity_tracker.get_source_entity(pkt.source_id);
                    let local_character_id = id_tracker
                        .borrow()
                        .get_local_character_id(entity_tracker.local_entity_id);
                    let target_count = pkt.skill_damage_events.len() as i32;
                    for mut event in pkt.skill_damage_events.into_iter() {
                        if !damage_handler.decrypt_damage_event(&mut event) {
                            state.damage_is_valid = false;
                            continue;
                        }
                        let target_entity = entity_tracker.get_or_create_entity(event.target_id);
                        // source_entity is to determine battle item
                        let source_entity = entity_tracker.get_or_create_entity(pkt.source_id);
                        let (se_on_source, se_on_target) = status_tracker
                            .borrow_mut()
                            .get_status_effects(&owner, &target_entity, local_character_id);
                        let damage_data = DamageData {
                            skill_id: pkt.skill_id,
                            skill_effect_id: pkt.skill_effect_id.unwrap_or_default(),
                            damage: event.damage,
                            shield_damage: event.shield_damage.p64_0,
                            modifier: event.modifier as i32,
                            target_current_hp: event.cur_hp,
                            target_max_hp: event.max_hp,
                            damage_attribute: event.damage_attr,
                            damage_type: event.damage_type,
                        };
                        state.on_damage(
                            app.clone(),
                            &owner,
                            &source_entity,
                            &target_entity,
                            damage_data,
                            se_on_source,
                            se_on_target,
                            target_count,
                            &entity_tracker,
                            now,
                        );
                    }
                }
            }
            Pkt::PartyInfo => {
                if let Some(pkt) = parse_pkt(&data, PKTPartyInfo::new, ) {
                    entity_tracker.party_info(pkt, &local_info);
                    let local_player_id = entity_tracker.local_entity_id;
                    if let Some(entity) = entity_tracker.entities.get(&local_player_id) {
                        state.update_local_player(entity);
                    }
                    party_cache = None;
                }
            }
            Pkt::PartyLeaveResult => {
                if let Some(pkt) = parse_pkt(&data, PKTPartyLeaveResult::new)
                {
                    party_tracker
                        .borrow_mut()
                        .remove(pkt.party_instance_id, pkt.name);
                    party_cache = None;
                }
            }
            Pkt::PartyStatusEffectAddNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTPartyStatusEffectAddNotify::new) {
                    // info!("{:?}", pkt);
                    let shields =
                        entity_tracker.party_status_effect_add(pkt, &state.encounter.entities);
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
                        // info!("SHIELD SOURCE: {} > TARGET: {}", source.name, target.name);
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
                if let Some(pkt) = parse_pkt(&data, PKTPartyStatusEffectRemoveNotify::new) {
                    let (is_shield, shields_broken, _effects_removed, _left_workshop) =
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
                if let Some(pkt) = parse_pkt(&data, PKTPartyStatusEffectResultNotify::new) {
                    // info!("{:?}", pkt);
                    party_tracker.borrow_mut().add(
                        pkt.raid_instance_id,
                        pkt.party_instance_id,
                        pkt.character_id,
                        0,
                        None,
                    );
                }
            }
            Pkt::StatusEffectAddNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTStatusEffectAddNotify::new) {
                    let status_effect = entity_tracker.build_and_register_status_effect(
                        &pkt.status_effect_data,
                        pkt.object_id,
                        Utc::now(),
                        Some(&state.encounter.entities),
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

                    if status_effect.status_effect_type == StatusEffectType::HardCrowdControl {
                        let target = entity_tracker.get_source_entity(status_effect.target_id);
                        if target.entity_type == EntityType::PLAYER {
                            state.on_cc_applied(&target, &status_effect);
                        }
                    }
                }
            }
            // Pkt::StatusEffectDurationNotify => {
            //     if let Some(pkt) = parse_pkt(
            //         &data,
            //         PKTStatusEffectDurationNotif:new,
            //         "PKTStatusEffectDurationNotify",
            //     ) {
            //         status_tracker.borrow_mut().update_status_duration(
            //             pkt.effect_instance_id,
            //             pkt.target_id,
            //             pkt.expiration_tick,
            //             StatusEffectTargetType::Local,
            //         );
            //     }
            // }
            Pkt::StatusEffectRemoveNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTStatusEffectRemoveNotify::new) {
                    let (is_shield, shields_broken, effects_removed, _left_workshop) =
                        status_tracker.borrow_mut().remove_status_effects(
                            pkt.object_id,
                            pkt.status_effect_instance_ids,
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
                    let now = Utc::now().timestamp_millis();
                    for effect_removed in effects_removed {
                        if effect_removed.status_effect_type == StatusEffectType::HardCrowdControl {
                            let target = entity_tracker.get_source_entity(effect_removed.target_id);
                            if target.entity_type == EntityType::PLAYER {
                                state.on_cc_removed(&target, &effect_removed, now);
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
                    state.on_phase_transition(app.clone(), 3, &mut stats_api);
                    debug_print(format_args!(
                        "phase: 3 - resetting encounter - TriggerBossBattleStatus"
                    ));
                }
            }
            Pkt::TriggerStartNotify => {
                if let Some(pkt) =
                    parse_pkt(&data, PKTTriggerStartNotify::new, )
                {
                    match pkt.signal {
                        57 | 59 | 61 | 63 | 74 | 76 => {
                            party_freeze = true;
                            state.party_info = if let Some(party) = party_cache.take() {
                                party
                            } else {
                                update_party(&party_tracker, &entity_tracker)
                            };
                            state.raid_clear = true;
                            state.on_phase_transition(app.clone(), 2, &mut stats_api);
                            raid_end_cd = Instant::now();
                            info!("phase: 2 - clear - TriggerStartNotify");
                        }
                        58 | 60 | 62 | 64 | 75 | 77 => {
                            party_freeze = true;
                            state.party_info = if let Some(party) = party_cache.take() {
                                party
                            } else {
                                update_party(&party_tracker, &entity_tracker)
                            };
                            state.raid_clear = false;
                            state.on_phase_transition(app.clone(), 4, &mut stats_api);
                            raid_end_cd = Instant::now();
                            info!("phase: 4 - wipe - TriggerStartNotify");
                        }
                        27 | 10 | 11 => {
                            // debug_print(format_args!("old rdps sync time - {}", pkt.trigger_signal_type));
                        }
                        _ => {}
                    }
                }
            }
            Pkt::ZoneMemberLoadStatusNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTZoneMemberLoadStatusNotify::new) {
                    if state.raid_difficulty_id >= pkt.zone_id && !state.raid_difficulty.is_empty()
                    {
                        continue;
                    }
                    debug_print(format_args!("raid zone id: {}", &pkt.zone_id));
                    debug_print(format_args!("raid zone id: {}", &pkt.zone_level));
                    match pkt.zone_level {
                        0 => {
                            state.raid_difficulty = "Normal".to_string();
                            state.raid_difficulty_id = 0;
                        }
                        1 => {
                            state.raid_difficulty = "Hard".to_string();
                            state.raid_difficulty_id = 1;
                        }
                        2 => {
                            state.raid_difficulty = "Inferno".to_string();
                            state.raid_difficulty_id = 2;
                        }
                        3 => {
                            state.raid_difficulty = "Challenge".to_string();
                            state.raid_difficulty_id = 3;
                        }
                        4 => {
                            state.raid_difficulty = "Solo".to_string();
                            state.raid_difficulty_id = 4;
                        }
                        5 => {
                            state.raid_difficulty = "The First".to_string();
                            state.raid_difficulty_id = 5;
                        }
                        _ => {}
                    }
                }
            }
            Pkt::ZoneObjectUnpublishNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTZoneObjectUnpublishNotify::new) {
                    status_tracker
                        .borrow_mut()
                        .remove_local_object(pkt.object_id);
                }
            }
            Pkt::StatusEffectSyncDataNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTStatusEffectSyncDataNotify::new) {
                    let (status_effect, old_value) =
                        status_tracker.borrow_mut().sync_status_effect(
                            pkt.status_effect_instance_id,
                            pkt.character_id,
                            pkt.object_id,
                            pkt.value,
                            entity_tracker.local_character_id,
                        );
                    if let Some(status_effect) = status_effect {
                        if status_effect.status_effect_type == StatusEffectType::Shield {
                            let change = old_value
                                .checked_sub(status_effect.value)
                                .unwrap_or_default();
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
                if let Some(pkt) = parse_pkt(&data, PKTTroopMemberUpdateMinNotify::new) {
                    // info!("{:?}", pkt);
                    if let Some(object_id) = id_tracker.borrow().get_entity_id(pkt.character_id) {
                        if let Some(entity) = entity_tracker.get_entity_ref(object_id) {
                            state
                                .encounter
                                .entities
                                .entry(entity.name.clone())
                                .and_modify(|e| {
                                    e.current_hp = pkt.cur_hp;
                                    e.max_hp = pkt.max_hp;
                                });
                        }
                        for se in pkt.status_effect_datas.iter() {
                            let val = get_status_effect_value(&se.value.bytearray_0);
                            let (status_effect, old_value) =
                                status_tracker.borrow_mut().sync_status_effect(
                                    se.status_effect_instance_id,
                                    pkt.character_id,
                                    object_id,
                                    val,
                                    entity_tracker.local_character_id,
                                );
                            if let Some(status_effect) = status_effect {
                                if status_effect.status_effect_type == StatusEffectType::Shield {
                                    let change = old_value
                                        .checked_sub(status_effect.value)
                                        .unwrap_or_default();
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
            Pkt::NewTransit => {
                if let Some(pkt) = parse_pkt(&data, PKTNewTransit::new, ) {
                    damage_handler.update_zone_instance_id(pkt.zone_instance_id);
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
            let damage_valid = state.damage_is_valid;
            let app_handle = app.app_handle();

            let party_info: Option<Vec<Vec<String>>> =
                if last_party_update.elapsed() >= party_duration && !party_freeze {
                    last_party_update = Instant::now();

                    // use cache if available
                    // otherwise get party info
                    party_cache.clone().or_else(|| {
                        let party = update_party(&party_tracker, &entity_tracker);
                        if party.len() > 1 {
                            if party.iter().all(|p| p.len() == 4) {
                                party_cache = Some(party.clone());
                            }
                            Some(party)
                        } else {
                            None
                        }
                    })
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
                    if !damage_valid {
                        app_handle
                            .emit_all("invalid-damage", "")
                            .expect("failed to emit invalid-damage");
                    } else {
                        app_handle
                            .emit_all("encounter-update", Some(clone))
                            .expect("failed to emit encounter-update");

                        if party_info.is_some() {
                            app_handle
                                .emit_all("party-update", party_info)
                                .expect("failed to emit party-update");
                        }
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
        }

        if heartbeat_api.can_send() {
            let args = HeartbeatSendArgs {
                region: region.clone(),
                client_id: client_id.clone(),
                version: version.clone()
            };

            heartbeat_api.send(args);
        }
    }

    Ok(())
}

fn update_party(
    party_tracker: &Rc<RefCell<PartyTracker>>,
    entity_tracker: &EntityTracker,
) -> Vec<Vec<String>> {
    let mut party_info = HashMap::new();

    for (entity_id, party_id) in &party_tracker.borrow().entity_id_to_party_id {
        let members = party_info.entry(*party_id).or_insert_with(Vec::new);
        if let Some(entity) = entity_tracker.entities.get(entity_id) {
            if entity.character_id > 0
                && entity.class_id > 0
                && entity
                    .name
                    .chars()
                    .next()
                    .unwrap_or_default()
                    .is_uppercase()
            {
                members.push(entity.name.clone());
            }
        }
    }

    let mut sorted_parties = party_info.into_iter().collect::<Vec<(u32, Vec<String>)>>();
    sorted_parties.sort_unstable_by_key(|&(party_id, _)| party_id);
    sorted_parties
        .into_iter()
        .map(|(_, members)| members)
        .collect()
}

fn on_shield_change(
    entity_tracker: &mut EntityTracker,
    id_tracker: &Rc<RefCell<IdTracker>>,
    state: &mut EncounterState,
    status_effect: StatusEffectDetails,
    change: u64,
) {
    if change == 0 {
        return;
    }
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

fn parse_pkt<T, F>(data: &[u8], new_fn: F) -> Option<T>
where
    F: FnOnce(&[u8]) -> Result<T, anyhow::Error>,
{
    match new_fn(data) {
        Ok(packet) => Some(packet),
        Err(e) => {
            warn!("Error parsing {}: {}", type_name::<T>(), e);
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
