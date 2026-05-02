use crate::models::{CastEvent, SkillCast, SkillHit};
use hashbrown::{HashMap, hash_map::Entry};
use moka::sync::Cache;
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Debug)]
pub struct SkillTracker {
    pub fight_start: i64,
    pub skills: HashMap<(u64, u32, i64), SkillCast>,
    pub projectile_id_to_timestamp: Cache<u64, i64>,
    pub skill_timestamp: Cache<(u64, u32), i64>,

    // map of skill_id to a list of CastEvents
    pub skill_cooldowns: HashMap<u32, Vec<CastEvent>>,
}

impl SkillTracker {
    pub fn new() -> Self {
        SkillTracker {
            fight_start: -1,
            skills: HashMap::new(),
            projectile_id_to_timestamp: Cache::builder()
                .time_to_idle(Duration::from_secs(20))
                .build(),
            skill_timestamp: Cache::builder()
                .time_to_idle(Duration::from_secs(20))
                .build(),
            skill_cooldowns: HashMap::new(),
        }
    }

    pub fn new_cast(
        &mut self,
        entity_id: u64,
        skill_id: u32,
        summon_source: Option<Vec<u32>>,
        timestamp: i64,
    ) {
        let relative = timestamp - self.fight_start;
        if let Some(summon_source) = summon_source {
            for source in summon_source {
                if self.skill_timestamp.get(&(entity_id, source)).is_some() {
                    // info!("ignoring summon: {}|{}|{}", entity_id, source, relative);
                    return;
                }
            }
        }
        // info!("new skill CAST: {}|{}|{}", entity_id, skill_id, relative);
        self.skill_timestamp.insert((entity_id, skill_id), relative);
        self.skills
            .entry((entity_id, skill_id, relative))
            .or_insert(SkillCast {
                hits: Vec::new(),
                timestamp: relative,
                last: relative,
            });
    }

    pub fn on_hit(
        &mut self,
        entity_id: u64,
        projectile_id: u64,
        skill_id: u32,
        info: SkillHit,
        summon_source: Option<Vec<u32>>,
    ) {
        let skill_timestamp = if let Some(summon_source) = summon_source {
            let mut source_timestamp = info.timestamp;
            let mut found = false;
            for source in summon_source {
                if let Some(skill_timestamp) = self.skill_timestamp.get(&(entity_id, source)) {
                    found = true;
                    source_timestamp = skill_timestamp;
                    break;
                }
            }
            if !found {
                self.skill_timestamp
                    .insert((entity_id, skill_id), source_timestamp);
            }
            source_timestamp
        } else if let Some(skill_timestamp) = self.projectile_id_to_timestamp.get(&projectile_id) {
            skill_timestamp
        } else {
            self.skill_timestamp
                .get(&(entity_id, skill_id))
                .unwrap_or(-1)
        };

        if skill_timestamp >= 0 {
            // info!(
            //     "new skill HIT: {}|{}|{}|{}",
            //     entity_id, projectile_id, skill_id, skill_timestamp
            // );
            let timestamp = info.timestamp;
            self.skills
                .entry((entity_id, skill_id, skill_timestamp))
                .and_modify(|skill| {
                    skill.hits.push(info.clone());
                    skill.last = timestamp;
                })
                .or_insert(SkillCast {
                    hits: vec![info],
                    timestamp: skill_timestamp,
                    last: timestamp,
                });
        }
    }

    pub fn get_cast_log(&mut self) -> HashMap<u64, HashMap<u32, BTreeMap<i64, SkillCast>>> {
        let mut cast_log: HashMap<u64, HashMap<u32, BTreeMap<i64, SkillCast>>> = HashMap::new();
        for ((entity_id, skill_id, timestamp), cast) in self.skills.iter() {
            cast_log
                .entry(*entity_id)
                .or_default()
                .entry(*skill_id)
                .or_default()
                .insert(*timestamp, cast.clone());
        }

        cast_log
    }

