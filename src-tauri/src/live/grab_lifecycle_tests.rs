use super::*;
use crate::live::parse_complete_packet;
use crate::live::status_tracker::{StatusEffectDetails, StatusEffectTargetType};
use crate::models::{DamageStats, IncapacitationEventType};
use meter_defs::BiSerializable;

fn statuses() -> StatusTracker {
    use crate::live::{id_tracker::IdTracker, party_tracker::PartyTracker};
    use std::{cell::RefCell, rc::Rc};
    let ids = Rc::new(RefCell::new(IdTracker::new()));
    StatusTracker::new(Rc::new(RefCell::new(PartyTracker::new(ids))))
}

fn held() -> Encounter {
    let mut stats = DamageStats::default();
    stats.start_grab(20, 1_000);
    Encounter {
        entities: HashMap::from_iter([(
            "player".into(),
            EncounterEntity {
                id: 10,
                name: "player".into(),
                entity_type: EntityType::Player,
                damage_stats: stats,
                ..Default::default()
            },
        )]),
        last_combat_packet: 5_000,
        ..Default::default()
    }
}

fn open(encounter: &Encounter) -> bool {
    encounter.entities["player"]
        .damage_stats
        .incapacitations
        .iter()
        .any(|event| event.grab_source_id.is_some())
}

fn movement(move_type: u8, sequence: u16) -> PKTSkillMoveNotify {
    PKTSkillMoveNotify {
        object_id: 10,
        move_type,
        movement_sequence: sequence,
        ..Default::default()
    }
}

#[test]
fn grab_move_decoder_requires_a_complete_packet() {
    let body = PKTSkillMoveNotify {
        object_id: 0x1122334455667788,
        move_time: 0.25,
        freeze_time: 0.125,
        down_time: 1.5,
        stand_up_time: 0.5,
        has_delay_mode: 1,
        delay_mode: Some(2),
        has_delay_milliseconds: 1,
        delay_milliseconds: Some(1000),
        ..movement(5, 0x1234)
    }
    .to_bytes()
    .unwrap();
    let packet = parse_complete_packet::<PKTSkillMoveNotify>(&body).unwrap();
    assert_eq!(packet.object_id, 0x1122334455667788);
    assert_eq!(packet.movement_sequence, 0x1234);
    assert_eq!(packet.move_type, 5);
    assert_eq!(
        (
            packet.move_time,
            packet.freeze_time,
            packet.down_time,
            packet.stand_up_time
        ),
        (0.25, 0.125, 1.5, 0.5)
    );
    assert_eq!(
        (packet.delay_mode, packet.delay_milliseconds),
        (Some(2), Some(1000))
    );
    assert_eq!(packet.to_bytes().unwrap(), body);
    assert!(parse_complete_packet::<PKTSkillMoveNotify>(&body[..body.len() - 1]).is_err());
    let mut trailing = body.clone();
    trailing.push(0);
    assert!(parse_complete_packet::<PKTSkillMoveNotify>(&trailing).is_err());
}

#[test]
fn grab_effect_target_round_trips_named_fields() {
    let body = PKTSkillEffectTargetNotify {
        source_id: 20,
        target_ids: vec![10, 30],
        skill_effect_id: 418840400,
        ..Default::default()
    }
    .to_bytes()
    .unwrap();
    let packet = parse_complete_packet::<PKTSkillEffectTargetNotify>(&body).unwrap();
    assert_eq!(packet.target_ids, [10, 30]);
    assert_eq!(packet.source_id, 20);
    assert_eq!(packet.skill_effect_id, 418840400);
    assert_eq!(packet.has_position, 0);
    assert!(packet.position.is_none());
}

#[test]
fn delayed_grab_release_keeps_physical_times_distinct() {
    let packet = PKTSkillMoveNotify {
        move_time: 0.25,
        freeze_time: 0.125,
        down_time: 1.5,
        stand_up_time: 0.5,
        delay_mode: Some(2),
        delay_milliseconds: Some(1000),
        ..movement(5, 0x1234)
    };
    let mut encounter = held();
    on_skill_move(&mut encounter, &HashMap::new(), &packet, 1_500);
    encounter.advance_grab_movements(2_499);
    assert!(open(&encounter));
    encounter.advance_grab_movements(2_500);
    let events = &encounter.entities["player"].damage_stats.incapacitations;
    assert_eq!(events[0].duration, 1_500);
    assert_eq!(events[1].event_type, IncapacitationEventType::FALL_DOWN);
    assert_eq!((events[1].timestamp, events[1].duration), (2_500, 2_375));
}

