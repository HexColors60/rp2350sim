//! Bus trait.

use crate::{AccessWidth, Result};

/// Bus trait for memory access.
pub trait Bus: Send + Sync {
    /// Read from the bus.
    fn read(&mut self, addr: u32, width: AccessWidth) -> Result<u64>;

    /// Write to the bus.
    fn write(&mut self, addr: u32, value: u64, width: AccessWidth) -> Result<()>;

    /// Check if an address is mapped.
    fn is_mapped(&self, addr: u32) -> bool;

    /// Get the size of the bus address space.
    fn address_space_size(&self) -> u64;
}

/// Bus access hook.
pub trait BusHook: Send + Sync {
    fn on_read(&mut self, addr: u32, width: AccessWidth, value: &mut u64) -> bool;
    fn on_write(&mut self, addr: u32, width: AccessWidth, value: u64) -> bool;
}