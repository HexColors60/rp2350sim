//! RP2350 memory layout.

use rp2350sim_core::consts::*;

/// RP2350 memory layout.
pub struct Rp2350Layout;

impl Rp2350Layout {
    pub const SRAM_BASE: u32 = SRAM_BASE;
    pub const SRAM_SIZE: usize = SRAM_SIZE;
    pub const FLASH_BASE: u32 = XIP_BASE;
    pub const BOOTROM_BASE: u32 = BOOTROM_BASE;
}