//! GPIO device for RP2350.
//!
//! Implements the GPIO peripheral with full register support.

use rp2350sim_core::{Device, DeviceId, Result};

/// GPIO base address.
pub const GPIO_BASE: u32 = 0x4001_4000;

/// GPIO register offsets.
pub mod regs {
    pub const GPIO0_STATUS: u32 = 0x000;
    pub const GPIO0_CTRL: u32 = 0x004;
    pub const GPIO1_STATUS: u32 = 0x008;
    pub const GPIO1_CTRL: u32 = 0x00C;
    // ... continues for all 48 GPIOs
    
    pub const GPIO_STATUS_OFFSET: u32 = 0x000;
    pub const GPIO_CTRL_OFFSET: u32 = 0x004;
    pub const GPIO_STRIDE: u32 = 0x008;
    
    pub const INTR0: u32 = 0x0F0;
    pub const INTR1: u32 = 0x0F4;
    pub const INTR2: u32 = 0x0F8;
    pub const INTR3: u32 = 0x0FC;
    pub const PROC0_INTE0: u32 = 0x100;
    pub const PROC0_INTE1: u32 = 0x104;
    pub const PROC0_INTE2: u32 = 0x108;
    pub const PROC0_INTE3: u32 = 0x10C;
    pub const PROC0_INTF0: u32 = 0x110;
    pub const PROC0_INTF1: u32 = 0x114;
    pub const PROC0_INTF2: u32 = 0x118;
    pub const PROC0_INTF3: u32 = 0x11C;
    pub const PROC0_INTS0: u32 = 0x120;
    pub const PROC0_INTS1: u32 = 0x124;
    pub const PROC0_INTS2: u32 = 0x128;
    pub const PROC0_INTS3: u32 = 0x12C;
    
    pub const PROC1_INTE0: u32 = 0x130;
    pub const PROC1_INTE1: u32 = 0x134;
    pub const PROC1_INTE2: u32 = 0x138;
    pub const PROC1_INTE3: u32 = 0x13C;
    pub const PROC1_INTF0: u32 = 0x140;
    pub const PROC1_INTF1: u32 = 0x144;
    pub const PROC1_INTF2: u32 = 0x148;
    pub const PROC1_INTF3: u32 = 0x14C;
    pub const PROC1_INTS0: u32 = 0x150;
    pub const PROC1_INTS1: u32 = 0x154;
    pub const PROC1_INTS2: u32 = 0x158;
    pub const PROC1_INTS3: u32 = 0x15C;
    
    pub const DORMANT_WAKE_INTE0: u32 = 0x160;
    pub const DORMANT_WAKE_INTE1: u32 = 0x164;
    pub const DORMANT_WAKE_INTE2: u32 = 0x168;
    pub const DORMANT_WAKE_INTE3: u32 = 0x16C;
    pub const DORMANT_WAKE_INTF0: u32 = 0x170;
    pub const DORMANT_WAKE_INTF1: u32 = 0x174;
    pub const DORMANT_WAKE_INTF2: u32 = 0x178;
    pub const DORMANT_WAKE_INTF3: u32 = 0x17C;
    pub const DORMANT_WAKE_INTS0: u32 = 0x180;
    pub const DORMANT_WAKE_INTS1: u32 = 0x184;
    pub const DORMANT_WAKE_INTS2: u32 = 0x188;
    pub const DORMANT_WAKE_INTS3: u32 = 0x18C;
    
    // IO Bank 0 specific
    pub const IO_BANK0_BASE: u32 = 0x4001_4000;
    pub const IO_BANK0_STATUS_OFFSET: u32 = 0x000;
    pub const IO_BANK0_CTRL_OFFSET: u32 = 0x004;
    
    // QSPI IO Bank
    pub const IO_QSPI_BASE: u32 = 0x4001_8000;
    
    // Pad control
    pub const PADS_BANK0_BASE: u32 = 0x4002_0000;
    pub const PADS_QSPI_BASE: u32 = 0x4002_4000;
}

