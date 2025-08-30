use std::{fs::File, thread::{self, JoinHandle}};
use anyhow::anyhow;
use once_cell::sync::Lazy;
use hashbrown::{HashMap, HashSet};

use crate::live::models::*;

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

pub struct AssetPreloader(JoinHandle<()>);

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

        Self(handle)
    }

    pub fn wait(self) -> anyhow::Result<()> {
        self.0.join().map_err(|err| anyhow!("Could not load assets {:?}", err))?;
        anyhow::Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use std::io::Write;
    use std::collections::{HashMap, HashSet};
    use std::fs::File;

    fn collect_name_counts<T>(items: &[(u32, T)], normalize: impl Fn(&T) -> String) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for (_, item) in items {
            let norm = normalize(item);
            *counts.entry(norm).or_insert(0) += 1;
        }
        counts
    }

    fn normalize_name(raw: &str) -> String {
        raw.replace("'", "")
            .replace(".", "")
            .replace("/", "")
            .replace("\\", "")
            .replace("!", "")
            .replace("&", "")
            .replace("-", "_")
            .replace(":", "_")
            .replace("1", "one")
            .replace("2", "two")
            .replace("3", "three")
            .replace("4", "four")
            .replace(" ", "_")
            .replace("__", "_")
            .replace("__", "_")
            .to_uppercase()
    }

    fn make_name_with_id(raw: &str, id: u32, counts: &HashMap<String, usize>) -> String {
        let base = normalize_name(raw);
        if *counts.get(&base).unwrap_or(&0) > 1 {
            format!("{}_{}", base, id)
        } else {
            base
        }
    }

    fn class_id_to_enum(class_id: u32) -> &'static str {
        match class_id {
            101 => "Class::WarriorMale",
            102 => "Class::Berserker",
            103 => "Class::Destroyer",
            104 => "Class::Gunlancer",
            105 => "Class::Paladin",
            111 => "Class::WarriorFemale",
            112 => "Class::Slayer",
            113 => "Class::Valkyrie",
            201 => "Class::Mage",
            202 => "Class::Arcanist",
            203 => "Class::Summoner",
            204 => "Class::Bard",
            205 => "Class::Sorceress",
            301 => "Class::MartialArtistFemale",
            302 => "Class::Wardancer",
            303 => "Class::Scrapper",
            304 => "Class::Soulfist",
            305 => "Class::Glaivier",
            311 => "Class::MartialArtistMale",
            312 => "Class::Striker",
            313 => "Class::Breaker",
            401 => "Class::Assassin",
            402 => "Class::Deathblade",
            403 => "Class::Shadowhunter",
            404 => "Class::Reaper",
            405 => "Class::Souleater",
            501 => "Class::GunnerMale",
            502 => "Class::Sharpshooter",
            503 => "Class::Deadeye",
            504 => "Class::Artillerist",
            505 => "Class::Machinist",
            511 => "Class::GunnerFemale",
            512 => "Class::Gunslinger",
            601 => "Class::Specialist",
            602 => "Class::Artist",
            603 => "Class::Aeromancer",
            604 => "Class::Wildsoul",
            _ => "Class::Unknown",
        }
    }

    fn grade_to_skilltype(grade: &str, is_hyper_awakening: bool) -> &'static str {
        match grade {
            "normal" => "SkillType::Normal",
            "super" => "SkillType::HyperAwakeningTechnique",
            "awakening" if is_hyper_awakening => "SkillType::HyperAwakening",
            "awakening" => "SkillType::Awakening",
            _ => "SkillType::Unknown",
        }
    }

    fn buff_category_to_enum(cat: &str) -> String {
        match cat {
            "ability" => "BuffType::Ability".to_string(),
            "identity" => "BuffType::Identity".to_string(),
            "classskill" => "BuffType::ClassSkill".to_string(),
            "arkpassive" => "BuffType::ArkPassive".to_string(),
            "supportbuff" => "BuffType::SupportBuff".to_string(),
            other => format!("BuffType::{}", other),
        }
    }

    fn generate_skills(skills: HashMap<u32, SkillData>) {
        let mut skills_vec: Vec<_> = skills.into_iter().collect();
        skills_vec.sort_by_key(|(_, skill)| skill.class_id);

        let counts = collect_name_counts(&skills_vec, |s| normalize_name(s.name.as_deref().unwrap_or("")));
        let mut all_consts = Vec::new();
        let mut file = File::create("skill_dump_struct.txt").unwrap();

        for (id, skill) in skills_vec {
            if skill.class_id == 0 || skill.name.is_none() {
                continue;
            }

            let raw_name = skill.name.unwrap();

            // Skip basic attacks
            if matches!(
                raw_name.as_str(),
                "Weapon Attack" | "Hand Attack" | "Basic Attack" | "Stand Up"
            ) {
                continue;
            }

            let const_name = make_name_with_id(&raw_name, id, &counts);
            all_consts.push(const_name.clone());

            let skill_name = raw_name.replace("\"", "\\\"");
            let class_id = class_id_to_enum(skill.class_id);
            let enum_name = grade_to_skilltype(&skill.grade, skill.is_hyper_awakening);

            writeln!(
                file,
            "pub const {}: Skill = Skill {{
    id: {},
    class_id: {},
    name: \"{}\",
    cooldown: {},
    kind: {},
    is_counter: false,
    skill_buff: None,
    is_projectile: false,
    is_trap: false,
    summon: None
}};\n",
                const_name,
                id,
                class_id,
                skill_name,
                skill.cooldown / 1000,
                enum_name
            )
            .unwrap();
        }

        // All skills array
        writeln!(file, "\npub const ALL_CLASS_SKILLS: &'static [Skill] = &[").unwrap();
        for name in &all_consts {
            writeln!(file, "\t{},", name).unwrap();
        }
        writeln!(file, "];").unwrap();
    }

    fn generate_debuffs(buffs: HashMap<u32, SkillBuffData>) {
        let mut buffs_vec: Vec<_> = buffs.into_iter().collect();
        buffs_vec.sort_by_key(|(_, buff)| buff.buff_category.clone());

        let counts = collect_name_counts(&buffs_vec, |b| normalize_name(b.name.as_deref().unwrap_or("")));
        let mut file = File::create("skill_debuff_dump_struct.txt").unwrap();
        let mut all_consts: Vec<_> = Vec::new();

        for (id, buff) in buffs_vec {

            if buff.name.as_ref()
                .filter(|pr| pr.contains("[Sample]")).is_some() {
                continue;
            }

            if !matches!(
                buff.buff_type.as_str(),
                "stun" | "freeze" | "electrocution" | "fear" | "earthquake"
            ) {
                continue;
            }

            if matches!(buff.duration, -1 | 0) || buff.duration > 5000 {
                continue;
            }

            let (raw_name, is_digit) = match buff.name.filter(|pr| !pr.is_empty()) {
                Some(name) => (name, false),
                None => (format!("DEBUFF_{}", buff.id.to_string()), true),
            };

            let desc = buff.desc.unwrap_or_default().replace("\n", "");
            
            let const_name = if is_digit {
                raw_name.clone()
            } else {
                make_name_with_id(&raw_name, id, &counts)
            };

            let enum_name = match buff.buff_category.unwrap_or_default().as_str() {
                "battleitem" => "BuffType::BattleItem",
                "classskill" => "BuffType::ClassSkill",
                "identity" => "BuffType::Identity",
                "etc" => "BuffType::Etc",
                _ => "BuffType::None"
            };

            write!(
                file,
            "pub const {}: SkillBuff = SkillBuff {{
    id: {},
    name: \"{}\", // {}
    duration: {},
    is_party: false,
    buff_type: {},
    unique_group: {}
}};\n\n",
                const_name,
                id,
                raw_name,
                desc,
                buff.duration / 1000,
                enum_name,
                buff.unique_group,
            )
            .unwrap();

            all_consts.push(const_name);
        }

        writeln!(file, "\npub const ALL_DEBUFFS: &'static [SkillBuff] = &[").unwrap();

        for name in &all_consts {
            writeln!(file, "    {},", name).unwrap();
        }
        writeln!(file, "];").unwrap();
    }

    fn generate_buffs(buffs: HashMap<u32, SkillBuffData>) {
        let mut buffs_vec: Vec<_> = buffs.into_iter().collect();
        buffs_vec.sort_by_key(|(_, buff)| buff.buff_category.clone());

        let counts = collect_name_counts(
            &buffs_vec,
            |buff: &SkillBuffData| normalize_name(buff.name.as_deref().unwrap_or("")));
        let mut file = File::create("skill_buff_dump_struct.txt").unwrap();

        for (id, buff) in buffs_vec {
            if buff.name.is_none() {
                continue;
            }

            if !matches!(
                buff.buff_category.as_deref(),
                Some("ability" |"supportbuff" | "identity" | "classskill" | "arkpassive")
            ) {
                continue;
            }

            let raw_name = buff.name.unwrap();
            let desc = buff.desc.unwrap_or_default().replace("\n", "");
            let const_name = make_name_with_id(&raw_name, id, &counts);
            let enum_name = buff_category_to_enum(buff.buff_category.as_ref().unwrap());

            write!(
                file,
            "pub const {}: SkillBuff = SkillBuff {{
    id: {},
    name: \"{}\", // {}
    duration: {},
    is_party: false,
    buff_type: {},
    unique_group: {}
}};\n\n",
                const_name,
                id,
                raw_name,
                desc,
                buff.duration / 1000,
                enum_name,
                buff.unique_group,
            )
            .unwrap();
        }
    }

    #[test]
    fn extract_skills_and_buffs() {
        // Skills
        let bytes = include_bytes!("../../meter-data/Skill.json");
        let skills: HashMap<u32, SkillData> = serde_json::from_slice(bytes).unwrap();
        generate_skills(skills);

        // Buffs
        let bytes = include_bytes!("../../meter-data/SkillBuff.json");
        let buffs: HashMap<u32, SkillBuffData> = serde_json::from_slice(bytes).unwrap();
        generate_buffs(buffs);

        let bytes = include_bytes!("../../meter-data/SkillBuff.json");
        let buffs: HashMap<u32, SkillBuffData> = serde_json::from_slice(bytes).unwrap();
        generate_debuffs(buffs);        
    }
}