//! SysTick Timer for RP2350.
//!
//! Implements the ARM SysTick timer for both cores.

use rp2350sim_core::{Device, DeviceId, Result};

/// SysTick base addresses (per core).
pub const SYSTICK_BASE_CORE0: u32 = 0xE000_E010;
pub const SYSTICK_BASE_CORE1: u32 = 0xE002_E010;

/// SysTick register offsets.
pub mod regs {
    pub const CSR: u32 = 0x000;   // Control and Status Register
    pub const RVR: u32 = 0x004;   // Reload Value Register
    pub const CVR: u32 = 0x008;   // Current Value Register
    pub const CALIB: u32 = 0x00C; // Calibration Value Register
}

/// CSR register bits.
pub mod csr {
    pub const ENABLE: u32 = 1 << 0;
    pub const TICKINT: u32 = 1 << 1;
    pub const CLKSOURCE: u32 = 1 << 2;
    pub const COUNTFLAG: u32 = 1 << 16;
}

/// CALIB register bits.
pub mod calib {
    pub const TENMS_SHIFT: u32 = 0;
    pub const TENMS_MASK: u32 = 0x00FF_FFFF;
    pub const SKEW: u32 = 1 << 30;
    pub const NOREF: u32 = 1 << 31;
}

/// SysTick timer for a single core.
#[derive(Debug, Clone)]
pub struct SystickCore {
    /// Control and Status Register.
    csr: u32,
    /// Reload Value Register.
    rvr: u32,
    /// Current Value Register.
    cvr: u32,
    /// Calibration Value Register.
    calib: u32,
    /// Counter value (internal).
    #[allow(dead_code)]
    counter: u32,
    /// Interrupt pending.
    interrupt_pending: bool,
}

impl Default for SystickCore {
    fn default() -> Self {
        Self::new()
    }
}

impl SystickCore {
    /// Create a new SysTick core.
    pub fn new() -> Self {
        Self {
            csr: 0,
            rvr: 0,
            cvr: 0,
            calib: (12_000_000 / 100) & calib::TENMS_MASK, // 10ms at 12MHz
            counter: 0,
            interrupt_pending: false,
        }
    }

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        (self.csr & csr::ENABLE) != 0
    }

    /// Check if interrupt enabled.
    pub fn is_interrupt_enabled(&self) -> bool {
        (self.csr & csr::TICKINT) != 0
    }

    /// Check clock source (true = CPU clock, false = external).
    pub fn get_clock_source(&self) -> bool {
        (self.csr & csr::CLKSOURCE) != 0
    }

    /// Get reload value.
    pub fn get_reload(&self) -> u32 {
        self.rvr & 0x00FF_FFFF
    }

    /// Get current value.
    pub fn get_current(&self) -> u32 {
        self.cvr & 0x00FF_FFFF
    }

    /// Check if interrupt pending.
    pub fn is_interrupt_pending(&self) -> bool {
        self.interrupt_pending
    }

    /// Clear interrupt.
    pub fn clear_interrupt(&mut self) {
        self.interrupt_pending = false;
    }

    /// Tick the timer.
    pub fn tick(&mut self) -> bool {
        if !self.is_enabled() {
            return false;
        }

        // Decrement counter
        if self.cvr > 0 {
            self.cvr -= 1;
        } else {
            // Reload
            self.cvr = self.get_reload();
            
            // Set count flag
            self.csr |= csr::COUNTFLAG;
            
            // Trigger interrupt if enabled
            if self.is_interrupt_enabled() {
                self.interrupt_pending = true;
            }
            
            return true; // Wrapped
        }

        false
    }

    /// Read CSR.
    fn read_csr(&mut self) -> u32 {
        let csr = self.csr;
        // COUNTFLAG clears on read
        self.csr &= !csr::COUNTFLAG;
        csr
    }

    /// Write CSR.
    fn write_csr(&mut self, value: u32) {
        self.csr = (self.csr & csr::COUNTFLAG) | (value & 0x0001_0007);
        
        // If disabling, clear counter
        if (value & csr::ENABLE) == 0 {
            self.cvr = 0;
        }
    }

    /// Write RVR.
    fn write_rvr(&mut self, value: u32) {
        self.rvr = value & 0x00FF_FFFF;
    }

    /// Write CVR.
    fn write_cvr(&mut self, _value: u32) {
        // Writing any value clears to 0
        self.cvr = 0;
        self.csr &= !csr::COUNTFLAG;
    }

    /// Reset.
    fn reset(&mut self) {
        let calib = self.calib;
        *self = Self::new();
        self.calib = calib;
    }
}

