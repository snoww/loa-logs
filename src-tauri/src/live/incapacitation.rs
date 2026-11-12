use crate::data::SKILL_EFFECT_DATA;
use crate::live::entity_tracker::Entity;
use crate::live::status_tracker::StatusTracker;
use crate::live::utils::encounter_entity_from_entity;
use crate::models::{Encounter, EncounterEntity, EntityType, GrabMovement};
use hashbrown::HashMap;
use meter_defs::defs::{
    PKTSkillEffectTargetNotify, PKTSkillMoveNotify, PKTTeleportFinishNotify,
    SkillDamageAbnormalMoveEvent,
};

#[cfg(test)]
#[path = "grab_lifecycle_tests.rs"]
mod lifecycle_tests;

pub fn on_skill_effect_target(
    encounter: &mut Encounter,
    entities: &HashMap<u64, Entity>,
    packet: &PKTSkillEffectTargetNotify,
    timestamp: i64,
    statuses: &StatusTracker,
) {
    let Some(effect) = SKILL_EFFECT_DATA.get(&packet.skill_effect_id) else {
        return;
    };
    if !matches!(effect.effect_type.as_str(), "attach_pc" | "detach_pc") {
        return;
    }
    // The game's attach/detach handler operates on the first target only.
    let Some(target_id) = packet.target_ids.first() else {
        return;
    };
    let Some(entity) = player(encounter, entities, *target_id) else {
        return;
    };
    if entity.is_dead {
        return;
    }
    if effect.effect_type == "attach_pc" {
        entity.damage_stats.start_grab(packet.source_id, timestamp);
    } else {
        entity
            .damage_stats
            .detach_grab(timestamp, statuses.has_grab_detach_control(*target_id));
    }
}

fn player<'a>(
    encounter: &'a mut Encounter,
    entities: &HashMap<u64, Entity>,
    target_id: u64,
) -> Option<&'a mut EncounterEntity> {
    let name = if let Some(entity) = encounter
        .entities
        .values()
        .find(|entity| entity.id == target_id && entity.entity_type == EntityType::Player)
    {
        entity.name.clone()
    } else if let Some(entity) = entities
        .get(&target_id)
        .filter(|entity| entity.entity_type == EntityType::Player)
    {
        encounter
            .entities
            .entry(entity.name.clone())
            .or_insert_with(|| encounter_entity_from_entity(entity));
        entity.name.clone()
    } else {
        return None;
    };
    encounter
        .entities
        .get_mut(&name)
        .filter(|entity| entity.id == target_id)
}

pub fn on_skill_move(
    encounter: &mut Encounter,
    entities: &HashMap<u64, Entity>,
    packet: &PKTSkillMoveNotify,
    timestamp: i64,
) {
    let delay = if packet.delay_mode == Some(2) {
        packet.delay_milliseconds.unwrap_or_default()
    } else {
        0
    };
    let Some(at) = i64::try_from(delay)
        .ok()
        .and_then(|delay| timestamp.checked_add(delay))
    else {
        return;
    };
    if let Some(entity) =
        player(encounter, entities, packet.object_id).filter(|entity| !entity.is_dead)
    {
        entity.damage_stats.observe_grab_movement(
            GrabMovement {
                timestamp: at,
                sequence: packet.movement_sequence,
                move_type: packet.move_type,
                move_time: packet.move_time,
                freeze_time: packet.freeze_time,
                down_time: packet.down_time,
                stand_up_time: packet.stand_up_time,
            },
            timestamp,
        );
    }
}

pub fn process_abnormal_damage(
    encounter: &mut Encounter,
    entities: &HashMap<u64, Entity>,
    event: &mut SkillDamageAbnormalMoveEvent,
    timestamp: i64,
    decryption: &meter_decryption::DamageEncryptionHandler,
) -> bool {
    let encrypted_target = event.skill_damage_event.damage_type <= 2
        && event
            .skill_damage_event
            .encrypted_damage
            .bytearray_0
            .is_some();
    let decrypted = decryption.decrypt_damage_event(&mut event.skill_damage_event);
    // A failed encrypted target cannot identify an actor. Damage-only encryption
    // leaves the wire target usable even when its key is unavailable.
    if decrypted || !encrypted_target {
        on_abnormal_move(encounter, entities, event, timestamp);
    }
    decrypted
}