#[test]
fn grab_release_uses_native_mask_without_requiring_down_time() {
    for kind in 0..=13 {
        let mut encounter = held();
        on_skill_move(&mut encounter, &HashMap::new(), &movement(kind, 1), 1_500);
        let releases = [2, 3, 4, 5, 6, 7, 8, 10, 12].contains(&kind);
        assert_eq!(!open(&encounter), releases, "move type {kind}");
        assert_eq!(
            encounter.entities["player"]
                .damage_stats
                .incapacitations
                .len(),
            1
        );
    }
}

#[test]
fn grab_standalone_and_abnormal_movement_share_unsigned_sequence_and_zero_reset() {
    let mut encounter = held();
    on_skill_move(&mut encounter, &HashMap::new(), &movement(9, 50_000), 1_100);
    let mut event = SkillDamageAbnormalMoveEvent {
        move_type: 5,
        movement_sequence: 49_999,
        ..Default::default()
    };
    event.skill_damage_event.target_id = 10;
    on_abnormal_move(&mut encounter, &HashMap::new(), &event, 1_200);
    assert!(open(&encounter));
    event.movement_sequence = 50_000;
    on_abnormal_move(&mut encounter, &HashMap::new(), &event, 1_300);
    assert!(!open(&encounter));
    encounter
        .entities
        .get_mut("player")
        .unwrap()
        .damage_stats
        .start_grab(20, 1_400);
    on_skill_move(&mut encounter, &HashMap::new(), &movement(9, 0), 1_500);
    event.movement_sequence = 1;
    on_abnormal_move(&mut encounter, &HashMap::new(), &event, 1_600);
    assert!(!open(&encounter));
    assert_eq!(
        encounter.entities["player"].damage_stats.incapacitations[1].duration,
        200
    );
}

#[test]
fn grab_delayed_application_rechecks_sequence_and_snapshot_uses_encounter_time() {
    let mut encounter = held();
    let delayed = PKTSkillMoveNotify {
        has_delay_mode: 1,
        has_delay_milliseconds: 1,
        delay_mode: Some(2),
        delay_milliseconds: Some(1000),
        ..movement(5, 9)
    };
    on_skill_move(&mut encounter, &HashMap::new(), &delayed, 1_500);
    on_skill_move(&mut encounter, &HashMap::new(), &movement(9, 10), 1_600);
    encounter.advance_grab_movements(2_500);
    assert!(open(&encounter));

    let delayed = PKTSkillMoveNotify {
        movement_sequence: 11,
        ..delayed
    };
    on_skill_move(&mut encounter, &HashMap::new(), &delayed, 3_000);
    let mut snapshot = encounter.clone();
    snapshot.snapshot_incapacitations();
    assert_eq!(
        snapshot.entities["player"].damage_stats.incapacitations[0].duration,
        3_000
    );
    assert!(open(&encounter));
    encounter.advance_grab_movements(4_000);
    assert_eq!(
        encounter.entities["player"].damage_stats.incapacitations[0].duration,
        3_000
    );
    assert!(!open(&encounter));
}

#[test]
fn grab_deferred_detach_requires_real_start_and_duplicate_attach_replaces_pending_detach() {
    crate::live::test_data::initialize();
    let mut encounter = held();
    let statuses = statuses();
    let detach = PKTSkillEffectTargetNotify {
        skill_effect_id: 418840401,
        target_ids: vec![10],
        ..Default::default()
    };
    on_skill_effect_target(&mut encounter, &HashMap::new(), &detach, 1_500, &statuses);
    encounter.advance_grab_movements(1_600);
    assert!(open(&encounter));
    on_skill_start(&mut encounter, 99, 1_700);
    assert!(open(&encounter));
    on_skill_start(&mut encounter, 10, 1_800);
    assert!(!open(&encounter));
    assert_eq!(
        encounter.entities["player"].damage_stats.incapacitations[0].duration,
        800
    );

    let attach = PKTSkillEffectTargetNotify {
        skill_effect_id: 418840400,
        source_id: 20,
        target_ids: vec![10],
        ..Default::default()
    };
    on_skill_effect_target(&mut encounter, &HashMap::new(), &attach, 2_000, &statuses);
    on_skill_effect_target(&mut encounter, &HashMap::new(), &detach, 2_100, &statuses);
    on_skill_effect_target(&mut encounter, &HashMap::new(), &attach, 2_200, &statuses);
    on_skill_start(&mut encounter, 10, 2_300);
    assert!(open(&encounter));
    assert_eq!(
        encounter.entities["player"]
            .damage_stats
            .incapacitations
            .len(),
        2
    );
}

