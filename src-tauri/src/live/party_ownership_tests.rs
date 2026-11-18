use super::*;
use crate::live::rdps::filter_target_effects_for_attacker;
use crate::live::status_tracker::{MeasuredStat, StatusEffectDetails};

fn fixture() -> (
    EntityTracker,
    Rc<RefCell<PartyTracker>>,
    Rc<RefCell<StatusTracker>>,
) {
    crate::live::test_data::initialize();
    let ids = Rc::new(RefCell::new(IdTracker::new()));
    let parties = Rc::new(RefCell::new(PartyTracker::new(ids.clone())));
    let statuses = Rc::new(RefCell::new(StatusTracker::new(parties.clone())));
    let mut tracker = EntityTracker::new(statuses.clone(), ids.clone(), parties.clone());
    for id in 1..=3 {
        ids.borrow_mut().add_mapping(100 + id, id);
        parties
            .borrow_mut()
            .add(500, 40, 100 + id, id, Some(format!("Player{id}")));
        tracker.entities.insert(
            id,
            Entity {
                id,
                character_id: 100 + id,
                name: format!("Player{id}"),
                entity_type: Player,
                class_id: 204,
                inspect_snapshot: Some(InspectSnapshot::default()),
                ..Default::default()
            },
        );
    }
    (tracker, parties, statuses)
}

fn notification(buff: u32, instance: u32, source: u64) -> StatusEffectData {
    StatusEffectData {
        status_effect_id: buff,
        status_effect_instance_id: instance,
        source_id: source,
        skill_level: 1,
        stack_count: 1,
        ..Default::default()
    }
}

fn entry(statuses: &Rc<RefCell<StatusTracker>>, target: u64, instance: u32) -> StatusEffectDetails {
    statuses
        .borrow()
        .tracked_status_effect(&[target], instance)
        .unwrap()
        .clone()
}

#[test]
fn loading_order_cannot_overwrite_observed_membership() {
    let (mut tracker, parties, _) = fixture();
    parties.borrow_mut().add(500, 900, 108, 0, None);
    tracker.observe_loading_participants(10, (101..=116).rev().collect());
    assert_eq!(parties.borrow().get_party_id_for_character(102), Some(40));
    assert_eq!(parties.borrow().get_party_id_for_character(108), Some(900));
    assert_eq!(parties.borrow().get_party_id_for_character(116), None);
    assert_eq!(
        tracker.inspect_discovery_characters(),
        (101..=116).collect::<Vec<_>>()
    );
    tracker.observe_loading_participants(10, vec![102, 101, 120]);
    assert_eq!(
        tracker.inspect_discovery_characters(),
        vec![101, 102, 103, 108, 120]
    );
    tracker.observe_loading_zone(11);
    assert_eq!(
        tracker.inspect_discovery_characters(),
        vec![101, 102, 103, 108]
    );
}

#[test]
fn roster_replacement_and_leave_preserve_other_parties() {
    let (_, parties, _) = fixture();
    parties
        .borrow_mut()
        .add(500, 900, 108, 0, Some("Other".into()));
    parties
        .borrow_mut()
        .replace_party_members(500, 40, [(101, "First".into())]);
    assert_eq!(parties.borrow().get_party_id_for_character(102), None);
    assert_eq!(parties.borrow().get_party_id_for_character(108), Some(900));
    parties.borrow_mut().remove(40, "First".into());
    assert_eq!(parties.borrow().get_party_id_for_character(101), None);
    assert_eq!(parties.borrow().get_party_id_for_character(108), Some(900));
    parties.borrow_mut().add(600, 900, 109, 0, None);
    assert!(!parties.borrow().are_same_party_characters(108, 109));
}

