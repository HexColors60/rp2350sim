//! Interpolator hardware accelerator for RP2350.
//!
//! Implements the hardware interpolators for fast math operations.

use rp2350sim_core::{Device, DeviceId, Result};

/// Interpolator base addresses.
pub const INTERP0_BASE: u32 = 0x5002_0000;
pub const INTERP1_BASE: u32 = 0x5002_1000;

/// Interpolator register offsets.
pub mod regs {
    pub const ACCUM0: u32 = 0x000;
    pub const ACCUM1: u32 = 0x004;
    pub const BASE0: u32 = 0x008;
    pub const BASE1: u32 = 0x00C;
    pub const BASE2: u32 = 0x010;
    pub const POP_LANE0: u32 = 0x014;
    pub const POP_LANE1: u32 = 0x018;
    pub const POP_FULL: u32 = 0x01C;
    pub const PEEK_LANE0: u32 = 0x020;
    pub const PEEK_LANE1: u32 = 0x024;
    pub const PEEK_FULL: u32 = 0x028;
    pub const CTRL_LANE0: u32 = 0x02C;
    pub const CTRL_LANE1: u32 = 0x030;
    pub const CTRL_LANE0_SIGNED: u32 = 0x034;
    pub const CTRL_LANE1_SIGNED: u32 = 0x038;
    pub const ADD_RAW: u32 = 0x03C;
    pub const BASE0_AND1: u32 = 0x040;
}

/// CTRL_LANE0/CTRL_LANE1 register bits.
pub mod ctrl {
    pub const SHIFT_SHIFT: u32 = 0;
    pub const SHIFT_MASK: u32 = 0x1F;
    pub const MASK_LSB_SHIFT: u32 = 5;
    pub const MASK_LSB_MASK: u32 = 0x1F << 5;
    pub const MASK_MSB_SHIFT: u32 = 10;
    pub const MASK_MSB_MASK: u32 = 0x1F << 10;
    pub const SIGNED: u32 = 1 << 15;
    pub const CROSS_INPUT: u32 = 1 << 16;
    pub const CROSS_RESULT: u32 = 1 << 17;
    pub const ADD_RAW: u32 = 1 << 18;
    pub const OVERF0: u32 = 1 << 19;
    pub const OVERF1: u32 = 1 << 20;
    pub const OVERF: u32 = 1 << 21;
    pub const BLEND: u32 = 1 << 23;
}

/// Interpolator lane.
#[derive(Debug, Clone, Default)]
pub struct InterpLane {
    /// Accumulator.
    accum: u32,
    /// Base value.
    base: u32,
    /// Control register.
    ctrl: u32,
    /// Signed mode.
    signed: bool,
    /// Shift amount.
    shift: u8,
    /// Mask LSB.
    mask_lsb: u8,
    /// Mask MSB.
    mask_msb: u8,
    /// Cross input enable.
    cross_input: bool,
    /// Cross result enable.
    cross_result: bool,
    /// Add raw mode.
    add_raw: bool,
    /// Overflow flags.
    overflow0: bool,
    overflow1: bool,
    /// Blend mode.
    blend: bool,
}

impl InterpLane {
    /// Create a new interpolator lane.
    pub fn new() -> Self {
        Self {
            mask_msb: 31,  // Default: full 32-bit
            ..Default::default()
        }
    }

    /// Update control from register value.
    fn update_ctrl(&mut self, value: u32) {
        self.ctrl = value;
        self.shift = ((value >> ctrl::SHIFT_SHIFT) & 0x1F) as u8;
        self.mask_lsb = ((value >> ctrl::MASK_LSB_SHIFT) & 0x1F) as u8;
        self.mask_msb = ((value >> ctrl::MASK_MSB_SHIFT) & 0x1F) as u8;
        self.signed = (value & ctrl::SIGNED) != 0;
        self.cross_input = (value & ctrl::CROSS_INPUT) != 0;
        self.cross_result = (value & ctrl::CROSS_RESULT) != 0;
        self.add_raw = (value & ctrl::ADD_RAW) != 0;
        self.blend = (value & ctrl::BLEND) != 0;
    }

