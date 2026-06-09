//! XIP (Execute In Place) mapping.

use rp2350sim_core::consts::XIP_BASE;

/// XIP controller state.
#[derive(Debug, Default)]
pub struct XipMap {
    enabled: bool,
    cache_enabled: bool,
}

impl XipMap {
    pub fn new() -> Self {
        Self {
            enabled: true,
            cache_enabled: true,
        }
    }

    /// Check if XIP is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable/disable XIP.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if cache is enabled.
    pub fn is_cache_enabled(&self) -> bool {
        self.cache_enabled
    }

    /// Enable/disable cache.
    pub fn set_cache_enabled(&mut self, enabled: bool) {
        self.cache_enabled = enabled;
    }

    /// Convert a flash address to XIP address.
    pub fn flash_to_xip(flash_addr: u32) -> u32 {
        XIP_BASE + flash_addr
    }

    /// Convert an XIP address to flash address.
    pub fn xip_to_flash(xip_addr: u32) -> Option<u32> {
        if xip_addr >= XIP_BASE && xip_addr < XIP_BASE + 0x1000_0000 {
            Some(xip_addr - XIP_BASE)
        } else {
            None
        }
    }

    /// Check if an address is in XIP range.
    pub fn is_xip_addr(addr: u32) -> bool {
        addr >= XIP_BASE && addr < XIP_BASE + 0x1000_0000
    }
}