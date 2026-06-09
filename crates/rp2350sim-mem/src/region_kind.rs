//! Region kind enumeration.

use serde::{Deserialize, Serialize};

/// Kind of memory region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegionKind {
    /// Boot ROM
    BootRom,
    /// Flash memory
    Flash,
    /// SRAM
    Sram,
    /// Peripheral MMIO
    Peripheral,
    /// USB registers
    Usb,
    /// PIO registers
    Pio,
    /// GPIO/pad control
    Gpio,
    /// Timer/clock/reset
    Timer,
    /// Reserved/unmapped
    Reserved,
    /// Custom region
    Custom(u8),
}