use crate::live::id_tracker::IdTracker;
use hashbrown::{HashMap, HashSet};
use meter_defs::defs::PKTTroopInfo;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PartyMembership {
    pub raid_instance_id: u32,
    pub party_instance_id: u32,
}

pub struct PartyTracker {
    id_tracker: Rc<RefCell<IdTracker>>,

    memberships: HashMap<u64, PartyMembership>,
    explicit_memberships: HashMap<u64, PartyMembership>,
    troop_groups: Vec<Vec<u64>>,
    character_id_to_name: HashMap<u64, String>,
    name: Option<String>,
}

#[derive(Default)]
pub(super) struct RdpsPartyTrace {
    pub fight_start: i64,
    lines: usize,
    previous: HashMap<u64, PartyMembership>,
    context: VecDeque<RosterObservation>,
    dropped: usize,
}

struct RosterObservation {
    first_ms: i64,
    last_ms: i64,
    kind: RosterEvidence,
    event: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RosterEvidence {
    Troop,
    Reset,
    Membership,
}

impl RdpsPartyTrace {
    const MAX_LINES: usize = 16;

    pub fn observe(
        &mut self,
        timestamp: i64,
        fight_start: i64,
        tracker: &PartyTracker,
        reason: &str,
    ) {
        let starting = fight_start != 0 && self.fight_start != fight_start;
        if starting {
            self.fight_start = fight_start;
            self.lines = 0;
            for observation in &self.context {
                log::info!(
                    "rdps party fight={fight_start} context first_ms={} last_ms={} {}",
                    observation.first_ms - fight_start,
                    observation.last_ms - fight_start,
                    observation.event
                );
            }
        } else if self.previous == tracker.memberships
            || (fight_start != 0 && self.lines == Self::MAX_LINES)
        {
            return;
        }
        self.previous.clone_from(&tracker.memberships);
        let event = format!("event={reason} {}", tracker.describe_roster());
        if fight_start == 0 {
            self.record_context(timestamp, 0, RosterEvidence::Membership, event);
        } else {
            self.lines += 1;
            log::info!(
                "rdps party fight={fight_start} t_ms={} {event} sample={}/{} context_dropped={}",
                timestamp - fight_start,
                self.lines,
                Self::MAX_LINES,
                self.dropped
            );
        }
    }

    // Keep setup evidence across wipes: the server may send the raid roster only once.
    fn record_context(
        &mut self,
        timestamp: i64,
        fight_start: i64,
        kind: RosterEvidence,
        event: String,
    ) {
        if let Some(last) = self.context.back_mut()
            && last.event == event
            && (fight_start == 0 || last.last_ms >= fight_start)
        {
            last.last_ms = timestamp;
            return;
        }
        // Frequent single-member updates must not evict the grouped packet or reset evidence.
        let limit = match kind {
            RosterEvidence::Membership => 8,
            _ => 4,
        };
        let mut same_kind = self
            .context
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.kind == kind);
        if let Some((oldest, _)) = same_kind.next()
            && same_kind.count() + 1 == limit
        {
            self.context.remove(oldest);
            self.dropped += 1;
        }
        if fight_start != 0 && self.fight_start == fight_start && self.lines < Self::MAX_LINES {
            self.lines += 1;
            log::info!(
                "rdps party fight={fight_start} t_ms={} {event}",
                timestamp - fight_start
            );
        }
        self.context.push_back(RosterObservation {
            first_ms: timestamp,
            last_ms: timestamp,
            kind,
            event,
        });
    }

    pub fn troop_info(
        &mut self,
        timestamp: i64,
        fight_start: i64,
        body: &[u8],
    ) -> Option<PKTTroopInfo> {
        let packet = super::parse_complete_packet::<PKTTroopInfo>(body).ok();
        let event = match &packet {
            Some(pkt) => describe_troop_roster(pkt),
            None => format!("event=TroopInfo decode_failed bytes={}", body.len()),
        };
        self.record_context(timestamp, fight_start, RosterEvidence::Troop, event);
        packet
    }

