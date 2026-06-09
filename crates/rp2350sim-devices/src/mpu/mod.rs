//! MPU (Memory Protection Unit) for RP2350.
//!
//! Implements the ARM v8-M Memory Protection Unit.

use rp2350sim_core::{Device, DeviceId, Result};

/// MPU base addresses (per core).
pub const MPU_BASE_CORE0: u32 = 0xE000_ED90;
pub const MPU_BASE_CORE1: u32 = 0xE002_ED90;

/// MPU register offsets.
pub mod regs {
    pub const TYPE: u32 = 0x000;
    pub const CTRL: u32 = 0x004;
    pub const RNR: u32 = 0x008;
    pub const RBAR: u32 = 0x00C;
    pub const RLAR: u32 = 0x010;
    pub const RBAR_A1: u32 = 0x014;
    pub const RLAR_A1: u32 = 0x018;
    pub const RBAR_A2: u32 = 0x01C;
    pub const RLAR_A2: u32 = 0x020;
    pub const RBAR_A3: u32 = 0x024;
    pub const RLAR_A3: u32 = 0x028;
    pub const MAIR0: u32 = 0x030;
    pub const MAIR1: u32 = 0x034;
}

/// MPU TYPE register bits.
pub mod mpu_type {
    pub const IREGION_SHIFT: u32 = 16;
    pub const IREGION_MASK: u32 = 0xFF << 16;
    pub const DREGION_SHIFT: u32 = 8;
    pub const DREGION_MASK: u32 = 0xFF << 8;
    pub const SEPARATE: u32 = 1 << 0;
}

/// MPU CTRL register bits.
pub mod mpu_ctrl {
    pub const PRIVDEFENA: u32 = 1 << 2;
    pub const HFNMIENA: u32 = 1 << 1;
    pub const ENABLE: u32 = 1 << 0;
}

/// MPU RBAR register bits.
pub mod mpu_rbar {
    pub const ADDR_SHIFT: u32 = 5;
    pub const ADDR_MASK: u32 = 0x7FFFF << 5;
    pub const SH_SHIFT: u32 = 3;
    pub const SH_MASK: u32 = 0x3 << 3;
    pub const AP_SHIFT: u32 = 1;
    pub const AP_MASK: u32 = 0x3 << 1;
    pub const XN: u32 = 1 << 0;
}

/// MPU RLAR register bits.
pub mod mpu_rlar {
    pub const ADDR_SHIFT: u32 = 5;
    pub const ADDR_MASK: u32 = 0x7FFFF << 5;
    pub const ATTRINDX_SHIFT: u32 = 1;
    pub const ATTRINDX_MASK: u32 = 0x7 << 1;
    pub const EN: u32 = 1 << 0;
}

/// Access permissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessPermission {
    /// No access.
    None = 0,
    /// Privileged read/write.
    PrivRW = 1,
    /// Privileged read/write, user read.
    PrivRWUserR = 2,
    /// Full read/write.
    FullRW = 3,
}

/// Memory attributes (MAIR).
#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryAttrs {
    /// Memory attribute byte.
    pub attrs: u8,
}

impl MemoryAttrs {
    /// Create new memory attributes.
    pub fn new(attrs: u8) -> Self {
        Self { attrs }
    }

    /// Check if device memory.
    pub fn is_device(&self) -> bool {
        (self.attrs & 0xF0) == 0x00
    }

    /// Check if normal memory.
    pub fn is_normal(&self) -> bool {
        (self.attrs & 0xC0) != 0x00
    }

    /// Check if cacheable.
    pub fn is_cacheable(&self) -> bool {
        self.is_normal() && (self.attrs & 0x0C) != 0x00
    }

    /// Check if shareable.
    pub fn is_shareable(&self) -> bool {
        (self.attrs & 0x10) != 0
    }
}

/// MPU region.
#[derive(Debug, Clone, Copy, Default)]
pub struct MpuRegion {
    /// Base address register.
    pub rbar: u32,
    /// Limit address register.
    pub rlar: u32,
}