#[test]
fn grab_detach_uses_retained_freeze_evidence_past_estimated_expiry() {
    crate::live::test_data::initialize();
    let mut statuses = statuses();
    statuses.register_status_effect(StatusEffectDetails {
        target_id: 10,
        instance_id: 1,
        status_effect_id: 77300163,
        target_type: StatusEffectTargetType::Local,
        expiration_delay: 0.1,
        timestamp: chrono::DateTime::from_timestamp_millis(1_000).unwrap(),
        occur_time: chrono::DateTime::from_timestamp_millis(1_000).unwrap(),
        ..Default::default()
    });
    let expired = chrono::DateTime::from_timestamp_millis(1_500).unwrap();
    assert!(
        statuses
            .tracked_status_effect(&[10], 1)
            .unwrap()
            .expire_at
            .unwrap()
            < expired
    );
    assert!(
        statuses
            .actually_get_status_effects(
                10,
                StatusEffectTargetType::Local,
                expired,
                crate::live::status_tracker::DeadlineMode::HideExpired
            )
            .is_empty()
    );
    assert!(statuses.has_grab_detach_control(10));
    assert!(!statuses.has_grab_detach_control(30));
    let mut encounter = held();
    let detach = PKTSkillEffectTargetNotify {
        skill_effect_id: 418840401,
        target_ids: vec![10],
        ..Default::default()
    };
    on_skill_effect_target(&mut encounter, &HashMap::new(), &detach, 1_500, &statuses);
    assert!(!open(&encounter));
    assert_eq!(
        encounter.entities["player"].damage_stats.incapacitations[0].duration,
        500
    );
}

fn grab_party_status_fixture() -> (
    crate::live::entity_tracker::EntityTracker,
    std::rc::Rc<std::cell::RefCell<StatusTracker>>,
) {
    use crate::live::{id_tracker::IdTracker, party_tracker::PartyTracker};
    use std::{cell::RefCell, rc::Rc};
    crate::live::test_data::initialize();
    let ids = Rc::new(RefCell::new(IdTracker::new()));
    let parties = Rc::new(RefCell::new(PartyTracker::new(ids.clone())));
    for (actor, character) in [(1, 101), (10, 110), (30, 130)] {
        ids.borrow_mut().add_mapping(character, actor);
        parties.borrow_mut().add(500, 40, character, actor, None);
    }
    let statuses = Rc::new(RefCell::new(StatusTracker::new(parties.clone())));
    let mut tracker =
        crate::live::entity_tracker::EntityTracker::new(statuses.clone(), ids, parties);
    tracker.local_entity_id = 1;
    tracker.local_character_id = 101;
    tracker.entities.insert(
        20,
        Entity {
            id: 20,
            name: "holder".into(),
            entity_type: EntityType::Boss,
            ..Default::default()
        },
    );
    (tracker, statuses)
}

fn grab_freeze_status(instance_id: u32) -> meter_defs::defs::StatusEffectData {
    meter_defs::defs::StatusEffectData {
        status_effect_id: 77300163,
        status_effect_instance_id: instance_id,
        source_id: 20,
        skill_level: 1,
        stack_count: 1,
        total_time: 0.1,
        occur_time: chrono::Utc::now(),
        ..Default::default()
    }
}

#[test]
fn grab_detach_keeps_freeze_evidence_after_party_renotification() {
    for expire_before_detach in [false, true] {
        let (mut tracker, statuses) = grab_party_status_fixture();
        let freeze = grab_freeze_status(1);
        tracker.build_and_register_status_effect(&freeze, 10, chrono::Utc::now(), None, false);
        tracker.party_status_effect_add(
            meter_defs::defs::PKTPartyStatusEffectAddNotify {
                character_id: 110,
                status_effect_datas: vec![freeze],
                ..Default::default()
            },
            &HashMap::new(),
        );
        if expire_before_detach {
            assert!(
                statuses
                    .borrow_mut()
                    .actually_get_status_effects(
                        110,
                        StatusEffectTargetType::Party,
                        chrono::Utc::now() + chrono::Duration::seconds(1),
                        crate::live::status_tracker::DeadlineMode::HideExpired,
                    )
                    .is_empty()
            );
        }
        let mut encounter = held();
        let detach = PKTSkillEffectTargetNotify {
            skill_effect_id: 418840401,
            target_ids: vec![10],
            ..Default::default()
        };
        on_skill_effect_target(
            &mut encounter,
            &tracker.entities,
            &detach,
            1_500,
            &statuses.borrow(),
        );
        let pending_after_detach = open(&encounter);
        on_skill_start(&mut encounter, 10, 5_000);
        assert_eq!(
            encounter.entities["player"].damage_stats.incapacitations[0].duration, 500,
            "party re-notification must not defer a controlled detach (pending={pending_after_detach})",
        );
        assert!(!pending_after_detach);
    }
}

