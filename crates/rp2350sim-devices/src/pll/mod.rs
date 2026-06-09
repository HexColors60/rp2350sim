//! PLL (Phase-Locked Loop) controller for RP2350.
//!
//! Implements the PLL peripherals for clock generation.

use rp2350sim_core::{Device, DeviceId, Result};

/// PLL base addresses.
pub const PLL_SYS_BASE: u32 = 0x5002_8000;
pub const PLL_USB_BASE: u32 = 0x5002_C000;

/// PLL register offsets.
pub mod regs {
    pub const CS: u32 = 0x000;
    pub const PWR: u32 = 0x004;
    pub const FBDIV_INT: u32 = 0x008;
    pub const FBDIV_FRAC: u32 = 0x00C;
    pub const PRIM: u32 = 0x010;
    pub const PRIM2: u32 = 0x014;
}

/// CS register bits.
pub mod cs {
    pub const REFDIV_SHIFT: u32 = 0;
    pub const REFDIV_MASK: u32 = 0x3F;
    pub const BYPASS: u32 = 1 << 8;
    pub const LOCK: u32 = 1 << 31;
}

/// PWR register bits.
pub mod pwr {
    pub const PD: u32 = 1 << 0;
    pub const DSMPD: u32 = 1 << 2;
    pub const POSTDIVPD: u32 = 1 << 3;
    pub const VCOPD: u32 = 1 << 4;
    pub const FRACPD: u32 = 1 << 5;
}

/// PRIM register bits.
pub mod prim {
    pub const POSTDIV1_SHIFT: u32 = 0;
    pub const POSTDIV1_MASK: u32 = 0x7;
    pub const POSTDIV2_SHIFT: u32 = 4;
    pub const POSTDIV2_MASK: u32 = 0x7 << 4;
}

/// PLL type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PllType {
    /// System PLL.
    Sys,
    /// USB PLL.
    Usb,
}

impl Default for PllType {
    fn default() -> Self {
        Self::Sys
    }
}

/// PLL configuration.
#[derive(Debug, Clone, Copy)]
pub struct PllConfig {
    /// Reference divider (1-63).
    pub refdiv: u8,
    /// Feedback divider (16-320).
    pub fbdiv: u16,
    /// Post divider 1 (1-7).
    pub postdiv1: u8,
    /// Post divider 2 (1-7).
    pub postdiv2: u8,
    /// Fractional feedback divider (0-255).
    pub fbdiv_frac: u8,
}

impl Default for PllConfig {
    fn default() -> Self {
        Self {
            refdiv: 1,
            fbdiv: 133,
            postdiv1: 6,
            postdiv2: 2,
            fbdiv_frac: 0,
        }
    }
}

/// PLL controller.
#[derive(Debug)]
pub struct Pll {
    /// PLL type.
    pub pll_type: PllType,
    /// Base address.
    base: u32,
    /// CS register.
    cs: u32,
    /// PWR register.
    pwr: u32,
    /// FBDIV_INT register.
    fbdiv_int: u32,
    /// FBDIV_FRAC register.
    fbdiv_frac: u32,
    /// PRIM register.
    prim: u32,
    /// PRIM2 register.
    prim2: u32,
    /// Reference clock frequency.
    ref_freq: u32,
    /// Output frequency.
    output_freq: u32,
    /// VCO frequency.
    vco_freq: u32,
    /// Lock status.
    locked: bool,
    /// Configuration.
    config: PllConfig,
}

impl Default for Pll {
    fn default() -> Self {
        Self::new(PllType::Sys)
    }
}

