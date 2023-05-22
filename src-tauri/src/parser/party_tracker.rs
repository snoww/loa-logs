use crate::parser::id_tracker::IdTracker;
use hashbrown::{HashMap, HashSet};
use pcap_test::packets::definitions::PKTPartyInfo;
use std::cell::RefCell;
use std::rc::Rc;

pub struct PartyTracker {
    id_tracker: Rc<RefCell<IdTracker>>,

    pub character_id_to_party_id: HashMap<u64, u32>,
    pub entity_id_to_party_id: HashMap<u64, u32>,
    raid_instance_to_party_instance: HashMap<u32, HashSet<u32>>,
    character_name_to_character_id: HashMap<String, u64>,
    name: Option<String>,
}

impl PartyTracker {
    pub fn new(id_tracker: Rc<RefCell<IdTracker>>) -> Self {
        Self {
            id_tracker,
            character_id_to_party_id: HashMap::new(),
            entity_id_to_party_id: HashMap::new(),
            raid_instance_to_party_instance: HashMap::new(),
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
                .borrow_mut()
                .get_entity_id(character_id)
                .unwrap_or(0);
        } else if character_id == 0 && entity_id > 0 {
            character_id = self
                .id_tracker
                .borrow_mut()
                .get_character_id(entity_id)
                .unwrap_or(0);
        }
        if character_id > 0 {
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

    pub fn remove_party_mappings(&mut self, party_instance_id: u32) {
        if let Some((_raid_id, party_instances)) = self
            .raid_instance_to_party_instance
            .iter()
            .find(|(_, party_instance)| party_instance.contains(&party_instance_id))
        {
            let to_remove: Vec<_> = self
                .character_id_to_party_id
                .iter()
                .filter_map(|(character_id, party_id)| {
                    if party_instances.contains(party_id) {
                        self.character_name_to_character_id
                            .retain(|_, v| v != character_id);
                        Some(*character_id)
                    } else {
                        None
                    }
                })
                .collect();

            for character_id in to_remove {
                self.character_id_to_party_id.remove(&character_id);
            }
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
            .raid_instance_to_party_instance
            .entry(raid_instance_id)
            .or_insert(HashSet::new());
        party_instance.insert(party_id);
    }
}