#[test]
fn grab_detach_controls_follow_the_receiver_and_explicit_removal_across_registries() {
    for removal_type in [StatusEffectTargetType::Local, StatusEffectTargetType::Party] {
        let (mut tracker, statuses) = grab_party_status_fixture();
        for (character, instance) in [(110, 1), (130, 2)] {
            tracker.party_status_effect_add(
                meter_defs::defs::PKTPartyStatusEffectAddNotify {
                    character_id: character,
                    status_effect_datas: vec![grab_freeze_status(instance)],
                    ..Default::default()
                },
                &HashMap::new(),
            );
        }
        assert!(statuses.borrow().has_grab_detach_control(10));
        assert!(statuses.borrow().has_grab_detach_control(30));
        assert!(!statuses.borrow().has_grab_detach_control(110));

        // The object channel can re-notify an instance previously supplied by the party channel.
        tracker.build_and_register_status_effect(
            &grab_freeze_status(1),
            10,
            chrono::Utc::now(),
            None,
            false,
        );
        assert!(statuses.borrow().has_grab_detach_control(10));
        let removal_target = match removal_type {
            StatusEffectTargetType::Local => 10,
            StatusEffectTargetType::Party => 110,
        };
        statuses
            .borrow_mut()
            .remove_status_effects(removal_target, vec![1], 0, removal_type);
        assert!(!statuses.borrow().has_grab_detach_control(10));
        assert!(statuses.borrow().has_grab_detach_control(30));

        let mut encounter = held();
        let detach = PKTSkillEffectTargetNotify {
            skill_effect_id: 418840401,
            target_ids: vec![10],
            ..Default::default()
        };
        on_skill_effect_target(
            &mut encounter,
            &tracker.entities,
            &detach,
            1_500,
            &statuses.borrow(),
        );
        assert!(
            open(&encounter),
            "another actor's control must not release this grab"
        );
        on_skill_start(&mut encounter, 10, 5_000);
        assert_eq!(
            encounter.entities["player"].damage_stats.incapacitations[0].duration,
            4_000
        );
    }
}

#[test]
fn grab_teleport_releases_only_native_origins_and_types() {
    for origin in 0..=22 {
        let mut encounter = held();
        on_teleport(
            &mut encounter,
            &PKTTeleportFinishNotify {
                object_id: 10,
                teleport_origin: origin,
                teleport_type: 1,
                ..Default::default()
            },
            1_500,
        );
        assert_eq!(!open(&encounter), [5, 18, 20].contains(&origin));
    }
    for kind in 0..=7 {
        let mut encounter = held();
        on_teleport(
            &mut encounter,
            &PKTTeleportFinishNotify {
                object_id: 10,
                teleport_origin: 0,
                teleport_type: kind,
                ..Default::default()
            },
            1_500,
        );
        assert_eq!(!open(&encounter), [2, 5, 6].contains(&kind));
    }
}

#[test]
fn grab_actor_removal_and_death_clear_delayed_movements_and_sequences() {
    let mut encounter = held();
    let delayed = PKTSkillMoveNotify {
        has_delay_mode: 1,
        has_delay_milliseconds: 1,
        delay_mode: Some(2),
        delay_milliseconds: Some(1000),
        down_time: 2.0,
        ..movement(5, 100)
    };
    on_skill_move(&mut encounter, &HashMap::new(), &delayed, 1_500);
    encounter.remove_grab_actor(20, 1_600);
    assert!(!open(&encounter));
    encounter.remove_grab_actor(10, 1_700);
    encounter
        .entities
        .get_mut("player")
        .unwrap()
        .damage_stats
        .start_grab(20, 1_800);
    encounter.advance_grab_movements(2_500);
    assert!(open(&encounter));
    assert_eq!(
        encounter.entities["player"]
            .damage_stats
            .incapacitations
            .len(),
        2
    );
    on_skill_move(&mut encounter, &HashMap::new(), &movement(5, 1), 2_600);
    assert!(!open(&encounter));
    let stats = &mut encounter.entities.get_mut("player").unwrap().damage_stats;
    stats.start_grab(20, 3_000);
    stats.end_incapacitations(3_100);
    assert!(
        stats
            .incapacitations
            .iter()
            .all(|event| event.grab_source_id.is_none())
    );
    let json = serde_json::to_value(stats).unwrap();
    assert!(json.get("grabState").is_none());
}

