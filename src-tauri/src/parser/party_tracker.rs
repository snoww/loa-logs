use crate::parser::id_tracker::IdTracker;
use hashbrown::{HashMap, HashSet};
use std::cell::RefCell;
use std::rc::Rc;

pub struct PartyTracker {
    id_tracker: Rc<RefCell<IdTracker>>,

    pub character_id_to_party_id: HashMap<u64, u32>,
    pub entity_id_to_party_id: HashMap<u64, u32>,
    raid_instance_to_party_ids: HashMap<u32, HashSet<u32>>,
    character_name_to_character_id: HashMap<String, u64>,
    name: Option<String>,
}

impl PartyTracker {
    pub fn new(id_tracker: Rc<RefCell<IdTracker>>) -> Self {
        Self {
            id_tracker,
            character_id_to_party_id: HashMap::new(),
            entity_id_to_party_id: HashMap::new(),
            raid_instance_to_party_ids: HashMap::new(),
            character_name_to_character_id: HashMap::new(),
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
            if let Some(name) = name {
                self.character_name_to_character_id
                    .insert(name, character_id);
            }
        }
        if entity_id > 0 {
            self.entity_id_to_party_id.insert(entity_id, party_id);
        }
        self.register_party_id(raid_instance_id, party_id);
    }

    pub fn remove(&mut self, party_instance_id: u32, name: String) {
        if let Some(local_name) = self.name.as_ref() {
            if local_name == &name {
                self.remove_party_mappings(party_instance_id);
            }
        }
    }

    pub fn reset_party_mappings(&mut self) {
        self.character_id_to_party_id.clear();
        self.entity_id_to_party_id.clear();
        self.raid_instance_to_party_ids.clear();
    }

    pub fn remove_party_mappings(&mut self, party_id: u32) {
        self.character_id_to_party_id
            .retain(|_, &mut p_id| p_id != party_id);
        self.entity_id_to_party_id
            .retain(|_, &mut p_id| p_id != party_id);
    }

    pub fn change_entity_id(&mut self, old: u64, new: u64) {
        if let Some(party_id) = self.entity_id_to_party_id.get(&old).cloned() {
            self.entity_id_to_party_id.remove(&old);
            self.entity_id_to_party_id.insert(new, party_id);
        }
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
}
