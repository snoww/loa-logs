pub mod entity_tracker;
pub mod id_tracker;
pub mod models;
pub mod parser;
pub mod party_tracker;
pub mod status_tracker;

use crate::parser::entity_tracker::{get_current_and_max_hp, EntityTracker};
use crate::parser::id_tracker::IdTracker;
use crate::parser::models::EntityType;
use crate::parser::parser::Parser;
use crate::parser::party_tracker::PartyTracker;
use crate::parser::status_tracker::{StatusEffectTargetType, StatusTracker};
use anyhow::Result;
use chrono::Utc;
use pcap_test::packets::definitions::*;
use pcap_test::packets::opcodes::Pkt;
use pcap_test::start_capture;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{Manager, Window, Wry};
use tokio::runtime::Runtime;

pub fn start(window: Window<Wry>) -> Result<()> {
    let id_tracker = Rc::new(RefCell::new(IdTracker::new()));
    let party_tracker = Rc::new(RefCell::new(PartyTracker::new(id_tracker.clone())));
    let status_tracker = Rc::new(RefCell::new(StatusTracker::new(party_tracker.clone())));
    let mut entity_tracker = EntityTracker::new(
        status_tracker.clone(),
        id_tracker.clone(),
        party_tracker.clone(),
    );
    let mut parser = Parser::new(window.clone());
    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();
    let rx = start_capture();
    let mut last_update = Instant::now();
    let duration = Duration::from_millis(100);

    let reset = Arc::new(Mutex::new(false));
    let pause = Arc::new(Mutex::new(false));
    let reset_clone = reset.clone();
    let pause_clone = pause.clone();
    let meter_window_clone = window.clone();
    window.listen_global("reset-request", move |_event| {
        if let Ok(ref mut reset) = reset_clone.try_lock() {
            **reset = true;
            meter_window_clone.emit("reset-encounter", "").ok();
        }
    });
    let meter_window_clone = window.clone();
    window.listen_global("pause-request", move |_event| {
        if let Ok(ref mut pause) = pause_clone.try_lock() {
            **pause = !(**pause);
            meter_window_clone.emit("pause-encounter", "").ok();
        }
    });

    while let Ok((op, data)) = rx.recv() {
        if let Ok(ref mut reset) = reset.try_lock() {
            if **reset {
                parser.soft_reset();
                **reset = false;
            }
        }
        if let Ok(ref mut pause) = pause.try_lock() {
            if **pause {
                continue;
            }
        }
        match op {
            Pkt::CounterAttackNotify => {
                let pkt = PKTCounterAttackNotify::new(&data)?;
                if let Some(entity) = entity_tracker.entities.get(&pkt.source_id) {
                    parser.on_counterattack(entity);
                }
                // println!("{:?}", pkt);
            }
            Pkt::DeathNotify => {
                let pkt = PKTDeathNotify::new(&data)?;
                if let Some(entity) = entity_tracker.entities.get(&pkt.target_id) {
                    parser.on_death(entity);
                }
                // println!("{:?}", pkt);
            }
            Pkt::IdentityGaugeChangeNotify => {
                let pkt = PKTIdentityGaugeChangeNotify::new(&data)?;
                parser.on_identity_gain(pkt);
                // println!("{:?}", pkt);
            }
            Pkt::InitEnv => {
                let pkt = PKTInitEnv::new(&data)?;
                let entity = entity_tracker.init_env(pkt);
                println!("init env {:?}", &entity);
                parser.on_init_env(entity);
                // println!("{:?}", pkt);
            }
            Pkt::InitPC => {
                println!("init pc");
                let pkt = PKTInitPC::new(&data)?;
                let (hp, max_hp) = get_current_and_max_hp(&pkt.stat_pair);
                let entity = entity_tracker.init_pc(pkt);
                println!("init pc {:?}", &entity);

                parser.on_init_pc(entity, hp, max_hp)
                // println!("{:?}", pkt);
            }
            Pkt::MigrationExecute => {
                let pkt = PKTMigrationExecute::new(&data)?;
                entity_tracker.migration_execute(pkt);

                // println!("{:?}", pkt);
            }
            Pkt::NewPC => {
                let pkt = PKTNewPC::new(&data)?;
                let (hp, max_hp) = get_current_and_max_hp(&pkt.pc_struct.stat_pair);
                let entity = entity_tracker.new_pc(pkt);
                // println!("new pc {:?}", &entity);
                parser.on_new_pc(entity, hp, max_hp);
                // println!("{:?}", pkt);
            }
            Pkt::NewNpc => {
                let pkt = PKTNewNpc::new(&data)?;
                let (hp, max_hp) = get_current_and_max_hp(&pkt.npc_data.stat_pair);
                let entity = entity_tracker.new_npc(pkt, max_hp);
                // println!("new npc {:?}", &entity);
                parser.on_new_npc(entity, hp, max_hp);
                // println!("{:?}", pkt);
            }
            Pkt::NewNpcSummon => {
                let pkt = PKTNewNpcSummon::new(&data)?;
                let (hp, max_hp) = get_current_and_max_hp(&pkt.npc_data.stat_pair);
                let entity = entity_tracker.new_npc_summon(pkt, max_hp);
                parser.on_new_npc(entity, hp, max_hp);
                // println!("{:?}", pkt);
            }
            Pkt::NewProjectile => {
                let pkt = PKTNewProjectile::new(&data)?;
                entity_tracker.new_projectile(pkt);
                // println!("{:?}", pkt);
            }
            Pkt::ParalyzationStateNotify => {
                let pkt = PKTParalyzationStateNotify::new(&data)?;
                parser.on_stagger_change(pkt);
                // println!("{:?}", pkt);
            }
            Pkt::PartyInfo => {
                let pkt = PKTPartyInfo::new(&data)?;
                entity_tracker.party_info(pkt);
                // println!("{:?}", pkt);
            }
            Pkt::PartyLeaveResult => {
                let pkt = PKTPartyLeaveResult::new(&data)?;
                party_tracker
                    .borrow_mut()
                    .remove(pkt.party_instance_id, pkt.name);
                // println!("{:?}", pkt);
            }
            Pkt::PartyStatusEffectAddNotify => {
                let pkt = PKTPartyStatusEffectAddNotify::new(&data)?;
                entity_tracker.party_status_effect_add(pkt);
                // println!("{:?}", pkt);
            }
            Pkt::PartyStatusEffectRemoveNotify => {
                let pkt = PKTPartyStatusEffectRemoveNotify::new(&data)?;
                entity_tracker.party_status_effect_remove(pkt);
                // println!("{:?}", pkt);
            }
            Pkt::PartyStatusEffectResultNotify => {
                let pkt = PKTPartyStatusEffectResultNotify::new(&data)?;
                party_tracker.borrow_mut().add(
                    pkt.raid_instance_id,
                    pkt.party_instance_id,
                    pkt.character_id,
                    0,
                    None,
                );
                // println!("{:?}", pkt);
            }
            Pkt::RaidBossKillNotify => {
                parser.on_phase_transition(1);
                // println!("{:?}", pkt);
            }
            Pkt::RaidResult => {
                parser.on_phase_transition(0);
                // println!("{:?}", pkt);
            }
            Pkt::RemoveObject => {
                let pkt = PKTRemoveObject::new(&data)?;
                for upo in pkt.unpublished_objects {
                    status_tracker
                        .borrow_mut()
                        .remove_local_object(upo.object_id);
                }
                // println!("{:?}", pkt);
            }
            Pkt::SkillCastNotify => {
                // identity skills
                // idk if i want to use this
                // only gets sent on certain identity casts
                // e.g. arcana cards, sorc ignite (only turning off)
                // let pkt = PKTSkillCastNotify::new(&data)?;
                // let mut entity = entity_tracker.get_source_entity(pkt.caster);
                // entity = entity_tracker.guess_is_player(entity, pkt.skill_id);
                // println!("skill cast notify {:?}", &entity);
                // parser.on_skill_start(entity, pkt.skill_id as i32, Utc::now().timestamp_millis());
                // println!("{:?}", pkt);
            }
            Pkt::SkillStartNotify => {
                let pkt = PKTSkillStartNotify::new(&data)?;
                let mut entity = entity_tracker.get_source_entity(pkt.source_id);
                entity = entity_tracker.guess_is_player(entity, pkt.skill_id);
                parser.on_skill_start(entity, pkt.skill_id as i32, Utc::now().timestamp_millis());
                // println!("{:?}", pkt);
            }
            Pkt::SkillStageNotify => {
                // let pkt = PKTSkillStageNotify::new(&data);
                // println!("{:?}", pkt);
            }
            Pkt::SkillDamageAbnormalMoveNotify => {
                let pkt = PKTSkillDamageAbnormalMoveNotify::new(&data)?;
                let owner = entity_tracker.get_source_entity(pkt.source_id);
                let local_character_id = id_tracker
                    .borrow()
                    .get_local_character_id(entity_tracker.local_player_id);
                for event in pkt.skill_damage_abnormal_move_events.iter() {
                    let target_entity =
                        entity_tracker.get_or_create_entity(event.skill_damage_event.target_id);
                    let source_entity = entity_tracker.get_or_create_entity(pkt.source_id);
                    let (se_on_source, se_on_target) = status_tracker
                        .borrow_mut()
                        .get_status_effects(&owner, &target_entity, local_character_id);
                    parser.on_damage(
                        &owner,
                        &source_entity,
                        &target_entity,
                        event.skill_damage_event.damage,
                        pkt.skill_id as i32,
                        pkt.skill_effect_id as i32,
                        event.skill_damage_event.modifier as i32,
                        event.skill_damage_event.cur_hp,
                        event.skill_damage_event.max_hp,
                        se_on_source,
                        se_on_target,
                    );
                }
                // println!("{:?}", pkt);
            }
            Pkt::SkillDamageNotify => {
                let pkt = PKTSkillDamageNotify::new(&data)?;
                let owner = entity_tracker.get_source_entity(pkt.source_id);
                let local_character_id = id_tracker
                    .borrow()
                    .get_local_character_id(entity_tracker.local_player_id);
                for event in pkt.skill_damage_events.iter() {
                    let target_entity = entity_tracker.get_or_create_entity(event.target_id);
                    // source_entity is to determine battle item
                    let source_entity = entity_tracker.get_or_create_entity(pkt.source_id);
                    let (se_on_source, se_on_target) = status_tracker
                        .borrow_mut()
                        .get_status_effects(&owner, &target_entity, local_character_id);
                    parser.on_damage(
                        &owner,
                        &source_entity,
                        &target_entity,
                        event.damage,
                        pkt.skill_id as i32,
                        pkt.skill_effect_id as i32,
                        event.modifier as i32,
                        event.cur_hp,
                        event.max_hp,
                        se_on_source,
                        se_on_target,
                    );
                }
                // println!("{:?}", pkt);
            }
            Pkt::StatusEffectAddNotify => {
                let pkt = PKTStatusEffectAddNotify::new(&data)?;
                entity_tracker
                    .build_and_register_status_effect(&pkt.status_effect_data, pkt.object_id)
                // println!("{:?}", pkt);
            }
            Pkt::StatusEffectDurationNotify => {
                let pkt = PKTStatusEffectDurationNotify::new(&data)?;
                status_tracker.borrow_mut().update_status_duration(
                    pkt.effect_instance_id,
                    pkt.target_id,
                    pkt.expiration_tick,
                    StatusEffectTargetType::Local,
                );
                // println!("{:?}", pkt);
            }
            Pkt::StatusEffectRemoveNotify => {
                let pkt = PKTStatusEffectRemoveNotify::new(&data)?;
                for se_id in pkt.status_effect_ids {
                    status_tracker.borrow_mut().remove_status_effect(
                        pkt.object_id,
                        se_id,
                        StatusEffectTargetType::Local,
                    );
                }
                // println!("{:?}", pkt);
            }
            Pkt::TriggerBossBattleStatus => {
                parser.on_phase_transition(2);
                // let pkt = PKTTriggerBossBattleStatus::new(&data)?;
                // println!("{:?}", pkt);
            }
            Pkt::TriggerStartNotify => {
                // let pkt = PKTTriggerStartNotify::new(&data)?;
                // println!("{:?}", pkt);
            }
            Pkt::ZoneObjectUnpublishNotify => {
                let pkt = PKTZoneObjectUnpublishNotify::new(&data)?;
                status_tracker
                    .borrow_mut()
                    .remove_local_object(pkt.object_id);
                // println!("{:?}", pkt);
            }
            Pkt::StatusEffectSyncDataNotify => {
                // let pkt = PKTStatusEffectSyncDataNotify::new(&data);
                // println!("{:?}", pkt);
                // shields
            }
            Pkt::TroopMemberUpdateMinNotify => {
                // let pkt = PKTTroopMemberUpdateMinNotify::new(&data);
                // println!("{:?}", pkt);
                // shields
            }
            _ => {
                continue;
            }
        }
        if last_update.elapsed() >= duration || parser.raid_end {
            let mut clone = parser.encounter.clone();
            let window = window.clone();
            tokio::task::spawn(async move {
                if !clone.current_boss_name.is_empty() {
                    clone.current_boss = clone.entities.get(&clone.current_boss_name).cloned();
                    if clone.current_boss.is_none() {
                        clone.current_boss_name = String::new();
                    }
                }
                clone.entities.retain(|_, v| {
                    (v.entity_type == EntityType::PLAYER || v.entity_type == EntityType::ESTHER)
                        && v.skill_stats.hits > 0
                        && v.max_hp > 0
                });
                if !clone.entities.is_empty() {
                    window
                        .emit("encounter-update", Some(clone))
                        .expect("failed to emit encounter-update");
                }
            });
            last_update = Instant::now();
        }

        if parser.raid_end {
            parser.soft_reset();
            parser.raid_end = false;
            parser.saved = false;
        }
    }

    Ok(())
}
