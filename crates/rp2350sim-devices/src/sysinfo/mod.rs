//! Sysinfo (System Information) for RP2350.
//!
//! Provides system identification and configuration information.

use rp2350sim_core::{Device, DeviceId, Result};

/// Sysinfo base address.
pub const SYSINFO_BASE: u32 = 0x4000_8000;

/// Sysinfo register offsets.
pub mod regs {
    pub const CHIP_ID: u32 = 0x000;
    pub const PLATFORM: u32 = 0x004;
    pub const GITREF_RP2040: u32 = 0x008;
    pub const GITREF_RP2350: u32 = 0x00C;
    pub const GITREF_RP2350_RISCVCORE: u32 = 0x010;
    pub const GITREF_RP2350_BOOTROM: u32 = 0x014;
    pub const GITREF_RP2350_BOOTROM_RISCV: u32 = 0x018;
    pub const GITSPEC_RP2350_ARM_NS: u32 = 0x01C;
    pub const GITSPEC_RP2350_ARM_S: u32 = 0x020;
    pub const GITSPEC_RP2350_RISCV: u32 = 0x024;
    pub const PACKAGE: u32 = 0x028;
    pub const DEVICE_ID: u32 = 0x02C;
    pub const DEVICE_ID_RISCV: u32 = 0x030;
    pub const REFCLOCK_FREQ: u32 = 0x034;
    pub const USR_ACCESS: u32 = 0x038;
    pub const USR_ACCESS_RISCV: u32 = 0x03C;
    pub const JTAG_USERCODE: u32 = 0x040;
    pub const JTAG_USERCODE_RISCV: u32 = 0x044;
    pub const CHIP_INFO: u32 = 0x048;
    pub const CHIP_INFO_RISCV: u32 = 0x04C;
    pub const BOOTRAM_BASE: u32 = 0x050;
    pub const BOOTRAM_SIZE: u32 = 0x054;
    pub const XIP_SRAM_BASE: u32 = 0x058;
    pub const XIP_SRAM_SIZE: u32 = 0x05C;
    pub const SRAM_BASE: u32 = 0x060;
    pub const SRAM_SIZE: u32 = 0x064;
    pub const FLASH_BASE: u32 = 0x068;
    pub const FLASH_SIZE: u32 = 0x06C;
}

/// CHIP_ID values.
pub mod chip_id {
    pub const RP2350: u32 = 0x0000_2350;
    pub const REVISION_SHIFT: u32 = 0;
    pub const REVISION_MASK: u32 = 0x0F;
}

/// PLATFORM values.
pub mod platform {
    pub const ASIC: u32 = 0x0000_0001;
    pub const FPGA: u32 = 0x0000_0002;
    pub const SIMULATION: u32 = 0x0000_0003;
}

/// PACKAGE values.
pub mod package {
    pub const QFN_60: u32 = 0x0000_0001;
    pub const QFN_80: u32 = 0x0000_0002;
    pub const WLCSPI: u32 = 0x0000_0003;
}

/// Chip revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChipRevision {
    #[default]
    A0,
    A1,
    A2,
    B0,
    B1,
}

impl ChipRevision {
    pub fn value(&self) -> u32 {
        match self {
            Self::A0 => 0,
            Self::A1 => 1,
            Self::A2 => 2,
            Self::B0 => 3,
            Self::B1 => 4,
        }
    }
}

/// Sysinfo peripheral.
#[derive(Debug)]
pub struct Sysinfo {
    /// Chip ID.
    chip_id: u32,
    /// Platform type.
    platform: u32,
    /// Git references (simulated).
    gitrefs: [u32; 8],
    /// Package type.
    package: u32,
    /// Device ID (unique per chip).
    device_id: [u32; 2],
    /// Reference clock frequency.
    refclock_freq: u32,
    /// User access registers.
    usr_access: [u32; 2],
    /// JTAG user code.
    jtag_usercode: [u32; 2],
    /// Chip info.
    chip_info: [u32; 2],
    /// Memory configuration.
    bootram_base: u32,
    bootram_size: u32,
    xip_sram_base: u32,
    xip_sram_size: u32,
    sram_base: u32,
    sram_size: u32,
    flash_base: u32,
    flash_size: u32,
    /// Chip revision.
    revision: ChipRevision,
}

