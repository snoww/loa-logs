use crate::constants::DB_VERSION;
use crate::data::*;
use crate::database::utils::*;
use crate::live::entity_tracker::Entity;
use crate::live::skill_tracker::SkillTracker;
use crate::models::*;
use crate::utils::*;
use anyhow::Result;
use hashbrown::HashMap;
use rusqlite::{params, Transaction};
use serde_json::json;
use std::cmp::{max, Reverse};
use std::collections::BTreeMap;

pub fn encounter_entity_from_entity(entity: &Entity) -> EncounterEntity {
    let mut e = EncounterEntity {
        id: entity.id,
        name: entity.name.clone(),
        entity_type: entity.entity_type,
        npc_id: entity.npc_id,
        class_id: entity.class_id,
        class: get_class_from_id(&entity.class_id),
        gear_score: entity.gear_level,
        ..Default::default()
    };

    if entity.character_id > 0 {
        e.character_id = entity.character_id;
    }

    e
}

pub fn update_player_entity(old: &mut EncounterEntity, new: &Entity) {
    old.id = new.id;
    old.character_id = new.character_id;
    old.name.clone_from(&new.name);
    old.class_id = new.class_id;
    old.class = get_class_from_id(&new.class_id);
    old.gear_score = new.gear_level;
}

pub fn is_support(entity: &EncounterEntity) -> bool {
    if let Some(spec) = &entity.spec {
        is_support_spec(spec)
    } else {
        is_support_class(&entity.class_id)
    }
}

pub fn is_support_class(class_id: &u32) -> bool {
    matches!(class_id, 105 | 204 | 602 | 113)
}

pub fn is_support_spec(spec: &str) -> bool {
    matches!(
        spec,
        "Desperate Salvation" | "Full Bloom" | "Blessed Aura" | "Liberator"
    )
}

pub fn is_battle_item(skill_effect_id: &u32, _item_type: &str) -> bool {
    if let Some(item) = SKILL_EFFECT_DATA.get(skill_effect_id)
        && let Some(category) = item.item_type.as_ref()
    {
        return category == "useup";
    }
    false
}

pub fn get_status_effect_data(buff_id: u32, source_skill: Option<u32>) -> Option<StatusEffect> {
    let buff = SKILL_BUFF_DATA.get(&buff_id)?;

    let buff_category = if buff.buff_category == StatusEffectBuffCategory::Ability
        && [501, 502, 503, 504, 505].contains(&buff.unique_group)
    {
        StatusEffectBuffCategory::DropsOfEther
    } else {
        buff.buff_category
    };

    let target = if buff.target == "none" {
        StatusEffectTarget::OTHER
    } else if buff.target == "self" {
        StatusEffectTarget::SELF
    } else {
        StatusEffectTarget::PARTY
    };

    let buff_type = get_status_effect_buff_type_flags(
        buff.buff_type,
        buff.category,
        buff.per_level_data.get("1").map(|pr| pr.passive_options.as_slice()));

    let mut status_effect = StatusEffect {
        target,
        category: buff.category.as_ref().to_string(),
        buff_category: buff_category.as_ref().to_string(),
        buff_type,
        unique_group: buff.unique_group,
        source: StatusEffectSource {
            name: buff.name.clone()?,
            desc: buff.desc.clone()?,
            icon: buff.icon.clone()?,
            ..Default::default()
        },
    };

    let is_from_player = buff_category == StatusEffectBuffCategory::ClassSkill
        || buff_category == StatusEffectBuffCategory::ArkPassive
        || buff_category == StatusEffectBuffCategory::Identity
        || (buff_category == StatusEffectBuffCategory::Ability && buff.unique_group != 0)
        || buff_category == StatusEffectBuffCategory::SupportBuff;

    if is_from_player {
        if let Some(buff_source_skills) = buff.source_skills.as_ref() {
            if let Some(source_skill) = source_skill {
                let skill = SKILL_DATA.get(&source_skill);
                get_summon_source_skill(skill, &mut status_effect);
            } else {
                // get the first skill that has a name, fall back to first if none
                let source_skill = {
                    let mut first_any = None;
                    let mut first_named = None;
                    for id in buff_source_skills {
                        if let Some(skill) = SKILL_DATA.get(id) {
                            if first_any.is_none() {
                                first_any = Some(skill);
                            }
                            if skill.name.is_some() {
                                first_named = Some(skill);
                                break; // break once skill with name found
                            }
                        }
                    }
                    first_named.or(first_any)
                };
                get_summon_source_skill(source_skill, &mut status_effect);
            }
        } else if let Some(buff_source_skill) = SKILL_DATA.get(&(buff_id / 10)) {
            status_effect.source.skill = Some(buff_source_skill.clone());
        } else if let Some(buff_source_skill) = SKILL_DATA.get(&((buff_id / 100) * 10)) {
            status_effect.source.skill = Some(buff_source_skill.clone());
        } else {
            let skill_id = buff.unique_group / 10;
            let buff_source_skill = SKILL_DATA.get(&skill_id);
            status_effect.source.skill = buff_source_skill.cloned();
        }
    } else if buff_category == StatusEffectBuffCategory::Set && buff.set_name.is_some() {
        status_effect.source.set_name.clone_from(&buff.set_name);
    } else if buff_category == StatusEffectBuffCategory::BattleItem
        && let Some(buff_source_item) = SKILL_EFFECT_DATA.get(&buff_id)
    {
        if let Some(item_name) = buff_source_item.item_name.as_ref() {
            status_effect.source.name.clone_from(item_name);
        }
        if let Some(item_desc) = buff_source_item.item_desc.as_ref() {
            status_effect.source.desc.clone_from(item_desc);
        }
        if let Some(icon) = buff_source_item.icon.as_ref() {
            status_effect.source.icon.clone_from(icon);
        }
    }

    Some(status_effect)
}

