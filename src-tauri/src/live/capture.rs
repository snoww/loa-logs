use std::{path::PathBuf, sync::mpsc::Receiver};

use anyhow::{anyhow, Context, Result};
use meter_core::{decryption::DamageEncryptionHandler, packets::{opcodes::Pkt, structures::SkillDamageEvent}, start_capture};

use crate::{live::packet::*, region::{RegionAccessor, SavedToFileRegionAccessor}};

#[cfg(test)]
use mockall::{automock, predicate::*};

#[cfg_attr(test, automock)]
pub trait PacketCapture {
    fn start(&mut self) -> Result<()>;
    fn recv(&mut self) -> Result<LoaPacket>;
    fn create_region_accessor(&self) -> Box<dyn RegionAccessor>;
    fn update_zone_instance_id(&self, zone_id: u32);
    fn decrypt_damage_event(&self, event: &mut SkillDamageEvent) -> bool;
}

pub struct SnowPacketCapture {
    port: u16,
    region_file_path: PathBuf,
    receiver: Option<Receiver<Packet>>,
    damage_handler: Option<DamageEncryptionHandler>,
}

impl PacketCapture for SnowPacketCapture {
    fn start(&mut self) -> Result<()> {
        let path = self.region_file_path.display().to_string();
        let receiver = start_capture(self.port, path).with_context(|| "Could not start packet capture")?;
        let mut damage_handler = DamageEncryptionHandler::new();
        damage_handler = damage_handler.start().with_context(|| "Could not start damage handler")?;
        
        self.damage_handler = Some(damage_handler);
        self.receiver = Some(receiver);

        Ok(())
    }
    
    fn recv(&mut self) -> Result<LoaPacket> {
        let packet = self.receiver.as_ref().with_context(|| "Call start first")?.recv()?;
        let packet = LoaPacket::try_from(packet).with_context(|| "Error parsing {}: {}")?;
        Ok(packet)
    }
    
    fn create_region_accessor(&self) -> Box<dyn RegionAccessor> {
        Box::new(SavedToFileRegionAccessor::new(self.region_file_path.clone()))
    }
    
    fn update_zone_instance_id(&self, channel_id: u32) {
        let damage_handler = self.damage_handler.as_ref().expect("damage handler is not initialized");
        damage_handler.update_zone_instance_id(channel_id);
    }
    
    fn decrypt_damage_event(&self, event: &mut SkillDamageEvent) -> bool {
        let damage_handler = self.damage_handler.as_ref().expect("damage handler is not initialized");
        damage_handler.decrypt_damage_event(event)
    }
}

impl SnowPacketCapture {
    pub fn new(port: u16, region_file_path: PathBuf) -> Self {
        Self {
            port,
            region_file_path,
            receiver: None,
            damage_handler: None
        }
    }
}