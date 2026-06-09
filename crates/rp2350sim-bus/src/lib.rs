//! RP2350 Bus System
//!
//! This crate provides the bus infrastructure for memory-mapped
//! access to devices and memory regions.

pub mod access;
pub mod bus;
pub mod dispatch;
pub mod error;
pub mod map;
pub mod region;
pub mod trace;

pub use bus::Bus;
pub use error::{BusError, Result};
pub use map::MemoryMap;
pub use region::{Region, RegionKind};