#[test]
fn grab_abnormal_movement_survives_missing_damage_key_but_requires_known_target() {
    let decryption = meter_decryption::DamageEncryptionHandler::new();
    let mut event = SkillDamageAbnormalMoveEvent {
        move_type: 5,
        movement_sequence: 1,
        ..Default::default()
    };
    event.skill_damage_event.target_id = 10;
    event.skill_damage_event.modifier = 0x40;
    let mut encounter = held();
    assert!(!process_abnormal_damage(
        &mut encounter,
        &HashMap::new(),
        &mut event,
        1500,
        &decryption
    ));
    assert!(!open(&encounter));
    assert_eq!(
        encounter.entities["player"].damage_stats.incapacitations[0].duration,
        500
    );

    let mut encounter = held();
    event.skill_damage_event.encrypted_damage.bytearray_0 = Some(vec![0; 32]);
    assert!(!process_abnormal_damage(
        &mut encounter,
        &HashMap::new(),
        &mut event,
        1500,
        &decryption
    ));
    assert!(open(&encounter));
}

#[test]
fn skill_cancel_decoder_round_trips_and_requires_complete_packet() {
    use meter_defs::defs::PKTSkillCancelNotify;

    let body = PKTSkillCancelNotify {
        object_id: 0x1122334455667788,
        skill_id: 16050,
        cancel_reason: 1,
        direction_yaw: 0x1234,
        bytearray_0: vec![0],
        bytearray_1: vec![0],
        ..Default::default()
    }
    .to_bytes()
    .unwrap();
    let packet = parse_complete_packet::<PKTSkillCancelNotify>(&body).unwrap();
    assert_eq!(packet.object_id, 0x1122334455667788);
    assert_eq!(packet.skill_id, 16050);
    assert_eq!(packet.cancel_reason, 1);
    assert_eq!(packet.direction_yaw, 0x1234);
    assert_eq!(packet.to_bytes().unwrap(), body);
    assert!(parse_complete_packet::<PKTSkillCancelNotify>(&body[..body.len() - 1]).is_err());
    let mut trailing = body.clone();
    trailing.push(0);
    assert!(parse_complete_packet::<PKTSkillCancelNotify>(&trailing).is_err());
}