    pub fn before_reset(&mut self, timestamp: i64, fight_start: i64, tracker: &PartyTracker) {
        self.record_context(
            timestamp,
            fight_start,
            RosterEvidence::Reset,
            format!("event=InitEnv cleared {}", tracker.describe_roster()),
        );
        self.previous.clear();
    }
}

fn describe_troop_roster(pkt: &PKTTroopInfo) -> String {
    let groups = &pkt
        .sub_p_k_t_troop_info_1_5_130
        .sub_p_k_t_troop_info_2_4_116
        .party_member_datas;
    let mut remaining = 32;
    let mut group_members = Vec::new();
    for group in groups.iter().take(32) {
        let members = group
            .iter()
            .take(remaining)
            .map(|member| format!("{}({})", member.name, member.character_id))
            .collect::<Vec<_>>();
        remaining -= members.len();
        group_members.push(format!("[{}]", members.join(" ")));
        if remaining == 0 {
            break;
        }
    }
    format!(
        "event=TroopInfo groups={} members={} {}",
        groups.len(),
        groups.iter().map(Vec::len).sum::<usize>(),
        group_members.join(" ")
    )
}

impl PartyTracker {
    fn describe_roster(&self) -> String {
        let mut roster = self.memberships().collect::<Vec<_>>();
        roster.sort_unstable_by_key(|(character, party)| (*party, *character));
        let members = roster
            .iter()
            .take(32)
            .map(|(character, party)| {
                let name = self
                    .character_id_to_name
                    .get(character)
                    .map_or("?", String::as_str);
                format!(
                    "{name}({character})={}/{}",
                    party.raid_instance_id, party.party_instance_id
                )
            })
            .collect::<Vec<_>>()
            .join(" ");
        format!("members={} [ {members} ]", roster.len())
    }

