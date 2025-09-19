#![allow(dead_code)]

use crate::models::{ArkPassiveNode, EncounterEntity};

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
