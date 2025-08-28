use std::sync::mpsc::Receiver;

use anyhow::*;
use meter_core::packets::opcodes::Pkt;
use meter_core::packets::structures::SkillDamageEvent;
use meter_core::start_capture;
use meter_core::decryption::DamageEncryptionHandler as MeterCoreDamageEncryptionHandler;

pub trait PacketCapture {
    fn start(&self, port: u16) -> Result<Receiver<(Pkt, Vec<u8>)>>;
}

pub trait DamageEncryptionHandler {
    fn start(&mut self) -> Result<()>;
    fn decrypt_damage_event(&self, skill_damage_event: &mut SkillDamageEvent) -> bool;
    fn update_zone_instance_id(&mut self, zone_instance_id: u32);
}

pub struct DefaultDamageEncryptionHandler(Option<MeterCoreDamageEncryptionHandler>);
pub struct WindivertPacketCapture {
    region_file_path: String
}

impl PacketCapture for WindivertPacketCapture {
    fn start(&self, port: u16) -> Result<Receiver<(Pkt, Vec<u8>)>> {
        start_capture(port, self.region_file_path.clone())
    }
}

impl DamageEncryptionHandler for DefaultDamageEncryptionHandler {
    fn start(&mut self) -> Result<()> {
        let handler = MeterCoreDamageEncryptionHandler::new();
        let handler = handler.start()?;
        self.0 = Some(handler);
        Ok(())
    }

    fn decrypt_damage_event(&self, skill_damage_event: &mut SkillDamageEvent) -> bool {
        let handler = self.0.as_ref().expect("DamageEncryptionHandler is not initialized");

        handler.decrypt_damage_event(skill_damage_event)
    }

    fn update_zone_instance_id(&mut self, zone_instance_id: u32) {
        let handler = self.0.as_mut().expect("DamageEncryptionHandler is not initialized");

        handler.update_zone_instance_id(zone_instance_id);
    }
}


impl WindivertPacketCapture {
    pub fn new(region_file_path: String) -> Self {
        Self {
            region_file_path
        }
    }
}

impl DefaultDamageEncryptionHandler {
    pub fn new() -> Self {
        Self(None)
    }
}