use std::collections::BTreeMap;

use chrono::Utc;
use hashbrown::{HashSet, HashMap};
use rand::{rngs::ThreadRng, seq::IndexedRandom, Rng};

use crate::{database::models::InsertEncounterArgs, models::*};

#[derive(Clone)]
pub struct PlayerSpec {
    class_id: u32,
    class_name: String,
    specialisation: &'static str,
    crit_rate: f64,
    gear_score: f32,
    hp: i64,
    info: InspectInfo,
}

pub struct RaidBuilder {
    parties: Vec<Vec<PlayerSpec>>,
    boss_name: String,
    boss_npc_id: u32,
    boss_hp: i64,
    duration_minutes: i64,
    region: String,
    version: String,
    difficulty: String,
    cleared: bool,
    rng: ThreadRng,
    damage_range: (i64, i64),
    damage_taken_range: (i64, i64),
}

impl RaidBuilder {
    fn new() -> Self {
        Self {
            parties: Vec::new(),
            boss_name: String::new(),
            boss_npc_id: 0,
            boss_hp: 0,
            cleared: false,
            duration_minutes: 15,
            region: "EUC".to_string(),
            version: "0.0.1".to_string(),
            difficulty: "Hard".to_string(),
            rng: rand::rng(),
            damage_range: (500, 1500),
            damage_taken_range: (500, 1000),
        }
    }

    fn add_party(mut self, players: (PlayerSpec, PlayerSpec, PlayerSpec, PlayerSpec)) -> Self {
        self.parties
            .push(vec![players.0, players.1, players.2, players.3]);
        self
    }

    fn set_boss(mut self, name: &str, npc_id: u32, hp: i64, duration_minutes: i64) -> Self {
        self.boss_name = name.to_string();
        self.boss_npc_id = npc_id;
        self.boss_hp = hp;
        self.duration_minutes = duration_minutes;
        self
    }

    fn set_region(mut self, region: &str) -> Self {
        self.region = region.to_string();
        self
    }

    fn set_version(mut self, version: &str) -> Self {
        self.version = version.to_string();
        self
    }

    fn set_cleared(mut self, cleared: bool) -> Self {
        self.cleared = cleared;
        self
    }

    fn set_difficulty(mut self, difficulty: &str) -> Self {
        self.difficulty = difficulty.to_string();
        self
    }

    fn set_damage_range(mut self, min: i64, max: i64) -> Self {
        assert!(min > 0 && max >= min, "Invalid damage range");
        self.damage_range = (min, max);
        self
    }

