pub mod traits;
pub mod heartbeat;
pub mod region;

pub use traits::*;
pub use heartbeat::*;
pub use region::*;

#[cfg(feature = "meter-core")]
pub mod snow_meter;

#[cfg(feature = "meter-core")]
pub use snow_meter::*;

#[cfg(feature = "meter-core-fake")]
pub mod fake_meter;

#[cfg(feature = "meter-core-fake")]
pub use fake_meter::*;

#[cfg(feature = "meter-core")]
pub use snow_meter::packets::opcodes::Pkt;

#[cfg(feature = "meter-core-fake")]
pub use meter_core_fake::packets::opcodes::Pkt;

pub mod definitions {
    #[cfg(feature = "meter-core")]
    pub use meter_core::packets::definitions::*;

    #[cfg(feature = "meter-core-fake")]
    pub use meter_core_fake::packets::definitions::*;
}

pub mod structures {
    #[cfg(feature = "meter-core")]
    pub use meter_core::packets::structures::*;

    #[cfg(feature = "meter-core")]
    pub use meter_core::packets::common::*;

    #[cfg(feature = "meter-core-fake")]
    pub use meter_core_fake::packets::structures::*;

    #[cfg(feature = "meter-core-fake")]
    pub use meter_core_fake::packets::common::*;
}