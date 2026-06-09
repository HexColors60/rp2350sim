//! Device trait.

use crate::{DeviceId, Result};

/// Device trait for peripherals.
pub trait Device: Send + Sync {
    /// Get the device ID.
    fn id(&self) -> DeviceId;

    /// Reset the device.
    fn reset(&mut self);

    /// Read a register.
    fn read(&mut self, addr: u32) -> Result<u32>;

    /// Write a register.
    fn write(&mut self, addr: u32, value: u32) -> Result<()>;
}