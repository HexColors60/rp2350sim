//! PWM device for RP2350.
//!
//! Implements the PWM peripheral with full register support.

use rp2350sim_core::{Device, DeviceId, Result};

/// PWM base address.
pub const PWM_BASE: u32 = 0x4005_0000;

/// PWM register offsets (slice-based).
pub mod regs {
    pub const CH0_CSR: u32 = 0x000;
    pub const CH0_DIV: u32 = 0x004;
    pub const CH0_CTR: u32 = 0x008;
    pub const CH0_CC: u32 = 0x00C;
    pub const CH0_TOP: u32 = 0x010;
    // Each slice is 16 bytes, 8 slices total
    pub const EN: u32 = 0x0A0;
    pub const INTR: u32 = 0x0A4;
    pub const INTE: u32 = 0x0A8;
    pub const INTF: u32 = 0x0AC;
    pub const INTS: u32 = 0x0B0;
}

/// CSR register bits.
pub mod csr {
    pub const EN: u32 = 1 << 0;
    pub const MODE_FREE: u32 = 0x00 << 1;
    pub const MODE_HIGH: u32 = 0x01 << 1;
    pub const MODE_LOW: u32 = 0x02 << 1;
    pub const MODE_MASK: u32 = 0x03 << 1;
    pub const PH_CORRECT: u32 = 1 << 3;
    pub const A_INV: u32 = 1 << 4;
    pub const B_INV: u32 = 1 << 5;
    pub const DIVMODE_DIV: u32 = 0x00 << 6;
    pub const DIVMODE_FALL: u32 = 0x01 << 6;
    pub const DIVMODE_RISE: u32 = 0x02 << 6;
    pub const DIVMODE_MASK: u32 = 0x03 << 6;
}

/// Number of PWM slices.
const NUM_SLICES: usize = 8;

/// Bytes per PWM slice (5 registers * 4 bytes).
const SLICE_SIZE: u32 = 20;

/// PWM slice (contains 2 channels: A and B).
#[derive(Debug, Clone, Copy, Default)]
pub struct PwmSlice {
    /// Control and status register.
    pub csr: u32,
    /// Clock divider (8.4 fixed point).
    pub div: u32,
    /// Counter value.
    pub ctr: u16,
    /// Compare values (A and B).
    pub cc: (u16, u16),
    /// Top value.
    pub top: u16,
}

/// PWM device.
#[derive(Debug)]
pub struct Pwm {
    /// PWM slices.
    slices: [PwmSlice; NUM_SLICES],
    /// Global enable mask.
    en: u32,
    /// Interrupt status.
    intr: u32,
    /// Interrupt enable.
    inte: u32,
    /// Interrupt force.
    intf: u32,
}

impl Default for Pwm {
    fn default() -> Self {
        Self::new()
    }
}

impl Pwm {
    /// Create a new PWM device.
    pub fn new() -> Self {
        let mut slices = [PwmSlice::default(); NUM_SLICES];
        for slice in &mut slices {
            slice.top = 0xFFFF; // Default top value
        }
        Self {
            slices,
            en: 0,
            intr: 0,
            inte: 0,
            intf: 0,
        }
    }

    /// Get duty cycle for channel (0-65535).
    pub fn get_duty(&self, channel: usize) -> u16 {
        let slice = channel / 2;
        let is_b = channel % 2 == 1;
        if slice < NUM_SLICES {
            if is_b {
                self.slices[slice].cc.1
            } else {
                self.slices[slice].cc.0
            }
        } else {
            0
        }
    }

    /// Set duty cycle for channel (0-65535).
    pub fn set_duty(&mut self, channel: usize, duty: u16) {
        let slice = channel / 2;
        let is_b = channel % 2 == 1;
        if slice < NUM_SLICES {
            if is_b {
                self.slices[slice].cc.1 = duty;
            } else {
                self.slices[slice].cc.0 = duty;
            }
        }
    }

    /// Check if slice is enabled.
    pub fn is_slice_enabled(&self, slice: usize) -> bool {
        if slice < NUM_SLICES {
            (self.en & (1 << slice)) != 0
        } else {
            false
        }
    }

    /// Enable/disable slice.
    pub fn set_slice_enabled(&mut self, slice: usize, enabled: bool) {
        if slice < NUM_SLICES {
            if enabled {
                self.en |= 1 << slice;
                self.slices[slice].csr |= csr::EN;
            } else {
                self.en &= !(1 << slice);
                self.slices[slice].csr &= !csr::EN;
            }
        }
    }

    /// Get top value for slice.
    pub fn get_top(&self, slice: usize) -> u16 {
        if slice < NUM_SLICES {
            self.slices[slice].top
        } else {
            0
        }
    }