/// GPIO function select values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GpioFunction {
    Jtag = 0,
    Spi = 1,
    Uart = 2,
    I2c = 3,
    Pwm = 4,
    Sio = 5,
    Pio0 = 6,
    Pio1 = 7,
    Gpck = 8,
    Usb = 9,
    None = 0xF,
}

impl From<u8> for GpioFunction {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Jtag,
            1 => Self::Spi,
            2 => Self::Uart,
            3 => Self::I2c,
            4 => Self::Pwm,
            5 => Self::Sio,
            6 => Self::Pio0,
            7 => Self::Pio1,
            8 => Self::Gpck,
            9 => Self::Usb,
            _ => Self::None,
        }
    }
}

/// Single GPIO pin state.
#[derive(Debug, Clone)]
pub struct GpioPin {
    /// Pin direction (true = output).
    pub direction: bool,
    /// Output value.
    pub output_value: bool,
    /// Input value.
    pub input_value: bool,
    /// Previous input value (for edge detection).
    prev_input_value: bool,
    /// Output enable override.
    pub oe_override: OverrideValue,
    /// Output value override.
    pub out_override: OverrideValue,
    /// Input override.
    pub in_override: OverrideValue,
    /// IRQ override.
    pub irq_override: OverrideValue,
    /// Function select.
    pub function: GpioFunction,
    /// Pull-up enabled.
    pub pull_up: bool,
    /// Pull-down enabled.
    pub pull_down: bool,
    /// Drive strength.
    pub drive_strength: DriveStrength,
    /// Input enable.
    pub input_enable: bool,
    /// Output enable from peripheral.
    pub oe_peripheral: bool,
    /// Output value from peripheral.
    pub out_peripheral: bool,
    /// Slew rate fast.
    pub slew_fast: bool,
    /// Schmitt trigger enable.
    pub schmitt: bool,
    /// Interrupt on rising edge.
    pub irq_rise: bool,
    /// Interrupt on falling edge.
    pub irq_fall: bool,
    /// Interrupt on level high.
    pub irq_level_high: bool,
    /// Interrupt on level low.
    pub irq_level_low: bool,
}

impl Default for GpioPin {
    fn default() -> Self {
        Self {
            direction: false,
            output_value: false,
            input_value: false,
            prev_input_value: false,
            oe_override: OverrideValue::Normal,
            out_override: OverrideValue::Normal,
            in_override: OverrideValue::Normal,
            irq_override: OverrideValue::Normal,
            function: GpioFunction::None,
            pull_up: false,
            pull_down: false,
            drive_strength: DriveStrength::Ma4,
            input_enable: true,
            oe_peripheral: false,
            out_peripheral: false,
            slew_fast: false,
            schmitt: true,
            irq_rise: false,
            irq_fall: false,
            irq_level_high: false,
            irq_level_low: false,
        }
    }
}

/// Override values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverrideValue {
    Normal = 0,
    Invert = 1,
    Low = 2,
    High = 3,
}

impl From<u8> for OverrideValue {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Normal,
            1 => Self::Invert,
            2 => Self::Low,
            3 => Self::High,
            _ => Self::Normal,
        }
    }
}

/// Drive strength values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveStrength {
    Ma2 = 0,
    Ma4 = 1,
    Ma8 = 2,
    Ma12 = 3,
}

impl From<u8> for DriveStrength {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Ma2,
            1 => Self::Ma4,
            2 => Self::Ma8,
            3 => Self::Ma12,
            _ => Self::Ma4,
        }
    }
}

