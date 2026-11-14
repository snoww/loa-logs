use crate::live::id_tracker::IdTracker;
use hashbrown::{HashMap, HashSet};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PartyMembership {
    pub raid_instance_id: u32,
    pub party_instance_id: u32,
}

pub struct PartyTracker {
    id_tracker: Rc<RefCell<IdTracker>>,

    memberships: HashMap<u64, PartyMembership>,
    character_id_to_name: HashMap<u64, String>,
    name: Option<String>,
}

impl PartyTracker {
    pub fn new(id_tracker: Rc<RefCell<IdTracker>>) -> Self {
        Self {
            id_tracker,
            memberships: HashMap::new(),
            character_id_to_name: HashMap::new(),
            name: None,
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    /// Only observed party packets establish membership.
    pub fn add(
        &mut self,
        raid_instance_id: u32,
        party_id: u32,
        character_id: u64,
        entity_id: u64,
        name: Option<String>,
    ) {
        let character_id = if character_id != 0 {
            character_id
        } else {
            self.id_tracker
                .borrow()
                .get_character_id(entity_id)
                .unwrap_or(0)
        };
        if character_id == 0 {
            return;
        }
        self.memberships.insert(
            character_id,
            PartyMembership {
                raid_instance_id,
                party_instance_id: party_id,
            },
        );
        if let Some(name) = name.filter(|name| !name.is_empty()) {
            self.character_id_to_name.insert(character_id, name);
        }
    }

    pub fn replace_party_members(
        &mut self,
        raid_instance_id: u32,
        party_id: u32,
        members: impl IntoIterator<Item = (u64, String)>,
    ) {
        let membership = PartyMembership {
            raid_instance_id,
            party_instance_id: party_id,
        };
        self.memberships.retain(|_, party| *party != membership);
        for (character, name) in members {
            self.add(raid_instance_id, party_id, character, 0, Some(name));
        }
    }

    pub fn observe_member_name(&mut self, character_id: u64, name: &str) {
        if self.memberships.contains_key(&character_id) && !name.is_empty() {
            self.character_id_to_name
                .insert(character_id, name.to_string());
        }
    }

    pub fn remove(&mut self, party_instance_id: u32, name: String) {
        self.memberships.retain(|character, party| {
            party.party_instance_id != party_instance_id
                || self.character_id_to_name.get(character) != Some(&name)
        });
    }

    pub fn reset_party_mappings(&mut self) {
        self.memberships.clear();
        self.character_id_to_name.clear();
    }

    pub fn membership_for_character(&self, character_id: u64) -> Option<PartyMembership> {
        self.memberships.get(&character_id).copied()
    }

    pub fn memberships(&self) -> impl Iterator<Item = (u64, PartyMembership)> + '_ {
        self.memberships
            .iter()
            .map(|(character, party)| (*character, *party))
    }

    pub fn get_party_id_for_character(&self, character_id: u64) -> Option<u32> {
        self.membership_for_character(character_id)
            .map(|party| party.party_instance_id)
    }

    pub fn get_character_id_for_entity(&self, entity_id: u64) -> Option<u64> {
        self.id_tracker.borrow().get_character_id(entity_id)
    }

    pub fn get_entity_id_for_character(&self, character_id: u64) -> Option<u64> {
        self.id_tracker.borrow().get_entity_id(character_id)
    }

    pub fn get_party_id_for_entity(&self, entity_id: u64) -> Option<u32> {
        self.id_tracker
            .borrow()
            .get_character_id(entity_id)
            .and_then(|character| self.get_party_id_for_character(character))
    }

    pub fn are_same_party_characters(&self, lhs: u64, rhs: u64) -> bool {
        matches!(
            (
                self.membership_for_character(lhs),
                self.membership_for_character(rhs)
            ),
            (Some(lhs_party), Some(rhs_party)) if lhs_party == rhs_party
        )
    }

    pub fn get_all_registered_party_characters(&self) -> Vec<u64> {
        let mut characters = self.memberships().collect::<Vec<_>>();
        characters.sort_unstable_by_key(|(character, party)| (*party, *character));
        let characters = characters
            .into_iter()
            .map(|(character, _)| character)
            .collect();
        characters
    }

    pub fn get_name_for_character(&self, character_id: u64) -> Option<String> {
        self.character_id_to_name.get(&character_id).cloned()
    }

    pub fn get_tracked_party_names(&self) -> HashSet<String> {
        let mut names = self
            .memberships()
            .filter_map(|(character, _)| self.character_id_to_name.get(&character))
            .filter(|name| !name.is_empty())
            .cloned()
            .collect::<HashSet<_>>();
        if let Some(name) = self.name.as_ref().filter(|name| !name.is_empty()) {
            names.insert(name.clone());
        }
        names
    }

    pub fn get_registered_party_count(&self) -> usize {
        self.memberships()
            .map(|(_, party)| party)
            .collect::<HashSet<_>>()
            .len()
    }

    pub fn get_group_number_for_character(&self, character_id: u64) -> Option<usize> {
        let party_id = self.membership_for_character(character_id)?;
        let mut sorted_party_ids = self
            .memberships()
            .map(|(_, party)| party)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        sorted_party_ids.sort_unstable();
        sorted_party_ids
            .iter()
            .position(|tracked_party_id| *tracked_party_id == party_id)
    }
}