    /// Set top value for slice.
    pub fn set_top(&mut self, slice: usize, top: u16) {
        if slice < NUM_SLICES {
            self.slices[slice].top = top;
        }
    }

    /// Get clock divider for slice (8.4 fixed point).
    pub fn get_div(&self, slice: usize) -> u32 {
        if slice < NUM_SLICES {
            self.slices[slice].div
        } else {
            0
        }
    }

    /// Set clock divider for slice (8.4 fixed point).
    pub fn set_div(&mut self, slice: usize, div: u32) {
        if slice < NUM_SLICES {
            self.slices[slice].div = div;
        }
    }

    /// Get counter value for slice.
    pub fn get_counter(&self, slice: usize) -> u16 {
        if slice < NUM_SLICES {
            self.slices[slice].ctr
        } else {
            0
        }
    }

    /// Set counter value for slice.
    pub fn set_counter(&mut self, slice: usize, ctr: u16) {
        if slice < NUM_SLICES {
            self.slices[slice].ctr = ctr;
        }
    }

    /// Check if phase correct mode is enabled.
    pub fn is_phase_correct(&self, slice: usize) -> bool {
        if slice < NUM_SLICES {
            (self.slices[slice].csr & csr::PH_CORRECT) != 0
        } else {
            false
        }
    }

    /// Check if channel A output is inverted.
    pub fn is_a_inverted(&self, slice: usize) -> bool {
        if slice < NUM_SLICES {
            (self.slices[slice].csr & csr::A_INV) != 0
        } else {
            false
        }
    }

    /// Check if channel B output is inverted.
    pub fn is_b_inverted(&self, slice: usize) -> bool {
        if slice < NUM_SLICES {
            (self.slices[slice].csr & csr::B_INV) != 0
        } else {
            false
        }
    }

    /// Tick the PWM (advance counters).
    pub fn tick(&mut self) {
        for slice_idx in 0..NUM_SLICES {
            if self.is_slice_enabled(slice_idx) {
                let slice = &mut self.slices[slice_idx];
                let top = slice.top;
                
                if slice.ctr >= top {
                    slice.ctr = 0;
                    self.intr |= 1 << slice_idx;
                } else {
                    slice.ctr += 1;
                }
            }
        }
    }

    /// Check if there's a pending interrupt.
    pub fn has_interrupt(&self) -> bool {
        (self.intr & self.inte) != 0 || self.intf != 0
    }

    /// Get actual output value for channel.
    pub fn get_output(&self, channel: usize) -> bool {
        let slice = channel / 2;
        let is_b = channel % 2 == 1;
        
        if slice >= NUM_SLICES {
            return false;
        }

        let s = &self.slices[slice];
        if !self.is_slice_enabled(slice) {
            return false;
        }

        let duty = if is_b { s.cc.1 } else { s.cc.0 };
        let inverted = if is_b { 
            (s.csr & csr::B_INV) != 0 
        } else { 
            (s.csr & csr::A_INV) != 0 
        };

        let output = s.ctr < duty;
        
        if inverted { !output } else { output }
    }
}

