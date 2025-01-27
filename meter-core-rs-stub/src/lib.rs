#![allow(unused_variables)]

use std::{error::Error, sync::mpsc::{self, Receiver}};

use packets::opcodes::Pkt;

pub type GearLevel = f32;
pub type EntityId = u64;
pub type CharacterId = u64;
pub type NpcId = u32;
pub type SkillId = u32;
pub type SkillEffectId = u32;
pub type ClassId = u32;
pub type PartyInstanceId = u32;
pub type RaidInstanceId = u32;
pub type StatusEffectId = u32;
pub type StatusEffectInstanceId = u32;

macro_rules! impl_new_default {
    ($struct_name:ident) => {
        impl $struct_name {
            pub fn new(_data: &[u8]) -> anyhow::Result<Self> {
                Ok(Self::default())
            }
        }
    };
}

pub mod packets {
    pub mod definitions {
        use crate::{CharacterId, ClassId, EntityId, GearLevel, PartyInstanceId, RaidInstanceId, SkillEffectId, SkillId, StatusEffectInstanceId};

        use super::structures::{EquipItemData, NpcStruct, SkillDamageEvent, StatPair, StatusEffectData};


        #[derive(Debug, Default, Clone)]
        pub struct PKTCounterAttackNotify {
            pub source_id: u64
        }

        impl_new_default!(PKTCounterAttackNotify);

        #[derive(Debug, Default, Clone)]
        pub struct PKTDeathNotify {
            pub target_id: u64
        }

        impl_new_default!(PKTDeathNotify);

        #[derive(Debug, Default, Clone)]
        pub struct PKTIdentityGaugeChangeNotify {
            pub player_id: EntityId,
            pub identity_gauge1: u32,
            pub identity_gauge2: u32,
            pub identity_gauge3: u32
        }
        
        impl_new_default!(PKTIdentityGaugeChangeNotify);

        #[derive(Debug, Default, Clone)]
        pub struct PKTInitEnv {
            pub player_id: EntityId
        }
        
        impl_new_default!(PKTInitEnv);

        #[derive(Debug, Default, Clone)]
        pub struct PKTInitPC {
            pub player_id: EntityId,
            pub name: String,
            pub character_id: CharacterId,
            pub class_id: CharacterId,
            pub gear_level: GearLevel,
            pub stat_pairs: Vec<StatPair>,
            pub status_effect_datas: Vec<StatusEffectData>,
        }
        
        impl_new_default!(PKTInitPC);

        #[derive(Debug, Default, Clone)]
        pub struct PKTNewNpc {
            pub npc_struct: NpcStruct
        }
        
        impl_new_default!(PKTNewNpc);

        #[derive(Debug, Default, Clone)]
        pub struct PKTNewNpcSummon {
            pub owner_id: EntityId,
            pub npc_struct: NpcStruct
        }
        
        impl_new_default!(PKTNewNpcSummon);

        #[derive(Debug, Default, Clone)]
        pub struct PKTNewProjectileInner {
            pub projectile_id: EntityId,
            pub owner_id: EntityId,
            pub skill_id: SkillId,
            pub skill_effect: SkillEffectId,
        }

        #[derive(Debug, Default, Clone)]
        pub struct PKTNewProjectile {
            pub projectile_info: PKTNewProjectileInner
        }
        
        impl_new_default!(PKTNewProjectile);

        #[derive(Debug, Default, Clone)]
        pub struct PKTNewTrapInner {
            pub object_id: EntityId,
            pub owner_id: EntityId,
            pub skill_id: SkillId,
            pub skill_effect: SkillEffectId
        }

        #[derive(Debug, Default, Clone)]
        pub struct PKTNewTrap {
            pub trap_struct: PKTNewTrapInner
        }
        
        impl_new_default!(PKTNewTrap);

        #[derive(Debug, Default, Clone)]
        pub struct PKTRaidBegin {
            pub raid_id: u32,
        }
        
        impl_new_default!(PKTRaidBegin);

        #[derive(Debug, Default, Clone)]
        pub struct PKTRemoveObjectInner {
            pub object_id: EntityId
        }

        #[derive(Debug, Default, Clone)]
        pub struct PKTRemoveObject {
            pub unpublished_objects: Vec<PKTRemoveObjectInner>
        }
        
        impl_new_default!(PKTRemoveObject);

        #[derive(Debug, Default, Clone)]
        pub struct PKTSkillCastNotify {
            pub source_id: EntityId,
            pub skill_id: SkillId,
        }
        
        impl_new_default!(PKTSkillCastNotify);

        #[derive(Debug, Default, Clone, Copy)]
        pub struct TripodIndex {
            pub first: u8,
            pub second: u8,
            pub third: u8,
        }