    fn build(mut self) -> InsertEncounterArgs {
        let fight_start = Utc::now().timestamp_millis();
        let last_combat_packet = fight_start + self.duration_minutes * 60 * 1000;
        let duration_ms = last_combat_packet - fight_start;
        let duration_s = duration_ms / 1000;

        let total_players = self.parties.iter().map(|p| p.len()).sum::<usize>();
        let raid_dps: i64 = self.boss_hp / duration_s;
        let per_player_total_damage: i64 = raid_dps / total_players as i64 * duration_s;

        let (entities_with_spec, player_names) = generate_entities_for_parties(&self.parties);

        let local_player = player_names[0].clone();
        let mut boss = self.generate_boss_entity();

        boss.damage_stats.damage_dealt += self
            .rng
            .random_range(self.damage_range.0..self.damage_range.1);

        let mut boss_hp_logs: HashMap<String, Vec<BossHpLog>> = HashMap::new();
        let mut boss_hp_log = Vec::with_capacity(duration_s as usize + 1);
        for t in 0..=duration_s as i32 {
            let dealt = raid_dps * t as i64;
            let hp = (self.boss_hp - dealt).max(0);
            let percent = hp as f32 / self.boss_hp as f32;
            boss_hp_log.push(BossHpLog::new(t, hp, percent));
        }
        boss_hp_logs.insert(boss.name.clone(), boss_hp_log);

        let party_vec: Vec<Vec<String>> =
            player_names.chunks(4).map(|chunk| chunk.to_vec()).collect();

        let mut party_info: HashMap<i32, Vec<String>> = HashMap::new();
        for (idx, party) in player_names.chunks(4).enumerate() {
            party_info.insert(idx as i32 + 1, party.to_vec());
        }

        let mut encounter_entities_with_stats = HashMap::new();
        let mut damage_log: HashMap<String, Vec<(i64, i64)>> = HashMap::new();
        let mut cast_log: HashMap<String, HashMap<u32, Vec<i32>>> = HashMap::new();
        let mut skill_cast_log: HashMap<u64, HashMap<u32, BTreeMap<i64, SkillCast>>> =
            HashMap::new();
        let mut player_info: HashMap<String, InspectInfo> = HashMap::new();
        for (name, (spec, mut entity)) in entities_with_spec.into_iter() {
            if entity.entity_type == EntityType::Player {
                update_skill_and_damage_stats(
                    duration_s,
                    &spec,
                    &mut damage_log,
                    &mut cast_log,
                    &mut skill_cast_log,
                    self.damage_range,
                    &mut self.rng,
                    &mut entity,
                );
                player_info.insert(name.clone(), spec.info);
                update_damage_taken(self.damage_taken_range, &mut self.rng, &mut entity);
                update_buffs_heals_and_absorb(&mut self.rng, &mut entity);
            }
            encounter_entities_with_stats.insert(name, entity);
        }

        encounter_entities_with_stats.insert(boss.name.clone(), boss.clone());

        let local_player_entity = encounter_entities_with_stats.get(&local_player).unwrap();
        let mut skill_cooldowns = HashMap::new();

        for (id, _) in local_player_entity.skills.iter() {
            skill_cooldowns.insert(
                *id,
                vec![CastEvent {
                    cooldown_duration_ms: self.rng.random_range(1000..5000),
                    timestamp: self.rng.random_range(1000..5000),
                }],
            );
        }

        let misc = EncounterMisc {
            boss_hp_log: None,
            raid_clear: Some(true),
            party_info: Some(party_info.clone()),
            region: Some(self.region.clone()),
            version: Some(self.version.clone()),
            rdps_valid: None,
            rdps_message: None,
            ntp_fight_start: Some(fight_start),
            manual_save: None,
        };

        let encounter_damage_stats = EncounterDamageStats {
            total_damage_dealt: self.boss_hp,
            top_damage_dealt: per_player_total_damage,
            total_damage_taken: 0,
            top_damage_taken: 0,
            dps: raid_dps,
            buffs: HashMap::new(),
            debuffs: HashMap::new(),
            total_shielding: 0,
            total_effective_shielding: 0,
            applied_shield_buffs: HashMap::new(),
            unknown_buffs: HashSet::new(),
            misc: Some(misc.clone()),
            boss_hp_log: boss_hp_logs.clone(),
        };

        let encounter = Encounter {
            last_combat_packet,
            fight_start,
            local_player,
            entities: encounter_entities_with_stats.clone(),
            current_boss_name: boss.name.clone(),
            current_boss: Some(boss),
            encounter_damage_stats,
            duration: duration_ms,
            difficulty: Some(self.difficulty.clone()),
            favorite: false,
            cleared: true,
            boss_only_damage: false,
            sync: None,
            region: Some(self.region.clone()),
        };

        let insert_args = InsertEncounterArgs {
            encounter,
            damage_log,
            cast_log,
            boss_hp_log: boss_hp_logs,
            raid_clear: true,
            party_info: party_vec,
            raid_difficulty: self.difficulty.clone(),
            region: Some(self.region.clone()),
            player_info: Some(player_info),
            meter_version: self.version.clone(),
            ntp_fight_start: fight_start,
            rdps_valid: true,
            manual: false,
            skill_cast_log,
            skill_cooldowns,
        };

        insert_args
    }

    fn generate_boss_entity(&self) -> EncounterEntity {
        EncounterEntity {
            id: 1000,
            character_id: 0,
            npc_id: self.boss_npc_id,
            name: self.boss_name.clone(),
            entity_type: EntityType::Boss,
            class_id: 0,
            class: String::new(),
            gear_score: 0.0,
            current_hp: self.boss_hp,
            max_hp: self.boss_hp,
            current_shield: 0,
            is_dead: false,
            skills: HashMap::new(),
            damage_stats: DamageStats {
                damage_taken: self.boss_hp,
                ..Default::default()
            },
            skill_stats: SkillStats::default(),
            engraving_data: None,
            ark_passive_active: None,
            ark_passive_data: None,
            spec: None,
            loadout_hash: None,
            combat_power: None,
        }
    }
}