/// GPIO device.
#[derive(Debug)]
pub struct Gpio {
    /// GPIO pins (48 total: 30 IO_BANK0 + 6 QSPI + 12 reserved).
    pins: [GpioPin; 48],
    /// Raw interrupt status.
    intr: [u32; 4],
    /// Interrupt enable for processor 0.
    proc0_inte: [u32; 4],
    /// Interrupt force for processor 0.
    proc0_intf: [u32; 4],
    /// Interrupt status for processor 0.
    proc0_ints: [u32; 4],
    /// Interrupt enable for processor 1.
    proc1_inte: [u32; 4],
    /// Interrupt force for processor 1.
    proc1_intf: [u32; 4],
    /// Interrupt status for processor 1.
    proc1_ints: [u32; 4],
    /// Dormant wake interrupt enable.
    dormant_wake_inte: [u32; 4],
    /// Dormant wake interrupt force.
    dormant_wake_intf: [u32; 4],
    /// Dormant wake interrupt status.
    dormant_wake_ints: [u32; 4],
    /// Pad control registers.
    #[allow(dead_code)]
    pads: [u32; 48],
}

impl Default for Gpio {
    fn default() -> Self {
        Self::new()
    }
}

impl Gpio {
    /// Create a new GPIO device.
    pub fn new() -> Self {
        Self {
            pins: std::array::from_fn(|_| GpioPin::default()),
            intr: [0; 4],
            proc0_inte: [0; 4],
            proc0_intf: [0; 4],
            proc0_ints: [0; 4],
            proc1_inte: [0; 4],
            proc1_intf: [0; 4],
            proc1_ints: [0; 4],
            dormant_wake_inte: [0; 4],
            dormant_wake_intf: [0; 4],
            dormant_wake_ints: [0; 4],
            pads: [0; 48],
        }
    }

    /// Get the number of pins.
    pub fn pin_count(&self) -> usize {
        self.pins.len()
    }

    /// Set pin direction.
    pub fn set_dir(&mut self, pin: usize, output: bool) {
        if pin < self.pins.len() {
            self.pins[pin].direction = output;
        }
    }

    /// Set output value.
    pub fn set_output(&mut self, pin: usize, value: bool) {
        if pin < self.pins.len() {
            self.pins[pin].output_value = value;
        }
    }

    /// Set input value (external).
    pub fn set_input(&mut self, pin: usize, value: bool) {
        if pin < self.pins.len() {
            let prev = self.pins[pin].input_value;
            self.pins[pin].prev_input_value = prev;
            self.pins[pin].input_value = value;
            self.update_interrupt(pin);
        }
    }

    /// Get pin value.
    pub fn get_value(&self, pin: usize) -> bool {
        if pin < self.pins.len() {
            let p = &self.pins[pin];
            // Apply overrides
            let out_val = match p.out_override {
                OverrideValue::Normal => p.output_value,
                OverrideValue::Invert => !p.output_value,
                OverrideValue::Low => false,
                OverrideValue::High => true,
            };
            let oe = match p.oe_override {
                OverrideValue::Normal => p.direction,
                OverrideValue::Invert => !p.direction,
                OverrideValue::Low => false,
                OverrideValue::High => true,
            };
            if oe {
                out_val
            } else {
                // Apply input override
                match p.in_override {
                    OverrideValue::Normal => p.input_value,
                    OverrideValue::Invert => !p.input_value,
                    OverrideValue::Low => false,
                    OverrideValue::High => true,
                }
            }
        } else {
            false
        }
    }

    /// Get pin reference.
    pub fn get_pin(&self, pin: usize) -> Option<&GpioPin> {
        self.pins.get(pin)
    }

    /// Get mutable pin reference.
    pub fn get_pin_mut(&mut self, pin: usize) -> Option<&mut GpioPin> {
        self.pins.get_mut(pin)
    }