fn get_summon_source_skill(skill: Option<&SkillData>, status_effect: &mut StatusEffect) {
    if let Some(skill) = skill {
        if let Some(summon_skills) = skill.summon_source_skills.as_ref() {
            let summon_source_skill = summon_skills.first().unwrap_or(&0);
            if *summon_source_skill > 0
                && let Some(summon_skill) = SKILL_DATA.get(summon_source_skill)
            {
                status_effect.source.skill = Some(summon_skill.clone());
            }
        } else {
            status_effect.source.skill = Some(skill.clone());
        }
    }
}

pub fn get_status_effect_buff_type_flags(
    buff_type: StatusEffectType,
    category: StatusEffectCategory,
    passive_options: Option<&[PassiveOption]>
) -> u32 {
    let mut buff_type_flag = StatusEffectBuffTypeFlags::NONE;

    match buff_type {
        t if t.is_damage_amplify() => buff_type_flag |= StatusEffectBuffTypeFlags::DMG,
        StatusEffectType::MoveSpeedDown | StatusEffectType::AllSpeedDown => {
            buff_type_flag |= StatusEffectBuffTypeFlags::MOVESPEED
        }
        StatusEffectType::ResetCooldown => buff_type_flag |= StatusEffectBuffTypeFlags::COOLDOWN,
        StatusEffectType::ChangeAiPoint | StatusEffectType::AiPointAmplify => {
            buff_type_flag |= StatusEffectBuffTypeFlags::STAGGER
        }
        StatusEffectType::IncreaseIdentityGauge => buff_type_flag |= StatusEffectBuffTypeFlags::RESOURCE,
        _ => {}
    }

    let passive_options = match passive_options {
            Some(value) => value,
            None => {
                return buff_type_flag.bits()
            },
    };

    for option in passive_options {
        let key_stat: StatType = option.key_stat;
        let option_type = option.option_type;
        let buff_flag = if (category == StatusEffectCategory::Buff && option.value >= 0)
            || (category == StatusEffectCategory::Debuff && option.value <= 0)
        {
            StatusEffectBuffTypeFlags::DMG
        } else {
            StatusEffectBuffTypeFlags::DEFENSE
        };

        if option_type == PassiveOptionType::Stat {

            let stat = match STAT_TYPE_MAP.get(&key_stat) {
                Some(&s) => s,
                None => continue,
            };

            if key_stat.is_stag_stat() {
                buff_type_flag |= StatusEffectBuffTypeFlags::STAGGER;
            } else if key_stat.is_cooldown_stat() {
                buff_type_flag |= StatusEffectBuffTypeFlags::COOLDOWN;
            } else if key_stat.is_resource_stat() {
                buff_type_flag |= StatusEffectBuffTypeFlags::RESOURCE;
            } else if key_stat.is_hp_stat() {
                buff_type_flag |= StatusEffectBuffTypeFlags::HP;
            } else if StatType::Def as u32 <= stat && stat <= StatType::MagicalIncRate as u32
                || key_stat.is_endurance_stat() {
                buff_type_flag |= buff_flag;
            } else if StatType::MoveSpeed as u32 <= stat
                && stat <= StatType::VehicleMoveSpeedRate as u32
            {
                buff_type_flag |= StatusEffectBuffTypeFlags::MOVESPEED;
            }
            if key_stat.is_atk_speed_stat() {
                buff_type_flag |= StatusEffectBuffTypeFlags::ATKSPEED;
            } else if key_stat.is_crit_stat() {
                buff_type_flag |= StatusEffectBuffTypeFlags::CRIT;
            } else if StatType::AttackPowerSubRate1 as u32 <= stat
                && stat <= StatType::SkillDamageSubRate2 as u32
                || StatType::FireDamRate as u32 <= stat
                    && stat <= StatType::ElementsDamRate as u32
                || key_stat.is_offensive_stat() {
                buff_type_flag |= buff_flag
            }

            continue;
        }
        
        if option_type == PassiveOptionType::SkillCriticalRatio {
            buff_type_flag |= StatusEffectBuffTypeFlags::CRIT;
            continue;
        }
        
        if option_type.is_skill_option() {
            buff_type_flag |= buff_flag;
            continue;
        }
        
        if option_type.is_cooldown_reduction() {
            buff_type_flag |= StatusEffectBuffTypeFlags::COOLDOWN;
            continue;
        } 
        
        if option_type.is_resource() {
            buff_type_flag |= StatusEffectBuffTypeFlags::RESOURCE;
            continue;
        }
        
        if option_type == PassiveOptionType::CombatEffect
            && let Some(combat_effect) = COMBAT_EFFECT_DATA.get(&option.key_index)
        {
            for effect in combat_effect.effects.iter() {
                for action in effect.actions.iter() {
                    
                    if action.action_type.is_damage_modifier() {
                        buff_type_flag |= StatusEffectBuffTypeFlags::DMG;
                    }
                    
                    if action.action_type.is_crit_modifier() {
                        buff_type_flag |= StatusEffectBuffTypeFlags::CRIT;
                    }
                }
            }
        }
    }

    buff_type_flag.bits()
}

