use crate::emitter::AppEmitter;
use crate::live::encounter_state::EncounterState;
use crate::live::entity_tracker::{EntityTracker, get_current_and_max_hp};
use crate::live::id_tracker::IdTracker;
use crate::live::manager::EventManager;
use crate::live::packet::LoaPacket;
use crate::live::party_tracker::PartyTracker;
use crate::live::sntp::{SntpTimeSyncClient, TimeSyncClient};
use crate::live::status_tracker::{StatusTracker, get_status_effect_value,};
use crate::live::{EncounterService, PacketCapture};
use crate::local::{LocalInfo, LocalPlayer, LocalPlayerRepository};
use crate::models::*;
use crate::settings::Settings;
use crate::utils::get_class_from_id;
use anyhow::Result;
use chrono::Utc;
use hashbrown::HashMap;
use log::*;
use meter_core::packets::structures::SkillDamageEvent;
use tokio::sync::watch;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

pub struct StartArgs<C, R, E, M, S>
where
    C: PacketCapture,
    R: EncounterService,
    E: AppEmitter,
    M: EventManager,
    S: TimeSyncClient
{
    pub capture: C,
    pub encounter_service: R,
    pub emitter: E,
    pub settings: Option<Settings>,
    pub shutdown_rx: watch::Receiver<bool>,
    pub local_info: LocalInfo,
    pub local_player_repository: LocalPlayerRepository,
    pub manager: M,
    pub sntp_client: S
    // pub heartbeat_api: HeartBeatApi,
}

