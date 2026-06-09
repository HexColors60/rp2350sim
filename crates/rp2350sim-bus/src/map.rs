//! Memory map implementation.

use crate::{Region, RegionKind};
use rp2350sim_core::consts::*;
use std::collections::BTreeMap;

/// Memory map for the RP2350.
#[derive(Debug, Clone)]
pub struct MemoryMap {
    regions: BTreeMap<u32, Region>,
}

impl Default for MemoryMap {
    fn default() -> Self {
        Self::rp2350_default()
    }
}

impl MemoryMap {
    pub fn new() -> Self {
        Self {
            regions: BTreeMap::new(),
        }
    }

    /// Create the default RP2350 memory map.
    pub fn rp2350_default() -> Self {
        let mut map = Self::new();

        // Boot ROM
        map.add_region(Region::new("Boot ROM", BOOTROM_BASE, BOOTROM_SIZE as u32, RegionKind::BootRom).readonly().executable());

        // Flash/XIP
        map.add_region(Region::new("Flash/XIP", XIP_BASE, 0x1000_0000, RegionKind::Flash).executable());

        // SRAM
        map.add_region(Region::new("SRAM", SRAM_BASE, SRAM_SIZE as u32, RegionKind::Sram).executable());

        // Peripheral regions
        map.add_region(Region::new("GPIO", GPIO_BASE, 0x1000, RegionKind::Gpio));
        map.add_region(Region::new("UART0", UART0_BASE, 0x1000, RegionKind::Peripheral));
        map.add_region(Region::new("UART1", UART1_BASE, 0x1000, RegionKind::Peripheral));
        map.add_region(Region::new("SPI0", SPI0_BASE, 0x1000, RegionKind::Peripheral));
        map.add_region(Region::new("SPI1", SPI1_BASE, 0x1000, RegionKind::Peripheral));
        map.add_region(Region::new("I2C0", I2C0_BASE, 0x1000, RegionKind::Peripheral));
        map.add_region(Region::new("I2C1", I2C1_BASE, 0x1000, RegionKind::Peripheral));
        map.add_region(Region::new("PWM", PWM_BASE, 0x1000, RegionKind::Peripheral));
        map.add_region(Region::new("ADC", ADC_BASE, 0x1000, RegionKind::Peripheral));
        map.add_region(Region::new("TIMER", TIMER_BASE, 0x1000, RegionKind::Timer));
        map.add_region(Region::new("WATCHDOG", WATCHDOG_BASE, 0x1000, RegionKind::Timer));
        map.add_region(Region::new("CLOCKS", CLOCKS_BASE, 0x1000, RegionKind::Timer));
        map.add_region(Region::new("RESETS", RESETS_BASE, 0x1000, RegionKind::Timer));

        // USB
        map.add_region(Region::new("USB", USB_BASE, 0x1000, RegionKind::Usb));

        // PIO
        map.add_region(Region::new("PIO0", PIO0_BASE, 0x1000, RegionKind::Pio));
        map.add_region(Region::new("PIO1", PIO1_BASE, 0x1000, RegionKind::Pio));

        map
    }

    /// Add a region to the map.
    pub fn add_region(&mut self, region: Region) {
        self.regions.insert(region.base, region);
    }

    /// Remove a region from the map.
    pub fn remove_region(&mut self, base: u32) -> Option<Region> {
        self.regions.remove(&base)
    }

    /// Find the region containing an address.
    pub fn find_region(&self, addr: u32) -> Option<&Region> {
        // Find the region with the largest base address <= addr
        let iter = self.regions.iter().rev();
        for (_, region) in iter {
            if region.contains(addr) {
                return Some(region);
            }
            if region.base < addr {
                break;
            }
        }
        None
    }

    /// Find the region containing an address (mutable).
    pub fn find_region_mut(&mut self, addr: u32) -> Option<&mut Region> {
        let iter = self.regions.iter_mut().rev();
        for (_, region) in iter {
            if region.contains(addr) {
                return Some(region);
            }
            if region.base < addr {
                break;
            }
        }
        None
    }

    /// Check if an address is mapped.
    pub fn is_mapped(&self, addr: u32) -> bool {
        self.find_region(addr).is_some()
    }

    /// Get all regions.
    pub fn regions(&self) -> impl Iterator<Item = &Region> {
        self.regions.values()
    }

    /// Get the number of regions.
    pub fn len(&self) -> usize {
        self.regions.len()
    }

    /// Check if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }

    /// Clear all regions.
    pub fn clear(&mut self) {
        self.regions.clear();
    }
}