pub type SkillDetails = (
    String,           // name
    String,           // icon
    Option<Vec<u32>>, // summon source skills
    bool,             // if skill is special, i.e. cannot crit or be buffed
    bool,             // is hyper awakening
);

pub fn get_skill_name_and_icon(
    skill_id: u32,
    skill_effect_id: u32,
    skill_tracker: &SkillTracker,
    entity_id: u64,
) -> SkillDetails {
    if (skill_id == 0) && (skill_effect_id == 0) {
        (
            "Bleed".to_string(),
            "buff_168.png".to_string(),
            None,
            false,
            false,
        )
    } else if (skill_effect_id != 0) && (skill_id == 0) {
        if let Some(effect) = SKILL_EFFECT_DATA.get(&skill_effect_id) {
            // if ValueJ is greater than 1,
            // 1 = esther, 2 = fixed, 3 = not used, 4 = orb power
            // these effects are not affected by crits or buffs
            let special = effect.values[9] > 0;

            if let Some(item_name) = effect.item_name.as_ref() {
                return (
                    item_name.clone(),
                    effect.icon.as_ref().cloned().unwrap_or_default(),
                    None,
                    special,
                    false,
                );
            }
            if let Some(source_skill) = effect.source_skills.as_ref() {
                if let Some(skill) = SKILL_DATA.get(source_skill.iter().min().unwrap_or(&0)) {
                    return (
                        skill.name.clone().unwrap_or(skill.id.to_string()),
                        skill.icon.clone().unwrap_or_default(),
                        None,
                        special,
                        skill.is_hyper_awakening,
                    );
                }
            } else if let Some(skill) = SKILL_DATA.get(&(skill_effect_id / 10)) {
                return (
                    skill.name.clone().unwrap_or(skill.id.to_string()),
                    skill.icon.clone().unwrap_or_default(),
                    None,
                    special,
                    skill.is_hyper_awakening,
                );
            }

            if effect.comment.is_empty() {
                (effect.id.to_string(), "".to_string(), None, special, false)
            } else {
                (effect.comment.clone(), "".to_string(), None, special, false)
            }
        } else {
            (skill_id.to_string(), "".to_string(), None, false, false)
        }
    } else if let Some(skill) = SKILL_DATA.get(&skill_id) {
        if let Some(summon_source_skill) = skill.summon_source_skills.as_ref() {
            for source in summon_source_skill {
                if skill_tracker
                    .skill_timestamp
                    .get(&(entity_id, *source))
                    .is_some()
                    && let Some(skill) = SKILL_DATA.get(source)
                {
                    return (
                        skill.name.clone().unwrap_or(skill.id.to_string()) + " (Summon)",
                        skill.icon.clone().unwrap_or_default(),
                        Some(summon_source_skill.clone()),
                        false,
                        skill.is_hyper_awakening,
                    );
                }
            }
            if let Some(skill) = SKILL_DATA.get(summon_source_skill.iter().min().unwrap_or(&0)) {
                (
                    skill.name.clone().unwrap_or(skill.id.to_string()) + " (Summon)",
                    skill.icon.clone().unwrap_or_default(),
                    Some(summon_source_skill.clone()),
                    false,
                    skill.is_hyper_awakening,
                )
            } else {
                (skill_id.to_string(), "".to_string(), None, false, false)
            }
        } else if let Some(source_skill) = skill.source_skills.as_ref() {
            if let Some(skill) = SKILL_DATA.get(source_skill.iter().min().unwrap_or(&0)) {
                (
                    skill.name.clone().unwrap_or(skill.id.to_string()),
                    skill.icon.clone().unwrap_or_default(),
                    None,
                    false,
                    skill.is_hyper_awakening,
                )
            } else {
                (skill_id.to_string(), "".to_string(), None, false, false)
            }
        } else {
            (
                skill.name.clone().unwrap_or(skill.id.to_string()),
                skill.icon.clone().unwrap_or_default(),
                None,
                false,
                skill.is_hyper_awakening,
            )
        }
    } else if let Some(skill) = SKILL_DATA.get(&(skill_id - (skill_id % 10))) {
        (
            skill.name.clone().unwrap_or(skill.id.to_string()),
            skill.icon.clone().unwrap_or_default(),
            None,
            false,
            skill.is_hyper_awakening,
        )
    } else {
        (skill_id.to_string(), "".to_string(), None, false, false)
    }
}