    /// Update interrupt status for a pin with edge detection.
    fn update_interrupt(&mut self, pin: usize) {
        if pin >= self.pins.len() {
            return;
        }

        let reg_idx = pin / 8;
        let bit_idx = (pin % 8) * 4;

        let p = &self.pins[pin];
        let current = p.input_value;
        let prev = p.prev_input_value;

        // Detect edges and levels
        let rising = current && !prev;
        let falling = !current && prev;

        // Check interrupt conditions
        let mut irq_triggered = false;

        // Level triggers
        if p.irq_level_high && current {
            irq_triggered = true;
        }
        if p.irq_level_low && !current {
            irq_triggered = true;
        }

        // Edge triggers
        if p.irq_rise && rising {
            irq_triggered = true;
        }
        if p.irq_fall && falling {
            irq_triggered = true;
        }

        // Set interrupt bit if triggered
        if irq_triggered {
            self.intr[reg_idx] |= 1 << bit_idx;
        }

        self.update_proc_ints();
    }


    /// Update processor interrupt status.
    fn update_proc_ints(&mut self) {
        for i in 0..4 {
            self.proc0_ints[i] = (self.intr[i] & self.proc0_inte[i]) | self.proc0_intf[i];
            self.proc1_ints[i] = (self.intr[i] & self.proc1_inte[i]) | self.proc1_intf[i];
            self.dormant_wake_ints[i] = (self.intr[i] & self.dormant_wake_inte[i]) | self.dormant_wake_intf[i];
        }
    }

    /// Configure interrupt detection for a pin.
    pub fn configure_irq(&mut self, pin: usize, rise: bool, fall: bool, level_high: bool, level_low: bool) {
        if pin < self.pins.len() {
            let p = &mut self.pins[pin];
            p.irq_rise = rise;
            p.irq_fall = fall;
            p.irq_level_high = level_high;
            p.irq_level_low = level_low;
        }
    }

    /// Check if pin has pending interrupt.
    pub fn has_interrupt(&self, pin: usize) -> bool {
        if pin >= self.pins.len() {
            return false;
        }
        let reg_idx = pin / 8;
        if reg_idx >= self.intr.len() {
            return false;
        }
        let bit_idx = (pin % 8) * 4;
        (self.intr[reg_idx] >> bit_idx) & 1 != 0
    }

    /// Clear interrupt for a pin.
    pub fn clear_interrupt(&mut self, pin: usize) {
        if pin >= self.pins.len() {
            return;
        }
        let reg_idx = pin / 8;
        if reg_idx >= self.intr.len() {
            return;
        }
        let bit_idx = (pin % 8) * 4;
        self.intr[reg_idx] &= !(1 << bit_idx);
        self.update_proc_ints();
    }

    /// Read GPIO status register.
    fn read_status(&self, pin: usize) -> u32 {
        if pin >= self.pins.len() {
            return 0;
        }
        
        let p = &self.pins[pin];
        let mut status = 0u32;
        
        // Bit 8: IRQTOPROC
        status |= (self.proc0_ints[pin / 8] >> ((pin % 8) * 4)) & 1;
        // Bit 7: IRQFROMPAD
        status |= (p.input_value as u32) << 7;
        // Bit 6: OETOPAD
        status |= (p.direction as u32) << 6;
        // Bit 5: OEFROMPERI
        status |= (p.oe_peripheral as u32) << 5;
        // Bit 4: OUTTOPAD
        status |= (self.get_value(pin) as u32) << 4;
        // Bit 3: OUTFROMPERI
        status |= (p.out_peripheral as u32) << 3;
        // Bits 2: INFROMPAD
        status |= (p.input_value as u32) << 2;
        // Bits 1: INTOPERI
        status |= (p.input_enable as u32) << 1;
        
        status
    }

