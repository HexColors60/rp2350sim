//! Watchdog Timer device for RP2350.
//!
//! Implements the Watchdog Timer peripheral with multi-stage support.

use rp2350sim_core::{Device, DeviceId, Result};

/// Watchdog base address.
pub const WATCHDOG_BASE: u32 = 0x400D_8000;

/// Watchdog register offsets.
pub mod regs {
    pub const CTRL: u32 = 0x000;
    pub const LOAD: u32 = 0x004;
    pub const REASON: u32 = 0x008;
    pub const SCRATCH0: u32 = 0x00C;
    pub const SCRATCH1: u32 = 0x010;
    pub const SCRATCH2: u32 = 0x014;
    pub const SCRATCH3: u32 = 0x018;
    pub const SCRATCH4: u32 = 0x01C;
    pub const SCRATCH5: u32 = 0x020;
    pub const SCRATCH6: u32 = 0x024;
    pub const SCRATCH7: u32 = 0x028;
    pub const TICK: u32 = 0x02C;
}

/// Control register bits.
pub mod ctrl {
    pub const ENABLE: u32 = 1 << 0;
    pub const PAUSE_DBG0: u32 = 1 << 1;
    pub const PAUSE_DBG1: u32 = 1 << 2;
    pub const PAUSE_JTAG: u32 = 1 << 3;
    pub const PAUSE_CORE0: u32 = 1 << 4;
    pub const PAUSE_CORE1: u32 = 1 << 5;
}

/// Reason register bits.
pub mod reason {
    pub const FORCE: u32 = 1 << 0;
    pub const TIMER: u32 = 1 << 1;
}

/// Watchdog Timer device.
#[derive(Debug)]
pub struct Watchdog {
    /// Control register.
    ctrl: u32,
    /// Load value (timeout in watchdog ticks).
    load: u32,
    /// Current counter value.
    counter: u32,
    /// Reason for last reset.
    reason: u32,
    /// Scratch registers (8 registers).
    scratch: [u32; 8],
    /// Tick enable.
    tick_enable: bool,
    /// Tick cycles (how many clock cycles per watchdog tick).
    tick_cycles: u32,
    /// Tick counter.
    tick_counter: u32,
}

impl Default for Watchdog {
    fn default() -> Self {
        Self::new()
    }
}

impl Watchdog {
    /// Create a new Watchdog device.
    pub fn new() -> Self {
        Self {
            ctrl: 0,
            load: 0,
            counter: 0,
            reason: 0,
            scratch: [0; 8],
            tick_enable: false,
            tick_cycles: 1,
            tick_counter: 0,
        }
    }

    /// Check if watchdog is enabled.
    pub fn is_enabled(&self) -> bool {
        (self.ctrl & ctrl::ENABLE) != 0
    }