#[test]
fn paralysis_sway_and_freeze_track_duration_and_clip_cleanly() {
    let mut encounter = Encounter {
        entities: HashMap::from_iter([(
            "player".into(),
            EncounterEntity {
                id: 10,
                name: "player".into(),
                entity_type: EntityType::Player,
                ..Default::default()
            },
        )]),
        last_combat_packet: 10_000,
        ..Default::default()
    };

    // Sway via PKTSkillDamageAbnormalMoveNotify: duration = move_time + freeze_time = 650ms.
    let mut sway = SkillDamageAbnormalMoveEvent {
        move_type: 4,
        movement_sequence: 1,
        ..Default::default()
    };
    sway.skill_damage_event.target_id = 10;
    sway.skill_move_option_data.move_time = Some(0.25);
    sway.skill_move_option_data.freeze_time = Some(0.40);
    on_abnormal_move(&mut encounter, &HashMap::new(), &sway, 1_000);

    let events = &encounter.entities["player"].damage_stats.incapacitations;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, IncapacitationEventType::PARALYSIS);
    assert_eq!((events[0].timestamp, events[0].duration), (1_000, 650));

    // Early skill start clips active paralysis to elapsed duration (300ms).
    on_skill_start(&mut encounter, 10, 1_300);
    assert_eq!(
        encounter.entities["player"].damage_stats.incapacitations[0].duration,
        300
    );

    // Delayed standalone PKTSkillMoveNotify sway (move_type=4) applies after delay_milliseconds.
    let delayed_sway = PKTSkillMoveNotify {
        move_time: 0.2,
        freeze_time: 0.8,
        delay_mode: Some(2),
        delay_milliseconds: Some(200),
        ..movement(4, 2)
    };
    on_skill_move(&mut encounter, &HashMap::new(), &delayed_sway, 2_000);
    encounter.advance_grab_movements(2_199);
    assert_eq!(
        encounter.entities["player"]
            .damage_stats
            .incapacitations
            .len(),
        1
    );
    encounter.advance_grab_movements(2_200);
    assert_eq!(
        encounter.entities["player"].damage_stats.incapacitations[1].event_type,
        IncapacitationEventType::PARALYSIS
    );
    assert_eq!(
        (
            encounter.entities["player"].damage_stats.incapacitations[1].timestamp,
            encounter.entities["player"].damage_stats.incapacitations[1].duration
        ),
        (2_200, 1_000)
    );

    // Subsequent knockdown clips active paralysis at 2_500 (300ms elapsed) and records FALL_DOWN.
    let knockdown = PKTSkillMoveNotify {
        move_time: 0.2,
        down_time: 1.5,
        stand_up_time: 0.3,
        ..movement(5, 3)
    };
    on_skill_move(&mut encounter, &HashMap::new(), &knockdown, 2_500);
    let events = &encounter.entities["player"].damage_stats.incapacitations;
    assert_eq!(events[1].duration, 300);
    assert_eq!(events[2].event_type, IncapacitationEventType::FALL_DOWN);
    assert_eq!((events[2].timestamp, events[2].duration), (2_500, 2_000));

    // MoveType::STOP (0) terminates active abnormal movement.
    on_skill_move(&mut encounter, &HashMap::new(), &movement(0, 4), 3_000);
    assert_eq!(
        encounter.entities["player"].damage_stats.incapacitations[2].duration,
        500
    );

    // FREEZE_STRONG (9) and FREEZE_COUNTER (11) record paralysis when NOT grabbed.
    let freeze_strong = PKTSkillMoveNotify {
        move_time: 0.1,
        stand_up_time: 0.9,
        ..movement(9, 5)
    };
    on_skill_move(&mut encounter, &HashMap::new(), &freeze_strong, 4_000);
    let events = &encounter.entities["player"].damage_stats.incapacitations;
    assert_eq!(events[3].event_type, IncapacitationEventType::PARALYSIS);
    assert_eq!((events[3].timestamp, events[3].duration), (4_000, 900));

    // FREEZE_STRONG during an active grab does NOT double-record paralysis.
    encounter
        .entities
        .get_mut("player")
        .unwrap()
        .damage_stats
        .start_grab(20, 5_000);
    on_skill_move(&mut encounter, &HashMap::new(), &freeze_strong, 5_100);
    assert_eq!(
        encounter.entities["player"]
            .damage_stats
            .incapacitations
            .iter()
            .filter(|e| e.event_type == IncapacitationEventType::PARALYSIS)
            .count(),
        3
    );

    // Replacement movements with zero down_time (STAND_UP=3, push BRING_DOWN=8)
    // must clip active paralysis rather than leaving it running.
    let sway_move = PKTSkillMoveNotify {
        move_time: 0.2,
        freeze_time: 0.8,
        ..movement(4, 6)
    };
    on_skill_move(&mut encounter, &HashMap::new(), &sway_move, 6_000);
    let stand_up = PKTSkillMoveNotify {
        move_time: 0.3,
        ..movement(3, 7)
    };
    on_skill_move(&mut encounter, &HashMap::new(), &stand_up, 6_250);
    let last_paralysis = encounter.entities["player"]
        .damage_stats
        .incapacitations
        .iter()
        .rev()
        .find(|e| e.event_type == IncapacitationEventType::PARALYSIS)
        .unwrap();
    assert_eq!(
        (last_paralysis.timestamp, last_paralysis.duration),
        (6_000, 250)
    );

    let sway_move2 = PKTSkillMoveNotify {
        move_time: 0.2,
        freeze_time: 0.8,
        ..movement(4, 8)
    };
    on_skill_move(&mut encounter, &HashMap::new(), &sway_move2, 7_000);
    let zero_down_push = PKTSkillMoveNotify {
        move_time: 0.4,
        ..movement(8, 9)
    };
    on_skill_move(&mut encounter, &HashMap::new(), &zero_down_push, 7_350);
    let last_paralysis2 = encounter.entities["player"]
        .damage_stats
        .incapacitations
        .iter()
        .rev()
        .find(|e| e.event_type == IncapacitationEventType::PARALYSIS)
        .unwrap();
    assert_eq!(
        (last_paralysis2.timestamp, last_paralysis2.duration),
        (7_000, 350)
    );
}