impl Device for Pwm {
    fn id(&self) -> DeviceId {
        DeviceId::PWM
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - PWM_BASE;

        // Check for slice registers
        if offset < regs::EN {
            let slice = (offset / SLICE_SIZE) as usize;
            let reg = offset % SLICE_SIZE;
            
            if slice < NUM_SLICES {
                return match reg {
                    0x0 => Ok(self.slices[slice].csr),
                    0x4 => Ok(self.slices[slice].div),
                    0x8 => Ok(self.slices[slice].ctr as u32),
                    0xC => Ok((self.slices[slice].cc.1 as u32) << 16 | (self.slices[slice].cc.0 as u32)),
                    0x10 => Ok(self.slices[slice].top as u32),
                    _ => Ok(0),
                };
            }
        }

        match offset {
            regs::EN => Ok(self.en),
            regs::INTR => Ok(self.intr),
            regs::INTE => Ok(self.inte),
            regs::INTF => Ok(self.intf),
            regs::INTS => Ok(self.intr & self.inte | self.intf),
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - PWM_BASE;

        // Check for slice registers
        if offset < regs::EN {
            let slice = (offset / SLICE_SIZE) as usize;
            let reg = offset % SLICE_SIZE;
            
            if slice < NUM_SLICES {
                match reg {
                    0x0 => {
                        self.slices[slice].csr = value & 0x3FF;
                        if (value & csr::EN) != 0 {
                            self.en |= 1 << slice;
                        } else {
                            self.en &= !(1 << slice);
                        }
                    }
                    0x4 => {
                        self.slices[slice].div = value & 0xFFFF;
                    }
                    0x8 => {
                        self.slices[slice].ctr = (value & 0xFFFF) as u16;
                    }
                    0xC => {
                        self.slices[slice].cc.0 = (value & 0xFFFF) as u16;
                        self.slices[slice].cc.1 = ((value >> 16) & 0xFFFF) as u16;
                    }
                    0x10 => {
                        self.slices[slice].top = (value & 0xFFFF) as u16;
                    }
                    _ => {}
                }
                return Ok(());
            }
        }

        match offset {
            regs::EN => {
                self.en = value & 0xFF;
                for i in 0..NUM_SLICES {
                    if (value & (1 << i)) != 0 {
                        self.slices[i].csr |= csr::EN;
                    } else {
                        self.slices[i].csr &= !csr::EN;
                    }
                }
            }
            regs::INTR => {
                self.intr &= !value;
            }
            regs::INTE => {
                self.inte = value & 0xFF;
            }
            regs::INTF => {
                self.intf = value & 0xFF;
            }
            _ => {}
        }

        Ok(())
    }

    fn reset(&mut self) {
        *self = Self::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pwm_creation() {
        let pwm = Pwm::new();
        // Check that all slices have default top value
        for slice in &pwm.slices {
            assert_eq!(slice.top, 0xFFFF);
        }
    }

    #[test]
    fn test_pwm_duty_cycle() {
        let mut pwm = Pwm::new();

        // Set duty cycle for channel 0 (slice 0, channel A)
        pwm.set_duty(0, 0x8000);
        assert_eq!(pwm.get_duty(0), 0x8000);

        // Set duty cycle for channel 1 (slice 0, channel B)
        pwm.set_duty(1, 0x4000);
        assert_eq!(pwm.get_duty(1), 0x4000);

        // Set duty cycle for channel 2 (slice 1, channel A)
        pwm.set_duty(2, 0x2000);
        assert_eq!(pwm.get_duty(2), 0x2000);
    }

    #[test]
    fn test_pwm_slice_enable() {
        let mut pwm = Pwm::new();

        // Enable slice 0
        pwm.write(PWM_BASE + regs::EN, 0x01).unwrap();
        assert!(pwm.is_slice_enabled(0));
        assert!(!pwm.is_slice_enabled(1));

        // Enable multiple slices
        pwm.write(PWM_BASE + regs::EN, 0x0F).unwrap();
        assert!(pwm.is_slice_enabled(0));
        assert!(pwm.is_slice_enabled(1));
        assert!(pwm.is_slice_enabled(2));
        assert!(pwm.is_slice_enabled(3));
        assert!(!pwm.is_slice_enabled(4));
    }

    #[test]
    fn test_pwm_register_read_write() {
        let mut pwm = Pwm::new();

        // Test CH0_CSR register
        pwm.write(PWM_BASE + regs::CH0_CSR, csr::EN).unwrap();
        assert_eq!(pwm.read(PWM_BASE + regs::CH0_CSR).unwrap(), csr::EN);

        // Test CH0_DIV register
        pwm.write(PWM_BASE + regs::CH0_DIV, 0x0010).unwrap();
        assert_eq!(pwm.read(PWM_BASE + regs::CH0_DIV).unwrap(), 0x0010);

        // Test CH0_TOP register
        pwm.write(PWM_BASE + regs::CH0_TOP, 0x1000).unwrap();
        assert_eq!(pwm.read(PWM_BASE + regs::CH0_TOP).unwrap(), 0x1000);
    }

    #[test]
    fn test_pwm_counter() {
        let mut pwm = Pwm::new();

        // Set counter value
        pwm.write(PWM_BASE + regs::CH0_CTR, 0x1234).unwrap();
        assert_eq!(pwm.read(PWM_BASE + regs::CH0_CTR).unwrap(), 0x1234);
    }

    #[test]
    fn test_pwm_compare_values() {
        let mut pwm = Pwm::new();

        // Set compare values for channel A and B
        // CC register format: (B << 16) | A
        pwm.write(PWM_BASE + regs::CH0_CC, 0x2000_1000).unwrap();
        let cc = pwm.read(PWM_BASE + regs::CH0_CC).unwrap();
        assert_eq!(cc, 0x2000_1000);

        // Check duty cycle matches
        assert_eq!(pwm.get_duty(0), 0x1000); // Channel A (low 16 bits)
        assert_eq!(pwm.get_duty(1), 0x2000); // Channel B (high 16 bits)
    }

    #[test]
    fn test_pwm_interrupts() {
        let mut pwm = Pwm::new();

        // Enable interrupt for slice 0
        pwm.write(PWM_BASE + regs::INTE, 0x01).unwrap();
        assert_eq!(pwm.read(PWM_BASE + regs::INTE).unwrap(), 0x01);

        // Force interrupt for slice 0
        pwm.write(PWM_BASE + regs::INTF, 0x01).unwrap();
        assert_eq!(pwm.read(PWM_BASE + regs::INTF).unwrap(), 0x01);
    }
}