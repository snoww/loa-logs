use std::{sync::mpsc::{self, Receiver}, thread::{self, sleep}, time::Duration};
use log::info;
use lost_metrics_simulator::*;
use meter_core_fake::packets::structures::SkillDamageEvent;
use meter_core_fake::packets::opcodes::Pkt;

use crate::abstractions::{DamageEncryptionHandler, PacketSource, PacketReceiver};
use anyhow::Result;

pub struct DefaultDamageEncryptionHandler;
pub struct FakePacketSource;

pub struct FakeReceiver {
    inner: Receiver<(Pkt, Vec<u8>)>,
}

impl PacketReceiver for FakeReceiver {
    fn recv(&mut self) -> Result<(Pkt, Vec<u8>)> {
        // Just call the standard receiver
        Ok(self.inner.recv()?)
    }
}


impl PacketSource<FakeReceiver> for FakePacketSource {
    fn start(&self, _port: u16) -> Result<FakeReceiver> {

        let (emitter, rx) = mpsc::channel();

        let builder = thread::Builder::new()
            .name("fake-sniffer".to_string());

        let handle = builder.spawn(move || {
            let delay = Duration::from_secs(1);

            loop {
                run_encounter(emitter.clone(), delay)?;
            }

            anyhow::Ok::<()>(())
        })?;

        Ok(FakeReceiver { inner: rx })
    }
}

impl DamageEncryptionHandler for DefaultDamageEncryptionHandler {
    fn start(&mut self) -> Result<()> {
        
        Ok(())
    }

    fn decrypt_damage_event(&self, _skill_damage_event: &mut SkillDamageEvent) -> bool {
        true
    }

    fn update_zone_instance_id(&mut self, _zone_instance_id: u32) {
        
    }
}


impl FakePacketSource {
    pub fn new() -> Self {
        Self
    }
}

impl DefaultDamageEncryptionHandler {
    pub fn new() -> Self {
        Self
    }
}