        #[derive(Debug, Default, Clone, Copy)]
        pub struct TripodLevel {
            pub first: u16,
            pub second: u16,
            pub third: u16,
        }

        #[derive(Debug, Default, Clone)]
        pub struct PKTSkillStartNotifyInner {
            pub tripod_index: Option<TripodIndex>,
            pub tripod_level: Option<TripodLevel>,
        }

        #[derive(Debug, Default, Clone)]
        pub struct PKTSkillStartNotify {
            pub source_id: EntityId,
            pub skill_id: SkillId,
            pub skill_option_data: PKTSkillStartNotifyInner,
        }
        
        impl_new_default!(PKTSkillStartNotify);

        #[derive(Debug, Default, Clone)]
        pub struct PKTSkillDamageAbnormalMoveNotifyInner {
            pub skill_damage_event: SkillDamageEvent
        }

        #[derive(Debug, Default, Clone)]
        pub struct PKTSkillDamageAbnormalMoveNotify {
            pub source_id: EntityId,
            pub skill_damage_abnormal_move_events: Vec<PKTSkillDamageAbnormalMoveNotifyInner>,
            pub skill_id: SkillId,
            pub skill_effect_id: SkillEffectId,
        }
        
        impl_new_default!(PKTSkillDamageAbnormalMoveNotify);

        #[derive(Debug, Default, Clone)]
        pub struct PKTSkillDamageNotify {
            pub source_id: EntityId,
            pub skill_damage_events: Vec<SkillDamageEvent>,
            pub skill_id: SkillId,
            pub skill_effect_id: Option<SkillEffectId>,
        }

        impl_new_default!(PKTSkillDamageNotify);

        #[derive(Debug, Default, Clone)]
        pub struct PKTPartyInfoInner {
            pub name: String,
            pub class_id: ClassId,
            pub character_id: CharacterId,
            pub gear_level: GearLevel,
        }

        #[derive(Debug, Default, Clone)]
        pub struct PKTPartyInfo {
            pub party_instance_id: PartyInstanceId,
            pub raid_instance_id: RaidInstanceId,
            pub party_member_datas: Vec<PKTPartyInfoInner>
        }

        impl_new_default!(PKTPartyInfo);

        #[derive(Debug, Default, Clone)]
        pub struct PKTPartyLeaveResult {
            pub party_instance_id: PartyInstanceId,
            pub name: String
        }
        
        impl_new_default!(PKTPartyLeaveResult);

        #[derive(Debug, Default, Clone)]
        pub struct PKTPartyStatusEffectAddNotify {
            pub character_id: u64,
            pub status_effect_datas: Vec<StatusEffectData>
        }

        impl_new_default!(PKTPartyStatusEffectAddNotify);
        
        #[derive(Debug, Default, Clone)]
        pub struct PKTPartyStatusEffectRemoveNotify {
            pub character_id: CharacterId,
            pub status_effect_instance_ids: Vec<StatusEffectInstanceId>,
            pub reason: u8
        }

        impl_new_default!(PKTPartyStatusEffectRemoveNotify);
        
        #[derive(Debug, Default, Clone)]
        pub struct PKTPartyStatusEffectResultNotify {
            pub raid_instance_id: RaidInstanceId,
            pub party_instance_id: PartyInstanceId,
            pub character_id: CharacterId
        }
        
        impl_new_default!(PKTPartyStatusEffectResultNotify);

        #[derive(Debug, Default, Clone)]
        pub struct PKTStatusEffectAddNotify {
            pub object_id: EntityId,
            pub status_effect_data: StatusEffectData
        }

        impl_new_default!(PKTStatusEffectAddNotify);
        
        #[derive(Debug, Default, Clone)]
        pub struct PKTStatusEffectRemoveNotify {
            pub object_id: EntityId,
            pub character_id: CharacterId,
            pub status_effect_instance_ids: Vec<StatusEffectInstanceId>,
            pub reason: u8
        }
        
        impl_new_default!(PKTStatusEffectRemoveNotify);

        #[derive(Debug, Default, Clone)]
        pub struct PKTTriggerStartNotify {
            pub signal: u32,
        }

        impl_new_default!(PKTTriggerStartNotify);
        
        #[derive(Debug, Default, Clone)]
        pub struct PKTZoneMemberLoadStatusNotify {
            pub zone_id: u32,
            pub zone_level: u32
        }

        impl_new_default!(PKTZoneMemberLoadStatusNotify);
        
        #[derive(Debug, Default, Clone)]
        pub struct PKTZoneObjectUnpublishNotify {
            pub object_id: u64
        }
        
        impl_new_default!(PKTZoneObjectUnpublishNotify);

