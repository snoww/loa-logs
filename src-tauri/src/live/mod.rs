mod encounter_state;
mod entity_tracker;
mod id_tracker;
mod manager;
mod party_tracker;
mod skill_tracker;
mod stats_api;
mod status_tracker;
mod utils;

use crate::app;
use crate::live::encounter_state::EncounterState;
use crate::live::entity_tracker::{EntityTracker, get_current_and_max_hp};
use crate::live::id_tracker::IdTracker;
use crate::live::manager::EventManager;
use crate::live::party_tracker::PartyTracker;
use crate::live::stats_api::API_URL;
use crate::live::stats_api::StatsApi;
use crate::live::status_tracker::{
    StatusEffectDetails, StatusEffectTargetType, StatusEffectType, StatusTracker,
    get_status_effect_value,
};
use crate::local::{LocalInfo, LocalPlayer};
use crate::models::{DamageData, EntityType, Identity, RdpsData, TripodIndex};
use crate::settings::Settings;
use crate::utils::get_class_from_id;
use anyhow::Result;
use chrono::Utc;
use hashbrown::HashMap;
use log::{info, warn};
use meter_core::packets::definitions::*;
use meter_core::packets::opcodes::Pkt;
use meter_core::start_capture;
use reqwest::Client;
use serde_json::json;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