impl Pll {
    /// Create a new PLL controller.
    pub fn new(pll_type: PllType) -> Self {
        let base = match pll_type {
            PllType::Sys => PLL_SYS_BASE,
            PllType::Usb => PLL_USB_BASE,
        };

        let config = match pll_type {
            PllType::Sys => PllConfig {
                refdiv: 1,
                fbdiv: 133,
                postdiv1: 6,
                postdiv2: 2,
                fbdiv_frac: 0,
            },
            PllType::Usb => PllConfig {
                refdiv: 1,
                fbdiv: 40,
                postdiv1: 5,
                postdiv2: 2,
                fbdiv_frac: 0,
            },
        };

        Self {
            pll_type,
            base,
            cs: 1 << cs::REFDIV_SHIFT, // refdiv = 1
            pwr: pwr::PD,              // Powered down initially
            fbdiv_int: config.fbdiv as u32,
            fbdiv_frac: config.fbdiv_frac as u32,
            prim: (config.postdiv1 as u32) << prim::POSTDIV1_SHIFT
                | (config.postdiv2 as u32) << prim::POSTDIV2_SHIFT,
            prim2: 0,
            ref_freq: 12_000_000, // 12 MHz reference
            output_freq: 0,
            vco_freq: 0,
            locked: false,
            config,
        }
    }

    /// Get base address.
    pub fn base(&self) -> u32 {
        self.base
    }

    /// Check if PLL is powered.
    pub fn is_powered(&self) -> bool {
        (self.pwr & pwr::PD) == 0
    }

    /// Check if PLL is locked.
    pub fn is_locked(&self) -> bool {
        self.locked
    }

    /// Check if bypass is enabled.
    pub fn is_bypass(&self) -> bool {
        (self.cs & cs::BYPASS) != 0
    }

    /// Get reference divider.
    pub fn get_refdiv(&self) -> u8 {
        ((self.cs >> cs::REFDIV_SHIFT) & cs::REFDIV_MASK) as u8
    }

    /// Get feedback divider.
    pub fn get_fbdiv(&self) -> u16 {
        self.fbdiv_int as u16
    }

    /// Get post divider 1.
    pub fn get_postdiv1(&self) -> u8 {
        (self.prim & prim::POSTDIV1_MASK) as u8
    }

    /// Get post divider 2.
    pub fn get_postdiv2(&self) -> u8 {
        ((self.prim >> prim::POSTDIV2_SHIFT) & 0x7) as u8
    }

    /// Calculate output frequency.
    pub fn calculate_frequency(&mut self) {
        let refdiv = self.get_refdiv() as u32;
        let fbdiv = self.get_fbdiv() as u32;
        let postdiv1 = self.get_postdiv1() as u32;
        let postdiv2 = self.get_postdiv2() as u32;

        if refdiv == 0 || postdiv1 == 0 || postdiv2 == 0 {
            self.output_freq = 0;
            self.vco_freq = 0;
            return;
        }

        // VCO frequency = (ref_freq / refdiv) * fbdiv
        self.vco_freq = (self.ref_freq / refdiv) * fbdiv;

        // Output frequency = VCO / (postdiv1 * postdiv2)
        self.output_freq = self.vco_freq / (postdiv1 * postdiv2);
    }

    /// Get output frequency.
    pub fn get_output_freq(&self) -> u32 {
        self.output_freq
    }

    /// Get VCO frequency.
    pub fn get_vco_freq(&self) -> u32 {
        self.vco_freq
    }

    /// Set reference clock frequency.
    pub fn set_ref_freq(&mut self, freq: u32) {
        self.ref_freq = freq;
        self.calculate_frequency();
    }

    /// Update lock status.
    fn update_lock(&mut self) {
        if self.is_powered() && !self.is_bypass() {
            // Simulate lock time (in real hardware, this takes time)
            self.locked = true;
            self.calculate_frequency();
        } else {
            self.locked = false;
        }
    }

    /// Tick the PLL (simulate lock acquisition).
    pub fn tick(&mut self) {
        if self.is_powered() && !self.is_bypass() && !self.locked {
            self.update_lock();
        }
    }
}