fn update_skill_and_damage_stats(
    duration_seconds: i64,
    spec: &PlayerSpec,
    entities_damage_log: &mut HashMap<String, Vec<(i64, i64)>>,
    cast_log: &mut HashMap<String, HashMap<u32, Vec<i32>>>,
    skill_cast_log: &mut HashMap<u64, HashMap<u32, BTreeMap<i64, SkillCast>>>,
    damage_range: (i64, i64),
    rng: &mut ThreadRng,
    entity: &mut EncounterEntity,
) {
    entity.skill_stats = SkillStats::default();
    entity.damage_stats = DamageStats::default();

    let damage_log = entities_damage_log.entry(entity.name.clone()).or_default();

    for skill in entity.skills.values_mut() {
        skill.casts = 0;
        skill.hits = 0;
        skill.crits = 0;

        let mut per_skill_map: BTreeMap<i64, SkillCast> = BTreeMap::new();
        let mut per_skill_vec: Vec<i32> = Vec::new();

        for it in 0..100 {
            let dmg = rng.random_range(damage_range.0..=damage_range.1);
            let is_crit = rng.random_bool(spec.crit_rate);
            let timestamp = it * 1000;
            skill.casts += 1;
            entity.skill_stats.casts += 1;
            skill.hits += 1;
            entity.skill_stats.hits += 1;
            skill.total_damage += dmg;

            let mut skill_hit = SkillHit {
                damage: dmg,
                timestamp,
                ..Default::default()
            };

            if is_crit {
                entity.skill_stats.crits += 1;
                skill_hit.crit = true;
                skill.crits += 1;
            }

            entity.damage_stats.damage_dealt += dmg;

            damage_log.push((timestamp, dmg));

            let skill_cast = SkillCast {
                hits: vec![skill_hit],
                last: timestamp,
                timestamp,
                ..Default::default()
            };

            per_skill_vec.push(timestamp as i32);
            per_skill_map.insert(timestamp, skill_cast);
        }

        cast_log
            .entry(entity.name.clone())
            .or_default()
            .insert(skill.id, per_skill_vec);

        skill_cast_log
            .entry(entity.id)
            .or_default()
            .insert(skill.id, per_skill_map);
    }

    entity.damage_stats.hyper_awakening_damage +=
        rng.random_range(1_000_000_000..2_000_000_000);
}

fn update_damage_taken(
    damage_taken: (i64, i64),
    rng: &mut ThreadRng,
    entity: &mut EncounterEntity,
) {
    entity.damage_stats.damage_taken += rng.random_range(damage_taken.0..damage_taken.1)
}

fn update_buffs_heals_and_absorb(rng: &mut ThreadRng, entity: &mut EncounterEntity) {
    let support_buff_ids = [101u32, 102u32, 103u32];
    let pick_count = rng.random_range(1..=3);

    for buff_id in support_buff_ids.choose_multiple(rng, pick_count) {
        let value = rng.random_range(1_000..=2_000);
        entity.damage_stats.buffed_by_support += value;
        entity.damage_stats.debuffed_by.insert(*buff_id, value / 2);
    }

    entity.damage_stats.buffed_by_identity += rng.random_range(0..=5);
    entity.damage_stats.buffed_by_hat += rng.random_range(0..=5);
    entity.damage_stats.debuffed_by_support += rng.random_range(0..=3);

    let absorb_value = rng.random_range(10_000..=100_000);
    entity.damage_stats.damage_absorbed += absorb_value;
    entity
        .damage_stats
        .damage_absorbed_by
        .insert(1, absorb_value);

    let shield_value = rng.random_range(5_000..=50_000);
    entity.damage_stats.shields_given += shield_value;
    entity.damage_stats.shields_given_by.insert(1, shield_value);
    entity.damage_stats.shields_received += shield_value / 2;
    entity
        .damage_stats
        .shields_received_by
        .insert(1, shield_value / 2);
    entity.damage_stats.damage_absorbed_on_others += shield_value / 3;
    entity
        .damage_stats
        .damage_absorbed_on_others_by
        .insert(1, shield_value / 3);
}