#[test]
fn special_effect_refresh_preserves_creation_owner_before_round_robin() {
    for snapshot_reset in [false, true] {
        let (mut tracker, _, statuses) = fixture();
        for id in [1, 2] {
            tracker
                .entities
                .get_mut(&id)
                .unwrap()
                .inspect_snapshot
                .as_mut()
                .unwrap()
                .derived_stats
                .buff_id_ownership = vec![2000361];
        }
        let now = Utc::now();
        tracker.build_and_register_status_effect(&notification(2000361, 1, 3), 3, now, None, false);
        assert_eq!(entry(&statuses, 3, 1).source_id, 1);
        let original_stats = Arc::new(PlayerStats::default());
        statuses.borrow_mut().update_effects_owned_by(1, |effect| {
            effect.owner_player_stats_snapshot = Some(original_stats.clone());
            effect.source_skill_id = Some(31050);
            effect.source_skill_runtime_snapshot = Some(SkillRuntimeData {
                skill_level: 7,
                ..Default::default()
            });
            effect.measured_stat = Some(MeasuredStat {
                stat_type: crate::live::stat_type::StatType::SKILL_DAMAGE_SUB_RATE_1,
                value: 3140,
            });
        });
        let original = entry(&statuses, 3, 1);
        if snapshot_reset {
            statuses.borrow_mut().remove_local_object(3);
        }
        tracker.build_and_register_status_effect(&notification(2000361, 1, 3), 3, now, None, true);
        let refreshed = entry(&statuses, 3, 1);
        assert_eq!(refreshed.source_id, 1);
        assert_eq!(refreshed.source_skill_id, Some(31050));
        assert_eq!(
            refreshed.source_skill_runtime_snapshot.unwrap().skill_level,
            7
        );
        assert!(Arc::ptr_eq(
            refreshed.owner_player_stats_snapshot.as_ref().unwrap(),
            &original_stats
        ));
        assert_eq!(refreshed.first_tracked, original.first_tracked);
        assert_eq!(refreshed.measured_stat.unwrap().value, 3140);
        tracker.build_and_register_status_effect(&notification(2000361, 2, 3), 3, now, None, false);
        assert_eq!(entry(&statuses, 3, 2).source_id, 2);
    }
}

#[test]
fn buffered_brand_uses_recorded_party_facts_after_a_real_party_change() {
    for initially_same_party in [false, true] {
        let (tracker, parties, _) = fixture();
        if !initially_same_party {
            parties.borrow_mut().add(500, 900, 101, 1, None);
        }
        let attacker = tracker.snapshot_entity_for_hit(&tracker.entities[&2]);
        let source = tracker.snapshot_entity_for_hit(&tracker.entities[&1]);
        let buffered = HashMap::from([(2, attacker.clone()), (1, source)]);
        parties.borrow_mut().add(
            500,
            if initially_same_party { 900 } else { 40 },
            101,
            1,
            None,
        );
        let boss = Entity {
            id: 99,
            npc_id: 486101,
            entity_type: Boss,
            ..Default::default()
        };
        let brand = StatusEffectDetails {
            status_effect_id: 314260,
            source_id: 1,
            owner_is_player: true,
            ..Default::default()
        };
        let earlier = filter_target_effects_for_attacker(
            &attacker,
            &boss,
            std::slice::from_ref(&brand),
            &tracker,
            Some(&buffered),
        );
        let later = filter_target_effects_for_attacker(
            &tracker.entities[&2],
            &boss,
            &[brand],
            &tracker,
            None,
        );
        assert_eq!(earlier.len(), usize::from(initially_same_party));
        assert_eq!(later.len(), usize::from(!initially_same_party));
    }
}

#[test]
fn local_and_party_notifications_share_one_receiver_instance() {
    let (mut tracker, _, statuses) = fixture();
    tracker.build_and_register_status_effect(
        &notification(314010, 1, 1),
        2,
        Utc::now(),
        None,
        false,
    );
    let boss = Entity {
        id: 99,
        npc_id: 486101,
        entity_type: Boss,
        ..Default::default()
    };
    let (source_effects, _) = statuses
        .borrow_mut()
        .get_status_effects(&tracker.entities[&2], &boss);
    assert_eq!(source_effects.len(), 1);
    let original = entry(&statuses, 2, 1);
    let creation_stats = Arc::new(PlayerStats::default());
    statuses.borrow_mut().update_effects_owned_by(1, |effect| {
        effect.owner_player_stats_snapshot = Some(creation_stats.clone());
    });
    let mut party_copy = original;
    party_copy.target_id = 102;
    party_copy.target_type = StatusEffectTargetType::Party;
    party_copy.owner_player_stats_snapshot = None;
    statuses.borrow_mut().register_status_effect(party_copy);
    assert!(statuses.borrow().tracked_status_effect(&[2], 1).is_none());
    assert!(Arc::ptr_eq(
        entry(&statuses, 102, 1)
            .owner_player_stats_snapshot
            .as_ref()
            .unwrap(),
        &creation_stats
    ));
    let (_, _, removed, _) =
        statuses
            .borrow_mut()
            .remove_status_effects(2, vec![1], 0, StatusEffectTargetType::Local);
    assert_eq!(removed.len(), 1);
    assert!(
        statuses
            .borrow_mut()
            .get_source_status_effects(
                &tracker.entities[&2],
                Utc::now(),
                DeadlineMode::KeepUntilRemoved
            )
            .is_empty()
    );
}