impl MpuRegion {
    /// Create a new MPU region.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        (self.rlar & mpu_rlar::EN) != 0
    }

    /// Get base address.
    pub fn get_base(&self) -> u32 {
        (self.rbar & mpu_rbar::ADDR_MASK) << (32 - 20)
    }

    /// Get limit address.
    pub fn get_limit(&self) -> u32 {
        (self.rlar & mpu_rlar::ADDR_MASK) << (32 - 20) | 0x1F
    }

    /// Get attribute index.
    pub fn get_attr_index(&self) -> u32 {
        (self.rlar >> mpu_rlar::ATTRINDX_SHIFT) & 0x7
    }

    /// Get access permission.
    pub fn get_ap(&self) -> AccessPermission {
        match (self.rbar >> mpu_rbar::AP_SHIFT) & 0x3 {
            0 => AccessPermission::None,
            1 => AccessPermission::PrivRW,
            2 => AccessPermission::PrivRWUserR,
            _ => AccessPermission::FullRW,
        }
    }

    /// Check if executable.
    pub fn is_executable(&self) -> bool {
        (self.rbar & mpu_rbar::XN) == 0
    }

    /// Get shareability.
    pub fn get_shareability(&self) -> u32 {
        (self.rbar >> mpu_rbar::SH_SHIFT) & 0x3
    }

    /// Check if address is in region.
    pub fn contains(&self, addr: u32) -> bool {
        if !self.is_enabled() {
            return false;
        }
        let base = self.get_base();
        let limit = self.get_limit();
        addr >= base && addr <= limit
    }

    /// Check if access is permitted.
    pub fn check_access(&self, addr: u32, write: bool, privileged: bool) -> bool {
        if !self.contains(addr) {
            return false;
        }

        match self.get_ap() {
            AccessPermission::None => false,
            AccessPermission::PrivRW => privileged,
            AccessPermission::PrivRWUserR => privileged || !write,
            AccessPermission::FullRW => true,
        }
    }
}

/// MPU for a single core.
#[derive(Debug)]
pub struct MpuCore {
    /// Type register.
    mpu_type: u32,
    /// Control register.
    ctrl: u32,
    /// Region number register.
    rnr: u32,
    /// Regions (8 regions for v8-M).
    regions: [MpuRegion; 8],
    /// Memory attribute indirect registers.
    mair: [u32; 2],
}

impl Default for MpuCore {
    fn default() -> Self {
        Self::new()
    }
}