pub fn start(app: AppHandle, port: u16, settings: Option<Settings>) -> Result<()> {
    let manager = EventManager::new(app.clone());
    let id_tracker = Rc::new(RefCell::new(IdTracker::new()));
    let party_tracker = Rc::new(RefCell::new(PartyTracker::new(id_tracker.clone())));
    let status_tracker = Rc::new(RefCell::new(StatusTracker::new(party_tracker.clone())));
    let mut entity_tracker = EntityTracker::new(
        status_tracker.clone(),
        id_tracker.clone(),
        party_tracker.clone(),
    );
    let mut state = EncounterState::new(app.clone());
    let region_file_path = app::path::data_dir(&app).join("current_region");
    let mut stats_api = StatsApi::new(app.clone());
    let rx = match start_capture(port, region_file_path.display().to_string()) {
        Ok(rx) => rx,
        Err(e) => {
            warn!("Error starting capture: {e}");
            return Ok(());
        }
    };

    let damage_handler = meter_core::decryption::DamageEncryptionHandler::new();
    let damage_handler = damage_handler.start()?;

    let mut last_update = Instant::now();
    let mut duration = Duration::from_millis(200);
    let mut last_party_update = Instant::now();
    let party_duration = Duration::from_millis(2000);
    let mut raid_end_cd = Instant::now();

    let client = Client::new();
    let mut last_heartbeat = Instant::now();
    let heartbeat_duration = Duration::from_secs(60 * 15);

    if let Some(settings) = settings {
        if settings.general.boss_only_damage {
            manager.set_boss_only_damage();
            info!("boss only damage enabled")
        }
        if settings.general.low_performance_mode {
            duration = Duration::from_millis(1500);
            info!("low performance mode enabled")
        }
    }

    // read saved local players
    // this info is used in case meter was opened late
    let mut local_info: LocalInfo = LocalInfo::default();
    let local_player_path = app::path::data_dir(&app).join("local_players.json");
    let mut client_id = "".to_string();

    if local_player_path.exists() {
        let local_players_file = std::fs::read_to_string(local_player_path.clone())?;
        local_info = serde_json::from_str(&local_players_file).unwrap_or_default();
        client_id = local_info.client_id.clone();
        let valid_id = Uuid::try_parse(client_id.as_str()).is_ok();
        if client_id.is_empty() || !valid_id {
            client_id = Uuid::new_v4().to_string();
            stats_api.client_id.clone_from(&client_id);
            local_info.client_id.clone_from(&client_id);
            write_local_players(&local_info, &local_player_path)?;
        } else {
            stats_api.client_id.clone_from(&client_id);
        }
    }

    get_and_set_region(&region_file_path, &mut state);

    let mut party_freeze = false;
    let mut party_cache: Option<Vec<Vec<String>>> = None;

    while let Ok((op, data)) = rx.recv() {
        if manager.has_reset() {
            state.soft_reset(true);
        }

        if manager.has_paused() {
            continue;
        }

        if manager.has_saved() {
            state.party_info = update_party(&party_tracker, &entity_tracker);
            state.save_to_db(&mut stats_api, true);
            state.saved = true;
            state.resetting = true;
        }

        if manager.has_toggled_boss_only_damage() {
            state.boss_only_damage = true;
        } else {
            state.boss_only_damage = false;
            state.encounter.boss_only_damage = false;
        }

        match op {
            Pkt::BattleItemUseNotify => {
                if let Some(pkt) =
                    parse_pkt(&data, PKTBattleItemUseNotify::new, "PKTBattleItemUseNotify")
                {
                    state.on_battle_item_use(&pkt.item_id);
                }
            }
            Pkt::CounterAttackNotify => {
                if let Some(pkt) =
                    parse_pkt(&data, PKTCounterAttackNotify::new, "PKTCounterAttackNotify")
                    && let Some(entity) = entity_tracker.entities.get(&pkt.source_id)
                {
                    state.on_counterattack(entity);
                }
            }
            Pkt::DeathNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTDeathNotify::new, "PKTDeathNotify")
                    && let Some(entity) = entity_tracker.entities.get(&pkt.target_id)
                {
                    debug_print(format_args!(
                        "death: {}, {}, {}",
                        entity.name, entity.entity_type, entity.id
                    ));
                    state.on_death(entity);
                }
            }
            Pkt::IdentityGaugeChangeNotify => {
                if let Some(pkt) = parse_pkt(
                    &data,
                    PKTIdentityGaugeChangeNotify::new,
                    "PKTIdentityGaugeChangeNotify",
                ) && manager.can_emit_details()
                {
                    app.emit(
                        "identity-update",
                        Identity {
                            gauge1: pkt.identity_gauge1,
                            gauge2: pkt.identity_gauge2,
                            gauge3: pkt.identity_gauge3,
                        },
                    )?;
                }
            }
            // Pkt::IdentityStanceChangeNotify => {
            //     if let Some(pkt) = parse_pkt(
            //         &data,
            //         PKTIdentityStanceChangeNotify::new,
            //         "PKTIdentityStanceChangeNotify",
            //     ) {
            //         if let Some(entity) = entity_tracker.entities.get_mut(&pkt.object_id) {
            //             if entity.entity_type == EntityType::Player {
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

                if let Some(pkt) = parse_pkt(&data, PKTInitEnv::new, "PKTInitEnv") {
                    party_tracker.borrow_mut().reset_party_mappings();
                    party_cache = None;
                    let entity = entity_tracker.init_env(pkt);
                    state.on_init_env(entity, &stats_api);
                    get_and_set_region(&region_file_path, &mut state);
                    info!("region: {:?}", state.region);
                }
            }
            Pkt::InitPC => {
                if let Some(pkt) = parse_pkt(&data, PKTInitPC::new, "PKTInitPC") {
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

                    write_local_players(&local_info, &local_player_path)?;

                    state.on_init_pc(entity, hp, max_hp)
                }
            }
            // Pkt::InitItem => {
            //     if let Some(pkt) = parse_pkt(&data, PKTInitItem::new, "PKTInitItem") {
            //         if pkt.storage_type == 1 || pkt.storage_type == 20 {
            //             entity_tracker.get_local_player_set_options(pkt.item_data_list);
            //         }
            //     }
            // }
            // Pkt::MigrationExecute => {
            //     if let Some(pkt) = parse_pkt(&data, PKTMigrationExecute::new, "PKTMigrationExecute")
            //     {
            //         entity_tracker.migration_execute(pkt);
            //         get_and_set_region(region_file_path.as_ref(), &mut state);
            //     }
            // }
            Pkt::NewPC => {
                if let Some(pkt) = parse_pkt(&data, PKTNewPC::new, "PKTNewPC") {
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
                if let Some(pkt) = parse_pkt(&data, PKTNewVehicle::new, "PKTNewVehicle")
                    && let Some(pc_struct) = pkt.vehicle_struct.p_c_struct_conditional.p_c_struct
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
            Pkt::NewNpc => {
                if let Some(pkt) = parse_pkt(&data, PKTNewNpc::new, "PKTNewNpc") {
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
                if let Some(pkt) = parse_pkt(&data, PKTNewNpcSummon::new, "PKTNewNpcSummon") {
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
                if let Some(pkt) = parse_pkt(&data, PKTNewProjectile::new, "PKTNewProjectile") {
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
                if let Some(pkt) = parse_pkt(&data, PKTNewTrap::new, "PKTNewTrap") {
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
            //         PKTParalyzationStateNotify::new,
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
                if let Some(pkt) = parse_pkt(&data, PKTRaidBegin::new, "PKTRaidBegin") {
                    debug_print(format_args!("raid begin: {}", pkt.raid_id));
                    match pkt.raid_id {
                        308226 | 308227 | 308239 | 308339 => {
                            state.raid_difficulty = "Trial".to_string().into();
                        }
                        308428 | 308429 | 308420 | 308410 | 308411 | 308414 | 308422 | 308424
                        | 308421 | 308412 | 308423 | 308426 | 308416 | 308419 | 308415 | 308437
                        | 308417 | 308418 | 308425 | 308430 => {
                            state.raid_difficulty = "Challenge".to_string().into();
                        }
                        _ => {
                            state.raid_difficulty = None;
                        }
                    }
                }
            }
            Pkt::RaidBossKillNotify => {
                state.on_phase_transition(1, &mut stats_api);
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
                state.on_phase_transition(0, &mut stats_api);
                raid_end_cd = Instant::now();
                info!("phase: 0 - RaidResult");
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
                    parse_pkt(&data, PKTSkillCooldownNotify::new, "PKTSkillCooldownNotify")
                {
                    state.on_skill_cooldown(pkt.skill_cooldown_struct);
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
                    let timestamp = Utc::now().timestamp_millis();
                    let (skill_id, summon_source) =
                        state.on_skill_start(&entity, pkt.skill_id, tripod_index, timestamp);

                    if entity.entity_type == EntityType::Player && skill_id > 0 {
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
                    PKTSkillDamageAbnormalMoveNotify::new,
                    "PKTSkillDamageAbnormalMoveNotify",
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

                        let mut rdps_data = Vec::new();

                        if let Some(rdps) = event.skill_damage_event.rdps_data_conditional.rdps_data
                        {
                            for i in 0..rdps.event_type.len() {
                                rdps_data.push(RdpsData {
                                    rdps_type: rdps.event_type[i],
                                    value: rdps.value[i],
                                    source_character_id: rdps.source_character_id[i],
                                    skill_id: rdps.skill_id[i],
                                });
                            }
                        }

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
                            stagger: event.skill_damage_event.stagger_amount,
                            rdps_data,
                        };

                        state.on_damage(
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
                    parse_pkt(&data, PKTSkillDamageNotify::new, "PktSkillDamageNotify")
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
                        let mut rdps_data = Vec::new();

                        if let Some(rdps) = event.rdps_data_conditional.rdps_data {
                            for i in 0..rdps.event_type.len() {
                                rdps_data.push(RdpsData {
                                    rdps_type: rdps.event_type[i],
                                    value: rdps.value[i],
                                    source_character_id: rdps.source_character_id[i],
                                    skill_id: rdps.skill_id[i],
                                });
                            }
                        }

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
                            stagger: event.stagger_amount,
                            rdps_data,
                        };
                        state.on_damage(
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
                if let Some(pkt) = parse_pkt(&data, PKTPartyInfo::new, "PKTPartyInfo") {
                    entity_tracker.party_info(pkt, &local_info);
                    let local_player_id = entity_tracker.local_entity_id;
                    if let Some(entity) = entity_tracker.entities.get(&local_player_id) {
                        state.update_local_player(entity);
                    }
                    party_cache = None;
                }
            }
            Pkt::PartyLeaveResult => {
                if let Some(pkt) = parse_pkt(&data, PKTPartyLeaveResult::new, "PKTPartyLeaveResult")
                {
                    party_tracker
                        .borrow_mut()
                        .remove(pkt.party_instance_id, pkt.name);
                    party_cache = None;
                }
            }
            Pkt::PartyStatusEffectAddNotify => {
                if let Some(pkt) = parse_pkt(
                    &data,
                    PKTPartyStatusEffectAddNotify::new,
                    "PKTPartyStatusEffectAddNotify",
                ) {
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
                if let Some(pkt) = parse_pkt(
                    &data,
                    PKTPartyStatusEffectRemoveNotify::new,
                    "PKTPartyStatusEffectRemoveNotify",
                ) {
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
                if let Some(pkt) = parse_pkt(
                    &data,
                    PKTPartyStatusEffectResultNotify::new,
                    "PKTPartyStatusEffectResultNotify",
                ) {
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
                if let Some(pkt) = parse_pkt(
                    &data,
                    PKTStatusEffectAddNotify::new,
                    "PKTStatusEffectAddNotify",
                ) {
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
                        if target.entity_type == EntityType::Player {
                            state.on_cc_applied(&target, &status_effect);
                        }
                    }
                }
            }
            // Pkt::StatusEffectDurationNotify => {
            //     if let Some(pkt) = parse_pkt(
            //         &data,
            //         PKTStatusEffectDurationNotify::new,
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
                if let Some(pkt) = parse_pkt(
                    &data,
                    PKTStatusEffectRemoveNotify::new,
                    "PKTStatusEffectRemoveNotify",
                ) {
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
                            if target.entity_type == EntityType::Player {
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
                    state.on_phase_transition(3, &mut stats_api);
                    debug_print(format_args!(
                        "phase: 3 - resetting encounter - TriggerBossBattleStatus"
                    ));
                }
            }
            Pkt::TriggerStartNotify => {
                if let Some(pkt) =
                    parse_pkt(&data, PKTTriggerStartNotify::new, "PKTTriggerStartNotify")
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
                            state.on_phase_transition(2, &mut stats_api);
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
                            state.on_phase_transition(4, &mut stats_api);
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
                if let Some(pkt) = parse_pkt(
                    &data,
                    PKTZoneMemberLoadStatusNotify::new,
                    "PKTZoneMemberLoadStatusNotify",
                ) {
                    if state.raid_difficulty.is_some() {
                        continue;
                    }
                    debug_print(format_args!("raid zone id: {}", &pkt.zone_id));
                    debug_print(format_args!("raid zone id: {}", &pkt.zone_level));
                    match pkt.zone_level {
                        0 => {
                            state.raid_difficulty = "Normal".to_string().into();
                        }
                        1 => {
                            state.raid_difficulty = "Hard".to_string().into();
                        }
                        2 => {
                            state.raid_difficulty = "Inferno".to_string().into();
                        }
                        3 => {
                            state.raid_difficulty = "Challenge".to_string().into();
                        }
                        4 => {
                            state.raid_difficulty = "Solo".to_string().into();
                        }
                        5 => {
                            state.raid_difficulty = "The First".to_string().into();
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
                            pkt.status_effect_instance_id,
                            pkt.character_id,
                            pkt.object_id,
                            pkt.value,
                            entity_tracker.local_character_id,
                        );
                    if let Some(status_effect) = status_effect
                        && status_effect.status_effect_type == StatusEffectType::Shield
                    {
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
            Pkt::TroopMemberUpdateMinNotify => {
                if let Some(pkt) = parse_pkt(
                    &data,
                    PKTTroopMemberUpdateMinNotify::new,
                    "PKTTroopMemberUpdateMinNotify",
                ) {
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
                            if let Some(status_effect) = status_effect
                                && status_effect.status_effect_type == StatusEffectType::Shield
                            {
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
            Pkt::NewTransit => {
                if let Some(pkt) = parse_pkt(&data, PKTNewTransit::new, "PKTNewZoneKey") {
                    debug_print(format_args!("transit zone id: {}", &pkt.zone_id));
                    state.raid_difficulty = "".to_string().into();
                    state.damage_is_valid = true;
                    damage_handler.update_zone_instance_id(pkt.zone_instance_id);
                    state.on_transit();
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
            let app_handle = app.clone();

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
                    ((e.entity_type == EntityType::Player && e.class_id > 0)
                        || e.entity_type == EntityType::Esther
                        || e.entity_type == EntityType::Boss)
                        && e.damage_stats.damage_dealt > 0
                });

                if !clone.entities.is_empty() {
                    if !damage_valid {
                        app_handle
                            .emit("invalid-damage", "")
                            .expect("failed to emit invalid-damage");
                    } else {
                        app_handle
                            .emit("encounter-update", Some(clone))
                            .expect("failed to emit encounter-update");

                        if party_info.is_some() {
                            app_handle
                                .emit("party-update", party_info)
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

        if last_heartbeat.elapsed() >= heartbeat_duration {
            let client = client.clone();
            let client_id = client_id.clone();
            let version = app.package_info().version.to_string();
            let region = match state.region {
                Some(ref region) => region.clone(),
                None => continue,
            };
            tokio::task::spawn(async move {
                let request_body = json!({
                    "id": client_id,
                    "version": version,
                    "region": region,
                });

                match client
                    .post(format!("{API_URL}/analytics/heartbeat"))
                    .json(&request_body)
                    .send()
                    .await
                {
                    Ok(_) => {
                        debug_print(format_args!("sent heartbeat"));
                    }
                    Err(e) => {
                        warn!("failed to send heartbeat: {:?}", e);
                    }
                }
            });
            last_heartbeat = Instant::now();
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
        if let Some(entity) = entity_tracker.entities.get(entity_id)
            && entity.character_id > 0
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

fn write_local_players(local_info: &LocalInfo, path: &PathBuf) -> Result<()> {
    let local_players_file = serde_json::to_string(local_info)?;
    std::fs::write(path, local_players_file)?;
    Ok(())
}

fn get_and_set_region<P: AsRef<Path>>(path: P, state: &mut EncounterState) {
    match std::fs::read_to_string(path) {
        Ok(region) => {
            state.region = Some(region.clone());
            state.encounter.region = Some(region);
        }
        Err(_) => {
            // warn!("failed to read region file. {}", e);
        }
    }
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
