//! Memory map for RP2350.

use rp2350sim_core::consts::*;

/// Memory region descriptor.
#[derive(Debug, Clone)]
pub struct MemRegion {
    pub name: &'static str,
    pub base: u32,
    pub size: u32,
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
}

impl MemRegion {
    pub const fn new(name: &'static str, base: u32, size: u32) -> Self {
        Self {
            name,
            base,
            size,
            readable: true,
            writable: true,
            executable: false,
        }
    }

    pub const fn readonly(mut self) -> Self {
        self.writable = false;
        self
    }

    pub const fn executable(mut self) -> Self {
        self.executable = true;
        self
    }

    pub const fn contains(&self, addr: u32) -> bool {
        addr >= self.base && addr < self.base + self.size
    }

    pub const fn end(&self) -> u32 {
        self.base + self.size - 1
    }
}

/// RP2350 memory map.
pub struct MemoryMap;

/// Static memory regions for RP2350.
static MEMORY_REGIONS: &[MemRegion] = &[
    // Boot ROM
    MemRegion::new("Boot ROM", BOOTROM_BASE, BOOTROM_SIZE as u32)
        .readonly()
        .executable(),
    // Flash/XIP
    MemRegion::new("Flash/XIP", XIP_BASE, 0x1000_0000)
        .executable(),
    // SRAM banks
    MemRegion::new("SRAM Bank 0", SRAM_BASE, 0x4000),
    MemRegion::new("SRAM Bank 1", SRAM_BASE + 0x4000, 0x4000),
    MemRegion::new("SRAM Bank 2", SRAM_BASE + 0x8000, 0x4000),
    MemRegion::new("SRAM Bank 3", SRAM_BASE + 0xC000, 0x4000),
    MemRegion::new("SRAM Bank 4", SRAM_BASE + 0x10000, 0x4000),
    MemRegion::new("SRAM Bank 5", SRAM_BASE + 0x14000, 0x4000),
    MemRegion::new("SRAM Bank 6", SRAM_BASE + 0x18000, 0x4000),
    MemRegion::new("SRAM Bank 7", SRAM_BASE + 0x1C000, 0x4000),
    // Peripheral space
    MemRegion::new("Peripheral", PERIPH_BASE, 0x1000_0000),
    // APB space
    MemRegion::new("APB", APB_BASE, 0x1000_0000),
    // IOPORT (fast GPIO)
    MemRegion::new("IOPORT", IOPORT_BASE, 0x1000_0000),
];

impl MemoryMap {
    /// Get all memory regions.
    pub const fn regions() -> &'static [MemRegion] {
        MEMORY_REGIONS
    }

    /// Find the region containing an address.
    pub fn find_region(addr: u32) -> Option<&'static MemRegion> {
        Self::regions().iter().find(|r| r.contains(addr))
    }

    /// Check if an address is in SRAM.
    pub const fn is_sram(addr: u32) -> bool {
        addr >= SRAM_BASE && addr < SRAM_BASE + SRAM_SIZE as u32
    }

    /// Check if an address is in Flash/XIP.
    pub const fn is_flash(addr: u32) -> bool {
        addr >= XIP_BASE && addr < XIP_BASE + 0x1000_0000
    }

    /// Check if an address is in Boot ROM.
    pub const fn is_bootrom(addr: u32) -> bool {
        addr >= BOOTROM_BASE && addr < BOOTROM_BASE + BOOTROM_SIZE as u32
    }

    /// Check if an address is in peripheral space.
    pub const fn is_peripheral(addr: u32) -> bool {
        (addr >= PERIPH_BASE && addr < PERIPH_BASE + 0x1000_0000) ||
        (addr >= APB_BASE && addr < APB_BASE + 0x1000_0000)
    }
}