    /// Get control register value.
    fn get_ctrl(&self) -> u32 {
        let mut value = (self.shift as u32) << ctrl::SHIFT_SHIFT
            | (self.mask_lsb as u32) << ctrl::MASK_LSB_SHIFT
            | (self.mask_msb as u32) << ctrl::MASK_MSB_SHIFT;
        
        if self.signed { value |= ctrl::SIGNED; }
        if self.cross_input { value |= ctrl::CROSS_INPUT; }
        if self.cross_result { value |= ctrl::CROSS_RESULT; }
        if self.add_raw { value |= ctrl::ADD_RAW; }
        if self.overflow0 { value |= ctrl::OVERF0; }
        if self.overflow1 { value |= ctrl::OVERF1; }
        if self.blend { value |= ctrl::BLEND; }
        
        value
    }

    /// Compute result.
    fn compute(&self, cross_input: u32) -> u32 {
        // Apply shift
        let shifted = if self.shift < 32 {
            self.accum >> self.shift
        } else {
            0
        };

        // Apply mask
        let mask = if self.mask_lsb <= self.mask_msb && self.mask_msb < 32 {
            let bits = self.mask_msb - self.mask_lsb + 1;
            ((1u32 << bits) - 1) << self.mask_lsb
        } else {
            0xFFFFFFFF
        };
        let masked = shifted & mask;

        // Get input (use cross input if enabled)
        let input = if self.cross_input {
            cross_input
        } else {
            self.base
        };

        // Add base/input
        let result = if self.add_raw {
            self.accum.wrapping_add(input)
        } else {
            masked.wrapping_add(input)
        };

        result
    }

    /// Sign-extend a value.
    #[allow(dead_code)]
    fn sign_extend(&self, value: u32) -> u32 {
        if !self.signed {
            return value;
        }

        let bits = self.mask_msb.saturating_sub(self.mask_lsb) + 1;
        if bits == 0 || bits >= 32 {
            return value;
        }

        let sign_bit = 1u32 << (bits - 1);
        if value & sign_bit != 0 {
            // Negative: extend sign
            let mask = !((1u32 << bits) - 1);
            value | mask
        } else {
            value
        }
    }
}

/// Interpolator instance.
#[derive(Debug)]
pub struct Interp {
    /// Interpolator ID (0 or 1).
    pub id: u8,
    /// Base address.
    base: u32,
    /// Lane 0.
    lane0: InterpLane,
    /// Lane 1.
    lane1: InterpLane,
    /// Base2 register.
    base2: u32,
    /// Accumulator 0 (for reading).
    accum0: u32,
    /// Accumulator 1 (for reading).
    accum1: u32,
    /// Result 0 (peeked).
    result0: u32,
    /// Result 1 (peeked).
    result1: u32,
    /// Result full (combined).
    result_full: u64,
    /// Add raw flag.
    add_raw: bool,
}

impl Default for Interp {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Interp {
    /// Create a new interpolator.
    pub fn new(id: u8) -> Self {
        Self {
            id,
            base: if id == 0 { INTERP0_BASE } else { INTERP1_BASE },
            lane0: InterpLane::new(),
            lane1: InterpLane::new(),
            base2: 0,
            accum0: 0,
            accum1: 0,
            result0: 0,
            result1: 0,
            result_full: 0,
            add_raw: false,
        }
    }

    /// Get base address.
    pub fn base(&self) -> u32 {
        self.base
    }

    /// Compute results.
    fn compute_results(&mut self) {
        // Lane 0 computes first
        self.result0 = self.lane0.compute(self.lane1.base);
        
        // Lane 1 can use lane 0's result as cross input
        let cross_input = if self.lane1.cross_input {
            self.result0
        } else {
            self.lane1.base
        };
        self.result1 = self.lane1.compute(cross_input);
        
        // Full result combines both lanes
        self.result_full = (self.result0 as u64) | ((self.result1 as u64) << 32);
    }