impl Default for Sysinfo {
    fn default() -> Self {
        Self::new()
    }
}

impl Sysinfo {
    /// Create a new Sysinfo instance.
    pub fn new() -> Self {
        Self {
            chip_id: chip_id::RP2350,
            platform: platform::SIMULATION,
            gitrefs: [
                0x12345678, // GITREF_RP2040
                0x87654321, // GITREF_RP2350
                0xDEADBEEF, // GITREF_RP2350_RISCVCORE
                0xCAFEBABE, // GITREF_RP2350_BOOTROM
                0xFEEDFACE, // GITREF_RP2350_BOOTROM_RISCV
                0xABCDEF01, // GITSPEC_RP2350_ARM_NS
                0x23456789, // GITSPEC_RP2350_ARM_S
                0x98765432, // GITSPEC_RP2350_RISCV
            ],
            package: package::QFN_80,
            device_id: [
                0x0123_4567, // Unique ID low
                0x89AB_CDEF, // Unique ID high
            ],
            refclock_freq: 12_000_000, // 12 MHz
            usr_access: [0, 0],
            jtag_usercode: [0x1234_5678, 0x8765_4321],
            chip_info: [0x0000_0001, 0x0000_0001],
            bootram_base: 0x4000_0000,
            bootram_size: 8 * 1024, // 8 KB
            xip_sram_base: 0x1500_0000,
            xip_sram_size: 64 * 1024, // 64 KB
            sram_base: 0x2000_0000,
            sram_size: 512 * 1024, // 512 KB
            flash_base: 0x1000_0000,
            flash_size: 4 * 1024 * 1024, // 4 MB
            revision: ChipRevision::default(),
        }
    }

    /// Get chip ID.
    pub fn get_chip_id(&self) -> u32 {
        self.chip_id | (self.revision.value() << chip_id::REVISION_SHIFT)
    }

    /// Get platform type.
    pub fn get_platform(&self) -> u32 {
        self.platform
    }

    /// Set platform type.
    pub fn set_platform(&mut self, platform: u32) {
        self.platform = platform;
    }

    /// Get device ID.
    pub fn get_device_id(&self) -> u64 {
        (self.device_id[1] as u64) << 32 | self.device_id[0] as u64
    }

    /// Set device ID.
    pub fn set_device_id(&mut self, id: u64) {
        self.device_id[0] = id as u32;
        self.device_id[1] = (id >> 32) as u32;
    }

    /// Get chip revision.
    pub fn get_revision(&self) -> ChipRevision {
        self.revision
    }

    /// Set chip revision.
    pub fn set_revision(&mut self, revision: ChipRevision) {
        self.revision = revision;
    }

    /// Get reference clock frequency.
    pub fn get_refclock_freq(&self) -> u32 {
        self.refclock_freq
    }

    /// Set reference clock frequency.
    pub fn set_refclock_freq(&mut self, freq: u32) {
        self.refclock_freq = freq;
    }

    /// Get memory configuration.
    pub fn get_memory_config(&self) -> (u32, u32, u32, u32, u32, u32, u32, u32) {
        (
            self.bootram_base, self.bootram_size,
            self.xip_sram_base, self.xip_sram_size,
            self.sram_base, self.sram_size,
            self.flash_base, self.flash_size,
        )
    }

    /// Set memory configuration.
    pub fn set_memory_config(
        &mut self,
        bootram_base: u32, bootram_size: u32,
        xip_sram_base: u32, xip_sram_size: u32,
        sram_base: u32, sram_size: u32,
        flash_base: u32, flash_size: u32,
    ) {
        self.bootram_base = bootram_base;
        self.bootram_size = bootram_size;
        self.xip_sram_base = xip_sram_base;
        self.xip_sram_size = xip_sram_size;
        self.sram_base = sram_base;
        self.sram_size = sram_size;
        self.flash_base = flash_base;
        self.flash_size = flash_size;
    }
}