    pub fn new(id_tracker: Rc<RefCell<IdTracker>>) -> Self {
        Self {
            id_tracker,
            memberships: HashMap::new(),
            explicit_memberships: HashMap::new(),
            troop_groups: Vec::new(),
            character_id_to_name: HashMap::new(),
            name: None,
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    pub fn set_troop_groups(&mut self, pkt: PKTTroopInfo) {
        self.troop_groups.clear();
        for group in pkt
            .sub_p_k_t_troop_info_1_5_130
            .sub_p_k_t_troop_info_2_4_116
            .party_member_datas
        {
            let mut ids = Vec::with_capacity(group.len());
            for member in group.into_iter().filter(|m| m.character_id != 0) {
                ids.push(member.character_id);
                if !member.name.is_empty() {
                    self.character_id_to_name
                        .insert(member.character_id, member.name);
                }
            }
            self.troop_groups.push(ids);
        }
        self.refresh_memberships();
    }

    fn refresh_memberships(&mut self) {
        self.memberships.clone_from(&self.explicit_memberships);
        for group in &self.troop_groups {
            // TroopInfo identifies teammates. A later party packet can supply their instance ID.
            let mut known = group
                .iter()
                .filter_map(|id| self.explicit_memberships.get(id));
            let Some(&party) = known.next() else { continue };
            if party.party_instance_id != 0 && known.all(|other| *other == party) {
                for &id in group {
                    self.memberships.entry(id).or_insert(party);
                }
            }
        }
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
        let membership = PartyMembership {
            raid_instance_id,
            party_instance_id: party_id,
        };
        let previous = self.explicit_memberships.insert(character_id, membership);
        if previous != Some(membership) {
            if previous.is_some_and(|party| party.party_instance_id != 0) {
                // A confirmed move supersedes the older grouping for this player.
                for group in &mut self.troop_groups {
                    group.retain(|id| *id != character_id);
                }
            }
            self.refresh_memberships();
        }
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
        // A complete party roster supersedes its older troop group, including omitted members.
        let members: Vec<_> = members.into_iter().collect();
        self.troop_groups.retain(|group| {
            !group.iter().any(|id| {
                self.memberships.get(id) == Some(&membership)
                    || members.iter().any(|(member, _)| member == id)
            })
        });
        self.explicit_memberships
            .retain(|_, party| *party != membership);
        for (character, name) in members {
            if character != 0 {
                self.explicit_memberships.insert(character, membership);
                if !name.is_empty() {
                    self.character_id_to_name.insert(character, name);
                }
            }
        }
        self.refresh_memberships();
    }

    pub fn observe_member_name(&mut self, character_id: u64, name: &str) {
        if self.memberships.contains_key(&character_id) && !name.is_empty() {
            self.character_id_to_name
                .insert(character_id, name.to_string());
        }
    }

    pub fn remove(&mut self, party_instance_id: u32, name: String) {
        let keep = |character: &u64| {
            self.character_id_to_name.get(character) != Some(&name)
                || self
                    .memberships
                    .get(character)
                    .is_some_and(|party| party.party_instance_id != party_instance_id)
        };
        for group in &mut self.troop_groups {
            group.retain(keep);
        }
        self.explicit_memberships.retain(|id, _| keep(id));
        self.refresh_memberships();
    }

    pub fn reset_party_mappings(&mut self) {
        self.memberships.clear();
        self.explicit_memberships.clear();
        self.troop_groups.clear();
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

#[cfg(test)]
mod trace_tests {
    use super::*;
    use meter_defs::{
        BiSerializable,
        defs::{PKTTroopInfo, PartyMemberData},
    };

    #[test]
    fn grouped_packet_and_discarded_mapping_survive_pre_pull_member_updates() {
        let mut packet = PKTTroopInfo::default();
        let roster = &mut packet
            .sub_p_k_t_troop_info_1_5_130
            .sub_p_k_t_troop_info_2_4_116;
        roster.num = 2;
        roster.b_0 = vec![0, 1];
        roster.str_0 = vec![String::new(); 2];
        let member = |character_id, name: &str| PartyMemberData {
            character_id,
            name: name.into(),
            ..Default::default()
        };
        roster.party_member_datas = vec![
            vec![member(10, "Artist"), member(20, "DPS")],
            vec![member(30, "Bard")],
        ];
        let bytes = packet.to_bytes().unwrap();

        let mut trace = RdpsPartyTrace::default();
        let mut tracker = PartyTracker::new(Rc::new(RefCell::new(IdTracker::new())));
        trace.troop_info(100, 0, &bytes);
        trace.troop_info(110, 0, &bytes);
        tracker.add(7, 42, 10, 0, Some("Artist".into()));
        trace.observe(120, 0, &tracker, "PartyInfo");
        trace.before_reset(130, 0, &tracker);
        tracker.reset_party_mappings();
        for id in 20..32 {
            tracker.add(7, 42, id, 0, None);
            trace.observe(
                140 + id as i64,
                0,
                &tracker,
                "PartyStatusEffectResultNotify",
            );
        }
        trace.observe(200, 200, &tracker, "fight_start");
        let grouped = &trace.context[0];
        assert_eq!((grouped.first_ms, grouped.last_ms), (100, 110));
        assert_eq!(
            grouped.event,
            "event=TroopInfo groups=2 members=3 [Artist(10) DPS(20)] [Bard(30)]"
        );
        assert!(
            trace.context[1]
                .event
                .contains("event=InitEnv cleared members=1 [ Artist(10)=7/42 ]")
        );
        assert_eq!(trace.context.len(), 10);
        assert_eq!(trace.dropped, 5);
        assert!(!trace.previous.contains_key(&10));

        trace.troop_info(201, 200, &[]);
        assert_eq!(
            trace.context.back().unwrap().event,
            "event=TroopInfo decode_failed bytes=0"
        );
        assert_eq!(trace.lines, 2);
    }
}