/// SysTick timer peripheral (dual core).
#[derive(Debug)]
pub struct Systick {
    /// Core 0 SysTick.
    core0: SystickCore,
    /// Core 1 SysTick.
    core1: SystickCore,
}

impl Default for Systick {
    fn default() -> Self {
        Self::new()
    }
}

impl Systick {
    /// Create a new SysTick instance.
    pub fn new() -> Self {
        Self {
            core0: SystickCore::new(),
            core1: SystickCore::new(),
        }
    }

    /// Get core SysTick.
    pub fn get_core(&self, core: usize) -> &SystickCore {
        match core {
            0 => &self.core0,
            _ => &self.core1,
        }
    }

    /// Get mutable core SysTick.
    pub fn get_core_mut(&mut self, core: usize) -> &mut SystickCore {
        match core {
            0 => &mut self.core0,
            _ => &mut self.core1,
        }
    }

    /// Tick both cores.
    pub fn tick(&mut self) -> (bool, bool) {
        let core0_wrap = self.core0.tick();
        let core1_wrap = self.core1.tick();
        (core0_wrap, core1_wrap)
    }

    /// Check if interrupt pending for core.
    pub fn is_interrupt_pending(&self, core: usize) -> bool {
        self.get_core(core).is_interrupt_pending()
    }

    /// Clear interrupt for core.
    pub fn clear_interrupt(&mut self, core: usize) {
        self.get_core_mut(core).clear_interrupt();
    }

    /// Determine which core based on address.
    fn get_core_index(&self, addr: u32) -> usize {
        if addr >= SYSTICK_BASE_CORE1 {
            1
        } else {
            0
        }
    }

    /// Get base address for core.
    fn get_base(&self, core: usize) -> u32 {
        match core {
            0 => SYSTICK_BASE_CORE0,
            _ => SYSTICK_BASE_CORE1,
        }
    }
}