fn get_skills_by_spec(specialisation: &str) -> HashMap<u32, Skill> {
    let mut skills = HashMap::new();

    match specialisation {
        "Mayhem" => {
            for id in [16010, 16640, 16120, 16080, 16300, 16050, 16220, 16030] {
                skills.insert(
                    id,
                    Skill {
                        id,
                        name: format!("Skill {}", id),
                        ..Default::default()
                    },
                );
            }
        }
        "Loyal Companion" => {
            for id in [50010, 28220, 28090, 28250, 28070, 28110, 28130, 28150] {
                skills.insert(
                    id,
                    Skill {
                        id,
                        name: format!("Skill {}", id),
                        ..Default::default()
                    },
                );
            }
        }
        "Esoteric Skill Enhancement" => {
            for id in [22340, 22080, 22120, 22160, 22210, 22240, 22270, 22310] {
                skills.insert(
                    id,
                    Skill {
                        id,
                        name: format!("Skill {}", id),
                        ..Default::default()
                    },
                );
            }
        }
        "Desperate Salvation" => {
            for id in [21290, 21170, 21080, 21160, 21250, 21040, 21020, 21210] {
                skills.insert(
                    id,
                    Skill {
                        id,
                        name: format!("Skill {}", id),
                        ..Default::default()
                    },
                );
            }
        }
        "Drizzle" => {
            for id in [32010, 32150, 32160, 32170, 32190, 32210, 32220, 32230] {
                skills.insert(
                    id,
                    Skill {
                        id,
                        name: format!("Skill {}", id),
                        ..Default::default()
                    },
                );
            }
        }
        "Barrage Enhancement" => {
            for id in [30260, 30270, 30290, 30340, 30392, 30320, 30310, 30380] {
                skills.insert(
                    id,
                    Skill {
                        id,
                        name: format!("Skill {}", id),
                        ..Default::default()
                    },
                );
            }
        }
        "Remaining Energy" => {
            for id in [25010, 25180, 25160, 25110, 25120, 25030, 25040, 25050] {
                skills.insert(
                    id,
                    Skill {
                        id,
                        name: format!("Skill {}", id),
                        ..Default::default()
                    },
                );
            }
        }
        "Blessed Aura" => {
            for id in [36080, 36120, 36200, 36170, 36800, 36040, 36020, 36220] {
                skills.insert(
                    id,
                    Skill {
                        id,
                        name: format!("Skill {}", id),
                        ..Default::default()
                    },
                );
            }
        }
        _ => {}
    }

    skills
}

fn build_entity_from_spec(name: &str, spec: &PlayerSpec, idx: usize) -> EncounterEntity {
    let skills = get_skills_by_spec(&spec.specialisation);

    let entity = EncounterEntity {
        id: idx as u64 + 1,
        character_id: idx as u64 + 101,
        npc_id: 0,
        name: name.to_string(),
        entity_type: EntityType::Player,
        class_id: spec.class_id,
        class: spec.class_name.clone(),
        gear_score: spec.gear_score,
        current_hp: spec.hp,
        max_hp: spec.hp,
        current_shield: 0,
        is_dead: false,
        skills,
        damage_stats: DamageStats::default(),
        skill_stats: SkillStats::default(),
        ..Default::default()
    };

    entity
}

pub fn generate_entities_for_parties(
    parties: &[Vec<PlayerSpec>],
) -> (HashMap<String, (PlayerSpec, EncounterEntity)>, Vec<String>) {
    let mut entities: HashMap<String, (PlayerSpec, EncounterEntity)> = HashMap::new();
    let mut player_names = Vec::new();

    for party in parties {
        for (idx, spec) in party.iter().enumerate() {
            let name = format!("Player{}", player_names.len() + 1);
            let entity = build_entity_from_spec(&name, spec, idx);

            entities.insert(name.clone(), (spec.clone(), entity));
            player_names.push(name);
        }
    }

    (entities, player_names)
}

