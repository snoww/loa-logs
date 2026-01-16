use std::sync::mpsc::Receiver;

use anyhow::{Context, Result};
use meter_core::{packets::{definitions::{PKTNewTransit, PKTSkillDamageAbnormalMoveNotify, PKTSkillDamageNotify}, opcodes::Pkt}, start_capture, DamageEncryptionHandler};

use crate::live::packet::{PKTSkillDamage, LoaPacket, Packet};

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait PacketCapture {
    fn start(&mut self, port: u16) -> Result<()>;
    fn recv(&self) -> Result<Result<LoaPacket>>;
}

pub struct SnowPacketCapture {
    region_file_path: String,
    receiver: Receiver<Packet>,
    damage_handler: Option<DamageEncryptionHandler>,
}

impl PacketCapture for SnowPacketCapture {
    fn start(&mut self, port: u16) -> Result<()> {
        
        let receiver = start_capture(port, self.region_file_path.clone()).with_context(|| "Could not start packet capture")?;

        let mut damage_handler = DamageEncryptionHandler::new();
        damage_handler = damage_handler.start().with_context(|| "Could not start damage encryption handler")?;

        self.receiver = receiver;
        self.damage_handler = Some(damage_handler);

        Ok(())
    }
    
    fn recv(&self) -> Result<Result<LoaPacket>> {
        let (opcode, data) = self.receiver.recv()?;

        let packet = match opcode {
            Pkt::SkillDamageAbnormalMoveNotify => {
                
                let PKTSkillDamageAbnormalMoveNotify {
                    skill_damage_abnormal_move_events,
                    skill_effect_id,
                    skill_id,
                    source_id,
                    ..
                } = PKTSkillDamageAbnormalMoveNotify::new(&data)?;
                let damage_handler = self.damage_handler.as_ref().expect("damage encryption handler is not initialized");
                let mut damage_is_valid = true;

                let events: Vec<_> = skill_damage_abnormal_move_events.into_iter().filter_map(|mut event| {
                    
                    if damage_handler.decrypt_damage_event(&mut event.skill_damage_event) {
                        damage_is_valid = false;
                        Some((event.skill_damage_event, Some(event.skill_move_option_data)))
                    }
                    else {
                        None
                    }
                }).collect();

                let packet = PKTSkillDamage {
                    source_id,
                    skill_effect_id: Some(skill_effect_id),
                    skill_id,
                    damage_is_valid,
                    events
                };
                Ok(LoaPacket::SkillDamage(packet))
            },
            Pkt::SkillDamageNotify => {
                   
                let PKTSkillDamageNotify {
                    skill_damage_events,
                    skill_effect_id,
                    skill_id,
                    source_id,
                    ..
                } = PKTSkillDamageNotify::new(&data)?;
                let damage_handler = self.damage_handler.as_ref().expect("damage encryption handler is not initialized");
                let mut damage_is_valid = true;

                let events: Vec<_> = skill_damage_events.into_iter().filter_map(|mut event| {
                    if damage_handler.decrypt_damage_event(&mut event) {
                        damage_is_valid = false;
                        Some((event, None))
                    }
                    else {
                        None
                    }
                }).collect();

                let packet = PKTSkillDamage {
                    source_id,
                    skill_effect_id,
                    skill_id,
                    damage_is_valid,
                    events
                };
                Ok(LoaPacket::SkillDamage(packet))
            },
            Pkt::NewTransit => {
                
                let packet = PKTNewTransit::new(&data)?;
                let damage_handler = self.damage_handler.as_ref().expect("damage encryption handler is not initialized");
                damage_handler.update_zone_instance_id(packet.zone_instance_id);

                Ok(LoaPacket::NewTransit(packet))
            },
            _ => TryInto::<LoaPacket>::try_into((opcode, data))
        };

        Ok(packet)
    }
}

impl SnowPacketCapture {
    pub fn new(region_file_path: String) -> Self {
        // stub
        let (_tx, receiver) = std::sync::mpsc::channel();

        Self {
            region_file_path,
            receiver,
            damage_handler: None
        }
    }
}