impl Device for Systick {
    fn id(&self) -> DeviceId {
        DeviceId::SYSTICK
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let core_idx = self.get_core_index(addr);
        let base = self.get_base(core_idx);
        let offset = addr - base;

        match core_idx {
            0 => {
                match offset {
                    regs::CSR => Ok(self.core0.read_csr()),
                    regs::RVR => Ok(self.core0.rvr),
                    regs::CVR => Ok(self.core0.cvr),
                    regs::CALIB => Ok(self.core0.calib),
                    _ => Ok(0),
                }
            }
            _ => {
                match offset {
                    regs::CSR => Ok(self.core1.read_csr()),
                    regs::RVR => Ok(self.core1.rvr),
                    regs::CVR => Ok(self.core1.cvr),
                    regs::CALIB => Ok(self.core1.calib),
                    _ => Ok(0),
                }
            }
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let core_idx = self.get_core_index(addr);
        let base = self.get_base(core_idx);
        let offset = addr - base;

        match core_idx {
            0 => {
                match offset {
                    regs::CSR => self.core0.write_csr(value),
                    regs::RVR => self.core0.write_rvr(value),
                    regs::CVR => self.core0.write_cvr(value),
                    regs::CALIB => {} // Read-only
                    _ => {}
                }
            }
            _ => {
                match offset {
                    regs::CSR => self.core1.write_csr(value),
                    regs::RVR => self.core1.write_rvr(value),
                    regs::CVR => self.core1.write_cvr(value),
                    regs::CALIB => {} // Read-only
                    _ => {}
                }
            }
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

    const BASE0: u32 = SYSTICK_BASE_CORE0;
    const BASE1: u32 = SYSTICK_BASE_CORE1;

    // ==================== SystickCore Tests ====================

    #[test]
    fn test_systick_core_creation() {
        let systick = SystickCore::new();

        assert!(!systick.is_enabled());
        assert!(!systick.is_interrupt_enabled());
        assert_eq!(systick.get_reload(), 0);
        assert_eq!(systick.get_current(), 0);
    }

    #[test]
    fn test_systick_core_default() {
        let systick = SystickCore::default();
        assert!(!systick.is_enabled());
    }

    #[test]
    fn test_systick_core_enable() {
        let mut systick = SystickCore::new();

        systick.write_csr(csr::ENABLE);
        assert!(systick.is_enabled());

        systick.write_csr(0);
        assert!(!systick.is_enabled());
    }

    #[test]
    fn test_systick_core_interrupt_enable() {
        let mut systick = SystickCore::new();

        systick.write_csr(csr::TICKINT);
        assert!(systick.is_interrupt_enabled());
    }

    #[test]
    fn test_systick_core_clock_source() {
        let mut systick = SystickCore::new();

        systick.write_csr(csr::CLKSOURCE);
        assert!(systick.get_clock_source());

        systick.write_csr(0);
        assert!(!systick.get_clock_source());
    }

    #[test]
    fn test_systick_core_reload_value() {
        let mut systick = SystickCore::new();

        systick.write_rvr(0x123456);
        assert_eq!(systick.get_reload(), 0x123456);

        // Only 24 bits valid
        systick.write_rvr(0x1FFFFFF);
        assert_eq!(systick.get_reload(), 0x00FFFFFF);
    }

    #[test]
    fn test_systick_core_current_value() {
        let mut systick = SystickCore::new();

        // Writing CVR clears it to 0
        systick.write_cvr(0x123456);
        assert_eq!(systick.get_current(), 0);
    }

    #[test]
    fn test_systick_core_tick_disabled() {
        let mut systick = SystickCore::new();

        // Disabled - tick does nothing
        assert!(!systick.tick());
        assert_eq!(systick.get_current(), 0);
    }

    #[test]
    fn test_systick_core_tick_enabled() {
        let mut systick = SystickCore::new();

        systick.write_rvr(10);
        systick.write_csr(csr::ENABLE);

        // First tick after enable (CVR is 0, so reloads)
        assert!(systick.tick());
        assert_eq!(systick.get_current(), 10);
    }

    #[test]
    fn test_systick_core_count_down() {
        let mut systick = SystickCore::new();

        systick.write_rvr(5);
        systick.write_csr(csr::ENABLE);

        // First tick reloads
        assert!(systick.tick());
        assert_eq!(systick.get_current(), 5);

        // Count down
        assert!(!systick.tick());
        assert_eq!(systick.get_current(), 4);

        assert!(!systick.tick());
        assert_eq!(systick.get_current(), 3);
    }

    #[test]
    fn test_systick_core_wrap() {
        let mut systick = SystickCore::new();

        systick.write_rvr(3);
        systick.write_csr(csr::ENABLE);

        // First tick: CVR 0 -> reload to 3
        assert!(systick.tick());
        assert_eq!(systick.get_current(), 3);

        // Count down
        assert!(!systick.tick()); // 3 -> 2
        assert!(!systick.tick()); // 2 -> 1
        assert!(!systick.tick()); // 1 -> 0

        // Wrap: 0 -> reload
        assert!(systick.tick());
        assert_eq!(systick.get_current(), 3);
    }

    #[test]
    fn test_systick_core_countflag() {
        let mut systick = SystickCore::new();

        systick.write_rvr(2);
        systick.write_csr(csr::ENABLE);

        // Tick to trigger reload
        systick.tick();

        // COUNTFLAG should be set
        let csr = systick.read_csr();
        assert_eq!(csr & csr::COUNTFLAG, csr::COUNTFLAG);

        // Read again - COUNTFLAG cleared
        let csr2 = systick.read_csr();
        assert_eq!(csr2 & csr::COUNTFLAG, 0);
    }

    #[test]
    fn test_systick_core_interrupt() {
        let mut systick = SystickCore::new();

        systick.write_rvr(2);
        systick.write_csr(csr::ENABLE | csr::TICKINT);

        assert!(!systick.is_interrupt_pending());

        // Tick triggers reload
        systick.tick();
        assert!(systick.is_interrupt_pending());

        // Clear interrupt
        systick.clear_interrupt();
        assert!(!systick.is_interrupt_pending());
    }

    #[test]
    fn test_systick_core_no_interrupt_without_tickint() {
        let mut systick = SystickCore::new();

        systick.write_rvr(2);
        systick.write_csr(csr::ENABLE); // No TICKINT

        systick.tick();
        assert!(!systick.is_interrupt_pending());
    }

    #[test]
    fn test_systick_core_reset() {
        let mut systick = SystickCore::new();

        systick.write_rvr(100);
        systick.write_csr(csr::ENABLE | csr::TICKINT);
        systick.reset();

        assert!(!systick.is_enabled());
        assert_eq!(systick.get_reload(), 0);
        // Calibration preserved
        assert_ne!(systick.calib, 0);
    }

    #[test]
    fn test_systick_core_disable_clears_cvr() {
        let mut systick = SystickCore::new();

        systick.write_rvr(10);
        systick.write_csr(csr::ENABLE);
        systick.tick();
        assert!(systick.get_current() > 0);

        // Disable
        systick.write_csr(0);
        assert_eq!(systick.get_current(), 0);
    }

    // ==================== Systick (Dual Core) Tests ====================

    #[test]
    fn test_systick_creation() {
        let systick = Systick::new();

        assert!(!systick.is_interrupt_pending(0));
        assert!(!systick.is_interrupt_pending(1));
    }

    #[test]
    fn test_systick_default() {
        let systick = Systick::default();
        assert!(!systick.get_core(0).is_enabled());
        assert!(!systick.get_core(1).is_enabled());
    }

    #[test]
    fn test_systick_core_independence() {
        let mut systick = Systick::new();

        // Configure core 0
        systick.write(BASE0 + regs::RVR, 100).unwrap();
        systick.write(BASE0 + regs::CSR, csr::ENABLE).unwrap();

        // Configure core 1 differently
        systick.write(BASE1 + regs::RVR, 200).unwrap();
        systick.write(BASE1 + regs::CSR, csr::ENABLE).unwrap();

        assert_eq!(systick.get_core(0).get_reload(), 100);
        assert_eq!(systick.get_core(1).get_reload(), 200);
    }

    #[test]
    fn test_systick_read_csr() {
        let mut systick = Systick::new();

        systick.write(BASE0 + regs::CSR, csr::ENABLE).unwrap();
        let csr = systick.read(BASE0 + regs::CSR).unwrap();
        assert_eq!(csr & csr::ENABLE, csr::ENABLE);
    }

    #[test]
    fn test_systick_read_rvr() {
        let mut systick = Systick::new();

        systick.write(BASE0 + regs::RVR, 0x123456).unwrap();
        let rvr = systick.read(BASE0 + regs::RVR).unwrap();
        assert_eq!(rvr, 0x123456);
    }

    #[test]
    fn test_systick_read_cvr() {
        let mut systick = Systick::new();

        systick.write(BASE0 + regs::CVR, 0x123456).unwrap();
        let cvr = systick.read(BASE0 + regs::CVR).unwrap();
        // CVR should be 0 after write
        assert_eq!(cvr, 0);
    }

    #[test]
    fn test_systick_read_calib() {
        let mut systick = Systick::new();

        let calib = systick.read(BASE0 + regs::CALIB).unwrap();
        assert_ne!(calib, 0); // Should have calibration value
    }

    #[test]
    fn test_systick_calib_read_only() {
        let mut systick = Systick::new();

        let original = systick.read(BASE0 + regs::CALIB).unwrap();
        systick.write(BASE0 + regs::CALIB, 0xFFFFFFFF).unwrap();
        let after = systick.read(BASE0 + regs::CALIB).unwrap();

        assert_eq!(original, after); // Should not change
    }

    #[test]
    fn test_systick_tick_both_cores() {
        let mut systick = Systick::new();

        systick.write(BASE0 + regs::RVR, 5).unwrap();
        systick.write(BASE0 + regs::CSR, csr::ENABLE).unwrap();

        systick.write(BASE1 + regs::RVR, 10).unwrap();
        systick.write(BASE1 + regs::CSR, csr::ENABLE).unwrap();

        let (wrap0, wrap1) = systick.tick();

        assert!(wrap0); // Core 0 wrapped
        assert!(wrap1); // Core 1 wrapped
    }

    #[test]
    fn test_systick_clear_interrupt() {
        let mut systick = Systick::new();

        systick.write(BASE0 + regs::RVR, 2).unwrap();
        systick.write(BASE0 + regs::CSR, csr::ENABLE | csr::TICKINT).unwrap();

        systick.tick();
        assert!(systick.is_interrupt_pending(0));

        systick.clear_interrupt(0);
        assert!(!systick.is_interrupt_pending(0));
    }

    #[test]
    fn test_systick_reset() {
        let mut systick = Systick::new();

        systick.write(BASE0 + regs::RVR, 100).unwrap();
        systick.write(BASE0 + regs::CSR, csr::ENABLE).unwrap();

        systick.write(BASE1 + regs::RVR, 200).unwrap();
        systick.write(BASE1 + regs::CSR, csr::ENABLE).unwrap();

        systick.reset();

        assert!(!systick.get_core(0).is_enabled());
        assert!(!systick.get_core(1).is_enabled());
        assert_eq!(systick.get_core(0).get_reload(), 0);
        assert_eq!(systick.get_core(1).get_reload(), 0);
    }

    // ==================== Device Trait Tests ====================

    #[test]
    fn test_systick_device_id() {
        let systick = Systick::new();
        assert_eq!(systick.id(), DeviceId::SYSTICK);
    }

    #[test]
    fn test_systick_invalid_register() {
        let mut systick = Systick::new();

        // Invalid offset
        let result = systick.read(BASE0 + 0x100).unwrap();
        assert_eq!(result, 0);

        systick.write(BASE0 + 0x100, 0x12345678).unwrap();
    }

    #[test]
    fn test_systick_core1_registers() {
        let mut systick = Systick::new();

        // Test core 1 registers
        systick.write(BASE1 + regs::RVR, 0xABCDEF).unwrap();
        systick.write(BASE1 + regs::CSR, csr::ENABLE | csr::TICKINT).unwrap();

        assert_eq!(systick.read(BASE1 + regs::RVR).unwrap(), 0x00ABCDEF & 0x00FFFFFF);
        assert!(systick.get_core(1).is_enabled());
        assert!(systick.get_core(1).is_interrupt_enabled());
    }

    #[test]
    fn test_systick_get_core_mut() {
        let mut systick = Systick::new();

        let core0 = systick.get_core_mut(0);
        core0.write_rvr(50);
        core0.write_csr(csr::ENABLE);

        assert_eq!(systick.get_core(0).get_reload(), 50);
        assert!(systick.get_core(0).is_enabled());
    }

    #[test]
    fn test_systick_all_csr_flags() {
        let mut systick = Systick::new();

        let csr_val = csr::ENABLE | csr::TICKINT | csr::CLKSOURCE;
        systick.write(BASE0 + regs::CSR, csr_val).unwrap();

        let read_val = systick.read(BASE0 + regs::CSR).unwrap();
        assert_eq!(read_val & csr::ENABLE, csr::ENABLE);
        assert_eq!(read_val & csr::TICKINT, csr::TICKINT);
        assert_eq!(read_val & csr::CLKSOURCE, csr::CLKSOURCE);
    }
}