fn on_abnormal_move(
    encounter: &mut Encounter,
    entities: &HashMap<u64, Entity>,
    event: &SkillDamageAbnormalMoveEvent,
    timestamp: i64,
) {
    if let Some(entity) = player(encounter, entities, event.skill_damage_event.target_id)
        .filter(|entity| !entity.is_dead)
    {
        let movement = &event.skill_move_option_data;
        entity.damage_stats.observe_grab_movement(
            GrabMovement {
                timestamp,
                sequence: event.movement_sequence,
                move_type: event.move_type,
                move_time: movement.move_time.unwrap_or_default(),
                freeze_time: movement.freeze_time.unwrap_or_default(),
                down_time: movement.down_time.unwrap_or_default(),
                stand_up_time: movement.stand_up_time.unwrap_or_default(),
            },
            timestamp,
        );
    }
}

pub fn on_teleport(encounter: &mut Encounter, packet: &PKTTeleportFinishNotify, timestamp: i64) {
    if matches!(packet.teleport_origin, 5 | 18 | 20) || matches!(packet.teleport_type, 2 | 5 | 6) {
        for entity in encounter
            .entities
            .values_mut()
            .filter(|entity| entity.id == packet.object_id)
        {
            entity.damage_stats.advance_grab_movements(timestamp);
            entity.damage_stats.end_grab(timestamp);
        }
    }
}

