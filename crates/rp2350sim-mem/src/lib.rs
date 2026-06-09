//! RP2350 Memory System
//!
//! This crate provides memory management for the RP2350 simulator,
//! including SRAM, Flash/XIP, Boot ROM, and firmware loading.

pub mod bootrom;
pub mod faults;
pub mod flash;
pub mod loader;
pub mod memory;
pub mod memory_map;
pub mod perms;
pub mod region_kind;
pub mod sram;
pub mod watch;

pub use memory::Memory;
pub use memory_map::MemoryMap;