impl Device for Sysinfo {
    fn id(&self) -> DeviceId {
        DeviceId::SYSINFO
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - SYSINFO_BASE;

        match offset {
            regs::CHIP_ID => Ok(self.get_chip_id()),
            regs::PLATFORM => Ok(self.platform),
            regs::GITREF_RP2040 => Ok(self.gitrefs[0]),
            regs::GITREF_RP2350 => Ok(self.gitrefs[1]),
            regs::GITREF_RP2350_RISCVCORE => Ok(self.gitrefs[2]),
            regs::GITREF_RP2350_BOOTROM => Ok(self.gitrefs[3]),
            regs::GITREF_RP2350_BOOTROM_RISCV => Ok(self.gitrefs[4]),
            regs::GITSPEC_RP2350_ARM_NS => Ok(self.gitrefs[5]),
            regs::GITSPEC_RP2350_ARM_S => Ok(self.gitrefs[6]),
            regs::GITSPEC_RP2350_RISCV => Ok(self.gitrefs[7]),
            regs::PACKAGE => Ok(self.package),
            regs::DEVICE_ID => Ok(self.device_id[0]),
            regs::DEVICE_ID_RISCV => Ok(self.device_id[1]),
            regs::REFCLOCK_FREQ => Ok(self.refclock_freq),
            regs::USR_ACCESS => Ok(self.usr_access[0]),
            regs::USR_ACCESS_RISCV => Ok(self.usr_access[1]),
            regs::JTAG_USERCODE => Ok(self.jtag_usercode[0]),
            regs::JTAG_USERCODE_RISCV => Ok(self.jtag_usercode[1]),
            regs::CHIP_INFO => Ok(self.chip_info[0]),
            regs::CHIP_INFO_RISCV => Ok(self.chip_info[1]),
            regs::BOOTRAM_BASE => Ok(self.bootram_base),
            regs::BOOTRAM_SIZE => Ok(self.bootram_size),
            regs::XIP_SRAM_BASE => Ok(self.xip_sram_base),
            regs::XIP_SRAM_SIZE => Ok(self.xip_sram_size),
            regs::SRAM_BASE => Ok(self.sram_base),
            regs::SRAM_SIZE => Ok(self.sram_size),
            regs::FLASH_BASE => Ok(self.flash_base),
            regs::FLASH_SIZE => Ok(self.flash_size),
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - SYSINFO_BASE;

        match offset {
            regs::USR_ACCESS => {
                self.usr_access[0] = value;
            }
            regs::USR_ACCESS_RISCV => {
                self.usr_access[1] = value;
            }
            _ => {
                // Most registers are read-only
            }
        }

        Ok(())
    }

    fn reset(&mut self) {
        // Preserve device ID across reset
        let device_id = self.device_id;
        *self = Self::new();
        self.device_id = device_id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SYSINFO_BASE: u32 = super::SYSINFO_BASE;

    #[test]
    fn test_sysinfo_creation() {
        let sysinfo = Sysinfo::new();

        assert_eq!(sysinfo.get_chip_id() & !0xF, chip_id::RP2350);
        assert_eq!(sysinfo.get_platform(), platform::SIMULATION);
    }

    #[test]
    fn test_sysinfo_default() {
        let sysinfo = Sysinfo::default();
        assert_eq!(sysinfo.platform, platform::SIMULATION);
    }

    #[test]
    fn test_chip_id() {
        let mut sysinfo = Sysinfo::new();
        let chip_id = sysinfo.read(SYSINFO_BASE + regs::CHIP_ID).unwrap();

        // Should be RP2350 with revision
        assert_eq!(chip_id & !0xF, chip_id::RP2350);
    }

    #[test]
    fn test_chip_revision() {
        let mut sysinfo = Sysinfo::new();

        sysinfo.set_revision(ChipRevision::B0);
        let chip_id = sysinfo.get_chip_id();
        assert_eq!(chip_id & chip_id::REVISION_MASK, 3);

        sysinfo.set_revision(ChipRevision::B1);
        let chip_id = sysinfo.get_chip_id();
        assert_eq!(chip_id & chip_id::REVISION_MASK, 4);
    }

    #[test]
    fn test_platform() {
        let mut sysinfo = Sysinfo::new();

        assert_eq!(sysinfo.get_platform(), platform::SIMULATION);

        sysinfo.set_platform(platform::ASIC);
        assert_eq!(sysinfo.get_platform(), platform::ASIC);

        sysinfo.set_platform(platform::FPGA);
        assert_eq!(sysinfo.get_platform(), platform::FPGA);
    }

    #[test]
    fn test_gitrefs() {
        let mut sysinfo = Sysinfo::new();

        // Read all git references
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::GITREF_RP2040).unwrap(), 0x12345678);
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::GITREF_RP2350).unwrap(), 0x87654321);
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::GITREF_RP2350_RISCVCORE).unwrap(), 0xDEADBEEF);
    }

    #[test]
    fn test_package() {
        let mut sysinfo = Sysinfo::new();

        let pkg = sysinfo.read(SYSINFO_BASE + regs::PACKAGE).unwrap();
        assert_eq!(pkg, package::QFN_80);
    }

    #[test]
    fn test_device_id() {
        let mut sysinfo = Sysinfo::new();

        // Default device ID
        let id = sysinfo.get_device_id();
        assert_eq!(id, 0x89AB_CDEF_0123_4567);

        // Set new device ID
        sysinfo.set_device_id(0x1122_3344_5566_7788);
        assert_eq!(sysinfo.get_device_id(), 0x1122_3344_5566_7788);

        // Read via registers
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::DEVICE_ID).unwrap(), 0x55667788);
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::DEVICE_ID_RISCV).unwrap(), 0x11223344);
    }

    #[test]
    fn test_refclock_freq() {
        let mut sysinfo = Sysinfo::new();

        assert_eq!(sysinfo.get_refclock_freq(), 12_000_000);

        sysinfo.set_refclock_freq(24_000_000);
        assert_eq!(sysinfo.get_refclock_freq(), 24_000_000);

        // Read via register
        let freq = sysinfo.read(SYSINFO_BASE + regs::REFCLOCK_FREQ).unwrap();
        assert_eq!(freq, 24_000_000);
    }

    #[test]
    fn test_usr_access() {
        let mut sysinfo = Sysinfo::new();

        // Write and read USR_ACCESS
        sysinfo.write(SYSINFO_BASE + regs::USR_ACCESS, 0x12345678).unwrap();
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::USR_ACCESS).unwrap(), 0x12345678);

        // USR_ACCESS_RISCV
        sysinfo.write(SYSINFO_BASE + regs::USR_ACCESS_RISCV, 0xABCDEF00).unwrap();
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::USR_ACCESS_RISCV).unwrap(), 0xABCDEF00);
    }

    #[test]
    fn test_jtag_usercode() {
        let mut sysinfo = Sysinfo::new();

        let code0 = sysinfo.read(SYSINFO_BASE + regs::JTAG_USERCODE).unwrap();
        let code1 = sysinfo.read(SYSINFO_BASE + regs::JTAG_USERCODE_RISCV).unwrap();

        assert_ne!(code0, 0);
        assert_ne!(code1, 0);
    }

    #[test]
    fn test_chip_info() {
        let mut sysinfo = Sysinfo::new();

        let info0 = sysinfo.read(SYSINFO_BASE + regs::CHIP_INFO).unwrap();
        let info1 = sysinfo.read(SYSINFO_BASE + regs::CHIP_INFO_RISCV).unwrap();

        assert_ne!(info0, 0);
        assert_ne!(info1, 0);
    }

    #[test]
    fn test_memory_config() {
        let sysinfo = Sysinfo::new();

        let (bootram_base, bootram_size, _xip_sram_base, _xip_sram_size, sram_base, sram_size, flash_base, flash_size) =
            sysinfo.get_memory_config();

        assert_eq!(bootram_base, 0x4000_0000);
        assert_eq!(bootram_size, 8 * 1024);
        assert_eq!(sram_base, 0x2000_0000);
        assert_eq!(sram_size, 512 * 1024);
        assert_eq!(flash_base, 0x1000_0000);
        assert_eq!(flash_size, 4 * 1024 * 1024);
    }

    #[test]
    fn test_memory_registers() {
        let mut sysinfo = Sysinfo::new();

        // Bootram
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::BOOTRAM_BASE).unwrap(), 0x4000_0000);
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::BOOTRAM_SIZE).unwrap(), 8 * 1024);

        // XIP SRAM
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::XIP_SRAM_BASE).unwrap(), 0x1500_0000);
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::XIP_SRAM_SIZE).unwrap(), 64 * 1024);

        // SRAM
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::SRAM_BASE).unwrap(), 0x2000_0000);
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::SRAM_SIZE).unwrap(), 512 * 1024);

        // Flash
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::FLASH_BASE).unwrap(), 0x1000_0000);
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::FLASH_SIZE).unwrap(), 4 * 1024 * 1024);
    }

    #[test]
    fn test_set_memory_config() {
        let mut sysinfo = Sysinfo::new();

        sysinfo.set_memory_config(
            0x5000_0000, 16 * 1024,  // bootram
            0x1600_0000, 128 * 1024, // xip_sram
            0x2100_0000, 1024 * 1024, // sram
            0x1100_0000, 8 * 1024 * 1024, // flash
        );

        let (bootram_base, bootram_size, _xip_sram_base, _xip_sram_size, sram_base, sram_size, _flash_base, _flash_size) =
            sysinfo.get_memory_config();

        assert_eq!(bootram_base, 0x5000_0000);
        assert_eq!(bootram_size, 16 * 1024);
        assert_eq!(sram_base, 0x2100_0000);
        assert_eq!(sram_size, 1024 * 1024);
    }

    #[test]
    fn test_read_only_registers() {
        let mut sysinfo = Sysinfo::new();

        // Try to write to read-only registers
        sysinfo.write(SYSINFO_BASE + regs::CHIP_ID, 0xFFFFFFFF).unwrap();
        sysinfo.write(SYSINFO_BASE + regs::PLATFORM, 0xFFFFFFFF).unwrap();
        sysinfo.write(SYSINFO_BASE + regs::PACKAGE, 0xFFFFFFFF).unwrap();

        // Values should not change
        assert_ne!(sysinfo.read(SYSINFO_BASE + regs::CHIP_ID).unwrap(), 0xFFFFFFFF);
        assert_ne!(sysinfo.read(SYSINFO_BASE + regs::PLATFORM).unwrap(), 0xFFFFFFFF);
        assert_ne!(sysinfo.read(SYSINFO_BASE + regs::PACKAGE).unwrap(), 0xFFFFFFFF);
    }

    #[test]
    fn test_invalid_register() {
        let mut sysinfo = Sysinfo::new();

        // Invalid offset
        let result = sysinfo.read(SYSINFO_BASE + 0x1000).unwrap();
        assert_eq!(result, 0);

        // Write to invalid offset should be ignored
        sysinfo.write(SYSINFO_BASE + 0x1000, 0x12345678).unwrap();
    }

    #[test]
    fn test_reset_preserves_device_id() {
        let mut sysinfo = Sysinfo::new();

        sysinfo.set_device_id(0xDEAD_BEEF_CAFE_BABE);
        sysinfo.set_revision(ChipRevision::B1);

        sysinfo.reset();

        // Device ID preserved
        assert_eq!(sysinfo.get_device_id(), 0xDEAD_BEEF_CAFE_BABE);
        // Revision reset to default
        assert_eq!(sysinfo.get_revision(), ChipRevision::A0);
    }

    #[test]
    fn test_sysinfo_device_id() {
        let sysinfo = Sysinfo::new();
        assert_eq!(sysinfo.id(), DeviceId::SYSINFO);
    }

    #[test]
    fn test_chip_revision_values() {
        assert_eq!(ChipRevision::A0.value(), 0);
        assert_eq!(ChipRevision::A1.value(), 1);
        assert_eq!(ChipRevision::A2.value(), 2);
        assert_eq!(ChipRevision::B0.value(), 3);
        assert_eq!(ChipRevision::B1.value(), 4);
    }

    #[test]
    fn test_all_gitspecs() {
        let mut sysinfo = Sysinfo::new();

        let arm_ns = sysinfo.read(SYSINFO_BASE + regs::GITSPEC_RP2350_ARM_NS).unwrap();
        let arm_s = sysinfo.read(SYSINFO_BASE + regs::GITSPEC_RP2350_ARM_S).unwrap();
        let riscv = sysinfo.read(SYSINFO_BASE + regs::GITSPEC_RP2350_RISCV).unwrap();

        assert_eq!(arm_ns, 0xABCDEF01);
        assert_eq!(arm_s, 0x23456789);
        assert_eq!(riscv, 0x98765432);
    }

    #[test]
    fn test_chip_revisions() {
        let mut sysinfo = Sysinfo::new();

        // Test all ChipRevision variants comprehensively
        let revisions = [
            (ChipRevision::A0, 0),
            (ChipRevision::A1, 1),
            (ChipRevision::A2, 2),
            (ChipRevision::B0, 3),
            (ChipRevision::B1, 4),
        ];

        for (rev, expected_val) in revisions {
            sysinfo.set_revision(rev);
            assert_eq!(sysinfo.get_revision(), rev);
            assert_eq!(rev.value(), expected_val);

            let chip_id = sysinfo.get_chip_id();
            assert_eq!(chip_id & chip_id::REVISION_MASK, expected_val);
            assert_eq!(chip_id & !0xF, chip_id::RP2350);
        }

        // Test Default trait
        assert_eq!(ChipRevision::default(), ChipRevision::A0);
    }

    #[test]
    fn test_device_id_64bit() {
        let mut sysinfo = Sysinfo::new();

        // Test maximum 64-bit device ID (edge case)
        let max_id = u64::MAX;
        sysinfo.set_device_id(max_id);
        assert_eq!(sysinfo.get_device_id(), max_id);
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::DEVICE_ID).unwrap(), 0xFFFFFFFF);
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::DEVICE_ID_RISCV).unwrap(), 0xFFFFFFFF);

        // Test zero device ID
        sysinfo.set_device_id(0);
        assert_eq!(sysinfo.get_device_id(), 0);

        // Test typical 64-bit ID with both halves populated
        let typical_id = 0x1234_5678_9ABC_DEF0_u64;
        sysinfo.set_device_id(typical_id);
        assert_eq!(sysinfo.get_device_id(), typical_id);
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::DEVICE_ID).unwrap(), 0x9ABC_DEF0);
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::DEVICE_ID_RISCV).unwrap(), 0x1234_5678);

        // Test bit pattern preservation
        let pattern_id = 0xAAAA_BBBB_CCCC_DDDD_u64;
        sysinfo.set_device_id(pattern_id);
        assert_eq!(sysinfo.get_device_id(), pattern_id);
    }

    #[test]
    fn test_platform_types() {
        let mut sysinfo = Sysinfo::new();

        // Test ASIC platform
        sysinfo.set_platform(platform::ASIC);
        assert_eq!(sysinfo.get_platform(), platform::ASIC);
        assert_eq!(sysinfo.get_platform(), 0x0000_0001);
        let read_val = sysinfo.read(SYSINFO_BASE + regs::PLATFORM).unwrap();
        assert_eq!(read_val, platform::ASIC);

        // Test FPGA platform
        sysinfo.set_platform(platform::FPGA);
        assert_eq!(sysinfo.get_platform(), platform::FPGA);
        assert_eq!(sysinfo.get_platform(), 0x0000_0002);
        let read_val = sysinfo.read(SYSINFO_BASE + regs::PLATFORM).unwrap();
        assert_eq!(read_val, platform::FPGA);

        // Test SIMULATION platform (default)
        sysinfo.set_platform(platform::SIMULATION);
        assert_eq!(sysinfo.get_platform(), platform::SIMULATION);
        assert_eq!(sysinfo.get_platform(), 0x0000_0003);
        let read_val = sysinfo.read(SYSINFO_BASE + regs::PLATFORM).unwrap();
        assert_eq!(read_val, platform::SIMULATION);

        // Test custom platform value
        sysinfo.set_platform(0x0000_00FF);
        assert_eq!(sysinfo.get_platform(), 0x0000_00FF);
    }

    #[test]
    fn test_gitref_registers() {
        let mut sysinfo = Sysinfo::new();

        // Verify all 8 gitref registers are readable
        let gitref_offsets = [
            (regs::GITREF_RP2040, 0x12345678_u32),
            (regs::GITREF_RP2350, 0x87654321),
            (regs::GITREF_RP2350_RISCVCORE, 0xDEADBEEF),
            (regs::GITREF_RP2350_BOOTROM, 0xCAFEBABE),
            (regs::GITREF_RP2350_BOOTROM_RISCV, 0xFEEDFACE),
            (regs::GITSPEC_RP2350_ARM_NS, 0xABCDEF01),
            (regs::GITSPEC_RP2350_ARM_S, 0x23456789),
            (regs::GITSPEC_RP2350_RISCV, 0x98765432),
        ];

        for (offset, expected) in gitref_offsets {
            let val = sysinfo.read(SYSINFO_BASE + offset);
            assert!(val.is_ok());
            assert_eq!(val.unwrap(), expected);
        }
    }

    #[test]
    fn test_device_read_only() {
        let mut sysinfo = Sysinfo::new();

        // Record original values
        let original_chip_id = sysinfo.read(SYSINFO_BASE + regs::CHIP_ID).unwrap();
        let original_platform = sysinfo.read(SYSINFO_BASE + regs::PLATFORM).unwrap();
        let original_package = sysinfo.read(SYSINFO_BASE + regs::PACKAGE).unwrap();
        let original_gitref = sysinfo.read(SYSINFO_BASE + regs::GITREF_RP2350).unwrap();
        let original_device_id = sysinfo.read(SYSINFO_BASE + regs::DEVICE_ID).unwrap();
        let original_refclock = sysinfo.read(SYSINFO_BASE + regs::REFCLOCK_FREQ).unwrap();

        // Attempt writes to read-only registers
        sysinfo.write(SYSINFO_BASE + regs::CHIP_ID, 0xAAAAAAAA).unwrap();
        sysinfo.write(SYSINFO_BASE + regs::PLATFORM, 0xBBBBBBBB).unwrap();
        sysinfo.write(SYSINFO_BASE + regs::PACKAGE, 0xCCCCCCCC).unwrap();
        sysinfo.write(SYSINFO_BASE + regs::GITREF_RP2350, 0xDDDDDDDD).unwrap();
        sysinfo.write(SYSINFO_BASE + regs::DEVICE_ID, 0xEEEEEEEE).unwrap();
        sysinfo.write(SYSINFO_BASE + regs::DEVICE_ID_RISCV, 0xFFFFFFFF).unwrap();
        sysinfo.write(SYSINFO_BASE + regs::REFCLOCK_FREQ, 0x12345678).unwrap();

        // Values should remain unchanged (writes ignored)
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::CHIP_ID).unwrap(), original_chip_id);
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::PLATFORM).unwrap(), original_platform);
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::PACKAGE).unwrap(), original_package);
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::GITREF_RP2350).unwrap(), original_gitref);
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::DEVICE_ID).unwrap(), original_device_id);
        assert_eq!(sysinfo.read(SYSINFO_BASE + regs::REFCLOCK_FREQ).unwrap(), original_refclock);

        // Memory config registers should also be read-only
        sysinfo.write(SYSINFO_BASE + regs::BOOTRAM_BASE, 0xFFFFFFFF).unwrap();
        sysinfo.write(SYSINFO_BASE + regs::BOOTRAM_SIZE, 0xFFFFFFFF).unwrap();
        sysinfo.write(SYSINFO_BASE + regs::SRAM_BASE, 0xFFFFFFFF).unwrap();
        sysinfo.write(SYSINFO_BASE + regs::SRAM_SIZE, 0xFFFFFFFF).unwrap();

        assert_ne!(sysinfo.read(SYSINFO_BASE + regs::BOOTRAM_BASE).unwrap(), 0xFFFFFFFF);
        assert_ne!(sysinfo.read(SYSINFO_BASE + regs::BOOTRAM_SIZE).unwrap(), 0xFFFFFFFF);
    }
}