use meter_core::packets::{definitions::*, opcodes::Pkt};
use anyhow::Result;

pub type Packet = (Pkt, Vec<u8>);

#[derive(Debug)]
pub enum LoaPacket {
    Unknown(Pkt),
    CounterAttackNotify(PKTCounterAttackNotify),
    DeathNotify(PKTDeathNotify),
    IdentityGaugeChangeNotify(PKTIdentityGaugeChangeNotify),
    /// three methods of getting local player info
    /// 1. MigrationExecute    + InitEnv      + PartyInfo
    /// 2. Cached Local Player + InitEnv      + PartyInfo
    ///    > character_id        > entity_id    > player_info
    /// 3. InitPC
    InitEnv(PKTInitEnv),
    /// Local player initialization
    InitPC(PKTInitPC),
    /// Player initialization
    NewPC(PKTNewPC),
    NewNpc(PKTNewNpc),
    NewVehicle(PKTNewVehicle),
    NewNpcSummon(PKTNewNpcSummon),
    NewProjectile(PKTNewProjectile),
    NewTrap(PKTNewTrap),
    RaidBegin(PKTRaidBegin),
    RaidBossKillNotify,
    RaidResult,
    RemoveObject(PKTRemoveObject),
    SkillCastNotify(PKTSkillCastNotify),
    SkillStartNotify(PKTSkillStartNotify),
    SkillCooldownNotify(PKTSkillCooldownNotify),
    SkillDamageAbnormalMoveNotify(PKTSkillDamageAbnormalMoveNotify),
    SkillDamageNotify(PKTSkillDamageNotify),
    PartyInfo(PKTPartyInfo),
    PartyLeaveResult(PKTPartyLeaveResult),
    PartyStatusEffectAddNotify(PKTPartyStatusEffectAddNotify),
    PartyStatusEffectRemoveNotify(PKTPartyStatusEffectRemoveNotify),
    PartyStatusEffectResultNotify(PKTPartyStatusEffectResultNotify),
    StatusEffectAddNotify(PKTStatusEffectAddNotify),
    StatusEffectRemoveNotify(PKTStatusEffectRemoveNotify),
    TriggerBossBattleStatus,
    TriggerStartNotify(PKTTriggerStartNotify),
    ZoneMemberLoadStatusNotify(PKTZoneMemberLoadStatusNotify),
    ZoneObjectUnpublishNotify(PKTZoneObjectUnpublishNotify),
    StatusEffectSyncDataNotify(PKTStatusEffectSyncDataNotify),
    TroopMemberUpdateMinNotify(PKTTroopMemberUpdateMinNotify),
    NewTransit(PKTNewTransit),
}

use std::convert::TryFrom;

impl TryFrom<Packet> for LoaPacket {
    type Error = anyhow::Error;

