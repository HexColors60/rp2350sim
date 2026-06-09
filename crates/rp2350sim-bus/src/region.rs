//! Memory region types.

use serde::{Deserialize, Serialize};

/// Region kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegionKind {
    /// Boot ROM
    BootRom,
    /// Flash/XIP
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

/// Memory region.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    /// Region name.
    pub name: String,
    /// Base address.
    pub base: u32,
    /// Size in bytes.
    pub size: u32,
    /// Region kind.
    pub kind: RegionKind,
    /// Readable.
    pub readable: bool,
    /// Writable.
    pub writable: bool,
    /// Executable.
    pub executable: bool,
}

impl Region {
    pub fn new(name: impl Into<String>, base: u32, size: u32, kind: RegionKind) -> Self {
        Self {
            name: name.into(),
            base,
            size,
            kind,
            readable: true,
            writable: true,
            executable: false,
        }
    }

    pub fn readonly(mut self) -> Self {
        self.writable = false;
        self
    }

    pub fn writeonly(mut self) -> Self {
        self.readable = false;
        self
    }

    pub fn executable(mut self) -> Self {
        self.executable = true;
        self
    }

    pub const fn end(&self) -> u32 {
        self.base + self.size - 1
    }

    pub const fn contains(&self, addr: u32) -> bool {
        addr >= self.base && addr < self.base + self.size
    }

    pub const fn offset(&self, addr: u32) -> u32 {
        addr - self.base
    }

    pub const fn overlaps(&self, other: &Self) -> bool {
        self.base < other.base + other.size && other.base < self.base + self.size
    }
}