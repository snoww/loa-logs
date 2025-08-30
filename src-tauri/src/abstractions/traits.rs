use std::sync::mpsc::Receiver;

use anyhow::*;

#[cfg(feature = "meter-core")]
use meter_core::packets::structures::SkillDamageEvent;

#[cfg(feature = "meter-core")]
use meter_core::packets::opcodes::Pkt;

#[cfg(feature = "meter-core-fake")]
use meter_core_fake::packets::structures::SkillDamageEvent;

#[cfg(feature = "meter-core-fake")]
use meter_core_fake::packets::opcodes::Pkt;

pub trait PacketReceiver {
    fn recv(&mut self) -> Result<(Pkt, Vec<u8>)>;
}

pub trait PacketSource<PR: PacketReceiver> {
    fn start(&self, port: u16) -> Result<PR>;
}

pub trait DamageEncryptionHandler {
    fn start(&mut self) -> Result<()>;
    fn decrypt_damage_event(&self, skill_damage_event: &mut SkillDamageEvent) -> bool;
    fn update_zone_instance_id(&mut self, zone_instance_id: u32);
}