    pub fn rebind_entity_id(&mut self, old_entity_id: u64, new_entity_id: u64) {
        if old_entity_id == 0 || old_entity_id == new_entity_id {
            return;
        }

        let old_skills = self
            .skills
            .keys()
            .filter_map(|(entity_id, skill_id, timestamp)| {
                (*entity_id == old_entity_id).then_some((*skill_id, *timestamp))
            })
            .collect::<Vec<_>>();
        let mut moved_skill_ids = Vec::new();

        for (skill_id, timestamp) in old_skills {
            let Some(old_cast) = self.skills.remove(&(old_entity_id, skill_id, timestamp)) else {
                continue;
            };

            match self.skills.entry((new_entity_id, skill_id, timestamp)) {
                Entry::Occupied(mut entry) => {
                    let cast = entry.get_mut();
                    cast.last = cast.last.max(old_cast.last);
                    Self::append_missing_hits(&mut cast.hits, old_cast.hits);
                }
                Entry::Vacant(entry) => {
                    entry.insert(old_cast);
                }
            }

            if !moved_skill_ids.contains(&skill_id) {
                moved_skill_ids.push(skill_id);
            }
        }

        for skill_id in moved_skill_ids {
            if let Some(timestamp) = self.skill_timestamp.get(&(old_entity_id, skill_id)) {
                if self
                    .skill_timestamp
                    .get(&(new_entity_id, skill_id))
                    .is_none()
                {
                    self.skill_timestamp
                        .insert((new_entity_id, skill_id), timestamp);
                }
            }
            self.skill_timestamp.invalidate(&(old_entity_id, skill_id));
        }
    }

    fn append_missing_hits(target: &mut Vec<SkillHit>, source: Vec<SkillHit>) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rebind_entity_id_moves_casts_and_hits() {
        let mut tracker = SkillTracker::new();
        tracker.fight_start = 1_000;
        tracker.new_cast(10, 20, None, 2_000);
        tracker.on_hit(
            10,
            0,
            20,
            SkillHit {
                damage: 30,
                timestamp: 1_500,
                ..Default::default()
            },
            None,
        );

        tracker.rebind_entity_id(10, 99);

        let cast_log = tracker.get_cast_log();
        assert!(!cast_log.contains_key(&10));
        let casts = cast_log
            .get(&99)
            .and_then(|skills| skills.get(&20))
            .expect("rebinding should move old entity casts to the new entity id");
        let cast = casts
            .values()
            .next()
            .expect("rebinding should retain the skill cast");
        assert_eq!(cast.hits.len(), 1);
        assert_eq!(cast.hits[0].damage, 30);
        assert!(tracker.skill_timestamp.get(&(10, 20)).is_none());
        assert_eq!(tracker.skill_timestamp.get(&(99, 20)), Some(1_000));
    }

    #[test]
    fn rebind_entity_id_preserves_existing_active_skill_timestamp() {
        let mut tracker = SkillTracker::new();
        tracker.fight_start = 1_000;
        tracker.new_cast(10, 20, None, 2_000);
        tracker.new_cast(99, 20, None, 3_000);

        tracker.rebind_entity_id(10, 99);

        assert!(tracker.skill_timestamp.get(&(10, 20)).is_none());
        assert_eq!(tracker.skill_timestamp.get(&(99, 20)), Some(2_000));
        assert!(tracker.skills.contains_key(&(99, 20, 1_000)));
        assert!(tracker.skills.contains_key(&(99, 20, 2_000)));
    }

    #[test]
    fn duplicate_new_cast_does_not_replace_existing_hits() {
        let mut tracker = SkillTracker::new();
        tracker.fight_start = 1_000;
        tracker.new_cast(10, 20, None, 2_000);
        tracker.on_hit(
            10,
            0,
            20,
            SkillHit {
                damage: 30,
                timestamp: 1_500,
                ..Default::default()
            },
            None,
        );

        tracker.new_cast(10, 20, None, 2_000);

        let cast = tracker
            .skills
            .get(&(10, 20, 1_000))
            .expect("duplicate cast should keep the existing cast entry");
        assert_eq!(cast.hits.len(), 1);
        assert_eq!(cast.hits[0].damage, 30);
    }

    #[test]
    fn rebind_entity_id_deduplicates_same_cast_hit_collisions() {
        let mut tracker = SkillTracker::new();
        tracker.fight_start = 1_000;
        let hit = SkillHit {
            damage: 30,
            timestamp: 1_500,
            crit: true,
            buffed_by: vec![1, 2],
            ..Default::default()
        };

        tracker.new_cast(10, 20, None, 2_000);
        tracker.on_hit(10, 0, 20, hit.clone(), None);
        tracker.new_cast(99, 20, None, 2_000);
        tracker.on_hit(99, 0, 20, hit, None);

        tracker.rebind_entity_id(10, 99);

        let cast_log = tracker.get_cast_log();
        let casts = cast_log
            .get(&99)
            .and_then(|skills| skills.get(&20))
            .expect("rebinding should keep the target cast");
        let cast = casts
            .get(&1_000)
            .expect("same-timestamp casts should merge into one cast");
        assert_eq!(cast.hits.len(), 1);
        assert_eq!(cast.hits[0].damage, 30);
        assert!(cast.hits[0].crit);
    }
}