    /// Read GPIO control register.
    fn read_ctrl(&self, pin: usize) -> u32 {
        if pin >= self.pins.len() {
            return 0;
        }
        
        let p = &self.pins[pin];
        let mut ctrl = 0u32;
        
        // Bits 28:29: IRQOVER
        ctrl |= (p.irq_override as u32) << 28;
        // Bits 24:25: INOVER
        ctrl |= (p.in_override as u32) << 24;
        // Bits 20:21: OEOVER
        ctrl |= (p.oe_override as u32) << 20;
        // Bits 16:17: OUTOVER
        ctrl |= (p.out_override as u32) << 16;
        // Bits 12:15: FUNCSEL
        ctrl |= (p.function as u32) << 12;
        // Bit 8: ODE (open drain)
        // Bit 7: PDE (pull down enable)
        ctrl |= (p.pull_down as u32) << 7;
        // Bit 6: PUE (pull up enable)
        ctrl |= (p.pull_up as u32) << 6;
        // Bit 5: DRIVE (MSB)
        // Bit 4: DRIVE (LSB)
        ctrl |= (p.drive_strength as u32) << 4;
        // Bit 3: IE (input enable)
        ctrl |= (p.input_enable as u32) << 3;
        // Bit 2: SLEWFAST
        ctrl |= (p.slew_fast as u32) << 2;
        // Bit 1: SCHMITT
        ctrl |= (p.schmitt as u32) << 1;
        
        ctrl
    }

    /// Write GPIO control register.
    fn write_ctrl(&mut self, pin: usize, value: u32) {
        if pin >= self.pins.len() {
            return;
        }
        
        let p = &mut self.pins[pin];
        
        p.irq_override = OverrideValue::from(((value >> 28) & 3) as u8);
        p.in_override = OverrideValue::from(((value >> 24) & 3) as u8);
        p.oe_override = OverrideValue::from(((value >> 20) & 3) as u8);
        p.out_override = OverrideValue::from(((value >> 16) & 3) as u8);
        p.function = GpioFunction::from(((value >> 12) & 0xF) as u8);
        p.pull_down = (value >> 7) & 1 != 0;
        p.pull_up = (value >> 6) & 1 != 0;
        p.drive_strength = DriveStrength::from(((value >> 4) & 3) as u8);
        p.input_enable = (value >> 3) & 1 != 0;
        p.slew_fast = (value >> 2) & 1 != 0;
        p.schmitt = (value >> 1) & 1 != 0;
    }

    /// Read pad control register.
    #[allow(dead_code)]
    fn read_pad(&self, pin: usize) -> u32 {
        if pin >= self.pads.len() {
            return 0;
        }
        self.pads[pin]
    }

    /// Write pad control register.
    #[allow(dead_code)]
    fn write_pad(&mut self, pin: usize, value: u32) {
        if pin >= self.pads.len() {
            return;
        }
        self.pads[pin] = value;
        
        // Update pin state from pad
        if pin < self.pins.len() {
            let p = &mut self.pins[pin];
            p.pull_up = (value >> 6) & 1 != 0;
            p.pull_down = (value >> 7) & 1 != 0;
            p.input_enable = (value >> 3) & 1 != 0;
            p.drive_strength = DriveStrength::from(((value >> 4) & 3) as u8);
            p.slew_fast = (value >> 2) & 1 != 0;
            p.schmitt = (value >> 1) & 1 != 0;
        }
    }
}

