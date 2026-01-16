use meter_core::packets::{structures::SkillDamageEvent, common::SkillMoveOptionData, definitions::*, opcodes::Pkt};
use anyhow::{Result, Context};
use serde::Serialize;

pub type Packet = (Pkt, Vec<u8>);

#[derive(Debug, Serialize, Clone)]
pub struct PKTSkillDamage {
    pub damage_is_valid: bool,
    pub events: Vec<(SkillDamageEvent, Option<SkillMoveOptionData>)>,
    pub source_id: u64,
    pub skill_id: u32,
    pub skill_effect_id: Option<u32>,
}

#[derive(Debug, Serialize, Clone)]
pub enum LoaPacket {
    #[cfg(test)]
    Test,
    Unknown(Packet),
    CounterAttackNotify(PKTCounterAttackNotify),
    DeathNotify(PKTDeathNotify),
    IdentityGaugeChangeNotify(PKTIdentityGaugeChangeNotify),
    InitEnv(PKTInitEnv),
    InitPC(PKTInitPC),
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
    SkillDamage(PKTSkillDamage),
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
        let (op, buf) = value;

        match op {
            Pkt::CounterAttackNotify => Ok(Self::CounterAttackNotify(
                parse_packet(buf, PKTCounterAttackNotify::new)?,
            )),

            Pkt::DeathNotify => Ok(Self::DeathNotify(
                parse_packet(buf, PKTDeathNotify::new)?,
            )),

            Pkt::IdentityGaugeChangeNotify => Ok(Self::IdentityGaugeChangeNotify(
                parse_packet(buf, PKTIdentityGaugeChangeNotify::new)?,
            )),

            Pkt::InitEnv => Ok(Self::InitEnv(
                parse_packet(buf, PKTInitEnv::new)?,
            )),

            Pkt::InitPC => Ok(Self::InitPC(
                parse_packet(buf, PKTInitPC::new)?,
            )),

            Pkt::NewPC => Ok(Self::NewPC(
                parse_packet(buf, PKTNewPC::new)?,
            )),

            Pkt::NewNpc => Ok(Self::NewNpc(
                parse_packet(buf, PKTNewNpc::new)?,
            )),

            Pkt::NewVehicle => Ok(Self::NewVehicle(
                parse_packet(buf, PKTNewVehicle::new)?,
            )),

            Pkt::NewNpcSummon => Ok(Self::NewNpcSummon(
                parse_packet(buf, PKTNewNpcSummon::new)?,
            )),

            Pkt::NewProjectile => Ok(Self::NewProjectile(
                parse_packet(buf, PKTNewProjectile::new)?,
            )),

            Pkt::NewTrap => Ok(Self::NewTrap(
                parse_packet(buf, PKTNewTrap::new)?,
            )),

            Pkt::RaidBegin => Ok(Self::RaidBegin(
                parse_packet(buf, PKTRaidBegin::new)?,
            )),

            Pkt::RaidBossKillNotify => Ok(Self::RaidBossKillNotify),

            Pkt::RaidResult => Ok(Self::RaidResult),

            Pkt::RemoveObject => Ok(Self::RemoveObject(
                parse_packet(buf, PKTRemoveObject::new)?,
            )),

            Pkt::SkillCastNotify => Ok(Self::SkillCastNotify(
                parse_packet(buf, PKTSkillCastNotify::new)?,
            )),

            Pkt::SkillStartNotify => Ok(Self::SkillStartNotify(
                parse_packet(buf, PKTSkillStartNotify::new)?,
            )),

            Pkt::SkillCooldownNotify => Ok(Self::SkillCooldownNotify(
                parse_packet(buf, PKTSkillCooldownNotify::new)?,
            )),

            Pkt::PartyInfo => Ok(Self::PartyInfo(
                parse_packet(buf, PKTPartyInfo::new)?,
            )),

            Pkt::PartyLeaveResult => Ok(Self::PartyLeaveResult(
                parse_packet(buf, PKTPartyLeaveResult::new)?,
            )),

            Pkt::PartyStatusEffectAddNotify => Ok(Self::PartyStatusEffectAddNotify(
                parse_packet(buf, PKTPartyStatusEffectAddNotify::new)?,
            )),

            Pkt::PartyStatusEffectRemoveNotify => Ok(Self::PartyStatusEffectRemoveNotify(
                parse_packet(buf, PKTPartyStatusEffectRemoveNotify::new)?,
            )),

            Pkt::PartyStatusEffectResultNotify => Ok(Self::PartyStatusEffectResultNotify(
                parse_packet(buf, PKTPartyStatusEffectResultNotify::new)?,
            )),

            Pkt::StatusEffectAddNotify => Ok(Self::StatusEffectAddNotify(
                parse_packet(buf, PKTStatusEffectAddNotify::new)?,
            )),

            Pkt::StatusEffectRemoveNotify => Ok(Self::StatusEffectRemoveNotify(
                parse_packet(buf, PKTStatusEffectRemoveNotify::new)?,
            )),

            Pkt::TriggerBossBattleStatus => Ok(Self::TriggerBossBattleStatus),

            Pkt::TriggerStartNotify => Ok(Self::TriggerStartNotify(
                parse_packet(buf, PKTTriggerStartNotify::new)?,
            )),

            Pkt::ZoneMemberLoadStatusNotify => Ok(Self::ZoneMemberLoadStatusNotify(
                parse_packet(buf, PKTZoneMemberLoadStatusNotify::new)?,
            )),

            Pkt::ZoneObjectUnpublishNotify => Ok(Self::ZoneObjectUnpublishNotify(
                parse_packet(buf, PKTZoneObjectUnpublishNotify::new)?,
            )),

            Pkt::StatusEffectSyncDataNotify => Ok(Self::StatusEffectSyncDataNotify(
                parse_packet(buf, PKTStatusEffectSyncDataNotify::new)?,
            )),

            Pkt::TroopMemberUpdateMinNotify => Ok(Self::TroopMemberUpdateMinNotify(
                parse_packet(buf, PKTTroopMemberUpdateMinNotify::new)?,
            )),

            _ => Ok(Self::Unknown((op, buf))),
        }
    }
}

fn parse_packet<T, B>(
    buf: B,
    f: impl FnOnce(&[u8]) -> Result<T>,
) -> Result<T>
where
    B: AsRef<[u8]>,
{
    let type_name = std::any::type_name::<T>();
    f(buf.as_ref()).with_context(|| format!("failed to parse {}", type_name))
}