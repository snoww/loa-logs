use crate::data::SKILL_EFFECT_DATA;
use crate::live::entity_tracker::Entity;
use crate::live::utils::encounter_entity_from_entity;
use crate::models::{Encounter, EntityType};
use hashbrown::HashMap;
use meter_defs::defs::PKTSkillEffectTargetNotify;

pub fn on_skill_effect_target(
    encounter: &mut Encounter,
    entities: &HashMap<u64, Entity>,
    packet: &PKTSkillEffectTargetNotify,
    timestamp: i64,
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
    let name = if let Some(entity) = encounter
        .entities
        .values()
        .find(|entity| entity.id == *target_id && entity.entity_type == EntityType::Player)
    {
        entity.name.clone()
    } else if let Some(entity) = entities
        .get(target_id)
        .filter(|entity| entity.entity_type == EntityType::Player)
    {
        encounter
            .entities
            .entry(entity.name.clone())
            .or_insert_with(|| encounter_entity_from_entity(entity));
        entity.name.clone()
    } else {
        return;
    };
    let entity = encounter.entities.get_mut(&name).unwrap();
    if entity.is_dead {
        return;
    }
    if effect.effect_type == "attach_pc" {
        entity.damage_stats.start_grab(packet.source_id, timestamp);
    } else {
        entity.damage_stats.end_grab(timestamp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DamageStats, IncapacitationEventType};
    use meter_defs::BiSerializable;

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
        let mut encounter = Encounter {
            last_combat_packet: 13_000,
            ..Default::default()
        };
        let attach = packet(418840400, &[10, 30]);
        on_skill_effect_target(&mut encounter, &entities, &attach, 10_000);
        on_skill_effect_target(&mut encounter, &entities, &attach, 11_000);
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
        on_skill_effect_target(&mut encounter, &HashMap::new(), &detach, 14_000);
        let event = &encounter.entities["player"].damage_stats.incapacitations[0];
        assert_eq!(event.duration, 4_000);
        assert!(event.grab_source_id.is_none());

        on_skill_effect_target(&mut encounter, &entities, &attach, 15_000);
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
        let mut encounter = Encounter::default();
        for pkt in [
            packet(418840400, &[]),
            packet(u32::MAX, &[10]),
            packet(418840400, &[20, 10]),
        ] {
            on_skill_effect_target(&mut encounter, &entities, &pkt, 1_000);
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