impl Device for Gpio {
    fn id(&self) -> DeviceId {
        DeviceId::GPIO
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - GPIO_BASE;
        
        // GPIO status/control registers
        if offset < 0x0F0 {
            let pin = (offset / regs::GPIO_STRIDE) as usize;
            let reg_offset = offset % regs::GPIO_STRIDE;
            
            return match reg_offset {
                regs::GPIO_STATUS_OFFSET => Ok(self.read_status(pin)),
                regs::GPIO_CTRL_OFFSET => Ok(self.read_ctrl(pin)),
                _ => Ok(0),
            };
        }
        
        // Interrupt registers
        match offset {
            regs::INTR0..=regs::INTR3 => {
                let idx = ((offset - regs::INTR0) / 4) as usize;
                Ok(self.intr[idx])
            }
            regs::PROC0_INTE0..=regs::PROC0_INTE3 => {
                let idx = ((offset - regs::PROC0_INTE0) / 4) as usize;
                Ok(self.proc0_inte[idx])
            }
            regs::PROC0_INTF0..=regs::PROC0_INTF3 => {
                let idx = ((offset - regs::PROC0_INTF0) / 4) as usize;
                Ok(self.proc0_intf[idx])
            }
            regs::PROC0_INTS0..=regs::PROC0_INTS3 => {
                let idx = ((offset - regs::PROC0_INTS0) / 4) as usize;
                Ok(self.proc0_ints[idx])
            }
            regs::PROC1_INTE0..=regs::PROC1_INTE3 => {
                let idx = ((offset - regs::PROC1_INTE0) / 4) as usize;
                Ok(self.proc1_inte[idx])
            }
            regs::PROC1_INTF0..=regs::PROC1_INTF3 => {
                let idx = ((offset - regs::PROC1_INTF0) / 4) as usize;
                Ok(self.proc1_intf[idx])
            }
            regs::PROC1_INTS0..=regs::PROC1_INTS3 => {
                let idx = ((offset - regs::PROC1_INTS0) / 4) as usize;
                Ok(self.proc1_ints[idx])
            }
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - GPIO_BASE;
        
        // GPIO status/control registers
        if offset < 0x0F0 {
            let pin = (offset / regs::GPIO_STRIDE) as usize;
            let reg_offset = offset % regs::GPIO_STRIDE;
            
            match reg_offset {
                regs::GPIO_CTRL_OFFSET => self.write_ctrl(pin, value),
                _ => {}
            }
            return Ok(());
        }
        
        // Interrupt registers
        match offset {
            regs::INTR0..=regs::INTR3 => {
                let idx = ((offset - regs::INTR0) / 4) as usize;
                // Write to clear
                self.intr[idx] &= !value;
                self.update_proc_ints();
            }
            regs::PROC0_INTE0..=regs::PROC0_INTE3 => {
                let idx = ((offset - regs::PROC0_INTE0) / 4) as usize;
                self.proc0_inte[idx] = value;
                self.update_proc_ints();
            }
            regs::PROC0_INTF0..=regs::PROC0_INTF3 => {
                let idx = ((offset - regs::PROC0_INTF0) / 4) as usize;
                self.proc0_intf[idx] = value;
                self.update_proc_ints();
            }
            regs::PROC1_INTE0..=regs::PROC1_INTE3 => {
                let idx = ((offset - regs::PROC1_INTE0) / 4) as usize;
                self.proc1_inte[idx] = value;
                self.update_proc_ints();
            }
            regs::PROC1_INTF0..=regs::PROC1_INTF3 => {
                let idx = ((offset - regs::PROC1_INTF0) / 4) as usize;
                self.proc1_intf[idx] = value;
                self.update_proc_ints();
            }
            _ => {}
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        *self = Self::new();
    }
}

/// SIO GPIO registers (direct access).
pub struct SioGpio {
    /// GPIO output value (bits 0-29).
    pub gpio_out: u32,
    /// GPIO output enable set.
    pub gpio_oe_set: u32,
    /// GPIO output enable clear.
    pub gpio_oe_clr: u32,
    /// GPIO input value.
    pub gpio_in: u32,
}

impl Default for SioGpio {
    fn default() -> Self {
        Self::new()
    }
}

impl SioGpio {
    pub fn new() -> Self {
        Self {
            gpio_out: 0,
            gpio_oe_set: 0,
            gpio_oe_clr: 0,
            gpio_in: 0,
        }
    }
    
    /// SIO base address.
    pub const BASE: u32 = 0xD000_0000;
    
    /// Register offsets.
    pub const GPIO_OUT: u32 = 0x010;
    pub const GPIO_OUT_SET: u32 = 0x014;
    pub const GPIO_OUT_CLR: u32 = 0x018;
    pub const GPIO_OUT_XOR: u32 = 0x01C;
    pub const GPIO_OE: u32 = 0x020;
    pub const GPIO_OE_SET: u32 = 0x024;
    pub const GPIO_OE_CLR: u32 = 0x028;
    pub const GPIO_OE_XOR: u32 = 0x02C;
    pub const GPIO_IN: u32 = 0x004;
    
    /// Read SIO GPIO register.
    pub fn read(&self, offset: u32) -> u32 {
        match offset {
            Self::GPIO_OUT => self.gpio_out,
            Self::GPIO_IN => self.gpio_in,
            Self::GPIO_OE => self.gpio_oe_set,
            _ => 0,
        }
    }
    
    /// Write SIO GPIO register.
    pub fn write(&mut self, offset: u32, value: u32) {
        match offset {
            Self::GPIO_OUT => self.gpio_out = value,
            Self::GPIO_OUT_SET => self.gpio_out |= value,
            Self::GPIO_OUT_CLR => self.gpio_out &= !value,
            Self::GPIO_OUT_XOR => self.gpio_out ^= value,
            Self::GPIO_OE => self.gpio_oe_set = value,
            Self::GPIO_OE_SET => self.gpio_oe_set |= value,
            Self::GPIO_OE_CLR => self.gpio_oe_set &= !value,
            Self::GPIO_OE_XOR => self.gpio_oe_set ^= value,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpio_creation() {
        let gpio = Gpio::new();
        assert_eq!(gpio.pin_count(), 48);
    }

    #[test]
    fn test_gpio_direction() {
        let mut gpio = Gpio::new();
        gpio.set_dir(0, true);
        assert!(gpio.pins[0].direction);
    }

    #[test]
    fn test_gpio_output() {
        let mut gpio = Gpio::new();
        gpio.set_dir(0, true);
        gpio.set_output(0, true);
        assert!(gpio.get_value(0));
    }

    #[test]
    fn test_gpio_input() {
        let mut gpio = Gpio::new();
        gpio.set_input(0, true);
        assert!(gpio.get_value(0));
    }

    #[test]
    fn test_gpio_function() {
        let mut gpio = Gpio::new();
        gpio.pins[0].function = GpioFunction::Uart;
        assert_eq!(gpio.pins[0].function, GpioFunction::Uart);
    }

    #[test]
    fn test_gpio_ctrl_register() {
        let mut gpio = Gpio::new();
        // FUNCSEL is bits 12-15, value 4 = Pwm
        gpio.write(GPIO_BASE + regs::GPIO_CTRL_OFFSET, 0x0000_4000);
        assert_eq!(gpio.pins[0].function, GpioFunction::Pwm);
    }

    #[test]
    fn test_gpio_edge_detection() {
        let mut gpio = Gpio::new();

        // Configure rising edge detection on pin 0
        gpio.configure_irq(0, true, false, false, false);

        // Initial state is low
        gpio.set_input(0, false);

        // Rising edge should trigger interrupt
        gpio.set_input(0, true);
        assert!(gpio.has_interrupt(0));

        // Clear interrupt
        gpio.clear_interrupt(0);
        assert!(!gpio.has_interrupt(0));

        // No edge (staying high) should not trigger
        gpio.set_input(0, true);
        assert!(!gpio.has_interrupt(0));
    }

    #[test]
    fn test_gpio_falling_edge_detection() {
        let mut gpio = Gpio::new();

        // Configure falling edge detection on pin 5
        gpio.configure_irq(5, false, true, false, false);

        // Start high
        gpio.set_input(5, true);

        // Falling edge should trigger interrupt
        gpio.set_input(5, false);
        assert!(gpio.has_interrupt(5));
    }

    #[test]
    fn test_gpio_level_detection() {
        let mut gpio = Gpio::new();

        // Configure level high detection on pin 10
        gpio.configure_irq(10, false, false, true, false);

        // Level high should trigger interrupt
        gpio.set_input(10, true);
        assert!(gpio.has_interrupt(10));

        // Clear and set low
        gpio.clear_interrupt(10);
        gpio.set_input(10, false);
        assert!(!gpio.has_interrupt(10));
    }
}