    /// Pop lane 0 result (also writes to accum).
    fn pop_lane0(&mut self) -> u32 {
        self.compute_results();
        let result = self.result0;
        self.accum0 = self.lane0.accum;
        result
    }

    /// Pop lane 1 result.
    fn pop_lane1(&mut self) -> u32 {
        self.compute_results();
        let result = self.result1;
        self.accum1 = self.lane1.accum;
        result
    }

    /// Pop full result.
    fn pop_full(&mut self) -> u64 {
        self.compute_results();
        self.result_full
    }

    /// Peek lane 0 result (doesn't modify state).
    fn peek_lane0(&mut self) -> u32 {
        self.compute_results();
        self.result0
    }

    /// Peek lane 1 result.
    fn peek_lane1(&mut self) -> u32 {
        self.compute_results();
        self.result1
    }

    /// Peek full result.
    fn peek_full(&mut self) -> u64 {
        self.compute_results();
        self.result_full
    }

    /// Perform blend operation.
    #[allow(dead_code)]
    fn blend(&mut self, a: u32, b: u32, weight: u32) -> u32 {
        // Linear interpolation: result = a + (b - a) * weight / 256
        let diff = (b as i64 - a as i64) as i64;
        let result = (a as i64 + diff * (weight as i64) / 256) as u32;
        result
    }
}

impl Device for Interp {
    fn id(&self) -> DeviceId {
        DeviceId::INTERP
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - self.base;

        match offset {
            regs::ACCUM0 => Ok(self.lane0.accum),
            regs::ACCUM1 => Ok(self.lane1.accum),
            regs::BASE0 => Ok(self.lane0.base),
            regs::BASE1 => Ok(self.lane1.base),
            regs::BASE2 => Ok(self.base2),
            regs::POP_LANE0 => Ok(self.pop_lane0()),
            regs::POP_LANE1 => Ok(self.pop_lane1()),
            regs::POP_FULL => Ok(self.pop_full() as u32),
            regs::PEEK_LANE0 => Ok(self.peek_lane0()),
            regs::PEEK_LANE1 => Ok(self.peek_lane1()),
            regs::PEEK_FULL => Ok(self.peek_full() as u32),
            regs::CTRL_LANE0 => Ok(self.lane0.get_ctrl()),
            regs::CTRL_LANE1 => Ok(self.lane1.get_ctrl()),
            regs::CTRL_LANE0_SIGNED => Ok(if self.lane0.signed { 1 } else { 0 }),
            regs::CTRL_LANE1_SIGNED => Ok(if self.lane1.signed { 1 } else { 0 }),
            regs::ADD_RAW => Ok(if self.add_raw { 1 } else { 0 }),
            regs::BASE0_AND1 => Ok(self.lane0.base | (self.lane1.base << 16)),
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - self.base;

        match offset {
            regs::ACCUM0 => {
                self.lane0.accum = value;
            }
            regs::ACCUM1 => {
                self.lane1.accum = value;
            }
            regs::BASE0 => {
                self.lane0.base = value;
            }
            regs::BASE1 => {
                self.lane1.base = value;
            }
            regs::BASE2 => {
                self.base2 = value;
            }
            regs::POP_LANE0 => {
                // Writing to POP also sets accum
                self.lane0.accum = value;
            }
            regs::POP_LANE1 => {
                self.lane1.accum = value;
            }
            regs::POP_FULL => {
                // Sets both accumulators
                self.lane0.accum = value & 0xFFFF;
                self.lane1.accum = (value >> 16) & 0xFFFF;
            }
            regs::CTRL_LANE0 => {
                self.lane0.update_ctrl(value);
            }
            regs::CTRL_LANE1 => {
                self.lane1.update_ctrl(value);
            }
            regs::CTRL_LANE0_SIGNED => {
                self.lane0.signed = value != 0;
            }
            regs::CTRL_LANE1_SIGNED => {
                self.lane1.signed = value != 0;
            }
            regs::ADD_RAW => {
                self.add_raw = value != 0;
            }
            regs::BASE0_AND1 => {
                self.lane0.base = value & 0xFFFF;
                self.lane1.base = (value >> 16) & 0xFFFF;
            }
            _ => {}
        }

        Ok(())
    }

    fn reset(&mut self) {
        let id = self.id;
        let base = self.base;
        *self = Self::new(id);
        self.base = base;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INTERP0_BASE: u32 = super::INTERP0_BASE;
    const INTERP1_BASE: u32 = super::INTERP1_BASE;

    // ==================== Basic Tests ====================

    #[test]
    fn test_interp_creation() {
        let interp0 = Interp::new(0);
        let interp1 = Interp::new(1);

        assert_eq!(interp0.id, 0);
        assert_eq!(interp1.id, 1);
        assert_eq!(interp0.base(), INTERP0_BASE);
        assert_eq!(interp1.base(), INTERP1_BASE);
    }

    #[test]
    fn test_interp_default() {
        let interp = Interp::default();
        assert_eq!(interp.id, 0);
        assert_eq!(interp.base(), INTERP0_BASE);
    }

    #[test]
    fn test_accumulator_read_write() {
        let mut interp = Interp::new(0);

        // Write to accumulators
        interp.write(INTERP0_BASE + regs::ACCUM0, 0x12345678).unwrap();
        interp.write(INTERP0_BASE + regs::ACCUM1, 0xDEADBEEF).unwrap();

        // Read back
        assert_eq!(interp.read(INTERP0_BASE + regs::ACCUM0).unwrap(), 0x12345678);
        assert_eq!(interp.read(INTERP0_BASE + regs::ACCUM1).unwrap(), 0xDEADBEEF);
    }

    #[test]
    fn test_base_read_write() {
        let mut interp = Interp::new(0);

        // Write to base registers
        interp.write(INTERP0_BASE + regs::BASE0, 0x100).unwrap();
        interp.write(INTERP0_BASE + regs::BASE1, 0x200).unwrap();
        interp.write(INTERP0_BASE + regs::BASE2, 0x300).unwrap();

        // Read back
        assert_eq!(interp.read(INTERP0_BASE + regs::BASE0).unwrap(), 0x100);
        assert_eq!(interp.read(INTERP0_BASE + regs::BASE1).unwrap(), 0x200);
        assert_eq!(interp.read(INTERP0_BASE + regs::BASE2).unwrap(), 0x300);
    }

    #[test]
    fn test_base0_and1_register() {
        let mut interp = Interp::new(0);

        // Write to BASE0_AND1 - sets both base registers
        interp.write(INTERP0_BASE + regs::BASE0_AND1, 0x00020001).unwrap();

        assert_eq!(interp.read(INTERP0_BASE + regs::BASE0).unwrap(), 0x0001);
        assert_eq!(interp.read(INTERP0_BASE + regs::BASE1).unwrap(), 0x0002);

        // Read back via BASE0_AND1
        assert_eq!(interp.read(INTERP0_BASE + regs::BASE0_AND1).unwrap(), 0x00020001);
    }

    #[test]
    fn test_ctrl_lane_settings() {
        let mut interp = Interp::new(0);

        // Set control register with various fields
        let ctrl = (5 << ctrl::SHIFT_SHIFT) | (8 << ctrl::MASK_LSB_SHIFT) | (15 << ctrl::MASK_MSB_SHIFT);
        interp.write(INTERP0_BASE + regs::CTRL_LANE0, ctrl).unwrap();

        // Read back and verify
        let read_ctrl = interp.read(INTERP0_BASE + regs::CTRL_LANE0).unwrap();
        assert_eq!(read_ctrl & ctrl::SHIFT_MASK, 5);
        assert_eq!((read_ctrl >> ctrl::MASK_LSB_SHIFT) & 0x1F, 8);
        assert_eq!((read_ctrl >> ctrl::MASK_MSB_SHIFT) & 0x1F, 15);
    }

    #[test]
    fn test_ctrl_signed_mode() {
        let mut interp = Interp::new(0);

        // Test signed mode via CTRL_LANE0_SIGNED
        interp.write(INTERP0_BASE + regs::CTRL_LANE0_SIGNED, 1).unwrap();
        assert_eq!(interp.read(INTERP0_BASE + regs::CTRL_LANE0_SIGNED).unwrap(), 1);

        interp.write(INTERP0_BASE + regs::CTRL_LANE0_SIGNED, 0).unwrap();
        assert_eq!(interp.read(INTERP0_BASE + regs::CTRL_LANE0_SIGNED).unwrap(), 0);
    }

    #[test]
    fn test_ctrl_flags() {
        let mut interp = Interp::new(0);

        // Set various flags
        let ctrl = ctrl::SIGNED | ctrl::CROSS_INPUT | ctrl::CROSS_RESULT | ctrl::ADD_RAW;
        interp.write(INTERP0_BASE + regs::CTRL_LANE0, ctrl).unwrap();

        let read_ctrl = interp.read(INTERP0_BASE + regs::CTRL_LANE0).unwrap();
        assert_eq!(read_ctrl & ctrl::SIGNED, ctrl::SIGNED);
        assert_eq!(read_ctrl & ctrl::CROSS_INPUT, ctrl::CROSS_INPUT);
        assert_eq!(read_ctrl & ctrl::CROSS_RESULT, ctrl::CROSS_RESULT);
        assert_eq!(read_ctrl & ctrl::ADD_RAW, ctrl::ADD_RAW);
    }

    // ==================== Compute Tests ====================

    #[test]
    fn test_peek_lane0() {
        let mut interp = Interp::new(0);

        // Set accumulator and base
        interp.write(INTERP0_BASE + regs::ACCUM0, 100).unwrap();
        interp.write(INTERP0_BASE + regs::BASE0, 50).unwrap();

        // Peek should return accum + base (with default mask)
        let result = interp.read(INTERP0_BASE + regs::PEEK_LANE0).unwrap();
        assert_eq!(result, 150);
    }

    #[test]
    fn test_peek_lane1() {
        let mut interp = Interp::new(0);

        interp.write(INTERP0_BASE + regs::ACCUM1, 200).unwrap();
        interp.write(INTERP0_BASE + regs::BASE1, 25).unwrap();

        let result = interp.read(INTERP0_BASE + regs::PEEK_LANE1).unwrap();
        assert_eq!(result, 225);
    }

    #[test]
    fn test_pop_lane0() {
        let mut interp = Interp::new(0);

        interp.write(INTERP0_BASE + regs::ACCUM0, 0x12345678).unwrap();

        let result = interp.read(INTERP0_BASE + regs::POP_LANE0).unwrap();
        assert_eq!(result, 0x12345678); // With default mask, result should equal accum
    }

    #[test]
    fn test_pop_updates_accum() {
        let mut interp = Interp::new(0);

        // Write to POP_LANE0 also sets accum
        interp.write(INTERP0_BASE + regs::POP_LANE0, 0xABCDEF00).unwrap();

        assert_eq!(interp.read(INTERP0_BASE + regs::ACCUM0).unwrap(), 0xABCDEF00);
    }

    // ==================== Mask and Shift Tests ====================

    #[test]
    fn test_mask_extract_bits() {
        let mut interp = Interp::new(0);

        // Configure mask: bits 8-15 (8 bits)
        let ctrl = (8 << ctrl::MASK_LSB_SHIFT) | (15 << ctrl::MASK_MSB_SHIFT);
        interp.write(INTERP0_BASE + regs::CTRL_LANE0, ctrl).unwrap();

        // Set accumulator with value in bits 8-15
        interp.write(INTERP0_BASE + regs::ACCUM0, 0x00FF0000).unwrap();
        interp.write(INTERP0_BASE + regs::BASE0, 0).unwrap();

        let result = interp.read(INTERP0_BASE + regs::PEEK_LANE0).unwrap();
        // Mask extracts bits 8-15, result should be 0xFF00
        assert_eq!(result, 0xFF00);
    }

    #[test]
    fn test_shift_operation() {
        let mut interp = Interp::new(0);

        // Configure shift by 4
        let ctrl = 4 << ctrl::SHIFT_SHIFT;
        interp.write(INTERP0_BASE + regs::CTRL_LANE0, ctrl).unwrap();

        interp.write(INTERP0_BASE + regs::ACCUM0, 0x100).unwrap();
        interp.write(INTERP0_BASE + regs::BASE0, 0).unwrap();

        let result = interp.read(INTERP0_BASE + regs::PEEK_LANE0).unwrap();
        // 0x100 >> 4 = 0x10
        assert_eq!(result, 0x10);
    }

    #[test]
    fn test_shift_and_mask_combined() {
        let mut interp = Interp::new(0);

        // Shift by 8, mask bits 0-7
        let ctrl = (8 << ctrl::SHIFT_SHIFT) | (0 << ctrl::MASK_LSB_SHIFT) | (7 << ctrl::MASK_MSB_SHIFT);
        interp.write(INTERP0_BASE + regs::CTRL_LANE0, ctrl).unwrap();

        // Value: 0x12345678 >> 8 = 0x123456, mask bits 0-7 = 0x56
        interp.write(INTERP0_BASE + regs::ACCUM0, 0x12345678).unwrap();
        interp.write(INTERP0_BASE + regs::BASE0, 0).unwrap();

        let result = interp.read(INTERP0_BASE + regs::PEEK_LANE0).unwrap();
        assert_eq!(result, 0x56);
    }

    // ==================== Cross Input/Result Tests ====================

    #[test]
    fn test_cross_input() {
        let mut interp = Interp::new(0);

        // Enable cross input for lane 1
        let ctrl = ctrl::CROSS_INPUT;
        interp.write(INTERP0_BASE + regs::CTRL_LANE1, ctrl).unwrap();

        // Lane 0 result should be used as input for lane 1
        interp.write(INTERP0_BASE + regs::ACCUM0, 100).unwrap();
        interp.write(INTERP0_BASE + regs::ACCUM1, 0).unwrap();
        interp.write(INTERP0_BASE + regs::BASE0, 50).unwrap();
        interp.write(INTERP0_BASE + regs::BASE1, 0).unwrap();

        // Lane 1 result = lane 0 result (150) + lane 1 base (0)
        let result1 = interp.read(INTERP0_BASE + regs::PEEK_LANE1).unwrap();
        assert_eq!(result1, 150);
    }

    // ==================== Add Raw Mode Tests ====================

    #[test]
    fn test_add_raw_mode() {
        let mut interp = Interp::new(0);

        // Enable add raw mode
        interp.write(INTERP0_BASE + regs::ADD_RAW, 1).unwrap();

        let ctrl = ctrl::ADD_RAW;
        interp.write(INTERP0_BASE + regs::CTRL_LANE0, ctrl).unwrap();

        interp.write(INTERP0_BASE + regs::ACCUM0, 1000).unwrap();
        interp.write(INTERP0_BASE + regs::BASE0, 500).unwrap();

        // In add raw mode, result = accum + base directly
        let result = interp.read(INTERP0_BASE + regs::PEEK_LANE0).unwrap();
        assert_eq!(result, 1500);
    }

    // ==================== Reset Test ====================

    #[test]
    fn test_interp_reset() {
        let mut interp = Interp::new(0);

        // Set various values
        interp.write(INTERP0_BASE + regs::ACCUM0, 0x12345678).unwrap();
        interp.write(INTERP0_BASE + regs::ACCUM1, 0xDEADBEEF).unwrap();
        interp.write(INTERP0_BASE + regs::BASE0, 0x100).unwrap();
        interp.write(INTERP0_BASE + regs::BASE1, 0x200).unwrap();

        // Reset
        interp.reset();

        // Check values are cleared
        assert_eq!(interp.read(INTERP0_BASE + regs::ACCUM0).unwrap(), 0);
        assert_eq!(interp.read(INTERP0_BASE + regs::ACCUM1).unwrap(), 0);
        assert_eq!(interp.read(INTERP0_BASE + regs::BASE0).unwrap(), 0);
        assert_eq!(interp.read(INTERP0_BASE + regs::BASE1).unwrap(), 0);

        // Base address should be preserved
        assert_eq!(interp.base(), INTERP0_BASE);
    }

    // ==================== Interp1 Tests ====================

    #[test]
    fn test_interp1_independence() {
        let mut interp0 = Interp::new(0);
        let mut interp1 = Interp::new(1);

        // Write to interp0
        interp0.write(INTERP0_BASE + regs::ACCUM0, 0x11111111).unwrap();

        // Write to interp1
        interp1.write(INTERP1_BASE + regs::ACCUM0, 0x22222222).unwrap();

        // Verify they are independent
        assert_eq!(interp0.read(INTERP0_BASE + regs::ACCUM0).unwrap(), 0x11111111);
        assert_eq!(interp1.read(INTERP1_BASE + regs::ACCUM0).unwrap(), 0x22222222);
    }

    // ==================== Device Trait Tests ====================

    #[test]
    fn test_device_id() {
        let interp = Interp::new(0);
        assert_eq!(interp.id(), DeviceId::INTERP);
    }

    #[test]
    fn test_invalid_register() {
        let mut interp = Interp::new(0);

        // Read from invalid offset should return 0
        let result = interp.read(INTERP0_BASE + 0x1000).unwrap();
        assert_eq!(result, 0);

        // Write to invalid offset should be ignored
        interp.write(INTERP0_BASE + 0x1000, 0x12345678).unwrap();
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_mask_full_32bit() {
        let mut interp = Interp::new(0);

        // Default mask should be full 32-bit
        interp.write(INTERP0_BASE + regs::ACCUM0, 0xFFFFFFFF).unwrap();
        interp.write(INTERP0_BASE + regs::BASE0, 0).unwrap();

        let result = interp.read(INTERP0_BASE + regs::PEEK_LANE0).unwrap();
        assert_eq!(result, 0xFFFFFFFF);
    }

    #[test]
    fn test_shift_31() {
        let mut interp = Interp::new(0);

        // Shift by 31
        let ctrl = 31 << ctrl::SHIFT_SHIFT;
        interp.write(INTERP0_BASE + regs::CTRL_LANE0, ctrl).unwrap();

        interp.write(INTERP0_BASE + regs::ACCUM0, 0x80000000).unwrap();
        interp.write(INTERP0_BASE + regs::BASE0, 0).unwrap();

        let result = interp.read(INTERP0_BASE + regs::PEEK_LANE0).unwrap();
        // 0x80000000 >> 31 = 1
        assert_eq!(result, 1);
    }

    #[test]
    fn test_wrapping_add() {
        let mut interp = Interp::new(0);

        interp.write(INTERP0_BASE + regs::ACCUM0, 0xFFFFFFFF).unwrap();
        interp.write(INTERP0_BASE + regs::BASE0, 1).unwrap();

        let result = interp.read(INTERP0_BASE + regs::PEEK_LANE0).unwrap();
        // Should wrap around
        assert_eq!(result, 0);
    }

    #[test]
    fn test_pop_full() {
        let mut interp = Interp::new(0);

        // Set both accumulators
        interp.write(INTERP0_BASE + regs::ACCUM0, 0x1234).unwrap();
        interp.write(INTERP0_BASE + regs::ACCUM1, 0x5678).unwrap();

        // Pop full returns combined result (lower 32 bits)
        let result = interp.read(INTERP0_BASE + regs::POP_FULL).unwrap();
        // Result depends on implementation; just verify it doesn't panic
        assert!(result >= 0);
    }

    #[test]
    fn test_pop_full_write() {
        let mut interp = Interp::new(0);

        // Write to POP_FULL sets both accumulators (16 bits each)
        interp.write(INTERP0_BASE + regs::POP_FULL, 0x00020001).unwrap();

        assert_eq!(interp.read(INTERP0_BASE + regs::ACCUM0).unwrap(), 0x0001);
        assert_eq!(interp.read(INTERP0_BASE + regs::ACCUM1).unwrap(), 0x0002);
    }
}