pub fn get_class_from_id(class_id: &u32) -> String {
    let class = match class_id {
        0 => "",
        101 => "Warrior (Male)",
        102 => "Berserker",
        103 => "Destroyer",
        104 => "Gunlancer",
        105 => "Paladin",
        111 => "Female Warrior",
        112 => "Slayer",
        113 => "Valkyrie",
        201 => "Mage",
        202 => "Arcanist",
        203 => "Summoner",
        204 => "Bard",
        205 => "Sorceress",
        301 => "Martial Artist (Female)",
        302 => "Wardancer",
        303 => "Scrapper",
        304 => "Soulfist",
        305 => "Glaivier",
        311 => "Martial Artist (Male)",
        312 => "Striker",
        313 => "Breaker",
        401 => "Assassin",
        402 => "Deathblade",
        403 => "Shadowhunter",
        404 => "Reaper",
        405 => "Souleater",
        501 => "Gunner (Male)",
        502 => "Sharpshooter",
        503 => "Deadeye",
        504 => "Artillerist",
        505 => "Machinist",
        511 => "Gunner (Female)",
        512 => "Gunslinger",
        601 => "Specialist",
        602 => "Artist",
        603 => "Aeromancer",
        604 => "Wildsoul",
        _ => "Unknown",
    };

    class.to_string()
}

pub fn is_hat_buff(buff_id: &u32) -> bool {
    matches!(buff_id, 362600 | 212305 | 319503 | 319504 | 485100)
}

pub fn is_hyper_hat_buff(buff_id: &u32) -> bool {
    matches!(buff_id, 362601 | 212306 | 319506 | 485101)
}

pub fn check_tripod_index_change(before: Option<TripodIndex>, after: Option<TripodIndex>) -> bool {
    if before.is_none() && after.is_none() {
        return false;
    }

    if before.is_none() || after.is_none() {
        return true;
    }

    let before = before.unwrap();
    let after = after.unwrap();

    before != after
}

pub fn check_tripod_level_change(before: Option<TripodLevel>, after: Option<TripodLevel>) -> bool {
    if before.is_none() && after.is_none() {
        return false;
    }

    if before.is_none() || after.is_none() {
        return true;
    }

    let before = before.unwrap();
    let after = after.unwrap();

    before != after
}

const WINDOW_MS: i64 = 5_000;
const WINDOW_S: i64 = 5;