pub fn build_args(version: &str) -> InsertEncounterArgs {
    let player11 = PlayerSpec {
        class_id: 102,
        class_name: "Berserker".to_string(),
        specialisation: "Mayhem",
        crit_rate: 0.25,
        gear_score: 1620.0,
        hp: 1_000_000,
        info: InspectInfo {
            combat_power: Some(CombatPower {
                id: 1,
                score: 1800.0,
            }),
            ark_passive_enabled: true,
            ark_passive_data: Some(ArkPassiveData {
                evolution: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                enlightenment: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                leap: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
            }),
            engravings: Some(vec![1118, 1299]),
            gems: Some(vec![
                GemData {
                    tier: 2,
                    skill_id: 16640,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 16120,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 16080,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 16300,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 16050,
                    gem_type: 63,
                    value: 4400,
                },
            ]),
            loadout_snapshot: Some(String::from("")),
        },
    };
    let player12 = PlayerSpec {
        class_id: 502,
        class_name: "Sharpshooter".to_string(),
        specialisation: "Loyal Companion",
        crit_rate: 0.28,
        gear_score: 1600.0,
        hp: 1_000_000,
        info: InspectInfo {
            combat_power: Some(CombatPower {
                id: 1,
                score: 1800.0,
            }),
            ark_passive_enabled: true,
            ark_passive_data: Some(ArkPassiveData {
                evolution: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                enlightenment: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                leap: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
            }),
            engravings: Some(vec![1118, 1299]),
            gems: Some(vec![
                GemData {
                    tier: 2,
                    skill_id: 50010,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 28220,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 28090,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 28250,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 28070,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 28110,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 28130,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 28150,
                    gem_type: 63,
                    value: 4400,
                },
            ]),
            loadout_snapshot: Some(String::from("")),
        },
    };
    let player13 = PlayerSpec {
        class_id: 302,
        class_name: "Wardancer".to_string(),
        specialisation: "Esoteric Skill Enhancement",
        crit_rate: 0.30,
        gear_score: 1580.0,
        hp: 1_000_000,
        info: InspectInfo {
            combat_power: Some(CombatPower {
                id: 1,
                score: 1800.0,
            }),
            ark_passive_enabled: true,
            ark_passive_data: Some(ArkPassiveData {
                evolution: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                enlightenment: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                leap: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
            }),
            engravings: Some(vec![1118, 1299]),
            gems: Some(vec![
                GemData {
                    tier: 2,
                    skill_id: 22340,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 22080,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 22120,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 22310,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 22270,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 22240,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 22210,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 22160,
                    gem_type: 63,
                    value: 4400,
                },
            ]),
            loadout_snapshot: Some(String::from("")),
        },
    };
    let player14 = PlayerSpec {
        class_id: 204,
        class_name: "Bard".to_string(),
        specialisation: "Desperate Salvation",
        crit_rate: 0.15,
        gear_score: 1500.0,
        hp: 1_000_000,
        info: InspectInfo {
            combat_power: Some(CombatPower {
                id: 2,
                score: 1800.0,
            }),
            ark_passive_enabled: true,
            ark_passive_data: Some(ArkPassiveData {
                evolution: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                enlightenment: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                leap: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
            }),
            engravings: Some(vec![1255, 1251, 1134, 1167, 77300001]),
            gems: Some(vec![
                GemData {
                    tier: 2,
                    skill_id: 21170,
                    gem_type: 35,
                    value: 2400,
                },
                GemData {
                    tier: 2,
                    skill_id: 21080,
                    gem_type: 35,
                    value: 2400,
                },
                GemData {
                    tier: 2,
                    skill_id: 21250,
                    gem_type: 35,
                    value: 2400,
                },
                GemData {
                    tier: 2,
                    skill_id: 21290,
                    gem_type: 35,
                    value: 2400,
                },
                GemData {
                    tier: 2,
                    skill_id: 21160,
                    gem_type: 35,
                    value: 2400,
                },
                GemData {
                    tier: 2,
                    skill_id: 21160,
                    gem_type: 64,
                    value: 1000,
                },
            ]),
            loadout_snapshot: Some(String::from("")),
        },
    };

    let player21 = PlayerSpec {
        class_id: 603,
        class_name: "Aeromancer".to_string(),
        specialisation: "Drizzle",
        crit_rate: 0.25,
        gear_score: 1620.0,
        hp: 0,
        info: InspectInfo {
            combat_power: Some(CombatPower {
                id: 1,
                score: 1800.0,
            }),
            ark_passive_enabled: true,
            ark_passive_data: Some(ArkPassiveData {
                evolution: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                enlightenment: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                leap: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
            }),
            engravings: Some(vec![1118, 1299]),
            gems: Some(vec![
                GemData {
                    tier: 2,
                    skill_id: 32010,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 32150,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 32160,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 32170,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 32190,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 32210,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 32220,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 32230,
                    gem_type: 63,
                    value: 4400,
                },
            ]),
            loadout_snapshot: Some(String::from("")),
        },
    };
    let player22 = PlayerSpec {
        class_id: 504,
        class_name: "Artillerist".to_string(),
        specialisation: "Barrage Enhancement",
        crit_rate: 0.28,
        gear_score: 1600.0,
        hp: 1_000_000,
        info: InspectInfo {
            combat_power: Some(CombatPower {
                id: 1,
                score: 1800.0,
            }),
            ark_passive_enabled: true,
            ark_passive_data: Some(ArkPassiveData {
                evolution: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                enlightenment: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                leap: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
            }),
            engravings: Some(vec![1118, 1299]),
            gems: Some(vec![
                GemData {
                    tier: 2,
                    skill_id: 30260,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 30270,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 30290,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 30340,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 30380,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 30310,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 30320,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 30392,
                    gem_type: 63,
                    value: 4400,
                },
            ]),
            loadout_snapshot: Some(String::from("")),
        },
    };
    let player23 = PlayerSpec {
        class_id: 402,
        class_name: "Deathblade".to_string(),
        specialisation: "Remaining Energy",
        crit_rate: 0.30,
        gear_score: 1580.0,
        hp: 1_000_000,
        info: InspectInfo {
            combat_power: Some(CombatPower {
                id: 1,
                score: 1800.0,
            }),
            ark_passive_enabled: true,
            ark_passive_data: Some(ArkPassiveData {
                evolution: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                enlightenment: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                leap: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
            }),
            engravings: Some(vec![1118, 1299]),
            gems: Some(vec![
                GemData {
                    tier: 2,
                    skill_id: 25010,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 25180,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 25160,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 25110,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 25120,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 25030,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 25040,
                    gem_type: 63,
                    value: 4400,
                },
                GemData {
                    tier: 2,
                    skill_id: 25050,
                    gem_type: 63,
                    value: 4400,
                },
            ]),
            loadout_snapshot: Some(String::from("")),
        },
    };
    let player24 = PlayerSpec {
        class_id: 105,
        class_name: "Paladin".to_string(),
        specialisation: "Blessed Aura",
        crit_rate: 0.15,
        gear_score: 1500.0,
        hp: 1_000_000,
        info: InspectInfo {
            combat_power: Some(CombatPower {
                id: 2,
                score: 1800.0,
            }),
            ark_passive_enabled: true,
            ark_passive_data: Some(ArkPassiveData {
                evolution: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                enlightenment: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
                leap: Some(vec![ArkPassiveNode { id: 1, lv: 1 }]),
            }),
            engravings: Some(vec![1255, 1251, 1134, 1167, 77300001]),
            gems: Some(vec![
                GemData {
                    tier: 2,
                    skill_id: 36080,
                    gem_type: 35,
                    value: 2400,
                },
                GemData {
                    tier: 2,
                    skill_id: 36120,
                    gem_type: 35,
                    value: 2400,
                },
                GemData {
                    tier: 2,
                    skill_id: 36220,
                    gem_type: 35,
                    value: 2400,
                },
                GemData {
                    tier: 2,
                    skill_id: 36170,
                    gem_type: 35,
                    value: 2400,
                },
                GemData {
                    tier: 2,
                    skill_id: 36200,
                    gem_type: 35,
                    value: 2400,
                },
                GemData {
                    tier: 2,
                    skill_id: 36200,
                    gem_type: 64,
                    value: 1000,
                },
            ]),
            loadout_snapshot: Some(String::from("")),
        },
    };

    let raid_builder = RaidBuilder::new()
        .add_party((player11, player12, player13, player14))
        .add_party((player21, player22, player23, player24))
        .set_boss(
            "Mordum, the Abyssal Punisher",
            485800,
            1_100_000_000_000,
            15,
        )
        .set_region("EUC")
        .set_version(version)
        .set_damage_range(1_000_000, 2_000_000)
        .set_difficulty("Hard")
        .set_cleared(true);

    let args = raid_builder.build();

    args
}