impl MpuCore {
    /// Create a new MPU core.
    pub fn new() -> Self {
        Self {
            mpu_type: 8 << mpu_type::DREGION_SHIFT, // 8 data regions
            ctrl: 0,
            rnr: 0,
            regions: [MpuRegion::new(); 8],
            mair: [0; 2],
        }
    }

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        (self.ctrl & mpu_ctrl::ENABLE) != 0
    }

    /// Check if privileged default enabled.
    pub fn is_privdef_enabled(&self) -> bool {
        (self.ctrl & mpu_ctrl::PRIVDEFENA) != 0
    }

    /// Get current region number.
    pub fn get_region_num(&self) -> usize {
        (self.rnr & 0x7) as usize
    }

    /// Get memory attributes for index.
    pub fn get_memory_attrs(&self, index: u32) -> MemoryAttrs {
        let mair_idx = (index / 4) as usize;
        let byte_idx = (index % 4) as usize;
        
        if mair_idx < 2 {
            let attrs = ((self.mair[mair_idx] >> (byte_idx * 8)) & 0xFF) as u8;
            MemoryAttrs::new(attrs)
        } else {
            MemoryAttrs::default()
        }
    }

    /// Check memory access.
    pub fn check_access(&self, addr: u32, write: bool, privileged: bool) -> bool {
        // If MPU is disabled, allow all access
        if !self.is_enabled() {
            return true;
        }

        // Check each enabled region
        for region in &self.regions {
            if region.is_enabled() && region.contains(addr) {
                return region.check_access(addr, write, privileged);
            }
        }

        // If no region matches and privileged default is enabled
        if self.is_privdef_enabled() && privileged {
            return true;
        }

        // Default: deny access
        false
    }

    /// Find region containing address.
    pub fn find_region(&self, addr: u32) -> Option<&MpuRegion> {
        for region in &self.regions {
            if region.is_enabled() && region.contains(addr) {
                return Some(region);
            }
        }
        None
    }

    /// Read register.
    fn read(&self, offset: u32) -> u32 {
        match offset {
            regs::TYPE => self.mpu_type,
            regs::CTRL => self.ctrl,
            regs::RNR => self.rnr,
            regs::RBAR => {
                let idx = self.get_region_num();
                self.regions[idx].rbar
            }
            regs::RLAR => {
                let idx = self.get_region_num();
                self.regions[idx].rlar
            }
            regs::RBAR_A1 => {
                let idx = self.get_region_num();
                if idx + 1 < 8 {
                    self.regions[idx + 1].rbar
                } else {
                    0
                }
            }
            regs::RLAR_A1 => {
                let idx = self.get_region_num();
                if idx + 1 < 8 {
                    self.regions[idx + 1].rlar
                } else {
                    0
                }
            }
            regs::RBAR_A2 => {
                let idx = self.get_region_num();
                if idx + 2 < 8 {
                    self.regions[idx + 2].rbar
                } else {
                    0
                }
            }
            regs::RLAR_A2 => {
                let idx = self.get_region_num();
                if idx + 2 < 8 {
                    self.regions[idx + 2].rlar
                } else {
                    0
                }
            }
            regs::RBAR_A3 => {
                let idx = self.get_region_num();
                if idx + 3 < 8 {
                    self.regions[idx + 3].rbar
                } else {
                    0
                }
            }
            regs::RLAR_A3 => {
                let idx = self.get_region_num();
                if idx + 3 < 8 {
                    self.regions[idx + 3].rlar
                } else {
                    0
                }
            }
            regs::MAIR0 => self.mair[0],
            regs::MAIR1 => self.mair[1],
            _ => 0,
        }
    }

    /// Write register.
    fn write(&mut self, offset: u32, value: u32) {
        match offset {
            regs::CTRL => {
                self.ctrl = value & 0x7;
            }
            regs::RNR => {
                self.rnr = value & 0x7;
            }
            regs::RBAR => {
                let idx = self.get_region_num();
                self.regions[idx].rbar = value;
            }
            regs::RLAR => {
                let idx = self.get_region_num();
                self.regions[idx].rlar = value;
            }
            regs::RBAR_A1 => {
                let idx = self.get_region_num();
                if idx + 1 < 8 {
                    self.regions[idx + 1].rbar = value;
                }
            }
            regs::RLAR_A1 => {
                let idx = self.get_region_num();
                if idx + 1 < 8 {
                    self.regions[idx + 1].rlar = value;
                }
            }
            regs::RBAR_A2 => {
                let idx = self.get_region_num();
                if idx + 2 < 8 {
                    self.regions[idx + 2].rbar = value;
                }
            }
            regs::RLAR_A2 => {
                let idx = self.get_region_num();
                if idx + 2 < 8 {
                    self.regions[idx + 2].rlar = value;
                }
            }
            regs::RBAR_A3 => {
                let idx = self.get_region_num();
                if idx + 3 < 8 {
                    self.regions[idx + 3].rbar = value;
                }
            }
            regs::RLAR_A3 => {
                let idx = self.get_region_num();
                if idx + 3 < 8 {
                    self.regions[idx + 3].rlar = value;
                }
            }
            regs::MAIR0 => self.mair[0] = value,
            regs::MAIR1 => self.mair[1] = value,
            _ => {}
        }
    }

    /// Reset.
    fn reset(&mut self) {
        let mpu_type = self.mpu_type;
        *self = Self::new();
        self.mpu_type = mpu_type;
    }
}

/// MPU peripheral (dual core).
#[derive(Debug)]
pub struct Mpu {
    /// Core 0 MPU.
    pub core0: MpuCore,
    /// Core 1 MPU.
    pub core1: MpuCore,
}

impl Default for Mpu {
    fn default() -> Self {
        Self::new()
    }
}

impl Mpu {
    /// Create a new MPU instance.
    pub fn new() -> Self {
        Self {
            core0: MpuCore::new(),
            core1: MpuCore::new(),
        }
    }

    /// Get core MPU.
    pub fn get_core(&self, core: usize) -> &MpuCore {
        match core {
            0 => &self.core0,
            _ => &self.core1,
        }
    }

    /// Get mutable core MPU.
    pub fn get_core_mut(&mut self, core: usize) -> &mut MpuCore {
        match core {
            0 => &mut self.core0,
            _ => &mut self.core1,
        }
    }

    /// Check memory access for core.
    pub fn check_access(&self, core: usize, addr: u32, write: bool, privileged: bool) -> bool {
        self.get_core(core).check_access(addr, write, privileged)
    }