#[allow(clippy::too_many_arguments)]
pub fn insert_data(
    tx: &Transaction,
    mut encounter: Encounter,
    damage_log: HashMap<String, Vec<(i64, i64)>>,
    cast_log: HashMap<String, HashMap<u32, Vec<i32>>>,
    boss_hp_log: HashMap<String, Vec<BossHpLog>>,
    raid_clear: bool,
    party_info: Vec<Vec<String>>,
    raid_difficulty: String,
    region: Option<String>,
    player_info: Option<HashMap<String, InspectInfo>>,
    meter_version: String,
    ntp_fight_start: i64,
    rdps_valid: bool,
    manual: bool,
    skill_cast_log: HashMap<u64, HashMap<u32, BTreeMap<i64, SkillCast>>>,
    skill_cooldowns: HashMap<u32, Vec<CastEvent>>,
) -> Result<i64> {
    let mut encounter_stmt = tx
        .prepare_cached(
            "
    INSERT INTO encounter (
        last_combat_packet,
        total_damage_dealt,
        top_damage_dealt,
        total_damage_taken,
        top_damage_taken,
        dps,
        buffs,
        debuffs,
        total_shielding,
        total_effective_shielding,
        applied_shield_buffs,
        misc,
        version,
        boss_hp_log
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        )
        .expect("failed to prepare encounter statement");

    encounter.duration = encounter.last_combat_packet - encounter.fight_start;
    let duration_seconds = max(encounter.duration / 1000, 1);
    encounter.encounter_damage_stats.dps =
        encounter.encounter_damage_stats.total_damage_dealt / duration_seconds;

    let misc: EncounterMisc = EncounterMisc {
        raid_clear: if raid_clear { Some(true) } else { None },
        party_info: if party_info.is_empty() {
            None
        } else {
            Some(
                party_info
                    .iter()
                    .enumerate()
                    .map(|(index, party)| (index as i32, party.clone()))
                    .collect(),
            )
        },
        region,
        version: Some(meter_version),
        rdps_valid: Some(rdps_valid),
        rdps_message: if rdps_valid {
            None
        } else {
            Some("invalid_stats".to_string())
        },
        ntp_fight_start: Some(ntp_fight_start),
        manual_save: Some(manual),
        ..Default::default()
    };

    let compressed_boss_hp = compress_json(&boss_hp_log)?;
    let compressed_buffs = compress_json(&encounter.encounter_damage_stats.buffs)?;
    let compressed_debuffs = compress_json(&encounter.encounter_damage_stats.debuffs)?;
    let compressed_shields = compress_json(&encounter.encounter_damage_stats.applied_shield_buffs)?;

    encounter_stmt
        .execute(params![
            encounter.last_combat_packet,
            encounter.encounter_damage_stats.total_damage_dealt,
            encounter.encounter_damage_stats.top_damage_dealt,
            encounter.encounter_damage_stats.total_damage_taken,
            encounter.encounter_damage_stats.top_damage_taken,
            encounter.encounter_damage_stats.dps,
            compressed_buffs,
            compressed_debuffs,
            encounter.encounter_damage_stats.total_shielding,
            encounter.encounter_damage_stats.total_effective_shielding,
            compressed_shields,
            json!(misc),
            DB_VERSION,
            compressed_boss_hp,
        ])
        .expect("failed to insert encounter");

    let last_insert_id = tx.last_insert_rowid();

    let mut entity_stmt = tx
        .prepare_cached(
            "
    INSERT INTO entity (
        name,
        encounter_id,
        npc_id,
        entity_type,
        class_id,
        class,
        gear_score,
        current_hp,
        max_hp,
        is_dead,
        skills,
        damage_stats,
        skill_stats,
        dps,
        character_id,
        engravings,
        loadout_hash,
        combat_power,
        ark_passive_active,
        spec,
        ark_passive_data,
        support_ap,
        support_brand,
        support_identity,
        support_hyper,
        unbuffed_damage,
        unbuffed_dps
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27)",
        )
        .expect("failed to prepare entity statement");

    let fight_start = encounter.fight_start;
    let fight_end = encounter.last_combat_packet;

    // get average support buffs for supports
    let mut buffs = HashMap::new();
    for party in party_info.iter() {
        let party_members: Vec<_> = encounter
            .entities
            .iter()
            .filter(|(name, _)| party.contains(name))
            .map(|(name, entity)| entity)
            .collect();

        // specs are not determined for dps classes, but should be set for supports
        let party_without_support: Vec<_> = party_members
            .iter()
            .filter(|entity| !is_support(entity))
            .collect();

        if party_members.len() - party_without_support.len() == 1 {
            let party_damage_total: i64 = party_without_support
                .iter()
                .map(|e| get_damage_without_hyper_or_special(e))
                .sum();

            if party_damage_total <= 0 {
                continue;
            }

            let mut average_brand = 0.0;
            let mut average_buff = 0.0;
            let mut average_identity = 0.0;
            let mut average_hyper = 0.0;

            for player in party_without_support {
                let damage_dealt = get_damage_without_hyper_or_special(player) as f64;

                if damage_dealt <= 0.0 {
                    continue;
                }

                let party_damage_percent = damage_dealt / party_damage_total as f64;

                let brand_ratio = player.damage_stats.debuffed_by_support as f64 / damage_dealt;
                let buff_ratio = player.damage_stats.buffed_by_support as f64 / damage_dealt;
                let identity_ratio = player.damage_stats.buffed_by_identity as f64 / damage_dealt;

                average_brand += brand_ratio * party_damage_percent;
                average_buff += buff_ratio * party_damage_percent;
                average_identity += identity_ratio * party_damage_percent;
                average_hyper += (player.damage_stats.buffed_by_hat as f64
                    / player.damage_stats.damage_dealt as f64)
                    * party_damage_percent;
            }

            if let Some(support) = party_members.iter().find(|entity| is_support(entity)) {
                buffs.insert(
                    support.name.clone(),
                    SupportBuffs {
                        brand: average_brand,
                        buff: average_buff,
                        identity: average_identity,
                        hyper: average_hyper,
                    },
                );
            }
        }
    }

    for (_key, entity) in encounter.entities.iter_mut().filter(|(_, e)| {
        ((e.entity_type == EntityType::Player && e.class_id > 0)
            || e.name == encounter.local_player
            || e.entity_type == EntityType::Esther
            || (e.entity_type == EntityType::Boss && e.max_hp > 0))
            && e.damage_stats.damage_dealt > 0
    }) {
        if entity.entity_type == EntityType::Player {
            let intervals = generate_intervals(fight_start, fight_end);
            if let Some(damage_log) = damage_log.get(&entity.name) {
                if !intervals.is_empty() {
                    for interval in intervals {
                        let start = fight_start + interval - WINDOW_MS;
                        let end = fight_start + interval + WINDOW_MS;

                        let damage = sum_in_range(damage_log, start, end);
                        entity
                            .damage_stats
                            .dps_rolling_10s_avg
                            .push(damage / (WINDOW_S * 2));
                    }
                }
                let fight_start_sec = encounter.fight_start / 1000;
                let fight_end_sec = encounter.last_combat_packet / 1000;
                entity.damage_stats.dps_average =
                    calculate_average_dps(damage_log, fight_start_sec, fight_end_sec);
            }

            let spec = get_player_spec(entity, &encounter.encounter_damage_stats.buffs, false);
            entity.spec = Some(spec.clone());

            if let Some(info) = player_info
                .as_ref()
                .and_then(|stats| stats.get(&entity.name))
            {
                for gem in info.gems.iter().flatten() {
                    let skill_ids = if matches!(gem.gem_type, 34 | 35 | 65 | 63 | 61) {
                        GEM_SKILL_MAP
                            .get(&gem.skill_id)
                            .cloned()
                            .unwrap_or_default()
                    } else {
                        vec![gem.skill_id]
                    };

                    for skill_id in skill_ids {
                        if let Some(skill) = entity.skills.get_mut(&skill_id) {
                            match gem.gem_type {
                                27 | 35 => {
                                    // cooldown gems
                                    skill.gem_cooldown =
                                        Some(cooldown_gem_value_to_level(gem.value, gem.tier));
                                    skill.gem_tier = Some(gem.tier);
                                }
                                64 | 65 => {
                                    // support effect damage gems
                                    skill.gem_damage =
                                        Some(support_damage_gem_value_to_level(gem.value));
                                    skill.gem_tier_dmg = Some(gem.tier);
                                }
                                _ => {
                                    // damage gems
                                    skill.gem_damage =
                                        Some(damage_gem_value_to_level(gem.value, gem.tier));
                                    skill.gem_tier_dmg = Some(gem.tier);
                                }
                            }
                        }
                    }
                }

                entity.ark_passive_active = Some(info.ark_passive_enabled);

                let engravings = get_engravings(&info.engravings);
                if entity.class_id == 104
                    && engravings.as_ref().is_some_and(|engravings| {
                        engravings
                            .iter()
                            .any(|e| e == "Awakening" || e == "Drops of Ether")
                    })
                {
                    entity.spec = Some("Princess".to_string());
                } else if spec == "Unknown" {
                    // not reliable enough to be used on its own
                    if let Some(tree) = info.ark_passive_data.as_ref()
                        && let Some(enlightenment) = tree.enlightenment.as_ref()
                    {
                        for node in enlightenment.iter() {
                            let spec = get_spec_from_ark_passive(node);
                            if spec != "Unknown" {
                                entity.spec = Some(spec);
                                break;
                            }
                        }
                    }
                }

                if entity.combat_power.is_none() {
                    entity.combat_power = info.combat_power.as_ref().map(|c| c.score);
                }

                entity.engraving_data = engravings;
                entity.ark_passive_data = info.ark_passive_data.clone();
                entity.loadout_hash = info.loadout_snapshot.clone();
            }

            entity.damage_stats.unbuffed_dps =
                entity.damage_stats.unbuffed_damage / duration_seconds;
        }

        if entity.name == encounter.local_player {
            for (skill_id, events) in skill_cooldowns.iter() {
                if let Some(skill) = entity.skills.get_mut(skill_id) {
                    skill.time_available =
                        Some(get_total_available_time(events, fight_start, fight_end));
                }
            }
        }

        entity.damage_stats.dps = entity.damage_stats.damage_dealt / duration_seconds;

        for (_, skill) in entity.skills.iter_mut() {
            skill.dps = skill.total_damage / duration_seconds;
        }

        for (_, cast_log) in cast_log.iter().filter(|&(s, _)| *s == entity.name) {
            for (skill, log) in cast_log {
                entity.skills.entry(*skill).and_modify(|e| {
                    e.cast_log.clone_from(log);
                });
            }
        }

        for (_, skill_cast_log) in skill_cast_log.iter().filter(|&(s, _)| *s == entity.id) {
            for (skill, log) in skill_cast_log {
                entity.skills.entry(*skill).and_modify(|e| {
                    let average_cast = e.total_damage as f64 / e.casts as f64;
                    let filter = average_cast * 0.05;
                    let mut adj_hits = 0;
                    let mut adj_crits = 0;
                    for cast in log.values() {
                        for hit in cast.hits.iter() {
                            if hit.damage as f64 > filter {
                                adj_hits += 1;
                                if hit.crit {
                                    adj_crits += 1;
                                }
                            }
                        }
                    }

                    if adj_hits > 0 {
                        e.adjusted_crit = Some(adj_crits as f64 / adj_hits as f64);
                    }

                    e.max_damage_cast = log
                        .values()
                        .map(|cast| cast.hits.iter().map(|hit| hit.damage).sum::<i64>())
                        .max()
                        .unwrap_or_default();
                    e.skill_cast_log = log.values().cloned().collect();
                });
            }
        }

        let compressed_skills = compress_json(&entity.skills)?;
        let compressed_damage_stats = compress_json(&entity.damage_stats)?;

        let damage_dealt = entity.damage_stats.damage_dealt;
        let damage_without_hyper =
            (damage_dealt - entity.damage_stats.hyper_awakening_damage) as f64;
        let support_buffs = buffs.get(&entity.name);

        entity_stmt
            .execute(params![
                entity.name,
                last_insert_id,
                entity.npc_id,
                entity.entity_type.to_string(),
                entity.class_id,
                entity.class,
                entity.gear_score,
                entity.current_hp,
                entity.max_hp,
                entity.is_dead,
                compressed_skills,
                compressed_damage_stats,
                json!(entity.skill_stats),
                entity.damage_stats.dps,
                entity.character_id,
                json!(entity.engraving_data),
                entity.loadout_hash,
                entity.combat_power,
                entity.ark_passive_active,
                entity.spec,
                json!(entity.ark_passive_data),
                support_buffs
                    .map(|b| b.buff)
                    .unwrap_or(entity.damage_stats.buffed_by_support as f64 / damage_without_hyper),
                support_buffs.map(|b| b.brand).unwrap_or(
                    entity.damage_stats.debuffed_by_support as f64 / damage_without_hyper
                ),
                support_buffs.map(|b| b.identity).unwrap_or(
                    entity.damage_stats.buffed_by_identity as f64 / damage_without_hyper
                ),
                support_buffs
                    .map(|b| b.hyper)
                    .unwrap_or(entity.damage_stats.buffed_by_hat as f64 / damage_without_hyper),
                entity.damage_stats.unbuffed_damage,
                entity.damage_stats.unbuffed_dps
            ])
            .expect("failed to insert entity");
    }

    let mut players = encounter
        .entities
        .values()
        .filter(|e| {
            ((e.entity_type == EntityType::Player && e.class_id > 0)
                || e.name == encounter.local_player)
                && e.damage_stats.damage_dealt > 0
        })
        .collect::<Vec<_>>();
    let local_player_dps = players
        .iter()
        .find(|e| e.name == encounter.local_player)
        .map(|e| e.damage_stats.dps)
        .unwrap_or_default();
    players.sort_unstable_by_key(|e| Reverse(e.damage_stats.damage_dealt));
    let preview_players = players
        .into_iter()
        .map(|e| format!("{}:{}", e.class_id, e.name))
        .collect::<Vec<_>>()
        .join(",");

    let mut encounter_preview_stmt = tx
        .prepare_cached(
            "
    INSERT INTO encounter_preview (
        id,
        fight_start,
        current_boss,
        duration,
        players,
        difficulty,
        local_player,
        my_dps,
        cleared,
        boss_only_damage
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        )
        .expect("failed to prepare encounter preview statement");
    encounter_preview_stmt
        .execute(params![
            last_insert_id,
            encounter.fight_start,
            encounter.current_boss_name,
            encounter.duration,
            preview_players,
            raid_difficulty,
            encounter.local_player,
            local_player_dps,
            raid_clear,
            encounter.boss_only_damage
        ])
        .expect("failed to insert encounter preview");

    Ok(last_insert_id)
}

pub fn map_status_effect(se: &StatusEffectDetails, custom_id_map: &mut HashMap<u32, u32>) -> u32 {
    if se.custom_id > 0 {
        custom_id_map.insert(se.custom_id, se.status_effect_id);
        se.custom_id
    } else {
        se.status_effect_id
    }
}

pub fn is_valid_player(player: &EncounterEntity) -> bool {
    player.gear_score >= 0.0
        && player.entity_type == EntityType::Player
        && player.character_id != 0
        && player.class_id != 0
        && player.name != "You"
        && player
            .name
            .chars()
            .next()
            .unwrap_or_default()
            .is_uppercase()
}

pub fn get_new_id(source_skill: u32) -> u32 {
    source_skill + 1_000_000_000
}

pub fn get_skill_id(new_skill: u32, original_buff_id: u32) -> u32 {
    new_skill - 1_000_000_000 - original_buff_id
}

pub fn update_current_boss_name(boss_name: &str) -> String {
    match boss_name {
        "Chaos Lightning Dragon Jade" => "Argeos",
        "Vicious Argeos" | "Ruthless Lakadroff" | "Untrue Crimson Yoho" | "Despicable Skolakia" => {
            "Behemoth, the Storm Commander"
        }
        _ => boss_name,
    }
    .to_string()
}

pub fn boss_to_raid_map(boss: &str, max_hp: i64) -> Option<String> {
    match boss {
        "Phantom Legion Commander Brelshaza" => {
            if max_hp > 100_000_000_000 {
                Some("Act 2: Brelshaza G2".to_string())
            } else {
                Some("Brelshaza G6".to_string())
            }
        }
        _ => RAID_MAP.get(boss).cloned(),
    }
}

fn get_damage_without_hyper_or_special(e: &EncounterEntity) -> i64 {
    let hyper = e.damage_stats.hyper_awakening_damage;
    let special = e
        .skills
        .values()
        .filter(|s| s.special.unwrap_or(false))
        .map(|s| s.total_damage)
        .sum::<i64>();
    e.damage_stats.damage_dealt - hyper - special
}

pub fn get_status_effect_value(value: Option<Vec<u8>>) -> u64 {
    value.map_or(0, |v| {
        let c1 = v
            .get(0..8)
            .map_or(0, |bytes| u64::from_le_bytes(bytes.try_into().unwrap()));
        let c2 = v
            .get(8..16)
            .map_or(0, |bytes| u64::from_le_bytes(bytes.try_into().unwrap()));
        c1.min(c2)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_calculate_flags() {
        let current_dir = std::env::current_dir().unwrap();
        AssetPreloader::new(&current_dir).unwrap();
       
        for (buff_type, category, passive_options, expected) in setup_test_data() {
            let actual = get_status_effect_buff_type_flags(buff_type, category, passive_options.as_deref());
            assert_eq!(actual, expected, "Failed for StatusEffectType::{:?}\nStatusEffectCategory::{:?}\nPassiveOptions: {:?}\nActual Flags: {:?}\nExpected Flags: {:?}",
                buff_type,
                category,
                passive_options,
                StatusEffectBuffTypeFlags::from_bits(actual).unwrap(),
                StatusEffectBuffTypeFlags::from_bits(expected).unwrap());
        }
    }

    fn setup_test_data() -> Vec<(StatusEffectType, StatusEffectCategory, Option<Vec<PassiveOption>>, u32)> {
        vec![
            (StatusEffectType::SkillDamageAmplify, StatusEffectCategory::Buff, None, StatusEffectBuffTypeFlags::DMG.bits()),
            (StatusEffectType::MoveSpeedDown, StatusEffectCategory::Buff, None, StatusEffectBuffTypeFlags::MOVESPEED.bits()),
            (StatusEffectType::ResetCooldown, StatusEffectCategory::Buff, None, StatusEffectBuffTypeFlags::COOLDOWN.bits()),
            (StatusEffectType::IncreaseIdentityGauge, StatusEffectCategory::Buff, None, StatusEffectBuffTypeFlags::RESOURCE.bits()),
            (StatusEffectType::Other, StatusEffectCategory::Buff, None, StatusEffectBuffTypeFlags::NONE.bits()),

            (
                StatusEffectType::Other,
                StatusEffectCategory::Buff,
                Some(vec![PassiveOption {
                    key_stat: StatType::AttackPowerRate,
                    option_type: PassiveOptionType::Stat,
                    value: 10,
                    key_index: 0,
                }]),
                StatusEffectBuffTypeFlags::DMG.bits()
            ),
            (
                StatusEffectType::Other,
                StatusEffectCategory::Buff,
                Some(vec![PassiveOption {
                    key_stat: StatType::MaxHp,
                    option_type: PassiveOptionType::Stat,
                    value: 5,
                    key_index: 1,
                }]),
                StatusEffectBuffTypeFlags::HP.bits()
            ),
            (
                StatusEffectType::Other,
                StatusEffectCategory::Buff,
                Some(vec![PassiveOption {
                    key_stat: StatType::CriticalHitRate,
                    option_type: PassiveOptionType::SkillCriticalRatio,
                    value: 0,
                    key_index: 2,
                }]),
                StatusEffectBuffTypeFlags::CRIT.bits()
            ),
            (
                StatusEffectType::Other,
                StatusEffectCategory::Debuff,
                Some(vec![PassiveOption {
                    key_stat: StatType::AttackPowerRate,
                    option_type: PassiveOptionType::Stat,
                    value: -10,
                    key_index: 3,
                }]),
                StatusEffectBuffTypeFlags::DMG.bits()
            ),
            (
                StatusEffectType::Other,
                StatusEffectCategory::Other,
                Some(vec![PassiveOption {
                    key_stat: StatType::Other,
                    option_type: PassiveOptionType::SkillCooldownReduction,
                    value: -10,
                    key_index: 3,
                }]),
                StatusEffectBuffTypeFlags::COOLDOWN.bits()
            ),
             (
                StatusEffectType::Other,
                StatusEffectCategory::Other,
                Some(vec![PassiveOption {
                    key_stat: StatType::MoveSpeed,
                    option_type: PassiveOptionType::Stat,
                    value: -10,
                    key_index: 3,
                }]),
                StatusEffectBuffTypeFlags::MOVESPEED.bits()
            ),
            (
                StatusEffectType::Other,
                StatusEffectCategory::Other,
                Some(vec![PassiveOption {
                    key_stat: StatType::Other,
                    option_type: PassiveOptionType::CombatEffect,
                    value: 0,
                    key_index: 1000,
                }]),
                StatusEffectBuffTypeFlags::DMG.bits()
            ),
              (
                StatusEffectType::Other,
                StatusEffectCategory::Other,
                Some(vec![PassiveOption {
                    key_stat: StatType::Other,
                    option_type: PassiveOptionType::CombatEffect,
                    value: 0,
                    key_index: 557300,
                }]),
                StatusEffectBuffTypeFlags::DMG.bits()
            ),
            (
                StatusEffectType::Other,
                StatusEffectCategory::Other,
                Some(vec![PassiveOption {
                    key_stat: StatType::Other,
                    option_type: PassiveOptionType::CombatEffect,
                    value: 0,
                    key_index: 2200400,
                }]),
                StatusEffectBuffTypeFlags::CRIT.bits()
            ),
        ]
    }
}