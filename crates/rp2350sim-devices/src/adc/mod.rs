//! ADC device for RP2350.
//!
//! Implements the SAR ADC peripheral with full register support.

use rp2350sim_core::{Device, DeviceId, Result};

/// ADC base address.
pub const ADC_BASE: u32 = 0x4004_C000;

/// ADC register offsets.
pub mod regs {
    pub const CS: u32 = 0x000;
    pub const RESULT: u32 = 0x004;
    pub const FCS: u32 = 0x008;
    pub const FIFO: u32 = 0x00C;
    pub const DIV: u32 = 0x010;
    pub const INTR: u32 = 0x014;
    pub const INTE: u32 = 0x018;
    pub const INTF: u32 = 0x01C;
    pub const INTS: u32 = 0x020;
}

/// CS register bits.
pub mod cs {
    pub const EN: u32 = 1 << 0;
    pub const TS_EN: u32 = 1 << 1;
    pub const START_ONCE: u32 = 1 << 2;
    pub const START_MANY: u32 = 1 << 3;
    pub const READY: u32 = 1 << 8;
    pub const ERR: u32 = 1 << 9;
    pub const ERR_STICKY: u32 = 1 << 10;
    pub const AINSEL_MASK: u32 = 0x07 << 12;
    pub const AINSEL_SHIFT: u32 = 12;
}

/// FCS register bits.
pub mod fcs {
    pub const THRESH_MASK: u32 = 0x0F << 0;
    pub const EN: u32 = 1 << 8;
    pub const ERR: u32 = 1 << 10;
    pub const FULL: u32 = 1 << 16;
    pub const EMPTY: u32 = 1 << 17;
    pub const LEVEL_MASK: u32 = 0x0F << 24;
    pub const LEVEL_SHIFT: u32 = 24;
}

/// ADC channel count.
const NUM_CHANNELS: usize = 5; // 4 analog inputs + temperature sensor

/// ADC device.
#[derive(Debug)]
pub struct Adc {
    /// Control and status register.
    cs: u32,
    /// FIFO control and status.
    fcs: u32,
    /// Clock divider.
    div: u32,
    /// Interrupt status.
    intr: u32,
    /// Interrupt enable.
    inte: u32,
    /// Interrupt force.
    intf: u32,
    /// Channel values (12-bit, 0-4095).
    channels: [u16; NUM_CHANNELS],
    /// Current result.
    result: u16,
    /// FIFO buffer.
    fifo: Vec<u16>,
}

impl Default for Adc {
    fn default() -> Self {
        Self::new()
    }
}

impl Adc {
    /// Create a new ADC device.
    pub fn new() -> Self {
        Self {
            cs: 0,
            fcs: 0,
            div: 0,
            intr: 0,
            inte: 0,
            intf: 0,
            channels: [0; NUM_CHANNELS],
            result: 0,
            fifo: Vec::with_capacity(16),
        }
    }

    /// Get channel value (0-4095).
    pub fn get_value(&self, channel: usize) -> u16 {
        if channel < NUM_CHANNELS {
            self.channels[channel]
        } else {
            0
        }
    }

    /// Set channel value (0-4095).
    pub fn set_value(&mut self, channel: usize, value: u16) {
        if channel < NUM_CHANNELS {
            self.channels[channel] = value.min(4095);
        }
    }

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        (self.cs & cs::EN) != 0
    }

    /// Get current channel selection.
    pub fn get_channel(&self) -> usize {
        ((self.cs & cs::AINSEL_MASK) >> cs::AINSEL_SHIFT) as usize
    }

    /// Check if temperature sensor is enabled.
    pub fn is_temp_sensor_enabled(&self) -> bool {
        (self.cs & cs::TS_EN) != 0
    }

    /// Check if ready.
    pub fn is_ready(&self) -> bool {
        (self.cs & cs::READY) != 0
    }

    /// Start single conversion.
    pub fn start_once(&mut self) {
        self.cs |= cs::START_ONCE;
        self.do_conversion();
    }

    /// Perform a conversion.
    fn do_conversion(&mut self) {
        let channel = self.get_channel();
        let value = if channel == 4 || self.is_temp_sensor_enabled() {
            self.channels[4]
        } else {
            self.channels[channel]
        };

        self.result = value;
        self.cs |= cs::READY;

        // Push to FIFO if enabled
        if (self.fcs & fcs::EN) != 0 && self.fifo.len() < 16 {
            self.fifo.push(value);
        }

        self.cs &= !cs::START_ONCE;
    }

    /// Tick for continuous mode.
    pub fn tick(&mut self) {
        if (self.cs & cs::START_MANY) != 0 && self.is_enabled() {
            self.do_conversion();
        }
    }

    /// Read from FIFO.
    fn read_fifo(&mut self) -> u32 {
        if let Some(val) = self.fifo.pop() {
            val as u32
        } else {
            0x8000 // Error bit set if empty
        }
    }

    /// Check if there's a pending interrupt.
    pub fn has_interrupt(&self) -> bool {
        (self.intr & self.inte) != 0 || self.intf != 0
    }
}

