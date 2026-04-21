use crate::live::id_tracker::IdTracker;
use hashbrown::{HashMap, HashSet};
use std::cell::RefCell;
use std::rc::Rc;

pub struct PartyTracker {
    id_tracker: Rc<RefCell<IdTracker>>,

    pub character_id_to_party_id: HashMap<u64, u32>,
    pub entity_id_to_party_id: HashMap<u64, u32>,
    character_id_to_name: HashMap<u64, String>,
    raid_instance_to_party_ids: HashMap<u32, HashSet<u32>>,
    name: Option<String>,
}

impl PartyTracker {
    pub fn new(id_tracker: Rc<RefCell<IdTracker>>) -> Self {
        Self {
            id_tracker,
            character_id_to_party_id: HashMap::new(),
            entity_id_to_party_id: HashMap::new(),
            character_id_to_name: HashMap::new(),
            raid_instance_to_party_ids: HashMap::new(),
            name: None,
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    pub fn add(
        &mut self,
        raid_instance_id: u32,
        party_id: u32,
        mut character_id: u64,
        mut entity_id: u64,
        name: Option<String>,
    ) {
        if character_id == 0 && entity_id == 0 {
            return;
        }
        if character_id > 0 && entity_id == 0 {
            entity_id = self
                .id_tracker
                .borrow()
                .get_entity_id(character_id)
                .unwrap_or(0);
        } else if character_id == 0 && entity_id > 0 {
            character_id = self
                .id_tracker
                .borrow()
                .get_character_id(entity_id)
                .unwrap_or(0);
        }
        if character_id > 0 {
            // println!("character_id: {}, entity_id: {}", character_id, entity_id);
            self.character_id_to_party_id.insert(character_id, party_id);
            if let Some(name) = name.filter(|name| !name.is_empty()) {
                self.character_id_to_name.insert(character_id, name);
            }
        }
        if entity_id > 0 {
            self.entity_id_to_party_id.insert(entity_id, party_id);
        }
        self.register_party_id(raid_instance_id, party_id);
    }

    pub fn remove(&mut self, party_instance_id: u32, name: String) {
        if let Some(local_name) = self.name.as_ref()
            && local_name == &name
        {
            self.remove_party_mappings(party_instance_id);
            return;
        }

        let removed_character_ids = self
            .character_id_to_party_id
            .iter()
            .filter_map(|(character_id, tracked_party_id)| {
                (*tracked_party_id == party_instance_id
                    && self
                        .character_id_to_name
                        .get(character_id)
                        .is_some_and(|tracked_name| tracked_name == &name))
                .then_some(*character_id)
            })
            .collect::<Vec<_>>();
        for character_id in removed_character_ids {
            self.character_id_to_party_id.remove(&character_id);
            self.character_id_to_name.remove(&character_id);
            if let Some(entity_id) = self.id_tracker.borrow().get_entity_id(character_id) {
                self.entity_id_to_party_id.remove(&entity_id);
            }
        }
    }

    pub fn reset_party_mappings(&mut self) {
        self.character_id_to_party_id.clear();
        self.entity_id_to_party_id.clear();
        self.character_id_to_name.clear();
        self.raid_instance_to_party_ids.clear();
    }

    pub fn remove_party_mappings(&mut self, party_id: u32) {
        let removed_character_ids = self
            .character_id_to_party_id
            .iter()
            .filter_map(|(character_id, tracked_party_id)| {
                (*tracked_party_id == party_id).then_some(*character_id)
            })
            .collect::<Vec<_>>();
        self.character_id_to_party_id
            .retain(|_, &mut p_id| p_id != party_id);
        for character_id in removed_character_ids {
            self.character_id_to_name.remove(&character_id);
        }
        self.entity_id_to_party_id
            .retain(|_, &mut p_id| p_id != party_id);
        for party_ids in self.raid_instance_to_party_ids.values_mut() {
            party_ids.remove(&party_id);
        }
        self.raid_instance_to_party_ids
            .retain(|_, party_ids| !party_ids.is_empty());
    }

    pub fn change_entity_id(&mut self, old: u64, new: u64) {
        if let Some(party_id) = self.entity_id_to_party_id.get(&old).cloned() {
            self.entity_id_to_party_id.remove(&old);
            self.entity_id_to_party_id.insert(new, party_id);
        }
    }

    pub fn remove_entity_mapping(&mut self, entity_id: u64) {
        self.entity_id_to_party_id.remove(&entity_id);
    }

    pub fn complete_entry(&mut self, character_id: u64, entity_id: u64) {
        let char_party_id = self.character_id_to_party_id.get(&character_id).cloned();
        let entity_party_id = self.entity_id_to_party_id.get(&entity_id).cloned();
        if let (Some(_char_party_id), Some(_entity_party_id)) = (char_party_id, entity_party_id) {
            return;
        }
        if let Some(entity_party_id) = entity_party_id {
            self.character_id_to_party_id
                .insert(character_id, entity_party_id);
        }
        if let Some(char_party_id) = char_party_id {
            self.entity_id_to_party_id.insert(entity_id, char_party_id);
        }
    }

    fn register_party_id(&mut self, raid_instance_id: u32, party_id: u32) {
        let party_instance = self
            .raid_instance_to_party_ids
            .entry(raid_instance_id)
            .or_default();
        party_instance.insert(party_id);
    }

    pub fn get_party_id_for_character(&self, character_id: u64) -> Option<u32> {
        self.character_id_to_party_id.get(&character_id).copied()
    }

    pub fn get_party_id_for_entity(&self, entity_id: u64) -> Option<u32> {
        self.entity_id_to_party_id.get(&entity_id).copied()
    }

    pub fn are_same_party_characters(&self, lhs: u64, rhs: u64) -> bool {
        matches!(
            (
                self.get_party_id_for_character(lhs),
                self.get_party_id_for_character(rhs)
            ),
            (Some(lhs_party), Some(rhs_party)) if lhs_party == rhs_party
        )
    }

    pub fn get_characters_in_party(&self, party_id: u32) -> Vec<u64> {
        let mut characters = self
            .character_id_to_party_id
            .iter()
            .filter_map(|(character_id, tracked_party_id)| {
                (*tracked_party_id == party_id).then_some(*character_id)
            })
            .collect::<Vec<_>>();
        characters.sort_unstable();
        characters
    }

    pub fn get_all_registered_party_characters(&self) -> Vec<u64> {
        let mut parties = self
            .character_id_to_party_id
            .values()
            .copied()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        parties.sort_unstable();

        let mut characters = Vec::new();
        for party_id in parties {
            characters.extend(self.get_characters_in_party(party_id));
        }
        characters
    }

    pub fn get_name_for_character(&self, character_id: u64) -> Option<String> {
        self.character_id_to_name.get(&character_id).cloned()
    }

    pub fn get_tracked_party_names(&self) -> HashSet<String> {
        let mut names = self
            .character_id_to_name
            .values()
            .filter(|name| !name.is_empty())
            .cloned()
            .collect::<HashSet<_>>();
        if let Some(name) = self.name.as_ref().filter(|name| !name.is_empty()) {
            names.insert(name.clone());
        }
        names
    }

    pub fn get_registered_party_count(&self) -> usize {
        self.character_id_to_party_id
            .values()
            .copied()
            .collect::<HashSet<_>>()
            .len()
    }

    pub fn get_group_number_for_character(&self, character_id: u64) -> Option<usize> {
        let party_id = self.get_party_id_for_character(character_id)?;
        let mut sorted_party_ids = self
            .character_id_to_party_id
            .values()
            .copied()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        sorted_party_ids.sort_unstable();
        sorted_party_ids
            .iter()
            .position(|tracked_party_id| *tracked_party_id == party_id)
    }
}
