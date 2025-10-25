use serde::Serialize;
use log::*;
use crate::{models::Encounter, utils::{boss_to_raid_map, is_valid_player}};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendHeartbeatArgs<'a> {
    pub id: &'a str,
    pub version: &'a str,
    pub region: &'a str
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCharacterInfoArgs<'a> {
    pub client_id: &'a str,
    pub version: &'a str,
    pub region: &'a str,
    pub raid_name: String,
    pub boss: &'a str,
    pub characters: Vec<&'a str>,
    pub difficulty: &'a str,
    pub cleared: bool,
}

impl<'a> GetCharacterInfoArgs<'a> {
    pub fn new(encounter: &'a Encounter, difficulty: &'a str) -> Option<Self> {

        if difficulty.is_empty() || matches!(difficulty, "Inferno" | "Trial") {
            return None
        }
        
        let boss: &str = match (!encounter.current_boss_name.is_empty()).then(|| encounter.current_boss_name.as_ref()) {
            Some(boss) => boss,
            None => return None
        };

        let region = match encounter.region.as_ref() {
            Some(region) => region,
            None => {
                warn!("region is not set");
                return None;
            }
        };

        let raid_name = encounter
            .entities
            .get(&encounter.current_boss_name)
            .and_then(|boss| boss_to_raid_map(&encounter.current_boss_name, boss.max_hp))
            .unwrap_or_default();

        let characters: Vec<&str> = encounter
            .entities
            .iter()
            .filter_map(|(_, e)| {
                if is_valid_player(e) {
                    Some(e.name.as_ref())
                } else {
                    None
                }
            })
            .collect();

        if characters.len() > 16 {
            return None;
        }

        Some(GetCharacterInfoArgs {
            version: "",
            client_id: "",
            boss,
            raid_name,
            characters,
            cleared: encounter.cleared,
            region,
            difficulty: difficulty
        })
    }
}