        #[derive(Debug, Default, Clone)]
        pub struct PKTStatusEffectSyncDataNotify {
            pub object_id: EntityId,
            pub status_effect_instance_id: StatusEffectInstanceId,
            pub character_id: CharacterId,
            pub value: u64,
        }
        
        impl_new_default!(PKTStatusEffectSyncDataNotify);

        #[derive(Debug, Default, Clone)]
        pub struct PKTTroopMemberUpdateMinNotify {
            pub character_id: u64,
            pub cur_hp: i64,
            pub max_hp: i64,
            pub status_effect_datas: Vec<StatusEffectData>,
        }

        impl_new_default!(PKTTroopMemberUpdateMinNotify);
        
        #[derive(Debug, Default, Clone)]
        pub struct PKTNewTransit {
            pub channel_id: u32
        }

        impl_new_default!(PKTNewTransit);

        #[derive(Debug, Default, Clone)]
        pub struct PKTNewPCInner {
            pub player_id: EntityId,
            pub name: String,
            pub class_id: ClassId,
            pub max_item_level: GearLevel,
            pub character_id: CharacterId,
            pub stat_pairs: Vec<StatPair>,
            pub equip_item_datas: Vec<EquipItemData>,
            pub status_effect_datas: Vec<StatusEffectData>
        }

        #[derive(Debug, Default, Clone)]
        pub struct PKTNewPC {
            pub pc_struct: PKTNewPCInner
        }

        impl_new_default!(PKTNewPC);
    }

    pub mod structures {
        use crate::{EntityId, NpcId, StatusEffectId, StatusEffectInstanceId};


        #[derive(Debug, Default, Clone)]
        pub struct StatusEffectData {
            pub source_id: EntityId,
            pub status_effect_id: StatusEffectId,
            pub status_effect_instance_id: StatusEffectInstanceId,
            pub value: Option<Vec<u8>>,
            pub total_time: f32,
            pub stack_count: u8,
            pub end_tick: u64
        }

        #[derive(Debug, Default, Clone)]
        pub struct EquipItemData {

        }
        
        #[derive(Debug, Default, Clone)]
        pub struct NpcStruct {
            pub object_id: EntityId,
            pub type_id: NpcId,
            pub level: u16,
            pub balance_level: Option<u16>,
            pub stat_pairs: Vec<StatPair>,
            pub status_effect_datas: Vec<StatusEffectData>
        }

        #[derive(Debug, Default, Clone)]
        pub struct StatPair {
            pub stat_type: u8,
            pub value: i64
        }

        #[derive(Debug, Default, Clone)]
        pub struct SkillDamageEvent {
            pub target_id: u64,
            pub damage: i64,
            pub modifier: i32,
            pub cur_hp: i64,
            pub max_hp: i64,
            pub damage_attr: Option<u8>,
            pub damage_type: u8,
        }
    }

    pub mod opcodes {
        pub enum Pkt {
            CounterAttackNotify,
            DeathNotify,
            IdentityGaugeChangeNotify,
            InitEnv,
            InitPC,
            NewPC,
            NewNpc,
            NewNpcSummon,
            NewProjectile,
            NewTrap,
            RaidBegin,
            RaidBossKillNotify,
            RaidResult,
            RemoveObject,
            SkillCastNotify,
            SkillStartNotify,
            SkillDamageAbnormalMoveNotify,
            SkillDamageNotify,
            PartyInfo,
            PartyLeaveResult,
            PartyStatusEffectAddNotify,
            PartyStatusEffectRemoveNotify,
            PartyStatusEffectResultNotify,
            StatusEffectAddNotify,
            StatusEffectRemoveNotify,
            TriggerBossBattleStatus,
            TriggerStartNotify,
            ZoneMemberLoadStatusNotify,
            ZoneObjectUnpublishNotify,
            StatusEffectSyncDataNotify,
            TroopMemberUpdateMinNotify,
            NewTransit
        }
    }
}

pub mod decryption {
    use crate::packets::structures::SkillDamageEvent;

    pub struct DamageEncryptionHandler{}
    pub struct DamageEncryptionHandlerInner{}

    impl DamageEncryptionHandlerInner{
        pub fn decrypt_damage_event(&self, event: &mut SkillDamageEvent) -> bool {
            true
        }

        pub fn update_zone_instance_id(&self, channel_id: u32) {

        }
    }

    impl DamageEncryptionHandler{
        pub fn new() -> Self {
            Self {}
        }

        pub fn start(&self) -> anyhow::Result<DamageEncryptionHandlerInner> {
            Ok(DamageEncryptionHandlerInner {})
        }
    }
}

pub fn start_capture(_port: u16, _region_file_path: String) -> Result<Receiver<(Pkt, Vec<u8>)>, Box<dyn Error>> {
    let (_tx, rx) = mpsc::channel::<(Pkt, Vec<u8>)>();

    Ok(rx)
}