    /// Enable or disable the watchdog.
    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled {
            self.ctrl |= ctrl::ENABLE;
        } else {
            self.ctrl &= !ctrl::ENABLE;
        }
    }

    /// Get the load value.
    pub fn get_load(&self) -> u32 {
        self.load
    }

    /// Set the load value and reload counter.
    pub fn set_load(&mut self, value: u32) {
        self.load = value;
        self.counter = value;
    }

    /// Feed the watchdog (reload counter).
    pub fn feed(&mut self) {
        self.counter = self.load;
    }

    /// Get the current counter value.
    pub fn get_counter(&self) -> u32 {
        self.counter
    }

    /// Get the reason for last reset.
    pub fn get_reason(&self) -> u32 {
        self.reason
    }

    /// Set the reason for reset.
    pub fn set_reason(&mut self, reason: u32) {
        self.reason = reason;
    }

    /// Get scratch register value.
    pub fn get_scratch(&self, index: usize) -> u32 {
        if index < 8 {
            self.scratch[index]
        } else {
            0
        }
    }

    /// Set scratch register value.
    pub fn set_scratch(&mut self, index: usize, value: u32) {
        if index < 8 {
            self.scratch[index] = value;
        }
    }

    /// Tick the watchdog timer.
    /// Returns true if a watchdog timeout occurred.
    pub fn tick(&mut self) -> bool {
        if !self.is_enabled() {
            return false;
        }

        // Handle tick cycles
        if self.tick_cycles > 1 {
            self.tick_counter += 1;
            if self.tick_counter < self.tick_cycles {
                return false;
            }
            self.tick_counter = 0;
        }

        if self.counter > 0 {
            self.counter -= 1;
            if self.counter == 0 {
                // Watchdog timeout
                self.reason |= reason::TIMER;
                return true;
            }
        }
        false
    }

    /// Force a watchdog reset.
    pub fn force_reset(&mut self) {
        self.reason |= reason::FORCE;
    }

    /// Check if paused by debug.
    pub fn is_paused_debug(&self) -> bool {
        (self.ctrl & (ctrl::PAUSE_DBG0 | ctrl::PAUSE_DBG1 | ctrl::PAUSE_JTAG)) != 0
    }

    /// Check if paused by core.
    pub fn is_paused_core(&self) -> bool {
        (self.ctrl & (ctrl::PAUSE_CORE0 | ctrl::PAUSE_CORE1)) != 0
    }

    /// Reset the watchdog to default state.
    pub fn reset(&mut self) {
        self.ctrl = 0;
        self.load = 0;
        self.counter = 0;
        // Keep reason and scratch registers across reset
        self.tick_enable = false;
        self.tick_cycles = 1;
        self.tick_counter = 0;
    }

    /// Full reset including reason and scratch.
    pub fn full_reset(&mut self) {
        self.ctrl = 0;
        self.load = 0;
        self.counter = 0;
        self.reason = 0;
        self.scratch = [0; 8];
        self.tick_enable = false;
        self.tick_cycles = 1;
        self.tick_counter = 0;
    }
}