pub fn on_skill_start(encounter: &mut Encounter, actor_id: u64, timestamp: i64) {
    for entity in encounter
        .entities
        .values_mut()
        .filter(|entity| entity.id == actor_id)
    {
        entity.damage_stats.start_grabbed_skill(timestamp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DamageStats, IncapacitationEventType};
    use meter_defs::BiSerializable;

    fn statuses() -> StatusTracker {
        use crate::live::{id_tracker::IdTracker, party_tracker::PartyTracker};
        use std::{cell::RefCell, rc::Rc};
        let ids = Rc::new(RefCell::new(IdTracker::new()));
        StatusTracker::new(Rc::new(RefCell::new(PartyTracker::new(ids))))
    }

    fn entities() -> HashMap<u64, Entity> {
        HashMap::from_iter([
            (
                10,
                Entity {
                    id: 10,
                    name: "player".into(),
                    entity_type: EntityType::Player,
                    ..Default::default()
                },
            ),
            (
                20,
                Entity {
                    id: 20,
                    name: "boss".into(),
                    entity_type: EntityType::Boss,
                    ..Default::default()
                },
            ),
            (
                30,
                Entity {
                    id: 30,
                    name: "other player".into(),
                    entity_type: EntityType::Player,
                    ..Default::default()
                },
            ),
        ])
    }

    fn packet(effect_id: u32, targets: &[u64]) -> PKTSkillEffectTargetNotify {
        // Current wire order: source, targets, effect, optional release position.
        let mut bytes = Vec::new();
        bytes.extend(20_u64.to_le_bytes());
        bytes.extend((targets.len() as u16).to_le_bytes());
        for target in targets {
            bytes.extend(target.to_le_bytes());
        }
        bytes.extend(effect_id.to_le_bytes());
        bytes.push(0); // no release position
        let packet = PKTSkillEffectTargetNotify::from_bytes(&bytes).unwrap();
        assert_eq!(packet.source_id, 20);
        assert_eq!(packet.target_ids, targets);
        assert_eq!(packet.skill_effect_id, effect_id);
        assert_eq!(packet.has_position, 0);
        packet
    }

    #[test]
    fn grab_packets_track_first_player_count_and_live_and_saved_durations() {
        crate::live::test_data::initialize();
        let entities = entities();
        let mut statuses = statuses();
        statuses.register_status_effect(crate::live::status_tracker::StatusEffectDetails {
            target_id: 10,
            status_effect_id: 77300163,
            instance_id: 1,
            target_type: crate::live::status_tracker::StatusEffectTargetType::Local,
            ..Default::default()
        });
        let mut encounter = Encounter {
            last_combat_packet: 13_000,
            ..Default::default()
        };
        let attach = packet(418840400, &[10, 30]);
        on_skill_effect_target(&mut encounter, &entities, &attach, 10_000, &statuses);
        on_skill_effect_target(&mut encounter, &entities, &attach, 11_000, &statuses);
        assert!(!encounter.entities.contains_key("other player"));
        assert_eq!(
            encounter.entities["player"]
                .damage_stats
                .incapacitations
                .len(),
            1
        );

        let mut live = encounter.clone();
        live.snapshot_incapacitations();
        let events = &live.entities["player"].damage_stats.incapacitations;
        assert_eq!(events[0].event_type, IncapacitationEventType::GRAB);
        assert_eq!((events[0].timestamp, events[0].duration), (10_000, 3_000));
        let json = serde_json::to_value(&live.entities["player"].damage_stats).unwrap();
        assert_eq!(
            json["incapacitations"][0],
            serde_json::json!({ "type": "GRAB", "timestamp": 10000, "duration": 3000 })
        );
        let saved: DamageStats = serde_json::from_value(json).unwrap();
        assert_eq!(saved.incapacitations[0].duration, 3_000);
        assert!(
            encounter.entities["player"].damage_stats.incapacitations[0]
                .grab_source_id
                .is_some()
        );

        // Detach applies to the target even if it comes from a different source, or the player left visibility.
        let mut detach = packet(418840401, &[10]);
        detach.source_id = 40;
        on_skill_effect_target(&mut encounter, &HashMap::new(), &detach, 14_000, &statuses);
        let event = &encounter.entities["player"].damage_stats.incapacitations[0];
        assert_eq!(event.duration, 4_000);
        assert!(event.grab_source_id.is_none());

        on_skill_effect_target(&mut encounter, &entities, &attach, 15_000, &statuses);
        encounter.last_combat_packet = 50_000;
        encounter.snapshot_incapacitations();
        let events = &encounter.entities["player"].damage_stats.incapacitations;
        assert_eq!(events.len(), 2);
        assert_eq!((events[0].duration, events[1].duration), (4_000, 35_000));
    }

    #[test]
    fn grab_packets_ignore_empty_unknown_and_non_player_first_targets() {
        crate::live::test_data::initialize();
        let entities = entities();
        let mut statuses = statuses();
        statuses.register_status_effect(crate::live::status_tracker::StatusEffectDetails {
            target_id: 10,
            status_effect_id: 77300163,
            instance_id: 1,
            target_type: crate::live::status_tracker::StatusEffectTargetType::Local,
            ..Default::default()
        });
        let mut encounter = Encounter::default();
        for pkt in [
            packet(418840400, &[]),
            packet(u32::MAX, &[10]),
            packet(418840400, &[20, 10]),
        ] {
            on_skill_effect_target(&mut encounter, &entities, &pkt, 1_000, &statuses);
        }
        assert!(encounter.entities.is_empty());
    }

    #[test]
    fn grab_death_and_replacement_close_intervals_without_extra_duplicate_counts() {
        let mut stats = DamageStats::default();
        stats.start_grab(20, 1_000);
        stats.start_grab(40, 1_500);
        stats.end_incapacitations(1_800);
        assert_eq!(stats.incapacitations.len(), 2);
        assert_eq!(
            (
                stats.incapacitations[0].duration,
                stats.incapacitations[1].duration
            ),
            (500, 300)
        );
        assert!(
            stats
                .incapacitations
                .iter()
                .all(|event| event.grab_source_id.is_none())
        );
        stats.end_grab(2_000);
        assert_eq!(stats.incapacitations[1].duration, 300);
        stats.start_grab(20, 3_000);
        stats.end_grab(3_000);
        assert_eq!(stats.incapacitations.len(), 3);
        assert_eq!(stats.incapacitations[2].duration, 0);
    }
}