    /// Determine core from address.
    fn get_core_index(&self, addr: u32) -> usize {
        if addr >= MPU_BASE_CORE1 {
            1
        } else {
            0
        }
    }

    /// Get base address for core.
    fn get_base(&self, core: usize) -> u32 {
        match core {
            0 => MPU_BASE_CORE0,
            _ => MPU_BASE_CORE1,
        }
    }
}

impl Device for Mpu {
    fn id(&self) -> DeviceId {
        DeviceId::MPU
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let core_idx = self.get_core_index(addr);
        let base = self.get_base(core_idx);
        let offset = addr - base;

        match core_idx {
            0 => Ok(self.core0.read(offset)),
            _ => Ok(self.core1.read(offset)),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let core_idx = self.get_core_index(addr);
        let base = self.get_base(core_idx);
        let offset = addr - base;

        match core_idx {
            0 => self.core0.write(offset, value),
            _ => self.core1.write(offset, value),
        }

        Ok(())
    }

    fn reset(&mut self) {
        self.core0.reset();
        self.core1.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE0: u32 = MPU_BASE_CORE0;
    const BASE1: u32 = MPU_BASE_CORE1;

    // ==================== MpuRegion Tests ====================

    #[test]
    fn test_mpu_region_creation() {
        let region = MpuRegion::new();
        assert!(!region.is_enabled());
        assert_eq!(region.rbar, 0);
        assert_eq!(region.rlar, 0);
    }

    #[test]
    fn test_mpu_region_default() {
        let region = MpuRegion::default();
        assert!(!region.is_enabled());
    }

    #[test]
    fn test_mpu_region_enable() {
        let mut region = MpuRegion::new();
        region.rlar = mpu_rlar::EN;
        assert!(region.is_enabled());
    }

    #[test]
    fn test_mpu_region_base_address() {
        let mut region = MpuRegion::new();
        region.rbar = 0x1000 << mpu_rbar::ADDR_SHIFT;
        let base = region.get_base();
        assert_eq!(base, 0x1000_0000);
    }

    #[test]
    fn test_mpu_region_limit_address() {
        let mut region = MpuRegion::new();
        region.rlar = 0x1100 << mpu_rlar::ADDR_SHIFT | mpu_rlar::EN;
        let limit = region.get_limit();
        assert!(limit > region.get_base());
    }

    #[test]
    fn test_mpu_region_contains() {
        let mut region = MpuRegion::new();
        region.rbar = 0x1000 << mpu_rbar::ADDR_SHIFT;
        region.rlar = 0x1100 << mpu_rlar::ADDR_SHIFT | mpu_rlar::EN;

        assert!(region.contains(0x1000_0000));
        assert!(region.contains(0x1050_0000));
        assert!(!region.contains(0x0000_0000));
    }

    #[test]
    fn test_mpu_region_contains_disabled() {
        let mut region = MpuRegion::new();
        region.rbar = 0x1000 << mpu_rbar::ADDR_SHIFT;
        region.rlar = 0x1100 << mpu_rlar::ADDR_SHIFT; // No EN bit

        assert!(!region.contains(0x1000_0000));
    }

    #[test]
    fn test_mpu_region_attr_index() {
        let mut region = MpuRegion::new();
        region.rlar = 3 << mpu_rlar::ATTRINDX_SHIFT;
        assert_eq!(region.get_attr_index(), 3);
    }

    #[test]
    fn test_mpu_region_access_permissions() {
        let mut region = MpuRegion::new();

        region.rbar = 0 << mpu_rbar::AP_SHIFT;
        assert_eq!(region.get_ap(), AccessPermission::None);

        region.rbar = 1 << mpu_rbar::AP_SHIFT;
        assert_eq!(region.get_ap(), AccessPermission::PrivRW);

        region.rbar = 2 << mpu_rbar::AP_SHIFT;
        assert_eq!(region.get_ap(), AccessPermission::PrivRWUserR);

        region.rbar = 3 << mpu_rbar::AP_SHIFT;
        assert_eq!(region.get_ap(), AccessPermission::FullRW);
    }

    #[test]
    fn test_mpu_region_executable() {
        let mut region = MpuRegion::new();

        assert!(region.is_executable()); // XN=0

        region.rbar = mpu_rbar::XN;
        assert!(!region.is_executable()); // XN=1
    }

    #[test]
    fn test_mpu_region_shareability() {
        let mut region = MpuRegion::new();

        region.rbar = 2 << mpu_rbar::SH_SHIFT;
        assert_eq!(region.get_shareability(), 2);
    }

    #[test]
    fn test_mpu_region_check_access_none() {
        let mut region = MpuRegion::new();
        region.rbar = 0x1000 << mpu_rbar::ADDR_SHIFT | (0 << mpu_rbar::AP_SHIFT);
        region.rlar = 0x1100 << mpu_rlar::ADDR_SHIFT | mpu_rlar::EN;

        assert!(!region.check_access(0x1000_0000, false, true));
        assert!(!region.check_access(0x1000_0000, true, true));
        assert!(!region.check_access(0x1000_0000, false, false));
    }

    #[test]
    fn test_mpu_region_check_access_priv_rw() {
        let mut region = MpuRegion::new();
        region.rbar = 0x1000 << mpu_rbar::ADDR_SHIFT | (1 << mpu_rbar::AP_SHIFT);
        region.rlar = 0x1100 << mpu_rlar::ADDR_SHIFT | mpu_rlar::EN;

        assert!(region.check_access(0x1000_0000, false, true));
        assert!(region.check_access(0x1000_0000, true, true));
        assert!(!region.check_access(0x1000_0000, false, false));
    }

    #[test]
    fn test_mpu_region_check_access_full_rw() {
        let mut region = MpuRegion::new();
        region.rbar = 0x1000 << mpu_rbar::ADDR_SHIFT | (3 << mpu_rbar::AP_SHIFT);
        region.rlar = 0x1100 << mpu_rlar::ADDR_SHIFT | mpu_rlar::EN;

        assert!(region.check_access(0x1000_0000, false, true));
        assert!(region.check_access(0x1000_0000, true, true));
        assert!(region.check_access(0x1000_0000, false, false));
        assert!(region.check_access(0x1000_0000, true, false));
    }

    // ==================== MemoryAttrs Tests ====================

    #[test]
    fn test_memory_attrs_creation() {
        let attrs = MemoryAttrs::new(0xFF);
        assert_eq!(attrs.attrs, 0xFF);
    }

    #[test]
    fn test_memory_attrs_default() {
        let attrs = MemoryAttrs::default();
        assert_eq!(attrs.attrs, 0);
    }

    #[test]
    fn test_memory_attrs_device() {
        let attrs = MemoryAttrs::new(0x00); // Device-nGnRnE
        assert!(attrs.is_device());
        assert!(!attrs.is_normal());
    }

    #[test]
    fn test_memory_attrs_normal() {
        let attrs = MemoryAttrs::new(0xFF); // Normal, WBWA, Shareable
        assert!(!attrs.is_device());
        assert!(attrs.is_normal());
    }

    #[test]
    fn test_memory_attrs_cacheable() {
        let attrs = MemoryAttrs::new(0xFF);
        assert!(attrs.is_cacheable());

        let attrs_nc = MemoryAttrs::new(0x40); // Normal, non-cacheable
        assert!(!attrs_nc.is_cacheable());
    }

    #[test]
    fn test_memory_attrs_shareable() {
        let attrs = MemoryAttrs::new(0x10);
        assert!(attrs.is_shareable());

        let attrs_ns = MemoryAttrs::new(0x00);
        assert!(!attrs_ns.is_shareable());
    }

    // ==================== MpuCore Tests ====================

    #[test]
    fn test_mpu_core_creation() {
        let mpu = MpuCore::new();

        assert!(!mpu.is_enabled());
        assert_eq!(mpu.get_region_num(), 0);

        // TYPE should indicate 8 data regions
        assert_eq!((mpu.mpu_type >> mpu_type::DREGION_SHIFT) & 0xFF, 8);
    }

    #[test]
    fn test_mpu_core_default() {
        let mpu = MpuCore::default();
        assert!(!mpu.is_enabled());
    }

    #[test]
    fn test_mpu_core_enable() {
        let mut mpu = MpuCore::new();
        mpu.ctrl = mpu_ctrl::ENABLE;
        assert!(mpu.is_enabled());
    }

    #[test]
    fn test_mpu_core_privdef() {
        let mut mpu = MpuCore::new();
        mpu.ctrl = mpu_ctrl::PRIVDEFENA;
        assert!(mpu.is_privdef_enabled());
    }

    #[test]
    fn test_mpu_core_region_num() {
        let mut mpu = MpuCore::new();

        mpu.rnr = 3;
        assert_eq!(mpu.get_region_num(), 3);

        mpu.rnr = 10; // Out of range, should be masked
        assert_eq!(mpu.get_region_num(), 2); // 10 & 0x7 = 2
    }

    #[test]
    fn test_mpu_core_check_access_disabled() {
        let mpu = MpuCore::new();
        // MPU disabled, all access allowed
        assert!(mpu.check_access(0x2000_0000, true, false));
        assert!(mpu.check_access(0x2000_0000, false, false));
    }

    #[test]
    fn test_mpu_core_check_access_enabled_no_region() {
        let mut mpu = MpuCore::new();
        mpu.ctrl = mpu_ctrl::ENABLE;

        // No regions configured, access denied
        assert!(!mpu.check_access(0x2000_0000, true, false));
    }

    #[test]
    fn test_mpu_core_check_access_privdef() {
        let mut mpu = MpuCore::new();
        mpu.ctrl = mpu_ctrl::ENABLE | mpu_ctrl::PRIVDEFENA;

        // No region match, but privdef enabled for privileged access
        assert!(mpu.check_access(0x2000_0000, true, true));
        assert!(!mpu.check_access(0x2000_0000, true, false));
    }

    #[test]
    fn test_mpu_core_find_region() {
        let mut mpu = MpuCore::new();

        // Configure region 0
        mpu.regions[0].rbar = 0x1000 << mpu_rbar::ADDR_SHIFT | (3 << mpu_rbar::AP_SHIFT);
        mpu.regions[0].rlar = 0x1100 << mpu_rlar::ADDR_SHIFT | mpu_rlar::EN;

        let region = mpu.find_region(0x1050_0000);
        assert!(region.is_some());

        let region = mpu.find_region(0x0000_0000);
        assert!(region.is_none());
    }

    #[test]
    fn test_mpu_core_get_memory_attrs() {
        let mut mpu = MpuCore::new();
        mpu.mair[0] = 0x12345678;

        // Index 0 = byte 0
        let attrs0 = mpu.get_memory_attrs(0);
        assert_eq!(attrs0.attrs, 0x78);

        // Index 1 = byte 1
        let attrs1 = mpu.get_memory_attrs(1);
        assert_eq!(attrs1.attrs, 0x56);
    }

    // ==================== MpuCore Register Tests ====================

    #[test]
    fn test_mpu_core_read_type() {
        let mpu = MpuCore::new();
        assert_eq!(mpu.read(regs::TYPE), mpu.mpu_type);
    }

    #[test]
    fn test_mpu_core_read_ctrl() {
        let mut mpu = MpuCore::new();
        mpu.ctrl = 0x5;
        assert_eq!(mpu.read(regs::CTRL), 0x5);
    }

    #[test]
    fn test_mpu_core_read_write_rnr() {
        let mut mpu = MpuCore::new();
        mpu.write(regs::RNR, 5);
        assert_eq!(mpu.read(regs::RNR), 5);
    }

    #[test]
    fn test_mpu_core_read_write_rbar() {
        let mut mpu = MpuCore::new();
        mpu.write(regs::RNR, 2);
        mpu.write(regs::RBAR, 0x12345678);
        assert_eq!(mpu.read(regs::RBAR), 0x12345678);
        assert_eq!(mpu.regions[2].rbar, 0x12345678);
    }

    #[test]
    fn test_mpu_core_read_write_rlar() {
        let mut mpu = MpuCore::new();
        mpu.write(regs::RNR, 1);
        mpu.write(regs::RLAR, 0xDEADBEEF);
        assert_eq!(mpu.read(regs::RLAR), 0xDEADBEEF);
        assert_eq!(mpu.regions[1].rlar, 0xDEADBEEF);
    }

    #[test]
    fn test_mpu_core_read_write_rbar_a1() {
        let mut mpu = MpuCore::new();
        mpu.write(regs::RNR, 0);
        mpu.write(regs::RBAR_A1, 0xAAAAAAAA);
        assert_eq!(mpu.regions[1].rbar, 0xAAAAAAAA);
    }

    #[test]
    fn test_mpu_core_read_write_mair() {
        let mut mpu = MpuCore::new();
        mpu.write(regs::MAIR0, 0x12345678);
        mpu.write(regs::MAIR1, 0xDEADBEEF);

        assert_eq!(mpu.read(regs::MAIR0), 0x12345678);
        assert_eq!(mpu.read(regs::MAIR1), 0xDEADBEEF);
    }

    #[test]
    fn test_mpu_core_reset() {
        let mut mpu = MpuCore::new();

        mpu.ctrl = 0x7;
        mpu.regions[0].rbar = 0xFFFFFFFF;
        mpu.mair[0] = 0xFFFFFFFF;

        mpu.reset();

        assert_eq!(mpu.ctrl, 0);
        assert_eq!(mpu.regions[0].rbar, 0);
        assert_eq!(mpu.mair[0], 0);
        // TYPE should be preserved
        assert_eq!((mpu.mpu_type >> mpu_type::DREGION_SHIFT) & 0xFF, 8);
    }

    // ==================== Mpu (Dual Core) Tests ====================

    #[test]
    fn test_mpu_creation() {
        let mpu = Mpu::new();
        assert!(!mpu.core0.is_enabled());
        assert!(!mpu.core1.is_enabled());
    }

    #[test]
    fn test_mpu_default() {
        let mpu = Mpu::default();
        assert!(!mpu.core0.is_enabled());
    }

    #[test]
    fn test_mpu_get_core() {
        let mpu = Mpu::new();
        assert!(!mpu.get_core(0).is_enabled());
        assert!(!mpu.get_core(1).is_enabled());
    }

    #[test]
    fn test_mpu_get_core_mut() {
        let mut mpu = Mpu::new();
        mpu.get_core_mut(0).ctrl = mpu_ctrl::ENABLE;
        assert!(mpu.core0.is_enabled());
        assert!(!mpu.core1.is_enabled());
    }

    #[test]
    fn test_mpu_core_independence() {
        let mut mpu = Mpu::new();

        // Configure core 0
        mpu.write(BASE0 + regs::CTRL, mpu_ctrl::ENABLE).unwrap();
        assert!(mpu.core0.is_enabled());
        assert!(!mpu.core1.is_enabled());

        // Configure core 1
        mpu.write(BASE1 + regs::CTRL, mpu_ctrl::ENABLE).unwrap();
        assert!(mpu.core1.is_enabled());
    }

    #[test]
    fn test_mpu_check_access() {
        let mut mpu = Mpu::new();

        // Configure region on core 0
        mpu.core0.ctrl = mpu_ctrl::ENABLE;
        mpu.core0.regions[0].rbar = 0x1000 << mpu_rbar::ADDR_SHIFT | (3 << mpu_rbar::AP_SHIFT);
        mpu.core0.regions[0].rlar = 0x1100 << mpu_rlar::ADDR_SHIFT | mpu_rlar::EN;

        assert!(mpu.check_access(0, 0x1000_0000, true, false));
        assert!(!mpu.check_access(0, 0x0000_0000, true, false));
    }

    #[test]
    fn test_mpu_device_id() {
        let mpu = Mpu::new();
        assert_eq!(mpu.id(), DeviceId::MPU);
    }

    #[test]
    fn test_mpu_read() {
        let mut mpu = Mpu::new();
        mpu.core0.ctrl = 0x5;
        assert_eq!(mpu.read(BASE0 + regs::CTRL).unwrap(), 0x5);
    }

    #[test]
    fn test_mpu_write() {
        let mut mpu = Mpu::new();
        mpu.write(BASE0 + regs::CTRL, 0x7).unwrap();
        assert_eq!(mpu.core0.ctrl, 0x7);
    }

    #[test]
    fn test_mpu_reset() {
        let mut mpu = Mpu::new();

        mpu.core0.ctrl = 0x7;
        mpu.core1.ctrl = 0x7;

        mpu.reset();

        assert!(!mpu.core0.is_enabled());
        assert!(!mpu.core1.is_enabled());
    }

    // ==================== AccessPermission Enum Tests ====================

    #[test]
    fn test_access_permission_values() {
        assert_eq!(AccessPermission::None as u8, 0);
        assert_eq!(AccessPermission::PrivRW as u8, 1);
        assert_eq!(AccessPermission::PrivRWUserR as u8, 2);
        assert_eq!(AccessPermission::FullRW as u8, 3);
    }

    #[test]
    fn test_access_permission_equality() {
        assert_eq!(AccessPermission::None, AccessPermission::None);
        assert_ne!(AccessPermission::None, AccessPermission::FullRW);
    }
}