pub fn start<C, R, E, M, S>(args: StartArgs<C, R, E, M, S>) -> Result<()>
where
    C: PacketCapture,
    R: EncounterService,
    E: AppEmitter,
    M: EventManager,
    S: TimeSyncClient,
{
    let StartArgs {
        mut capture,
        encounter_service,
        emitter,
        sntp_client,
        settings,
        shutdown_rx,
        mut local_info,
        local_player_repository,
        manager
        // heartbeat_api,
    } = args;

    capture.start()?;

    let region_accessor = capture.create_region_accessor();
   
    let id_tracker = Rc::new(RefCell::new(IdTracker::new()));
    let party_tracker = Rc::new(RefCell::new(PartyTracker::new(id_tracker.clone())));
    let status_tracker = Rc::new(RefCell::new(StatusTracker::new(party_tracker.clone())));
    let mut entity_tracker = EntityTracker::new(
        status_tracker.clone(),
        id_tracker.clone(),
        party_tracker.clone(),
    );
    
    let mut state = EncounterState::new(sntp_client, &encounter_service, &emitter);

    let mut last_update: Option<Instant> = None;
    let mut duration = Duration::from_millis(200);
    let mut last_party_update = Instant::now();
    let party_duration = Duration::from_millis(2000);
    let mut raid_end_cd: Option<Instant> = None;
    let damage_ignore_after_end = Duration::from_secs(10); 

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

    let mut party_freeze = false;
    let mut party_cache: Option<Vec<Vec<String>>> = None;

    while let Ok(packet) = capture.recv() {

        if manager.has_reset() {
            state.soft_reset(true);
        }

        if manager.has_paused() {
            continue;
        }

        if manager.has_saved() {
            state.party_info = update_party(&party_tracker, &entity_tracker);
            state.save_to_db(true);
            state.saved = true;
            state.resetting = true;
        }

        if manager.has_toggled_boss_only_damage() {
            state.boss_only_damage = true;
        } else {
            state.boss_only_damage = false;
            state.encounter.boss_only_damage = false;
        }

        // use this to make sure damage packets are not tracked after a raid just wiped
        let can_ignore = raid_end_cd
            .map(|t| Instant::now() - t < damage_ignore_after_end)
            .unwrap_or(false);

        match packet {
            LoaPacket::CounterAttackNotify(data) => {
                if let Some(entity) = entity_tracker.entities.get(&data.source_id) {
                    state.on_counterattack(entity);
                }
            }
            LoaPacket::DeathNotify(data) => {
                if let Some(entity) = entity_tracker.entities.get(&data.target_id) {
                    info!(
                        "death: {}, {}, {}",
                        entity.name, entity.entity_type, entity.id
                    );
                    state.on_death(entity);
                }
            }
            LoaPacket::IdentityGaugeChangeNotify(data) => {
                if manager.can_emit_details() {
                    emitter.emit(
                        "identity-update",
                        Identity {
                            gauge1: data.identity_gauge1,
                            gauge2: data.identity_gauge2,
                            gauge3: data.identity_gauge3,
                        },
                    );
                }
            }
            LoaPacket::InitEnv(pkt) => {
                party_tracker.borrow_mut().reset_party_mappings();
                state.raid_difficulty = "".to_string();
                state.raid_difficulty_id = 0;
                state.damage_is_valid = true;
                party_cache = None;
                let entity = entity_tracker.init_env(pkt);
                state.on_init_env(entity);
                
                if let Some(region) = region_accessor.get() {
                    state.region = Some(region.clone());
                    state.encounter.region = Some(region);
                }

                info!("region: {:?}", state.region);
            }
            LoaPacket::InitPC(pkt) => {
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
            LoaPacket::NewPC(pkt) => {
                let (hp, max_hp) = get_current_and_max_hp(&pkt.pc_struct.stat_pairs);
                let entity = entity_tracker.new_pc(pkt.pc_struct);
                info!(
                    "new PC: {}, {}, {}, eid: {}, cid: {}",
                    entity.name,
                    get_class_from_id(&entity.class_id),
                    entity.gear_level,
                    entity.id,
                    entity.character_id
                );
                state.on_new_pc(entity, hp, max_hp);
            }
            LoaPacket::NewVehicle(pkt) => {
                if let Some(pc_struct) = pkt.vehicle_struct.p_c_struct_conditional.p_c_struct {
                    let (hp, max_hp) = get_current_and_max_hp(&pc_struct.stat_pairs);
                    let entity = entity_tracker.new_pc(pc_struct);
                    info!(
                        "new PC from vehicle: {}, {}, {}, eid: {}, cid: {}",
                        entity.name,
                        get_class_from_id(&entity.class_id),
                        entity.gear_level,
                        entity.id,
                        entity.character_id
                    );
                    state.on_new_pc(entity, hp, max_hp);
                }
            }
            LoaPacket::NewNpc(pkt) => {
                 let (hp, max_hp) = get_current_and_max_hp(&pkt.npc_struct.stat_pairs);
                let entity = entity_tracker.new_npc(pkt, max_hp);
                info!(
                    "new {}: {}, eid: {}, id: {}, hp: {}",
                    entity.entity_type, entity.name, entity.id, entity.npc_id, max_hp
                );
                state.on_new_npc(entity, hp, max_hp);
            }
            LoaPacket::NewNpcSummon(pkt) => {
                 let (hp, max_hp) = get_current_and_max_hp(&pkt.npc_struct.stat_pairs);
                let entity = entity_tracker.new_npc_summon(pkt, max_hp);
                info!(
                    "new {}: {}, eid: {}, id: {}, hp: {}",
                    entity.entity_type, entity.name, entity.id, entity.npc_id, max_hp
                );
                state.on_new_npc(entity, hp, max_hp);
            }
            LoaPacket::NewProjectile(pkt) => {
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
            LoaPacket::NewTrap(pkt) => {
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
            LoaPacket::RaidBegin(pkt) => {
                info!("raid begin: {}", pkt.raid_id);
                let raid_difficulty = RaidDifficulty::from_raid_id(pkt.raid_id);
                state.raid_difficulty = raid_difficulty.as_ref().to_string();
                state.raid_difficulty_id = raid_difficulty as u32;
            }
            LoaPacket::RaidBossKillNotify => {
                state.on_phase_transition(1);
                state.raid_clear = true;
                info!("phase: 1 - RaidBossKillNotify");
            }
            LoaPacket::RaidResult => {
                party_freeze = true;
                state.party_info = if let Some(party) = party_cache.take() {
                    party
                } else {
                    update_party(&party_tracker, &entity_tracker)
                };
                state.on_phase_transition(0);
                raid_end_cd = Some(Instant::now());
                info!("phase: 0 - RaidResult");
            }
            LoaPacket::RemoveObject(pkt) => {
                for upo in pkt.unpublished_objects {
                    entity_tracker.entities.remove(&upo.object_id);
                    status_tracker
                        .borrow_mut()
                        .remove_local_object(upo.object_id);
                }
            }
            LoaPacket::SkillCastNotify(pkt) => {
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
            LoaPacket::SkillCooldownNotify(pkt) => {
                state.on_skill_cooldown(pkt.skill_cooldown_struct);
            }
            LoaPacket::SkillStartNotify(pkt) => {
                let mut entity = entity_tracker.get_source_entity(pkt.source_id);
                entity_tracker.guess_is_player(&mut entity, pkt.skill_id);
                let tripod_index =
                    pkt.skill_option_data
                        .tripod_index
                        .map(|tripod_index| crate::models::TripodIndex {
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
            LoaPacket::SkillDamageAbnormalMoveNotify(pkt) => {
                if can_ignore {
                    info!("ignoring damage - SkillDamageAbnormalMoveNotify");
                    continue;
                }

                let now = Utc::now().timestamp_millis();
                let owner = entity_tracker.get_source_entity(pkt.source_id);
                let local_character_id = id_tracker
                    .borrow()
                    .get_local_character_id(entity_tracker.local_entity_id);

                for mut event in pkt.skill_damage_abnormal_move_events.into_iter() {
                    if !capture.decrypt_damage_event(&mut event.skill_damage_event) {
                        state.damage_is_valid = false;
                        continue;
                    }

                    let SkillDamageEvent {
                        cur_hp,
                        damage,
                        max_hp,
                        modifier,
                        rdps_data_conditional,
                        shield_damage,
                        stagger_amount,
                        target_id,
                        ..
                    } = event.skill_damage_event;

                    let target_entity =
                        entity_tracker.get_or_create_entity(target_id);
                    let source_entity = entity_tracker.get_or_create_entity(pkt.source_id);

                    // track potential knockdown
                    state.on_abnormal_move(&target_entity, &event.skill_move_option_data, now);

                    let (se_on_source, se_on_target) = status_tracker
                        .borrow_mut()
                        .get_status_effects(&owner, &target_entity, local_character_id);

                    let se_on_source_ids = se_on_source
                        .into_iter()
                        .map(|se| se.id)
                        .collect::<Vec<_>>();
                    let se_on_target_ids = se_on_target
                        .into_iter()
                        .map(|se| se.id)
                        .collect::<Vec<_>>();

                     let rdps_data: Vec<_> = rdps_data_conditional
                        .rdps_data
                        .map(|rdps| {
                            rdps.event_type
                                .iter()
                                .enumerate()
                                .map(|(i, &rdps_type)| RdpsData {
                                    rdps_type,
                                    value: rdps.value[i],
                                    source_character_id: rdps.source_character_id[i],
                                    skill_id: rdps.skill_id[i],
                                })
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();

                    let HitInfo(hit_option, hit_flag) = match HitInfo::try_from(modifier) {
                        Ok(value) => value,
                        Err(_) => continue,
                    };

                    let context = DamageEventContext {
                        damage: damage + shield_damage.p64_0.unwrap_or_default(),
                        stagger: stagger_amount as i64,
                        skill_id: pkt.skill_id,
                        skill_effect_id: pkt.skill_effect_id,
                        target_current_hp: cur_hp,
                        target_max_hp: max_hp,
                        rdps_data,
                        dmg_src_entity: &owner,
                        proj_entity: &source_entity,
                        dmg_target_entity: &target_entity,
                        se_on_source_ids,
                        se_on_target_ids,
                        hit_flag,
                        hit_option,
                        timestamp: now,
                        character_id_to_name: &entity_tracker.character_id_to_name,
                    };

                    state.on_damage(context);
                }
            }
            LoaPacket::SkillDamageNotify(pkt) => {
                if can_ignore {
                    info!("ignoring damage - SkillDamageNotify");
                    continue;
                }

                let now = Utc::now().timestamp_millis();
                let owner = entity_tracker.get_source_entity(pkt.source_id);
                let local_character_id = id_tracker
                    .borrow()
                    .get_local_character_id(entity_tracker.local_entity_id);

                for mut event in pkt.skill_damage_events.into_iter() {
                    if !capture.decrypt_damage_event(&mut event) {
                        state.damage_is_valid = false;
                        continue;
                    }

                    let SkillDamageEvent {
                        cur_hp,
                        damage,
                        max_hp,
                        modifier,
                        rdps_data_conditional,
                        shield_damage,
                        stagger_amount,
                        target_id,
                        ..
                    } = event;

                    let target_entity = entity_tracker.get_or_create_entity(target_id);
                    // source_entity is to determine battle item
                    let source_entity = entity_tracker.get_or_create_entity(pkt.source_id);
                    let (se_on_source, se_on_target) = status_tracker
                        .borrow_mut()
                        .get_status_effects(&owner, &target_entity, local_character_id);

                    let se_on_source_ids = se_on_source
                        .into_iter()
                        .map(|se| se.id)
                        .collect::<Vec<_>>();
                    let se_on_target_ids = se_on_target
                        .into_iter()
                        .map(|se| se.id)
                        .collect::<Vec<_>>();

                    let rdps_data: Vec<_> = rdps_data_conditional
                        .rdps_data
                        .map(|rdps| {
                            rdps.event_type
                                .iter()
                                .enumerate()
                                .map(|(i, &rdps_type)| RdpsData {
                                    rdps_type,
                                    value: rdps.value[i],
                                    source_character_id: rdps.source_character_id[i],
                                    skill_id: rdps.skill_id[i],
                                })
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();

                    let HitInfo(hit_option, hit_flag) = match HitInfo::try_from(modifier) {
                        Ok(value) => value,
                        Err(_) => continue,
                    };

                    let context = DamageEventContext {
                        damage: damage + shield_damage.p64_0.unwrap_or_default(),
                        stagger: stagger_amount as i64,
                        skill_id: pkt.skill_id,
                        skill_effect_id: pkt.skill_effect_id.unwrap_or_default(),
                        target_current_hp: cur_hp,
                        target_max_hp: max_hp,
                        rdps_data,
                            dmg_src_entity: &owner,
                        proj_entity: &source_entity,
                        dmg_target_entity: &target_entity,
                        se_on_source_ids,
                        se_on_target_ids,
                        hit_flag,
                        hit_option,
                        timestamp: now,
                        character_id_to_name: &entity_tracker.character_id_to_name,
                    };
                    
                    state.on_damage(context);
                }
            }
            LoaPacket::PartyInfo(pkt) => {
                entity_tracker.party_info(pkt, &local_info);
                let local_player_id = entity_tracker.local_entity_id;
                if let Some(entity) = entity_tracker.entities.get(&local_player_id) {
                    state.update_local_player(entity);
                }
                party_cache = None;
            }
            LoaPacket::PartyLeaveResult(pkt) => {
                party_tracker
                    .borrow_mut()
                    .remove(pkt.party_instance_id, pkt.name);
                party_cache = None;
            }
            LoaPacket::PartyStatusEffectAddNotify(pkt) => {
                let shields =
                    entity_tracker.party_status_effect_add(
                        pkt,
                        &state.encounter.entities,
                        &mut state.custom_id_map);
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
            LoaPacket::PartyStatusEffectRemoveNotify(pkt) => {
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
            LoaPacket::PartyStatusEffectResultNotify(pkt) => {
                party_tracker.borrow_mut().add(
                    pkt.raid_instance_id,
                    pkt.party_instance_id,
                    pkt.character_id,
                    0,
                    None,
                );
            }
            LoaPacket::StatusEffectAddNotify(pkt) => {
                let status_effect = entity_tracker.build_and_register_status_effect(
                    &pkt.status_effect_data,
                    pkt.object_id,
                    Utc::now(),
                    Some(&state.encounter.entities),
                    &mut state.custom_id_map
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
            LoaPacket::StatusEffectRemoveNotify(pkt) => {
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
            LoaPacket::TriggerBossBattleStatus => {
                // need to hard code clown because it spawns before the trigger is sent???
                if state.encounter.current_boss_name.is_empty()
                    || state.encounter.fight_start == 0
                    || state.encounter.current_boss_name == "Saydon"
                {
                    state.on_phase_transition(3);
                    info!(
                        "phase: 3 - resetting encounter - TriggerBossBattleStatus"
                    );
                }   
            }
            LoaPacket::TriggerStartNotify(pkt) => {
                let signal = TriggerSignal::from(pkt.signal);

                match signal {
                    TriggerSignal::Clear(_) => {
                        party_freeze = true;
                        state.party_info = if let Some(party) = party_cache.take() {
                            party
                        } else {
                            update_party(&party_tracker, &entity_tracker)
                        };
                        state.raid_clear = true;
                        state.on_phase_transition(2);
                        raid_end_cd = Some(Instant::now());
                        info!("phase clear - TriggerStartNotify");
                    }
                    TriggerSignal::Wipe(_) => {
                        party_freeze = true;
                        state.party_info = if let Some(party) = party_cache.take() {
                            party
                        } else {
                            update_party(&party_tracker, &entity_tracker)
                        };
                        state.raid_clear = false;
                        state.on_phase_transition(4);
                        raid_end_cd = Some(Instant::now());
                        info!("phase wipe - TriggerStartNotify");
                    }
                    TriggerSignal::Unknown(_) => {
                        // debug_print(format_args!("old rdps sync time - {}", pkt.trigger_signal_type));
                    }
                }
            }
            LoaPacket::ZoneMemberLoadStatusNotify(pkt) => {
                if state.raid_difficulty_id >= pkt.zone_id && !state.raid_difficulty.is_empty() {
                    continue;
                }

                info!("raid zone id: {}, level: {}", &pkt.zone_id, pkt.zone_level);
                
                let raid_difficulty = RaidDifficulty::from(pkt.zone_level);
                state.raid_difficulty = raid_difficulty.as_ref().to_string();
                state.raid_difficulty_id = raid_difficulty as u32;
            }
            LoaPacket::ZoneObjectUnpublishNotify(pkt) => {
                status_tracker
                    .borrow_mut()
                    .remove_local_object(pkt.object_id);
            }
            LoaPacket::StatusEffectSyncDataNotify(pkt) => {
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
            LoaPacket::TroopMemberUpdateMinNotify(pkt) => {
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
                    for se in pkt.status_effect_datas {
                        let val = get_status_effect_value(se.value.bytearray_0);
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
            LoaPacket::NewTransit(pkt) => {
                capture.update_zone_instance_id(pkt.zone_instance_id);
            }
            _ => {}
        }

        let boss_dead = state.boss_dead_update;
        let can_send_to_ui = match last_update {
            Some(last) => last.elapsed() >= duration || state.resetting || state.boss_dead_update,
            None => true,
        };

        if can_send_to_ui {
            let clone = state.encounter.clone();
            let damage_valid = state.damage_is_valid;
            let should_refresh_party_info = last_party_update.elapsed() >= party_duration && !party_freeze;

            if should_refresh_party_info {
                last_party_update = Instant::now();
            }
            
           let party_info: Option<Vec<Vec<String>>> = match (should_refresh_party_info, party_cache.clone()) {
                (true, None) => {
                    let party = update_party(&party_tracker, &entity_tracker);

                    if party.len() > 1 {
                        if party.iter().all(|p| p.len() == 4) {
                            party_cache = Some(party.clone());
                        }
                        
                        Some(party)
                    } else {
                        None
                    }
                },
                (true, Some(party_cache)) => Some(party_cache),
                _ => None
            };

            encounter_service.send(boss_dead, damage_valid, clone, party_info);

            last_update = Some(Instant::now());
        }

        if state.boss_dead_update {
            state.boss_dead_update = false;
        }

        if state.resetting {
            state.soft_reset(true);
            state.resetting = false;
            state.saved = false;
            party_freeze = false;
            party_cache = None;
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

fn on_shield_change<T: TimeSyncClient, R: EncounterService, E: AppEmitter>(
    entity_tracker: &mut EntityTracker,
    id_tracker: &Rc<RefCell<IdTracker>>,
    state: &mut EncounterState<T, R, E>,
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use meter_core::packets::{
        definitions::{PKTNewNpc, PKTNewPC, PKTSkillDamageNotify, PKTTriggerStartNotify},
        structures::{NpcStruct, PCStruct, StatPair},
    };
    use mockall::predicate::{always, eq};
    use tokio::sync::watch;

    use crate::{
        data::AssetPreloader,
        emitter::MockAppEmitter,
        live::*,
        region::MockRegionAccessor,
    };

    use super::*;

    fn setup_mock_capture(call_count_limit: usize) -> MockPacketCapture {
        let mut capture = MockPacketCapture::new();

        capture.expect_start()
            .times(1)
            .returning(|| Ok(()));

        capture.expect_create_region_accessor()
            .times(1)
            .returning(|| Box::new(MockRegionAccessor::new()));

        let mut call_count = 0;
        capture.expect_recv()
            .times(call_count_limit)
            .returning(move || {
                call_count += 1;
                match call_count {
                    1 => Ok(LoaPacket::NewPC(PKTNewPC {
                        pc_struct: PCStruct {
                            player_id: 1,
                            name: "Player".into(),
                            character_id: 1,
                            class_id: 101,
                            gear_level: 1700.0,
                            stat_pairs: vec![],
                            max_item_level: 1700.0,
                            status_effect_datas: vec![],
                        },
                    })),
                    2 => Ok(LoaPacket::NewNpc(PKTNewNpc {
                        npc_struct: NpcStruct {
                            object_id: 2,
                            type_id: 620600,
                            stat_pairs: vec![
                                StatPair { stat_type: 1, value: 1_500_000_000 },
                                StatPair { stat_type: 27, value: 1_500_000_000 },
                            ],
                            status_effect_datas: vec![],
                            ..Default::default()
                        },
                    })),
                    3 => Ok(LoaPacket::SkillDamageNotify(PKTSkillDamageNotify {
                        source_id: 1,
                        skill_damage_events: vec![SkillDamageEvent {
                            damage: 1_500_000_000,
                            modifier: encode_modifier(HitFlag::Critical, HitOption::BackAttack),
                            target_id: 2,
                            cur_hp: 1_500_000_000,
                            max_hp: 1_500_000_000,
                            ..Default::default()
                        }],
                        skill_id: 16140,
                        skill_effect_id: None,
                    })),
                    4 if call_count_limit == 5 => Ok(LoaPacket::TriggerStartNotify(PKTTriggerStartNotify { signal: 57 })),
                    _ => Err(anyhow::anyhow!("end loop")),
                }
            });

        capture.expect_decrypt_damage_event()
            .with(always())
            .return_const(true);

        capture
    }

    fn setup_mock_manager() -> MockEventManager {
        let mut manager = MockEventManager::new();
        manager.expect_has_paused().return_const(false);
        manager.expect_has_reset().return_const(false);
        manager.expect_has_saved().return_const(false);
        manager.expect_has_toggled_boss_only_damage().return_const(false);
        manager
    }

    fn setup_mock_emitter() -> MockAppEmitter {
        let mut emitter = MockAppEmitter::new();
        emitter.expect_emit::<i64>()
            .with(eq("raid-start"), always())
            .times(1)
            .return_const(());

        emitter.expect_emit::<i32>()
            .with(eq("phase-transition"), always())
            .return_const(());

        emitter
    }

    fn setup_sntp_client() -> MockTimeSyncClient {
         let mut sntp_client = MockTimeSyncClient::new();

        sntp_client
            .expect_synchronize()
            .times(1)
            .return_const(1);

        sntp_client
    }

    fn setup_mock_encounter_service(save: bool) -> MockEncounterService {
        let mut encounter_service = MockEncounterService::new();
        encounter_service.expect_send()
            .with(always(), always(), always(), always())
            .return_const(());
        
        if save {
            encounter_service.expect_save()
                .times(1)
                .with(always())
                .return_const(());
        }
        
        encounter_service
    }

    pub fn encode_modifier(hit_flag: HitFlag, hit_option: HitOption) -> i32 {
        let flag_bits = if hit_flag == HitFlag::Unknown { 15u8 } else { hit_flag as u8 };
        let option_bits = if (hit_option as u8) >= 4 { 0u8 } else { hit_option as u8 };
        ((option_bits << 4) | flag_bits) as i32
    }

    fn build_common_args(call_count_limit: usize, save: bool) -> StartArgs<MockPacketCapture, MockEncounterService, MockAppEmitter, MockEventManager, MockTimeSyncClient> {
        let current_dir = std::env::current_dir().unwrap();
        AssetPreloader::new(&current_dir).unwrap();

        let capture = setup_mock_capture(call_count_limit);
        let manager = setup_mock_manager();
        let emitter = setup_mock_emitter();
        let encounter_service = setup_mock_encounter_service(save);
        let settings = None;
        let (_shutdown_tx, shutdown_rx) = watch::channel(false);
        let local_player_repository = LocalPlayerRepository::new(PathBuf::from("local.json")).unwrap();
        let local_info = local_player_repository.read().unwrap();
        let sntp_client = setup_sntp_client();

        StartArgs {
            capture,
            encounter_service,
            emitter,
            settings,
            shutdown_rx,
            local_info,
            local_player_repository,
            manager,
            sntp_client
        }
    }

    #[test]
    fn should_save_encounter_on_clear() {
        let args = build_common_args(5, true);
        start(args).unwrap();
    }

    #[test]
    fn should_send_encounter_to_ui() {
        let args = build_common_args(4, false);
        start(args).unwrap();
    }
}