impl Device for Watchdog {
    fn id(&self) -> DeviceId {
        DeviceId::WATCHDOG
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - WATCHDOG_BASE;

        match offset {
            regs::CTRL => Ok(self.ctrl),
            regs::LOAD => Ok(self.load),
            regs::REASON => Ok(self.reason),
            regs::SCRATCH0 => Ok(self.scratch[0]),
            regs::SCRATCH1 => Ok(self.scratch[1]),
            regs::SCRATCH2 => Ok(self.scratch[2]),
            regs::SCRATCH3 => Ok(self.scratch[3]),
            regs::SCRATCH4 => Ok(self.scratch[4]),
            regs::SCRATCH5 => Ok(self.scratch[5]),
            regs::SCRATCH6 => Ok(self.scratch[6]),
            regs::SCRATCH7 => Ok(self.scratch[7]),
            regs::TICK => Ok(if self.tick_enable { 1 } else { 0 } | (self.tick_cycles << 1)),
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - WATCHDOG_BASE;

        match offset {
            regs::CTRL => {
                self.ctrl = value;
            }
            regs::LOAD => {
                self.load = value;
                self.counter = value;
            }
            regs::REASON => {
                // Write 1 to clear
                self.reason &= !value;
            }
            regs::SCRATCH0 => self.scratch[0] = value,
            regs::SCRATCH1 => self.scratch[1] = value,
            regs::SCRATCH2 => self.scratch[2] = value,
            regs::SCRATCH3 => self.scratch[3] = value,
            regs::SCRATCH4 => self.scratch[4] = value,
            regs::SCRATCH5 => self.scratch[5] = value,
            regs::SCRATCH6 => self.scratch[6] = value,
            regs::SCRATCH7 => self.scratch[7] = value,
            regs::TICK => {
                self.tick_enable = (value & 1) != 0;
                self.tick_cycles = (value >> 1) & 0x1FF;
                if self.tick_cycles == 0 {
                    self.tick_cycles = 1;
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn reset(&mut self) {
        self.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watchdog_creation() {
        let watchdog = Watchdog::new();
        assert!(!watchdog.is_enabled());
        assert_eq!(watchdog.get_load(), 0);
        assert_eq!(watchdog.get_counter(), 0);
    }

    #[test]
    fn test_watchdog_enable() {
        let mut watchdog = Watchdog::new();
        assert!(!watchdog.is_enabled());

        watchdog.set_enabled(true);
        assert!(watchdog.is_enabled());

        watchdog.set_enabled(false);
        assert!(!watchdog.is_enabled());
    }

    #[test]
    fn test_watchdog_tick_disabled() {
        let mut watchdog = Watchdog::new();
        watchdog.load = 10;
        watchdog.counter = 10;

        // When disabled, tick should return false and not decrement
        assert!(!watchdog.tick());
        assert_eq!(watchdog.get_counter(), 10);
    }

    #[test]
    fn test_watchdog_tick_enabled() {
        let mut watchdog = Watchdog::new();
        watchdog.set_enabled(true);
        watchdog.load = 10;
        watchdog.counter = 10;

        // Tick should decrement counter
        for _ in 0..9 {
            assert!(!watchdog.tick());
        }
        assert_eq!(watchdog.get_counter(), 1);

        // Final tick should return true (timeout)
        assert!(watchdog.tick());
        assert_eq!(watchdog.get_counter(), 0);
    }

    #[test]
    fn test_watchdog_feed() {
        let mut watchdog = Watchdog::new();
        watchdog.set_enabled(true);
        watchdog.load = 100;
        watchdog.counter = 50;

        // Feed should reload counter
        watchdog.feed();
        assert_eq!(watchdog.get_counter(), 100);
    }

    #[test]
    fn test_watchdog_reset() {
        let mut watchdog = Watchdog::new();
        watchdog.set_enabled(true);
        watchdog.load = 100;
        watchdog.counter = 50;

        // Reset should clear control, load, counter
        watchdog.reset();
        assert!(!watchdog.is_enabled());
        assert_eq!(watchdog.get_load(), 0);
        assert_eq!(watchdog.get_counter(), 0);
    }

    #[test]
    fn test_watchdog_timeout_sequence() {
        let mut watchdog = Watchdog::new();
        watchdog.set_enabled(true);
        watchdog.load = 5;
        watchdog.counter = 5;

        // Run until timeout
        for _ in 0..4 {
            assert!(!watchdog.tick());
        }
        assert!(watchdog.tick()); // Timeout

        // Check reason
        assert_eq!(watchdog.get_reason(), reason::TIMER);

        // Reload and run again
        watchdog.feed();
        assert_eq!(watchdog.get_counter(), 5);

        for _ in 0..4 {
            assert!(!watchdog.tick());
        }
        assert!(watchdog.tick()); // Timeout again
    }

    #[test]
    fn test_watchdog_scratch_registers() {
        let mut watchdog = Watchdog::new();

        // Write to all scratch registers
        for i in 0..8 {
            watchdog.set_scratch(i, 0xDEAD0000 | i as u32);
        }

        // Read back
        for i in 0..8 {
            assert_eq!(watchdog.get_scratch(i), 0xDEAD0000 | i as u32);
        }

        // Invalid index should return 0
        assert_eq!(watchdog.get_scratch(8), 0);
    }

    #[test]
    fn test_watchdog_scratch_preserved_across_reset() {
        let mut watchdog = Watchdog::new();

        // Set scratch values
        watchdog.set_scratch(0, 0x12345678);
        watchdog.set_scratch(7, 0xABCDEF00);

        // Reset
        watchdog.reset();

        // Scratch should be preserved
        assert_eq!(watchdog.get_scratch(0), 0x12345678);
        assert_eq!(watchdog.get_scratch(7), 0xABCDEF00);
    }

    #[test]
    fn test_watchdog_reason() {
        let mut watchdog = Watchdog::new();

        // Force reset
        watchdog.force_reset();
        assert_eq!(watchdog.get_reason(), reason::FORCE);

        // Clear reason by writing
        watchdog.reason = 0;

        // Timer timeout
        watchdog.set_enabled(true);
        watchdog.load = 1;
        watchdog.counter = 1;
        assert!(watchdog.tick());
        assert_eq!(watchdog.get_reason(), reason::TIMER);
    }

    #[test]
    fn test_watchdog_reason_clear() {
        let mut watchdog = Watchdog::new();
        watchdog.reason = 0x3; // Both bits set

        // Write 1 to clear
        watchdog.reason &= !0x2; // Clear TIMER bit
        assert_eq!(watchdog.get_reason(), 0x1);

        watchdog.reason &= !0x1; // Clear FORCE bit
        assert_eq!(watchdog.get_reason(), 0);
    }

    #[test]
    fn test_watchdog_tick_cycles() {
        let mut watchdog = Watchdog::new();
        watchdog.set_enabled(true);
        watchdog.load = 2;
        watchdog.counter = 2;
        watchdog.tick_cycles = 3; // 3 clock cycles per watchdog tick
        watchdog.tick_counter = 0;

        // First two clock ticks should not decrement counter (tick_counter goes 1, 2)
        assert!(!watchdog.tick());
        assert_eq!(watchdog.tick_counter, 1);
        assert_eq!(watchdog.get_counter(), 2);

        assert!(!watchdog.tick());
        assert_eq!(watchdog.tick_counter, 2);
        assert_eq!(watchdog.get_counter(), 2);

        // Third tick should decrement counter (tick_counter reaches 3, resets to 0)
        assert!(!watchdog.tick());
        assert_eq!(watchdog.tick_counter, 0);
        assert_eq!(watchdog.get_counter(), 1);

        // Continue for next decrement (3 more ticks)
        assert!(!watchdog.tick()); // tick_counter = 1
        assert!(!watchdog.tick()); // tick_counter = 2
        // Sixth tick: tick_counter reaches 3, counter goes from 1 to 0, returns true
        assert!(watchdog.tick()); // Timeout!
        assert_eq!(watchdog.get_counter(), 0);
    }

    #[test]
    fn test_watchdog_register_access() {
        let mut watchdog = Watchdog::new();

        // Test CTRL register
        watchdog.write(WATCHDOG_BASE + regs::CTRL, 0x1).unwrap();
        assert_eq!(watchdog.read(WATCHDOG_BASE + regs::CTRL).unwrap(), 0x1);
        assert!(watchdog.is_enabled());

        // Test LOAD register
        watchdog.write(WATCHDOG_BASE + regs::LOAD, 1000).unwrap();
        assert_eq!(watchdog.read(WATCHDOG_BASE + regs::LOAD).unwrap(), 1000);
        assert_eq!(watchdog.get_counter(), 1000);

        // Test SCRATCH registers
        watchdog.write(WATCHDOG_BASE + regs::SCRATCH0, 0xDEADBEEF).unwrap();
        assert_eq!(watchdog.read(WATCHDOG_BASE + regs::SCRATCH0).unwrap(), 0xDEADBEEF);

        // Test REASON register
        watchdog.reason = 0x3;
        assert_eq!(watchdog.read(WATCHDOG_BASE + regs::REASON).unwrap(), 0x3);
        watchdog.write(WATCHDOG_BASE + regs::REASON, 0x1).unwrap(); // Clear bit 0
        assert_eq!(watchdog.read(WATCHDOG_BASE + regs::REASON).unwrap(), 0x2);
    }

    #[test]
    fn test_watchdog_ctrl_pause_bits() {
        let mut watchdog = Watchdog::new();

        // Set pause bits
        watchdog.ctrl = ctrl::PAUSE_DBG0 | ctrl::PAUSE_CORE0;
        assert!(watchdog.is_paused_debug());
        assert!(watchdog.is_paused_core());

        watchdog.ctrl = 0;
        assert!(!watchdog.is_paused_debug());
        assert!(!watchdog.is_paused_core());
    }

    #[test]
    fn test_watchdog_full_reset() {
        let mut watchdog = Watchdog::new();

        // Set all values
        watchdog.set_enabled(true);
        watchdog.load = 100;
        watchdog.counter = 50;
        watchdog.reason = 0x3;
        watchdog.scratch[0] = 0x12345678;

        // Full reset
        watchdog.full_reset();

        // Everything should be cleared
        assert!(!watchdog.is_enabled());
        assert_eq!(watchdog.get_load(), 0);
        assert_eq!(watchdog.get_counter(), 0);
        assert_eq!(watchdog.get_reason(), 0);
        assert_eq!(watchdog.get_scratch(0), 0);
    }

    #[test]
    fn test_watchdog_set_load_reloads_counter() {
        let mut watchdog = Watchdog::new();
        watchdog.counter = 50;

        // Setting load should also set counter
        watchdog.set_load(100);
        assert_eq!(watchdog.get_load(), 100);
        assert_eq!(watchdog.get_counter(), 100);
    }
}