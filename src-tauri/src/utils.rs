#![allow(dead_code)]

use hashbrown::HashMap;

use crate::{
    data::RAID_MAP,
    models::{ArkPassiveNode, EncounterEntity, EntityType, StatusEffect},
};

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

pub fn get_spec_from_ark_passive(node: &ArkPassiveNode) -> String {
    match node.id {
        2160000 => "Berserker Technique",
        2160010 => "Mayhem",
        2170000 => "Lone Knight",
        2170010 => "Combat Readiness",
        2180000 => "Rage Hammer",
        2180010 => "Gravity Training",
        2360000 => "Judgment",
        2360010 => "Blessed Aura",
        2450000 => "Punisher",
        2450010 => "Predator",
        2480000 => "Shining Knight",
        2480100 => "Liberator",
        2230000 => "Ultimate Skill: Taijutsu",
        2230100 => "Shock Training",
        2220000 => "First Intention",
        2220100 => "Esoteric Skill Enhancement",
        2240000 => "Energy Overflow",
        2240100 => "Robust Spirit",
        2340000 => "Control",
        2340100 => "Pinnacle",
        2470000 => "Brawl King Storm",
        2470100 => "Asura's Path",
        2390000 => "Esoteric Flurry",
        2390010 => "Deathblow",
        2300000 => "Barrage Enhancement",
        2300100 => "Firepower Enhancement",
        2290000 => "Enhanced Weapon",
        2290100 => "Pistoleer",
        2280000 => "Death Strike",
        2280100 => "Loyal Companion",
        2350000 => "Evolutionary Legacy",
        2350100 => "Arthetinean Skill",
        2380000 => "Peacemaker",
        2380100 => "Time to Hunt",
        2370000 => "Igniter",
        2370100 => "Reflux",
        2190000 => "Grace of the Empress",
        2190100 => "Order of the Emperor",
        2200000 => "Communication Overflow",
        2200100 => "Master Summoner",
        2210000 => "Desperate Salvation",
        2210100 => "True Courage",
        2270000 => "Demonic Impulse",
        2270600 => "Perfect Suppression",
        2250000 => "Surge",
        2250600 => "Remaining Energy",
        2260000 => "Lunar Voice",
        2260600 => "Hunger",
        2460000 => "Full Moon Harvester",
        2460600 => "Night's Edge",
        2320000 => "Wind Fury",
        2320600 => "Drizzle",
        2310000 => "Full Bloom",
        2310600 => "Recurrence",
        2330000 => "Ferality",
        2330100 => "Phantom Beast Awakening",
        _ => "Unknown",
    }
    .to_string()
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

pub fn damage_gem_value_to_level(value: u32, tier: u8) -> u8 {
    if tier == 4 {
        match value {
            4400 => 10,
            4000 => 9,
            3600 => 8,
            3200 => 7,
            2800 => 6,
            2400 => 5,
            2000 => 4,
            1600 => 3,
            1200 => 2,
            800 => 1,
            _ => 0,
        }
    } else {
        match value {
            4000 => 10,
            3000 => 9,
            2400 => 8,
            2100 => 7,
            1800 => 6,
            1500 => 5,
            1200 => 4,
            900 => 3,
            600 => 2,
            300 => 1,
            _ => 0,
        }
    }
}

pub fn cooldown_gem_value_to_level(value: u32, tier: u8) -> u8 {
    if tier == 4 {
        match value {
            2400 => 10,
            2200 => 9,
            2000 => 8,
            1800 => 7,
            1600 => 6,
            1400 => 5,
            1200 => 4,
            1000 => 3,
            800 => 2,
            600 => 1,
            _ => 0,
        }
    } else {
        match value {
            2000 => 10,
            1800 => 9,
            1600 => 8,
            1400 => 7,
            1200 => 6,
            1000 => 5,
            800 => 4,
            600 => 3,
            400 => 2,
            200 => 1,
            _ => 0,
        }
    }
}

pub fn support_damage_gem_value_to_level(value: u32) -> u8 {
    match value {
        1000 => 10,
        900 => 9,
        800 => 8,
        700 => 7,
        600 => 6,
        500 => 5,
        400 => 4,
        300 => 3,
        200 => 2,
        100 => 1,
        _ => 0,
    }
}

pub fn get_buff_names(player: &EncounterEntity, buffs: &HashMap<u32, StatusEffect>) -> Vec<String> {
    let mut names = Vec::new();
    for (id, _) in player.damage_stats.buffed_by.iter() {
        if let Some(buff) = buffs.get(id) {
            names.push(buff.source.name.clone());
        }
    }

    names
}

pub fn get_player_spec(
    player: &EncounterEntity,
    buffs: &HashMap<u32, StatusEffect>,
    skip_min_check: bool,
) -> String {
    if !skip_min_check && player.skills.len() < 8 {
        return "Unknown".to_string();
    }

    match player.class.as_str() {
        "Berserker" => {
            // if has bloody rush
            if player.skills.contains_key(&16140)
                || player.skills.contains_key(&16145)
                || player.skills.contains_key(&16146)
                || player.skills.contains_key(&16147)
            {
                "Berserker Technique"
            } else {
                "Mayhem"
            }
        }
        "Destroyer" => {
            // chain strike or vortex gravity or basic attack is highest dps
            if player.skills.contains_key(&18260)
                || player.skills.contains_key(&18011)
                || player
                    .skills
                    .values()
                    .max_by(|a, b| a.total_damage.cmp(&b.total_damage))
                    .is_some_and(|s| s.name == "Basic Attack")
            {
                "Gravity Training"
            } else {
                "Rage Hammer"
            }
        }
        "Gunlancer" => {
            // todo
            // surge cannon and no guardian thundercrack
            if player.skills.contains_key(&17200) && !player.skills.contains_key(&17140) {
                "Lone Knight"
            } else if player.skills.contains_key(&17140) && player.skills.contains_key(&17110) {
                // has guardian thundercrack and leap attack
                "Combat Readiness"
            } else {
                "Princess"
            }
        }
        "Paladin" => {
            // if has execution of judgement, judgement blade, or flash slash strength release tripod
            if player.skills.contains_key(&36250)
                || player.skills.contains_key(&36270)
                || player
                    .skills
                    .get(&36090)
                    .is_some_and(|s| s.tripod_index.is_some_and(|t| t.second == 3))
            {
                "Judgment"
            } else if player.skills.contains_key(&36200)
                || player.skills.contains_key(&36170)
                || player.skills.contains_key(&36800)
            {
                // if has heavenly blessing, wrath of god, or holy aura
                "Blessed Aura"
            } else {
                "Unknown"
            }
        }
        "Slayer" => {
            if player.skills.contains_key(&45004) {
                "Punisher"
            } else {
                "Predator"
            }
        }
        "Valkyrie" => {
            if player.skills.contains_key(&48060)
                || player.skills.contains_key(&48070)
                || player.skills.contains_key(&48500)
                || player.skills.contains_key(&48100)
            {
                // shining knight, final splendor, cataclysm, foresight slash
                "Shining Knight"
            } else if player.skills.contains_key(&48250)
                || player.skills.contains_key(&48270)
                || player.skills.contains_key(&48230)
                || player.skills.contains_key(&48220)
                || player.skills.contains_key(&48040)
                || player.skills.contains_key(&48041)
                || player.skills.contains_key(&48042)
            {
                // seraphic oath, seraphic leap,
                // circle of truth, truth's decree
                // release light
                "Liberator"
            } else {
                "Unknown"
            }
        }
        "Arcanist" => {
            if player.skills.contains_key(&19282) {
                "Order of the Emperor"
            } else {
                "Grace of the Empress"
            }
        }
        "Summoner" => {
            if player
                .skills
                .iter()
                .any(|(_, skill)| skill.name.contains("Kelsion"))
            {
                "Communication Overflow"
            } else {
                "Master Summoner"
            }
        }
        "Bard" => {
            // if has tempest skill has damage
            if (player.skills.get(&21147).is_some_and(|s| s.total_damage > 0)
                || player.skills.get(&21148).is_some_and(|s| s.total_damage > 0)
                || player.skills.get(&21149).is_some_and(|s| s.total_damage > 0))
            {
                return "True Courage".to_string();
            } else if player
                .skills
                .get(&21160)
                .is_some_and(|s| s.tripod_index.is_some_and(|t| t.third == 1))
            {
                // if heavenly tune has atk pwr tripod
                return "Desperate Salvation".to_string();
            }

            "Unknown"
        }
        "Sorceress" => {
            // if has arcane rupture
            if player.skills.contains_key(&37100) || player.skills.contains_key(&37101) {
                "Igniter"
            } else {
                "Reflux"
            }
        }
        "Wardancer" => {
            // has esoteric origin
            if player.skills.contains_key(&22400) {
                "First Intention"
            } else {
                "Esoteric Skill Enhancement"
            }
        }
        "Scrapper" => {
            // has blazing bombardment
            if player.skills.contains_key(&23420) {
                "Ultimate Skill: Taijutsu"
            } else {
                "Shock Training"
            }
        }
        "Soulfist" => {
            // if has hype level 1 or 2, or no hype
            if player.skills.contains_key(&24020)
                || player.skills.contains_key(&24021)
                || player.skills.values().all(|s| !s.name.contains("Hype"))
            {
                "Energy Overflow"
            } else {
                "Robust Spirit"
            }
        }
        "Glaivier" => {
            if player.skills.contains_key(&34590) {
                "Pinnacle"
            } else {
                "Control"
            }
        }
        "Striker" => {
            // has charging kick
            if player.skills.contains_key(&39360) {
                "Deathblow"
            } else {
                "Esoteric Flurry"
            }
        }
        "Breaker" => {
            if player.skills.contains_key(&47020) {
                "Asura's Path"
            } else {
                "Brawl King Storm"
            }
        }
        "Deathblade" => {
            if player.skills.contains_key(&25038) {
                "Surge"
            } else {
                "Remaining Energy"
            }
        }
        "Shadowhunter" => {
            if player.skills.contains_key(&27860) {
                "Demonic Impulse"
            } else {
                "Perfect Suppression"
            }
        }
        "Reaper" => {
            let buff_names = get_buff_names(player, buffs);
            if buff_names.iter().any(|s| s.contains("Lunar Voice")) {
                "Lunar Voice"
            } else {
                "Hunger"
            }
        }
        "Souleater" => {
            // has deathlord mode
            if player.skills.contains_key(&46050) {
                "Full Moon Harvester"
            } else {
                "Night's Edge"
            }
        }
        "Sharpshooter" => {
            let buff_names = get_buff_names(player, buffs);
            if buff_names
                .iter()
                .any(|s| s.contains("Loyal Companion") || s.contains("Hawk Support"))
            {
                "Loyal Companion"
            } else {
                "Death Strike"
            }
        }
        "Deadeye" => {
            // has secret weapon or quick fire
            if player.skills.contains_key(&29380) || player.skills.contains_key(&29330) {
                "Pistoleer"
            } else {
                "Enhanced Weapon"
            }
        }
        "Artillerist" => {
            // if has barrage attack (disables wheelchair)
            if player.skills.contains_key(&30370) {
                "Firepower Enhancement"
            } else {
                "Barrage Enhancement"
            }
        }
        "Machinist" => {
            let buff_names = get_buff_names(player, buffs);
            if buff_names
                .iter()
                .any(|s| s.contains("Combat Mode") || s.contains("Evolutionary Legacy"))
            {
                "Evolutionary Legacy"
            } else {
                "Arthetinean Skill"
            }
        }
        "Gunslinger" => {
            // has rose blossom
            if player.skills.contains_key(&38340) {
                "Time to Hunt"
            } else {
                "Peacemaker"
            }
        }
        "Artist" => {
            // dps if has shattering strike damage or rising moon damage
            if player.skills.get(&31060).is_some_and(|s| s.total_damage > 0)
                || player
                    .skills
                    .get(&31145)
                    .is_some_and(|s| s.total_damage > 0)
            {
                return "Recurrence".to_string();
            } else if player
                .skills
                .get(&31400)
                .is_some_and(|s| s.tripod_index.is_some_and(|t| t.third == 1))
            {
                // if sunsketch has atk pwr tripod
                return "Full Bloom".to_string();
            }

            "Unknown"
        }
        "Aeromancer" => {
            if player.skills.contains_key(&32250) && player.skills.contains_key(&32260) {
                "Wind Fury"
            } else {
                "Drizzle"
            }
        }
        "Wildsoul" => {
            if player.skills.contains_key(&33400) || player.skills.contains_key(&33410) {
                "Ferality"
            } else {
                "Phantom Beast Awakening"
            }
        }
        _ => "Unknown",
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
