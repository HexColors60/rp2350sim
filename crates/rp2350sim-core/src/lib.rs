//! RP2350 Simulator Core Library
//!
//! This crate provides the core types, traits, and infrastructure
//! for the RP2350 full-system simulator.

pub mod consts;
pub mod error;
pub mod event;
pub mod ids;
pub mod time;
pub mod traits;
pub mod types;
pub mod util;

pub use consts::*;
pub use error::{Error, Result};
pub use event::{Event, EventQueue, Scheduler};
pub use ids::{CoreId, DeviceId, IrqId, PioSmId, PinId, UartId, SpiId, I2cId};
pub use time::{ClockDomain, ClockDomainId, Ticks};
pub use traits::*;
pub use types::*;