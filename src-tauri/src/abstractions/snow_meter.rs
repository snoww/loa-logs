use meter_core::packets::opcodes::Pkt;
use meter_core::packets::structures::SkillDamageEvent;
use meter_core::start_capture;
use meter_core::decryption::DamageEncryptionHandler as MeterCoreDamageEncryptionHandler;
use std::{fs, path::Path};
use anyhow::Result;
use crate::abstractions::PacketCapture;

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

/// Ensures that the required WinDivert files are available in the current directory.
///
/// ### Windows-specific
/// This function is only relevant on Windows platforms. It is a workaround for cases where
/// the DLL (`WinDivert.dll`) and SYS (`WinDivert64.sys`) driver files are locked by an already running kernel driver.
pub fn load_windivert(current_dir: &Path) -> Result<()> {
    #[cfg(all(target_os = "windows"))]
    {
        let windivert_dll_path = current_dir.join("WinDivert.dll");

        if !windivert_dll_path.exists() {
            let bytes: &'static [u8] = include_bytes!("../WinDivert.dll");
            fs::write(windivert_dll_path, bytes)?;
        }

        let windivert_driver_path = current_dir.join("WinDivert64.sys");
        
        if !windivert_driver_path.exists() {
            let bytes: &'static [u8] = include_bytes!("../WinDivert64.sys");
            fs::write(windivert_driver_path, bytes)?;
        }
    }

    Ok(())
}