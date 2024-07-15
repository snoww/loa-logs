use crate::parser::models::{SkillCast, SkillHit};
use hashbrown::HashMap;
use moka::sync::Cache;
use std::collections::BTreeMap;
use std::time::Duration;
use log::info;

#[derive(Debug)]
pub struct SkillTracker {
    pub fight_start: i64,
    pub skills: HashMap<(u64, u32, i64), SkillCast>,
    pub projectile_id_to_timestamp: Cache<u64, i64>,
    pub skill_timestamp: Cache<(u64, u32), i64>,
}

impl SkillTracker {
    pub fn new() -> Self {
        SkillTracker {
            fight_start: -1,
            skills: HashMap::new(),
            projectile_id_to_timestamp: Cache::builder()
                .time_to_idle(Duration::from_secs(60))
                .build(),
            skill_timestamp: Cache::builder()
                .time_to_idle(Duration::from_secs(60))
                .build(),
        }
    }

    pub fn new_cast(&mut self, entity_id: u64, skill_id: u32, timestamp: i64) {
        let relative = timestamp - self.fight_start;
        // info!("new skill CAST: {}|{}|{}", entity_id, skill_id, relative);
        self.skill_timestamp
            .insert((entity_id, skill_id), relative);
        self.skills.insert(
            (entity_id, skill_id, relative),
            SkillCast {
                hits: Vec::new(),
                timestamp: relative,
                last: relative,
            },
        );
    }

    pub fn on_hit(&mut self, entity_id: u64, projectile_id: u64, skill_id: u32, info: SkillHit) {
        let skill_timestamp =
            if let Some(skill_timestamp) = self.projectile_id_to_timestamp.get(&projectile_id) {
                skill_timestamp
            } else if let Some(skill_timestamp) = self.skill_timestamp.get(&(entity_id, skill_id)) {
                skill_timestamp
            } else {
                -1
            };

        if skill_timestamp >= 0 {
            // info!("new skill HIT: {}|{}|{}", entity_id, skill_id, skill_timestamp);
            let timestamp = info.timestamp;
            self.skills
                .entry((entity_id, skill_id, skill_timestamp))
                .and_modify(|skill| {
                    skill.hits.push(info.clone());
                    skill.last = timestamp;
                })
                .or_insert(SkillCast {
                    hits: vec![info],
                    timestamp,
                    last: timestamp,
                });
        }
    }

    pub fn get_cast_log(&mut self) -> HashMap<u64, HashMap<u32, BTreeMap<i64, SkillCast>>> {
        let mut cast_log: HashMap<u64, HashMap<u32, BTreeMap<i64, SkillCast>>> = HashMap::new();
        for ((entity_id, skill_id, timestamp), cast) in self.skills.iter() {
            if cast.hits.is_empty() {
                continue;
            }
            cast_log
                .entry(*entity_id)
                .or_default()
                .entry(*skill_id)
                .or_default()
                .insert(*timestamp, cast.clone());
        }

        cast_log
    }
}