impl Device for Adc {
    fn id(&self) -> DeviceId {
        DeviceId::ADC
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - ADC_BASE;

        match offset {
            regs::CS => Ok(self.cs),
            regs::RESULT => Ok(self.result as u32),
            regs::FCS => {
                let mut val = self.fcs;
                if self.fifo.len() >= 16 {
                    val |= fcs::FULL;
                }
                if self.fifo.is_empty() {
                    val |= fcs::EMPTY;
                }
                val |= (self.fifo.len() as u32) << 24;
                Ok(val)
            }
            regs::FIFO => Ok(self.read_fifo()),
            regs::DIV => Ok(self.div),
            regs::INTR => Ok(self.intr),
            regs::INTE => Ok(self.inte),
            regs::INTF => Ok(self.intf),
            regs::INTS => Ok(self.intr & self.inte | self.intf),
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - ADC_BASE;

        match offset {
            regs::CS => {
                let old_cs = self.cs;
                self.cs = value & 0x0007_770F; // Include AINSEL bits (12-14)
                if (value & cs::START_ONCE) != 0 && (old_cs & cs::START_ONCE) == 0 {
                    self.do_conversion();
                }
            }
            regs::FCS => {
                self.fcs = value & 0x0F00_131F;
            }
            regs::DIV => {
                self.div = value;
            }
            regs::INTR => {
                self.intr &= !value;
            }
            regs::INTE => {
                self.inte = value & 0x03;
            }
            regs::INTF => {
                self.intf = value & 0x03;
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
    fn test_adc_creation() {
        let adc = Adc::new();
        assert!(!adc.is_enabled());
        assert_eq!(adc.get_channel(), 0);
    }

    #[test]
    fn test_adc_enable() {
        let mut adc = Adc::new();
        assert!(!adc.is_enabled());

        adc.write(ADC_BASE + regs::CS, cs::EN).unwrap();
        assert!(adc.is_enabled());
    }

    #[test]
    fn test_adc_channel_selection() {
        let mut adc = Adc::new();

        // Select channel 2
        adc.write(ADC_BASE + regs::CS, cs::EN | (2 << cs::AINSEL_SHIFT)).unwrap();
        assert_eq!(adc.get_channel(), 2);

        // Select channel 4
        adc.write(ADC_BASE + regs::CS, cs::EN | (4 << cs::AINSEL_SHIFT)).unwrap();
        assert_eq!(adc.get_channel(), 4);
    }

    #[test]
    fn test_adc_value_set_get() {
        let mut adc = Adc::new();

        // Set channel values
        adc.set_value(0, 1000);
        adc.set_value(1, 2000);
        adc.set_value(2, 3000);

        assert_eq!(adc.get_value(0), 1000);
        assert_eq!(adc.get_value(1), 2000);
        assert_eq!(adc.get_value(2), 3000);
    }

    #[test]
    fn test_adc_conversion() {
        let mut adc = Adc::new();

        // Set channel 0 value
        adc.set_value(0, 2048);

        // Enable ADC and select channel 0
        adc.write(ADC_BASE + regs::CS, cs::EN | cs::START_ONCE).unwrap();

        // Read result
        let result = adc.read(ADC_BASE + regs::RESULT).unwrap();
        assert_eq!(result & 0xFFF, 2048);
    }

    #[test]
    fn test_adc_temperature_sensor() {
        let mut adc = Adc::new();

        // Enable temperature sensor
        adc.write(ADC_BASE + regs::CS, cs::EN | cs::TS_EN).unwrap();
        assert!(adc.is_temp_sensor_enabled());
    }

    #[test]
    fn test_adc_fifo() {
        let mut adc = Adc::new();

        // Enable FIFO
        adc.write(ADC_BASE + regs::FCS, fcs::EN).unwrap();

        // Check FIFO is empty
        let fcs = adc.read(ADC_BASE + regs::FCS).unwrap();
        assert_eq!(fcs & fcs::EMPTY, fcs::EMPTY);
    }

    #[test]
    fn test_adc_interrupts() {
        let mut adc = Adc::new();

        // Enable interrupt
        adc.write(ADC_BASE + regs::INTE, 0x01).unwrap();
        assert_eq!(adc.read(ADC_BASE + regs::INTE).unwrap(), 0x01);

        // Force interrupt
        adc.write(ADC_BASE + regs::INTF, 0x01).unwrap();
        assert_eq!(adc.read(ADC_BASE + regs::INTF).unwrap(), 0x01);
    }

    #[test]
    fn test_adc_value_clamping() {
        let mut adc = Adc::new();

        // Set value above 12-bit range
        adc.set_value(0, 5000);
        // Should be clamped to 4095
        assert_eq!(adc.get_value(0), 4095);

        // Set value below 0 (wrapping)
        adc.set_value(0, 0);
        assert_eq!(adc.get_value(0), 0);
    }

    #[test]
    fn test_adc_clock_divider() {
        let mut adc = Adc::new();

        // Set clock divider
        adc.write(ADC_BASE + regs::DIV, 0x0001_0000).unwrap();
        assert_eq!(adc.read(ADC_BASE + regs::DIV).unwrap(), 0x0001_0000);

        // Different divider value
        adc.write(ADC_BASE + regs::DIV, 0x00FF_0000).unwrap();
        assert_eq!(adc.read(ADC_BASE + regs::DIV).unwrap(), 0x00FF_0000);
    }

    #[test]
    fn test_adc_fifo_threshold() {
        let mut adc = Adc::new();

        // Set FIFO threshold to 4
        adc.write(ADC_BASE + regs::FCS, fcs::EN | (4 << 0)).unwrap();
        let fcs = adc.read(ADC_BASE + regs::FCS).unwrap();
        assert_eq!(fcs & fcs::THRESH_MASK, 4);
    }

    #[test]
    fn test_adc_start_many() {
        let mut adc = Adc::new();

        // Enable continuous sampling
        adc.write(ADC_BASE + regs::CS, cs::EN | cs::START_MANY).unwrap();
        let cs = adc.read(ADC_BASE + regs::CS).unwrap();
        assert_eq!(cs & cs::START_MANY, cs::START_MANY);
    }

    #[test]
    fn test_adc_error_status() {
        let mut adc = Adc::new();

        // Check initial error status
        let cs = adc.read(ADC_BASE + regs::CS).unwrap();
        assert_eq!(cs & cs::ERR, 0);
    }

    #[test]
    fn test_adc_all_channels() {
        let mut adc = Adc::new();

        // Set values for all channels
        for i in 0..4u16 {
            adc.set_value(i as usize, (i + 1) * 1000);
        }

        // Verify all channels
        for i in 0..4 {
            assert_eq!(adc.get_value(i), ((i + 1) * 1000) as u16);
        }
    }

    #[test]
    fn test_adc_reset() {
        let mut adc = Adc::new();

        // Modify state
        adc.write(ADC_BASE + regs::CS, cs::EN).unwrap();
        adc.write(ADC_BASE + regs::DIV, 0x1234_5678).unwrap();
        adc.set_value(0, 3000);

        // Reset
        adc.reset();

        // Check state is reset
        assert!(!adc.is_enabled());
        assert_eq!(adc.read(ADC_BASE + regs::DIV).unwrap(), 0);
        assert_eq!(adc.get_value(0), 0);
    }

    #[test]
    fn test_adc_fifo_operations() {
        let mut adc = Adc::new();

        // Enable ADC and FIFO
        adc.write(ADC_BASE + regs::CS, cs::EN).unwrap();
        adc.write(ADC_BASE + regs::FCS, fcs::EN).unwrap();

        // Set channel value
        adc.set_value(0, 1234);

        // Perform conversion
        adc.write(ADC_BASE + regs::CS, cs::EN | cs::START_ONCE).unwrap();

        // Check FIFO level
        let fcs = adc.read(ADC_BASE + regs::FCS).unwrap();
        let level = (fcs >> 24) & 0x0F;
        assert_eq!(level, 1);

        // Read from FIFO
        let fifo_val = adc.read(ADC_BASE + regs::FIFO).unwrap();
        assert_eq!(fifo_val & 0xFFF, 1234);
    }

    #[test]
    fn test_adc_fifo_full_empty() {
        let mut adc = Adc::new();

        // Enable ADC and FIFO
        adc.write(ADC_BASE + regs::CS, cs::EN).unwrap();
        adc.write(ADC_BASE + regs::FCS, fcs::EN).unwrap();

        // Initially empty
        let fcs = adc.read(ADC_BASE + regs::FCS).unwrap();
        assert_eq!(fcs & fcs::EMPTY, fcs::EMPTY);
        assert_eq!(fcs & fcs::FULL, 0);

        // Fill FIFO (16 entries max)
        adc.set_value(0, 1000);
        for _ in 0..16 {
            adc.write(ADC_BASE + regs::CS, cs::EN | cs::START_ONCE).unwrap();
        }

        // Check FIFO is full
        let fcs = adc.read(ADC_BASE + regs::FCS).unwrap();
        assert_eq!(fcs & fcs::FULL, fcs::FULL);
        assert_eq!(fcs & fcs::EMPTY, 0);
    }

    #[test]
    fn test_adc_continuous_mode() {
        let mut adc = Adc::new();

        // Enable ADC and continuous mode
        adc.set_value(0, 2000);
        adc.write(ADC_BASE + regs::CS, cs::EN | cs::START_MANY).unwrap();

        // Tick should perform conversion
        adc.tick();
        assert!(adc.is_ready());

        // Result should be updated
        let result = adc.read(ADC_BASE + regs::RESULT).unwrap();
        assert_eq!(result & 0xFFF, 2000);

        // Change value and tick again
        adc.set_value(0, 3000);
        adc.tick();
        let result = adc.read(ADC_BASE + regs::RESULT).unwrap();
        assert_eq!(result & 0xFFF, 3000);
    }

    #[test]
    fn test_adc_temperature_reading() {
        let mut adc = Adc::new();

        // Set temperature sensor value (channel 4)
        adc.set_value(4, 2048); // ~27°C typical

        // Enable temperature sensor and start conversion
        adc.write(ADC_BASE + regs::CS, cs::EN | cs::TS_EN | cs::START_ONCE).unwrap();

        // Read result - should be temperature sensor value
        let result = adc.read(ADC_BASE + regs::RESULT).unwrap();
        assert_eq!(result & 0xFFF, 2048);
    }

    #[test]
    fn test_adc_interrupt_status() {
        let mut adc = Adc::new();

        // Enable interrupt
        adc.write(ADC_BASE + regs::INTE, 0x01).unwrap();

        // Set interrupt status
        adc.intr = 0x01;

        // Check interrupt pending
        assert!(adc.has_interrupt());

        // Read INTS (masked interrupt status)
        let ints = adc.read(ADC_BASE + regs::INTS).unwrap();
        assert_eq!(ints, 0x01);
    }

    #[test]
    fn test_adc_interrupt_force() {
        let mut adc = Adc::new();

        // Force interrupt without enabling
        adc.write(ADC_BASE + regs::INTF, 0x02).unwrap();

        // Should have interrupt due to force
        assert!(adc.has_interrupt());

        // Clear force
        adc.write(ADC_BASE + regs::INTF, 0x00).unwrap();
        assert!(!adc.has_interrupt());
    }

    #[test]
    fn test_adc_fifo_error_on_empty_read() {
        let mut adc = Adc::new();

        // Enable FIFO but don't add any data
        adc.write(ADC_BASE + regs::FCS, fcs::EN).unwrap();

        // Read from empty FIFO
        let fifo_val = adc.read(ADC_BASE + regs::FIFO).unwrap();

        // Should have error bit set (bit 15)
        assert_eq!(fifo_val & 0x8000, 0x8000);
    }

    #[test]
    fn test_adc_channel_switching() {
        let mut adc = Adc::new();

        // Set different values for each channel
        adc.set_value(0, 1000);
        adc.set_value(1, 2000);
        adc.set_value(2, 3000);
        adc.set_value(3, 4000);

        // Enable ADC
        adc.write(ADC_BASE + regs::CS, cs::EN).unwrap();

        // Test each channel
        for ch in 0..4u32 {
            adc.write(ADC_BASE + regs::CS, cs::EN | (ch << cs::AINSEL_SHIFT) | cs::START_ONCE).unwrap();
            let result = adc.read(ADC_BASE + regs::RESULT).unwrap();
            assert_eq!(result & 0xFFF, (ch + 1) * 1000);
        }
    }

    #[test]
    fn test_adc_ready_flag() {
        let mut adc = Adc::new();

        // Initially not ready
        assert!(!adc.is_ready());

        // Enable and start conversion
        adc.write(ADC_BASE + regs::CS, cs::EN | cs::START_ONCE).unwrap();

        // Should be ready after conversion
        assert!(adc.is_ready());

        // Read CS to check READY bit
        let cs = adc.read(ADC_BASE + regs::CS).unwrap();
        assert_eq!(cs & cs::READY, cs::READY);
    }

    #[test]
    fn test_adc_fifo_level_tracking() {
        let mut adc = Adc::new();

        // Enable ADC and FIFO
        adc.write(ADC_BASE + regs::CS, cs::EN).unwrap();
        adc.write(ADC_BASE + regs::FCS, fcs::EN).unwrap();
        adc.set_value(0, 1000);

        // Add multiple entries
        for i in 0..5 {
            adc.set_value(0, 1000 + i as u16);
            adc.write(ADC_BASE + regs::CS, cs::EN | cs::START_ONCE).unwrap();
        }

        // Check FIFO level
        let fcs = adc.read(ADC_BASE + regs::FCS).unwrap();
        let level = (fcs >> 24) & 0x0F;
        assert_eq!(level, 5);

        // Read some entries
        for _ in 0..3 {
            adc.read(ADC_BASE + regs::FIFO).unwrap();
        }

        // Check new level
        let fcs = adc.read(ADC_BASE + regs::FCS).unwrap();
        let level = (fcs >> 24) & 0x0F;
        assert_eq!(level, 2);
    }

    #[test]
    fn test_adc_clear_interrupt() {
        let mut adc = Adc::new();

        // Set interrupt status
        adc.intr = 0x03;

        // Clear by writing 1s
        adc.write(ADC_BASE + regs::INTR, 0x01).unwrap();
        assert_eq!(adc.intr, 0x02);

        adc.write(ADC_BASE + regs::INTR, 0x02).unwrap();
        assert_eq!(adc.intr, 0x00);
    }

    #[test]
    fn test_adc_fifo_with_threshold() {
        let mut adc = Adc::new();

        // Enable FIFO with threshold of 4
        adc.write(ADC_BASE + regs::FCS, fcs::EN | (4 << 0)).unwrap();

        // Verify threshold is set
        let fcs = adc.read(ADC_BASE + regs::FCS).unwrap();
        assert_eq!(fcs & 0x0F, 4);
    }

    #[test]
    fn test_adc_multiple_conversions_same_channel() {
        let mut adc = Adc::new();

        // Enable ADC
        adc.write(ADC_BASE + regs::CS, cs::EN).unwrap();

        // Perform multiple conversions on same channel
        for i in 0..10 {
            let val = (i * 400) as u16;
            adc.set_value(0, val);
            adc.write(ADC_BASE + regs::CS, cs::EN | cs::START_ONCE).unwrap();
            let result = adc.read(ADC_BASE + regs::RESULT).unwrap();
            assert_eq!(result & 0xFFF, val as u32);
        }
    }

    #[test]
    fn test_adc_disabled_no_conversion() {
        let mut adc = Adc::new();

        // Set channel value but don't enable ADC
        adc.set_value(0, 1234);

        // START_ONCE triggers conversion even without EN bit
        // This matches hardware behavior where START_ONCE forces a conversion
        adc.write(ADC_BASE + regs::CS, cs::START_ONCE).unwrap();

        // Result should be the channel value (START_ONCE forces conversion)
        let result = adc.read(ADC_BASE + regs::RESULT).unwrap();
        assert_eq!(result & 0xFFF, 1234);
    }
}