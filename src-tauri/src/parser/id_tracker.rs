use hashbrown::HashMap;

pub struct IdTracker {
    character_id_to_entity_id: HashMap<u64, u64>,
    entity_id_to_character_id: HashMap<u64, u64>,
}

impl IdTracker {
    pub fn new() -> Self {
        Self {
            character_id_to_entity_id: HashMap::new(),
            entity_id_to_character_id: HashMap::new(),
        }
    }

    pub fn add_mapping(&mut self, character_id: u64, entity_id: u64) {
        self.character_id_to_entity_id
            .insert(character_id, entity_id);
        self.entity_id_to_character_id
            .insert(entity_id, character_id);
    }

    pub fn get_character_id(&self, entity_id: u64) -> Option<u64> {
        self.entity_id_to_character_id.get(&entity_id).copied()
    }

    pub fn get_local_character_id(&self, entity_id: u64) -> u64 {
        self.entity_id_to_character_id
            .get(&entity_id)
            .copied()
            .unwrap_or_default()
    }

    pub fn get_entity_id(&self, character_id: u64) -> Option<u64> {
        self.character_id_to_entity_id.get(&character_id).copied()
    }

    pub fn clear(&mut self) {
        self.character_id_to_entity_id.clear();
        self.entity_id_to_character_id.clear();
    }
}