    fn try_from(value: Packet) -> Result<Self, Self::Error> {
        match value.0 {
            Pkt::CounterAttackNotify => {
                let packet = PKTCounterAttackNotify::new(&value.1)?;
                Ok(Self::CounterAttackNotify(packet))
            }
            Pkt::DeathNotify => {
                let packet = PKTDeathNotify::new(&value.1)?;
                Ok(Self::DeathNotify(packet))
            }
            Pkt::IdentityGaugeChangeNotify => {
                let packet = PKTIdentityGaugeChangeNotify::new(&value.1)?;
                Ok(Self::IdentityGaugeChangeNotify(packet))
            }
            Pkt::InitEnv => {
                let packet = PKTInitEnv::new(&value.1)?;
                Ok(Self::InitEnv(packet))
            }
            Pkt::InitPC => {
                let packet = PKTInitPC::new(&value.1)?;
                Ok(Self::InitPC(packet))
            }
            Pkt::NewPC => {
                let packet = PKTNewPC::new(&value.1)?;
                Ok(Self::NewPC(packet))
            }
            Pkt::NewNpc => {
                let packet = PKTNewNpc::new(&value.1)?;
                Ok(Self::NewNpc(packet))
            }
            Pkt::NewVehicle => {
                let packet = PKTNewVehicle::new(&value.1)?;
                Ok(Self::NewVehicle(packet))
            }
            Pkt::NewNpcSummon => {
                let packet = PKTNewNpcSummon::new(&value.1)?;
                Ok(Self::NewNpcSummon(packet))
            }
            Pkt::NewProjectile => {
                let packet = PKTNewProjectile::new(&value.1)?;
                Ok(Self::NewProjectile(packet))
            }
            Pkt::NewTrap => {
                let packet = PKTNewTrap::new(&value.1)?;
                Ok(Self::NewTrap(packet))
            }
            Pkt::RaidBegin => {
                let packet = PKTRaidBegin::new(&value.1)?;
                Ok(Self::RaidBegin(packet))
            }
            Pkt::RaidBossKillNotify => Ok(Self::RaidBossKillNotify),
            Pkt::RaidResult => Ok(Self::RaidResult),
            Pkt::RemoveObject => {
                let packet = PKTRemoveObject::new(&value.1)?;
                Ok(Self::RemoveObject(packet))
            }
            Pkt::SkillCastNotify => {
                let packet = PKTSkillCastNotify::new(&value.1)?;
                Ok(Self::SkillCastNotify(packet))
            }
            Pkt::SkillStartNotify => {
                let packet = PKTSkillStartNotify::new(&value.1)?;
                Ok(Self::SkillStartNotify(packet))
            }
            Pkt::SkillCooldownNotify => {
                let packet = PKTSkillCooldownNotify::new(&value.1)?;
                Ok(Self::SkillCooldownNotify(packet))
            }
            Pkt::SkillDamageAbnormalMoveNotify => {
                let packet = PKTSkillDamageAbnormalMoveNotify::new(&value.1)?;
                Ok(Self::SkillDamageAbnormalMoveNotify(packet))
            }
            Pkt::SkillDamageNotify => {
                let packet = PKTSkillDamageNotify::new(&value.1)?;
                Ok(Self::SkillDamageNotify(packet))
            }
            Pkt::PartyInfo => {
                let packet = PKTPartyInfo::new(&value.1)?;
                Ok(Self::PartyInfo(packet))
            }
            Pkt::PartyLeaveResult => {
                let packet = PKTPartyLeaveResult::new(&value.1)?;
                Ok(Self::PartyLeaveResult(packet))
            }
            Pkt::PartyStatusEffectAddNotify => {
                let packet = PKTPartyStatusEffectAddNotify::new(&value.1)?;
                Ok(Self::PartyStatusEffectAddNotify(packet))
            }
            Pkt::PartyStatusEffectRemoveNotify => {
                let packet = PKTPartyStatusEffectRemoveNotify::new(&value.1)?;
                Ok(Self::PartyStatusEffectRemoveNotify(packet))
            }
            Pkt::PartyStatusEffectResultNotify => {
                let packet = PKTPartyStatusEffectResultNotify::new(&value.1)?;
                Ok(Self::PartyStatusEffectResultNotify(packet))
            }
            Pkt::StatusEffectAddNotify => {
                let packet = PKTStatusEffectAddNotify::new(&value.1)?;
                Ok(Self::StatusEffectAddNotify(packet))
            }
            Pkt::StatusEffectRemoveNotify => {
                let packet = PKTStatusEffectRemoveNotify::new(&value.1)?;
                Ok(Self::StatusEffectRemoveNotify(packet))
            }
            Pkt::TriggerBossBattleStatus => Ok(Self::TriggerBossBattleStatus),
            Pkt::TriggerStartNotify => {
                let packet = PKTTriggerStartNotify::new(&value.1)?;
                Ok(Self::TriggerStartNotify(packet))
            }
            Pkt::ZoneMemberLoadStatusNotify => {
                let packet = PKTZoneMemberLoadStatusNotify::new(&value.1)?;
                Ok(Self::ZoneMemberLoadStatusNotify(packet))
            }
            Pkt::ZoneObjectUnpublishNotify => {
                let packet = PKTZoneObjectUnpublishNotify::new(&value.1)?;
                Ok(Self::ZoneObjectUnpublishNotify(packet))
            }
            Pkt::StatusEffectSyncDataNotify => {
                let packet = PKTStatusEffectSyncDataNotify::new(&value.1)?;
                Ok(Self::StatusEffectSyncDataNotify(packet))
            }
            Pkt::TroopMemberUpdateMinNotify => {
                let packet = PKTTroopMemberUpdateMinNotify::new(&value.1)?;
                Ok(Self::TroopMemberUpdateMinNotify(packet))
            }
            Pkt::NewTransit => {
                let packet = PKTNewTransit::new(&value.1)?;
                Ok(Self::NewTransit(packet))
            }
            #[allow(unreachable_patterns)]
            packet => Ok(Self::Unknown(packet))
        }
    }
}