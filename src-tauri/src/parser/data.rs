use std::{fs::File, thread::{self, JoinHandle}};
use log::error;
use once_cell::sync::Lazy;
use hashbrown::{HashMap, HashSet};

use crate::parser::models::*;

pub static COMBAT_EFFECT_DATA: Lazy<HashMap<i32, CombatEffectData>> = Lazy::new(|| {
    unsafe {
        let reader = File::open("meter-data/CombatEffect.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    }
});

pub static ENGRAVING_DATA: Lazy<HashMap<u32, EngravingData>> = Lazy::new(|| {
    unsafe {
        let reader = File::open("meter-data/Ability.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    }
});

pub static SKILL_BUFF_DATA: Lazy<HashMap<u32, SkillBuffData>> = Lazy::new(|| {
    unsafe {
        let reader = File::open("meter-data/SkillBuff.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    }
});

pub static SKILL_DATA: Lazy<HashMap<u32, SkillData>> = Lazy::new(|| {
    unsafe {
        let reader = File::open("meter-data/Skill.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    }
});

pub static SKILL_EFFECT_DATA: Lazy<HashMap<u32, SkillEffectData>> = Lazy::new(|| {
    unsafe {
        let reader = File::open("meter-data/SkillEffect.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    }
});

pub static SUPPORT_AP_GROUP: Lazy<HashSet<u32>> = Lazy::new(|| {
    let set = HashSet::from([
        101204, // bard
        101105, // paladin
        314004, // artist
        480030, // valkyrie
    ]);

    set
});

pub static SUPPORT_IDENTITY_GROUP: Lazy<HashSet<u32>> = Lazy::new(|| {
    let set = HashSet::from([
        211400, // bard serenade of courage
        368000, // paladin holy aura
        310501, // artist moonfall
        480018, // valkyrie release light
    ]);

    set
});

pub static VALID_ZONES: Lazy<HashSet<u32>> = Lazy::new(|| {
    unsafe {
        let reader = File::open("meter-data/Zone.json").unwrap_unchecked();
        let map: HashMap<u32, String> = serde_json::from_reader(reader).unwrap_unchecked();
        map.keys().cloned().collect()
    }
});

pub static STAT_TYPE_MAP: Lazy<HashMap<String, u32>> = Lazy::new(|| {
    unsafe {
        let reader = File::open("meter-data/StatType.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    }
});

pub static ESTHER_DATA: Lazy<Vec<Esther>> = Lazy::new(|| {
    unsafe {
        let reader = File::open("meter-data/Esther.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    }
});

pub static NPC_DATA: Lazy<HashMap<u32, Npc>> = Lazy::new(|| {
    unsafe {
        let reader = File::open("meter-data/Npc.json").unwrap_unchecked();
        serde_json::from_reader(reader).unwrap_unchecked()
    }
});

pub static GEM_SKILL_MAP: Lazy<HashMap<u32, Vec<u32>>> = Lazy::new(|| {
    unsafe {
        let reader = File::open("meter-data/GemSkillGroup.json").unwrap_unchecked();
        let raw_map: HashMap<String, (String, String, Vec<u32>)> = serde_json::from_reader(reader).unwrap_unchecked();

        raw_map
            .into_iter()
            .filter_map(|(key, entry)| key.parse::<u32>().ok().map(|id| (id, entry.2)))
            .collect()
    }
});

pub static RAID_MAP: Lazy<HashMap<String, String>> = Lazy::new(|| {
     unsafe {
        let reader = File::open("meter-data/encounters.json").unwrap_unchecked();
        let encounters: HashMap<String, HashMap<String, Vec<String>>> = serde_json::from_reader(reader).unwrap_unchecked();

        encounters
            .values()
            .flat_map(|raid| raid.iter())
            .flat_map(|(gate, bosses)| bosses.iter().map(move |boss| (boss.clone(), gate.clone())))
            .collect()
    }
});

pub struct AssetPreloader(Option<JoinHandle<()>>);

impl AssetPreloader {
    pub fn new() -> Self {
        let handle = thread::spawn(|| {
            SKILL_BUFF_DATA.iter().next();
            SKILL_BUFF_DATA.iter().next();
            SKILL_DATA.iter().next();
            VALID_ZONES.iter().next();
            STAT_TYPE_MAP.iter().next();
            ESTHER_DATA.iter().next();
            NPC_DATA.iter().next();
            GEM_SKILL_MAP.iter().next();
        });

        Self(Some(handle))
    }
}

//         let json_str = include_str!("../../meter-data/GemSkillGroup.json");

//         let raw_map: HashMap<String, (String, String, Vec<u32>)> = serde_json::from_str(json_str)
//             .unwrap_or_else(|e| {
//                 error!("Failed to parse GemSkillGroup.json: {}", e);
//                 HashMap::new()
//             });

//         raw_map
//             .into_iter()
//             .filter_map(|(key, entry)| key.parse::<u32>().ok().map(|id| (id, entry.2)))
//             .collect()
//     };

// lazy_static! {
//     pub static ref NPC_DATA: HashMap<u32, Npc> = {
//         let json_str = include_str!("../../meter-data/Npc.json");
//         serde_json::from_str(json_str).unwrap_or_else(|e| {
//             error!("Failed to parse NPC data: {}", e);
//             HashMap::new()
//         })
//     };
//     pub static ref SKILL_DATA: HashMap<u32, SkillData> = {
//         let json_str = include_str!("../../meter-data/Skill.json");
//         serde_json::from_str(json_str).unwrap_or_else(|e| {
//             error!("Failed to parse SkillData: {}", e);
//             HashMap::new()
//         })
//     };
//     pub static ref SKILL_EFFECT_DATA: HashMap<u32, SkillEffectData> = {
//         let json_str = include_str!("../../meter-data/SkillEffect.json");
//         serde_json::from_str(json_str).unwrap_or_else(|e| {
//             error!("Failed to parse SkillEffectData: {}", e);
//             HashMap::new()
//         })
//     };
//     pub static ref SKILL_BUFF_DATA: HashMap<u32, SkillBuffData> = {
//         let json_str = include_str!("../../meter-data/SkillBuff.json");
//         serde_json::from_str(json_str).unwrap_or_else(|e| {
//             error!("Failed to parse SkillBuffData: {}", e);
//             HashMap::new()
//         })
//     };
//     pub static ref COMBAT_EFFECT_DATA: HashMap<i32, CombatEffectData> = {
//         let json_str = include_str!("../../meter-data/CombatEffect.json");
//         serde_json::from_str(json_str).unwrap_or_else(|e| {
//             error!("Failed to parse CombatEffectData: {}", e);
//             HashMap::new()
//         })
//     };
//     pub static ref ENGRAVING_DATA: HashMap<u32, EngravingData> = {
//         let json_str = include_str!("../../meter-data/Ability.json");
//         serde_json::from_str(json_str).unwrap_or_else(|e| {
//             error!("Failed to parse EngravingData: {}", e);
//             HashMap::new()
//         })
//     };
//     pub static ref ESTHER_DATA: Vec<Esther> = {
//         let json_str = include_str!("../../meter-data/Esther.json");
//         serde_json::from_str(json_str).unwrap_or_else(|e| {
//             error!("Failed to parse EstherData: {}", e);
//             Vec::new()
//         })
//     };
//     pub static ref RAID_MAP: HashMap<String, String> = {
//         let json_str = include_str!("../../meter-data/encounters.json");
//         let encounters =
//             serde_json::from_str::<HashMap<String, HashMap<String, Vec<String>>>>(json_str)
//                 .unwrap_or_else(|e| {
//                     error!("Failed to parse encounters.json: {}", e);
//                     HashMap::new()
//                 });
//         encounters
//             .values()
//             .flat_map(|raid| raid.iter())
//             .flat_map(|(gate, bosses)| bosses.iter().map(move |boss| (boss.clone(), gate.clone())))
//             .collect()
//     };
//     pub static ref GEM_SKILL_MAP: HashMap<u32, Vec<u32>> = {
//         let json_str = include_str!("../../meter-data/GemSkillGroup.json");

//         let raw_map: HashMap<String, (String, String, Vec<u32>)> = serde_json::from_str(json_str)
//             .unwrap_or_else(|e| {
//                 error!("Failed to parse GemSkillGroup.json: {}", e);
//                 HashMap::new()
//             });

//         raw_map
//             .into_iter()
//             .filter_map(|(key, entry)| key.parse::<u32>().ok().map(|id| (id, entry.2)))
//             .collect()
//     };
//     pub static ref VALID_ZONES: HashSet<u32> = {
//         let valid_zones = [
//             30801, 30802, 30803, 30804, 30805, 30806, 30807, 30835, 37001, 37002, 37003, 37011,
//             37012, 37021, 37022, 37031, 37032, 37041, 37042, 37051, 37061, 37071, 37072, 37081,
//             37091, 37092, 37093, 37094, 37101, 37102, 37111, 37112, 37121, 37122, 37123, 37124,
//             308010, 308011, 308012, 308014, 308015, 308016, 308017, 308018, 308019, 308020, 308021,
//             308022, 308023, 308024, 308025, 308026, 308027, 308028, 308029, 308030, 308037, 308039,
//             308040, 308041, 308042, 308043, 308044, 308239, 308339, 308410, 308411, 308412, 308414,
//             308415, 308416, 308417, 308418, 308419, 308420, 308421, 308422, 308423, 308424, 308425,
//             308426, 308428, 308429, 308430, 308437, 309020, 30865, 30866,
//         ];

//         valid_zones.iter().cloned().collect()
//     };
//     pub static ref SUPPORT_AP_GROUP: HashSet<u32> = {
//         let mut set = HashSet::new();
//         set.insert(101204); // bard
//         set.insert(101105); // paladin
//         set.insert(314004); // artist
//         set.insert(480030); // valkyrie

//         set
//     };
//     pub static ref SUPPORT_IDENTITY_GROUP: HashSet<u32> = {
//         let mut set = HashSet::new();
//         set.insert(211400); // bard serenade of courage
//         set.insert(368000); // paladin holy aura
//         set.insert(310501); // artist moonfall
//         set.insert(480018); // valkyrie release light

//         set
//     };

//     pub static ref NPC_GRADE: HashMap<&'static str, i32> = {
//         let mut map = HashMap::new();
//         map.insert("none", 0);
//         map.insert("underling", 1);
//         map.insert("normal", 2);
//         map.insert("elite", 3);
//         map.insert("named", 4);
//         map.insert("seed", 5);
//         map.insert("boss", 6);
//         map.insert("raid", 7);
//         map.insert("lucky", 8);
//         map.insert("epic_raid", 9);
//         map.insert("commander", 10);
//         map
//     };
// }