impl Device for Pll {
    fn id(&self) -> DeviceId {
        DeviceId::PLL
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - self.base;

        match offset {
            regs::CS => {
                // Update lock bit
                let mut cs = self.cs;
                if self.locked {
                    cs |= cs::LOCK;
                } else {
                    cs &= !cs::LOCK;
                }
                Ok(cs)
            }
            regs::PWR => Ok(self.pwr),
            regs::FBDIV_INT => Ok(self.fbdiv_int),
            regs::FBDIV_FRAC => Ok(self.fbdiv_frac),
            regs::PRIM => Ok(self.prim),
            regs::PRIM2 => Ok(self.prim2),
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - self.base;

        match offset {
            regs::CS => {
                self.cs = value & !(cs::LOCK); // Lock bit is read-only
                self.config.refdiv = ((value >> cs::REFDIV_SHIFT) & cs::REFDIV_MASK) as u8;
            }
            regs::PWR => {
                self.pwr = value;
                self.update_lock();
            }
            regs::FBDIV_INT => {
                self.fbdiv_int = value & 0x1FF; // 9 bits
                self.config.fbdiv = self.fbdiv_int as u16;
                self.calculate_frequency();
            }
            regs::FBDIV_FRAC => {
                self.fbdiv_frac = value & 0xFF;
                self.config.fbdiv_frac = self.fbdiv_frac as u8;
            }
            regs::PRIM => {
                self.prim = value;
                self.config.postdiv1 = ((value >> prim::POSTDIV1_SHIFT) & prim::POSTDIV1_MASK) as u8;
                self.config.postdiv2 = ((value >> prim::POSTDIV2_SHIFT) & prim::POSTDIV2_MASK) as u8;
                self.calculate_frequency();
            }
            regs::PRIM2 => {
                self.prim2 = value;
            }
            _ => {}
        }

        Ok(())
    }

    fn reset(&mut self) {
        let pll_type = self.pll_type;
        *self = Self::new(pll_type);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pll_sys_creation() {
        let pll = Pll::new(PllType::Sys);
        assert_eq!(pll.pll_type, PllType::Sys);
        assert_eq!(pll.base(), PLL_SYS_BASE);
        assert!(!pll.is_powered()); // Powered down initially
        assert!(!pll.is_locked());
    }

    #[test]
    fn test_pll_usb_creation() {
        let pll = Pll::new(PllType::Usb);
        assert_eq!(pll.pll_type, PllType::Usb);
        assert_eq!(pll.base(), PLL_USB_BASE);
        assert!(!pll.is_powered());
    }

    #[test]
    fn test_pll_power_on() {
        let mut pll = Pll::new(PllType::Sys);

        // Power on PLL (clear PD bit)
        pll.write(PLL_SYS_BASE + regs::PWR, 0).unwrap();
        assert!(pll.is_powered());

        // Power off PLL
        pll.write(PLL_SYS_BASE + regs::PWR, pwr::PD).unwrap();
        assert!(!pll.is_powered());
    }

    #[test]
    fn test_pll_lock() {
        let mut pll = Pll::new(PllType::Sys);

        // Power on and tick to acquire lock
        pll.write(PLL_SYS_BASE + regs::PWR, 0).unwrap();
        pll.tick();

        assert!(pll.is_locked());

        // Check lock bit in CS register
        let cs = pll.read(PLL_SYS_BASE + regs::CS).unwrap();
        assert_eq!(cs & cs::LOCK, cs::LOCK);
    }

    #[test]
    fn test_pll_bypass() {
        let mut pll = Pll::new(PllType::Sys);

        // Enable bypass
        pll.write(PLL_SYS_BASE + regs::CS, cs::BYPASS).unwrap();
        assert!(pll.is_bypass());

        // Disable bypass
        pll.write(PLL_SYS_BASE + regs::CS, 0).unwrap();
        assert!(!pll.is_bypass());
    }

    #[test]
    fn test_pll_refdiv() {
        let mut pll = Pll::new(PllType::Sys);

        // Set reference divider
        pll.write(PLL_SYS_BASE + regs::CS, 2 << cs::REFDIV_SHIFT).unwrap();
        assert_eq!(pll.get_refdiv(), 2);

        // Read back
        let cs = pll.read(PLL_SYS_BASE + regs::CS).unwrap();
        assert_eq!((cs >> cs::REFDIV_SHIFT) & cs::REFDIV_MASK, 2);
    }

    #[test]
    fn test_pll_fbdiv() {
        let mut pll = Pll::new(PllType::Sys);

        // Set feedback divider
        pll.write(PLL_SYS_BASE + regs::FBDIV_INT, 150).unwrap();
        assert_eq!(pll.get_fbdiv(), 150);

        // Read back
        let fbdiv = pll.read(PLL_SYS_BASE + regs::FBDIV_INT).unwrap();
        assert_eq!(fbdiv, 150);
    }

    #[test]
    fn test_pll_postdiv() {
        let mut pll = Pll::new(PllType::Sys);

        // Set post dividers
        let prim = (3 << prim::POSTDIV1_SHIFT) | (4 << prim::POSTDIV2_SHIFT);
        pll.write(PLL_SYS_BASE + regs::PRIM, prim).unwrap();

        assert_eq!(pll.get_postdiv1(), 3);
        assert_eq!(pll.get_postdiv2(), 4);

        // Read back
        let prim_read = pll.read(PLL_SYS_BASE + regs::PRIM).unwrap();
        assert_eq!(prim_read, prim);
    }

    #[test]
    fn test_pll_frequency_calculation() {
        let mut pll = Pll::new(PllType::Sys);

        // Set reference frequency to 12 MHz
        pll.set_ref_freq(12_000_000);

        // Configure: refdiv=1, fbdiv=133, postdiv1=6, postdiv2=2
        pll.write(PLL_SYS_BASE + regs::CS, 1 << cs::REFDIV_SHIFT).unwrap();
        pll.write(PLL_SYS_BASE + regs::FBDIV_INT, 133).unwrap();
        pll.write(PLL_SYS_BASE + regs::PRIM, (6 << prim::POSTDIV1_SHIFT) | (2 << prim::POSTDIV2_SHIFT)).unwrap();

        // Calculate expected frequency:
        // VCO = (12 MHz / 1) * 133 = 1596 MHz
        // Output = 1596 MHz / (6 * 2) = 133 MHz
        pll.calculate_frequency();
        assert_eq!(pll.get_vco_freq(), 1_596_000_000);
        assert_eq!(pll.get_output_freq(), 133_000_000);
    }

    #[test]
    fn test_pll_fractional() {
        let mut pll = Pll::new(PllType::Sys);

        // Set fractional feedback divider
        pll.write(PLL_SYS_BASE + regs::FBDIV_FRAC, 128).unwrap();

        let frac = pll.read(PLL_SYS_BASE + regs::FBDIV_FRAC).unwrap();
        assert_eq!(frac, 128);
    }

    #[test]
    fn test_pll_power_modes() {
        let mut pll = Pll::new(PllType::Sys);

        // Test various power modes
        pll.write(PLL_SYS_BASE + regs::PWR, pwr::DSMPD).unwrap();
        assert_eq!(pll.read(PLL_SYS_BASE + regs::PWR).unwrap(), pwr::DSMPD);

        pll.write(PLL_SYS_BASE + regs::PWR, pwr::POSTDIVPD).unwrap();
        assert_eq!(pll.read(PLL_SYS_BASE + regs::PWR).unwrap(), pwr::POSTDIVPD);

        pll.write(PLL_SYS_BASE + regs::PWR, pwr::VCOPD).unwrap();
        assert_eq!(pll.read(PLL_SYS_BASE + regs::PWR).unwrap(), pwr::VCOPD);
    }

    #[test]
    fn test_pll_reset() {
        let mut pll = Pll::new(PllType::Sys);

        // Modify state
        pll.write(PLL_SYS_BASE + regs::PWR, 0).unwrap();
        pll.write(PLL_SYS_BASE + regs::FBDIV_INT, 200).unwrap();
        pll.write(PLL_SYS_BASE + regs::CS, cs::BYPASS).unwrap();

        // Reset
        pll.reset();

        // Check state is reset
        assert!(!pll.is_powered());
        assert!(!pll.is_locked());
        assert!(!pll.is_bypass());
        assert_eq!(pll.get_fbdiv(), 133); // Default for SYS PLL
    }
}