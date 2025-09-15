use std::fmt::Display;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum EntityType {
    #[default]
    #[serde(rename = "UNKNOWN")]
    Unknown,
    #[serde(rename = "MONSTER")]
    Monster,
    #[serde(rename = "BOSS")]
    Boss,
    #[serde(rename = "GUARDIAN")]
    Guardian,
    #[serde(rename = "PLAYER")]
    Player,
    #[serde(rename = "NPC")]
    Npc,
    #[serde(rename = "ESTHER")]
    Esther,
    #[serde(rename = "PROJECTILE")]
    Projectile,
    #[serde(rename = "SUMMON")]
    Summon,
}

impl Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            EntityType::Unknown => "UNKNOWN".to_string(),
            EntityType::Monster => "MONSTER".to_string(),
            EntityType::Boss => "BOSS".to_string(),
            EntityType::Guardian => "GUARDIAN".to_string(),
            EntityType::Player => "PLAYER".to_string(),
            EntityType::Npc => "NPC".to_string(),
            EntityType::Esther => "ESTHER".to_string(),
            EntityType::Projectile => "PROJECTILE".to_string(),
            EntityType::Summon => "SUMMON".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl FromStr for EntityType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "UNKNOWN" => Ok(EntityType::Unknown),
            "MONSTER" => Ok(EntityType::Monster),
            "BOSS" => Ok(EntityType::Boss),
            "GUARDIAN" => Ok(EntityType::Guardian),
            "PLAYER" => Ok(EntityType::Player),
            "NPC" => Ok(EntityType::Npc),
            "ESTHER" => Ok(EntityType::Esther),
            _ => Ok(EntityType::Unknown),
        }
    }
}