#[test]
fn explicit_remove_retires_retained_instance_without_an_active_receiver_bucket() {
    let (mut tracker, _, statuses) = fixture();
    tracker.build_and_register_status_effect(
        &notification(314010, 1, 1),
        2,
        Utc::now(),
        None,
        false,
    );
    let original = entry(&statuses, 2, 1).owner_player_stats_snapshot.unwrap();
    statuses.borrow_mut().remove_local_object(2);
    statuses
        .borrow_mut()
        .remove_status_effects(2, vec![1], 0, StatusEffectTargetType::Local);
    let prior = statuses.borrow_mut().creation_owner_for_notification(
        &notification(314010, 1, 1),
        2,
        StatusEffectTargetType::Local,
        Utc::now(),
    );
    assert_eq!(prior, None);
    tracker.build_and_register_status_effect(
        &notification(314010, 1, 1),
        2,
        Utc::now(),
        None,
        false,
    );
    assert!(!Arc::ptr_eq(
        &original,
        entry(&statuses, 2, 1)
            .owner_player_stats_snapshot
            .as_ref()
            .unwrap(),
    ));
}

#[test]
fn late_damage_preserves_retained_identity_without_visibility() {
    let (mut tracker, _, _) = fixture();
    tracker.entities.get_mut(&1).unwrap().class_id = 0;
    tracker.remove_object(1);
    let mut owner = tracker.get_or_create_entity(1);
    assert_eq!(owner.entity_type, Player);
    assert_eq!(owner.character_id, 101);
    tracker.infer_entity_class_from_skill(&mut owner, 21160);
    assert!(owner.class_id > 0);
    assert!(!tracker.entities.contains_key(&1));
    assert_eq!(tracker.known_effect_owner(1).unwrap().character_id, 101);
}

#[test]
fn player_id_change_preserves_active_and_retained_buff_creation_stats() {
    for retained in [false, true] {
        let (mut tracker, _, statuses) = fixture();
        let now = Utc::now();
        tracker.build_and_register_status_effect(&notification(314010, 1, 1), 2, now, None, false);
        let original = entry(&statuses, 2, 1);
        if retained {
            statuses.borrow_mut().remove_local_object(2);
        }

        tracker.new_pc(PCStruct {
            player_id: 10,
            character_id: 101,
            name: "Player1".into(),
            class_id: 204,
            ..Default::default()
        });
        if !retained {
            assert_eq!(entry(&statuses, 2, 1).source_id, 10);
        }
        tracker.build_and_register_status_effect(&notification(314010, 1, 10), 2, now, None, true);

        let updated = entry(&statuses, 2, 1);
        assert_eq!(updated.source_id, 10);
        assert!(Arc::ptr_eq(
            original.owner_player_stats_snapshot.as_ref().unwrap(),
            updated.owner_player_stats_snapshot.as_ref().unwrap(),
        ));
    }
}

#[test]
fn local_player_id_changes_preserve_buff_creation_stats_for_the_same_character() {
    for next_character_id in [101, 104] {
        let (mut tracker, _, statuses) = fixture();
        tracker.local_entity_id = 1;
        tracker.local_character_id = 101;
        let now = Utc::now();
        tracker.build_and_register_status_effect(&notification(314010, 1, 1), 1, now, None, false);
        let original = entry(&statuses, 1, 1);

        tracker.init_env(PKTInitEnv {
            player_id: 10,
            ..Default::default()
        });
        tracker.init_pc(PKTInitPC {
            player_id: 20,
            character_id: next_character_id,
            name: "Local".into(),
            class_id: 204,
            status_effect_datas: vec![notification(314010, 1, 20)],
            ..Default::default()
        });

        let updated = entry(&statuses, 20, 1);
        assert_eq!(updated.source_id, 20);
        if next_character_id == 101 {
            assert!(Arc::ptr_eq(
                updated.owner_player_stats_snapshot.as_ref().unwrap(),
                original.owner_player_stats_snapshot.as_ref().unwrap(),
            ));
        } else {
            assert!(updated.owner_player_stats_snapshot.is_none());
        }
    }
}

#[test]
fn player_snapshot_replaces_both_status_channels() {
    let (mut tracker, _, statuses) = fixture();
    tracker.local_entity_id = 1;
    tracker.local_character_id = 101;
    tracker.build_and_register_status_effect(
        &notification(314010, 1, 1),
        2,
        Utc::now(),
        None,
        false,
    );
    assert!(statuses.borrow().tracked_status_effect(&[2], 1).is_some());
    tracker.new_pc(PCStruct {
        player_id: 2,
        character_id: 102,
        name: "Player2".into(),
        class_id: 204,
        ..Default::default()
    });
    assert!(
        statuses
            .borrow_mut()
            .get_source_status_effects(
                &tracker.entities[&2],
                Utc::now(),
                DeadlineMode::KeepUntilRemoved
            )
            .is_empty()
    );
}
