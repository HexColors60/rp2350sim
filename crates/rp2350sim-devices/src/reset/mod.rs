//! Reset controller for RP2350.
//!
//! Implements the RESETS peripheral for resetting individual hardware blocks.

use rp2350sim_core::{Device, DeviceId, Result};

/// Reset controller base address.
pub const RESETS_BASE: u32 = 0x4002_0000;

/// Reset controller register offsets.
pub mod regs {
    pub const RESET: u32 = 0x000;
    pub const WDSEL: u32 = 0x004;
    pub const RESET_DONE: u32 = 0x008;
    pub const RESET_DONE_TBMAN: u32 = 0x00C;
}

/// RESET register bits - peripherals that can be reset.
pub mod reset {
    pub const USBCTRL: u32 = 1 << 0;
    pub const UART0: u32 = 1 << 1;
    pub const UART1: u32 = 1 << 2;
    pub const SPI0: u32 = 1 << 3;
    pub const SPI1: u32 = 1 << 4;
    pub const I2C0: u32 = 1 << 5;
    pub const I2C1: u32 = 1 << 6;
    pub const ADC: u32 = 1 << 7;
    pub const PWM: u32 = 1 << 8;
    pub const SIO: u32 = 1 << 9;
    pub const PIO0: u32 = 1 << 10;
    pub const PIO1: u32 = 1 << 11;
    pub const TIMER0: u32 = 1 << 12;
    pub const TIMER1: u32 = 1 << 13;
    pub const SPI_USB: u32 = 1 << 14;
    pub const UART_USB: u32 = 1 << 15;
    pub const ADC_ALT: u32 = 1 << 16;
    pub const PIO0_ALT: u32 = 1 << 17;
    pub const PIO1_ALT: u32 = 1 << 18;
    pub const I2C0_ALT: u32 = 1 << 19;
    pub const I2C1_ALT: u32 = 1 << 20;
    pub const SPI0_ALT: u32 = 1 << 21;
    pub const SPI1_ALT: u32 = 1 << 22;
    pub const UART0_ALT: u32 = 1 << 23;
    pub const UART1_ALT: u32 = 1 << 24;
    pub const HSTX: u32 = 1 << 25;
    pub const TIMER2: u32 = 1 << 26;
    pub const TIMER3: u32 = 1 << 27;
    pub const TRNG: u32 = 1 << 28;
    pub const TBMAN: u32 = 1 << 29;
    pub const PLL_USB: u32 = 1 << 30;
    pub const PLL_SYS: u32 = 1 << 31;
}

/// WDSEL register bits - watchdog reset select.
pub mod wdsel {
    pub const USBCTRL: u32 = 1 << 0;
    pub const UART0: u32 = 1 << 1;
    pub const UART1: u32 = 1 << 2;
    pub const SPI0: u32 = 1 << 3;
    pub const SPI1: u32 = 1 << 4;
    pub const I2C0: u32 = 1 << 5;
    pub const I2C1: u32 = 1 << 6;
    pub const ADC: u32 = 1 << 7;
    pub const PWM: u32 = 1 << 8;
    pub const SIO: u32 = 1 << 9;
    pub const PIO0: u32 = 1 << 10;
    pub const PIO1: u32 = 1 << 11;
    pub const TIMER0: u32 = 1 << 12;
    pub const TIMER1: u32 = 1 << 13;
    pub const TRNG: u32 = 1 << 14;
    pub const TBMAN: u32 = 1 << 15;
    pub const PLL_USB: u32 = 1 << 16;
    pub const PLL_SYS: u32 = 1 << 17;
    pub const CORESIGHT: u32 = 1 << 18;
    pub const PROC0: u32 = 1 << 19;
    pub const PROC1: u32 = 1 << 20;
}

/// Reset done status bits.
pub mod reset_done {
    pub const USBCTRL: u32 = 1 << 0;
    pub const UART0: u32 = 1 << 1;
    pub const UART1: u32 = 1 << 2;
    pub const SPI0: u32 = 1 << 3;
    pub const SPI1: u32 = 1 << 4;
    pub const I2C0: u32 = 1 << 5;
    pub const I2C1: u32 = 1 << 6;
    pub const ADC: u32 = 1 << 7;
    pub const PWM: u32 = 1 << 8;
    pub const SIO: u32 = 1 << 9;
    pub const PIO0: u32 = 1 << 10;
    pub const PIO1: u32 = 1 << 11;
    pub const TIMER0: u32 = 1 << 12;
    pub const TIMER1: u32 = 1 << 13;
    pub const TRNG: u32 = 1 << 14;
    pub const TBMAN: u32 = 1 << 15;
    pub const PLL_USB: u32 = 1 << 16;
    pub const PLL_SYS: u32 = 1 << 17;
}

/// Reset peripheral names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetPeripheral {
    Usbctrl,
    Uart0,
    Uart1,
    Spi0,
    Spi1,
    I2c0,
    I2c1,
    Adc,
    Pwm,
    Sio,
    Pio0,
    Pio1,
    Timer0,
    Timer1,
    Hstx,
    Timer2,
    Timer3,
    Trng,
    Tbman,
    PllUsb,
    PllSys,
}

impl ResetPeripheral {
    pub fn reset_bit(&self) -> u32 {
        match self {
            Self::Usbctrl => reset::USBCTRL,
            Self::Uart0 => reset::UART0,
            Self::Uart1 => reset::UART1,
            Self::Spi0 => reset::SPI0,
            Self::Spi1 => reset::SPI1,
            Self::I2c0 => reset::I2C0,
            Self::I2c1 => reset::I2C1,
            Self::Adc => reset::ADC,
            Self::Pwm => reset::PWM,
            Self::Sio => reset::SIO,
            Self::Pio0 => reset::PIO0,
            Self::Pio1 => reset::PIO1,
            Self::Timer0 => reset::TIMER0,
            Self::Timer1 => reset::TIMER1,
            Self::Hstx => reset::HSTX,
            Self::Timer2 => reset::TIMER2,
            Self::Timer3 => reset::TIMER3,
            Self::Trng => reset::TRNG,
            Self::Tbman => reset::TBMAN,
            Self::PllUsb => reset::PLL_USB,
            Self::PllSys => reset::PLL_SYS,
        }
    }

    pub fn done_bit(&self) -> u32 {
        match self {
            Self::Usbctrl => reset_done::USBCTRL,
            Self::Uart0 => reset_done::UART0,
            Self::Uart1 => reset_done::UART1,
            Self::Spi0 => reset_done::SPI0,
            Self::Spi1 => reset_done::SPI1,
            Self::I2c0 => reset_done::I2C0,
            Self::I2c1 => reset_done::I2C1,
            Self::Adc => reset_done::ADC,
            Self::Pwm => reset_done::PWM,
            Self::Sio => reset_done::SIO,
            Self::Pio0 => reset_done::PIO0,
            Self::Pio1 => reset_done::PIO1,
            Self::Timer0 => reset_done::TIMER0,
            Self::Timer1 => reset_done::TIMER1,
            Self::Hstx => 0,
            Self::Timer2 => 0,
            Self::Timer3 => 0,
            Self::Trng => reset_done::TRNG,
            Self::Tbman => reset_done::TBMAN,
            Self::PllUsb => reset_done::PLL_USB,
            Self::PllSys => reset_done::PLL_SYS,
        }
    }
}

/// Reset controller.
#[derive(Debug)]
pub struct Reset {
    /// Reset register (write 1 to assert reset, read returns done status).
    reset: u32,
    /// Watchdog select register.
    wdsel: u32,
    /// Reset done status register.
    reset_done: u32,
    /// Reset done for TBMAN.
    reset_done_tbman: u32,
    /// Pending resets (internal).
    pending_reset: u32,
    /// Cycle counter for reset duration.
    reset_cycles: u32,
}

impl Default for Reset {
    fn default() -> Self {
        Self::new()
    }
}

impl Reset {
    /// Create a new Reset controller.
    pub fn new() -> Self {
        Self {
            reset: 0,
            wdsel: 0,
            reset_done: 0xFFFFFFFF,
            reset_done_tbman: 1,
            pending_reset: 0,
            reset_cycles: 0,
        }
    }

    /// Assert reset for a peripheral.
    pub fn assert_reset(&mut self, peripheral: ResetPeripheral) {
        let bit = peripheral.reset_bit();
        self.reset |= bit;
        self.pending_reset |= bit;
        self.reset_done &= !peripheral.done_bit();
        self.reset_cycles = 0;
    }

    /// Deassert reset for a peripheral.
    pub fn deassert_reset(&mut self, peripheral: ResetPeripheral) {
        let bit = peripheral.reset_bit();
        self.reset &= !bit;
        self.reset_done |= peripheral.done_bit();
    }

    /// Reset a peripheral (assert then deassert).
    pub fn reset_device(&mut self, peripheral: ResetPeripheral) {
        self.assert_reset(peripheral);
        self.deassert_reset(peripheral);
    }

    /// Check if a peripheral is in reset.
    pub fn is_in_reset(&self, peripheral: ResetPeripheral) -> bool {
        (self.reset & peripheral.reset_bit()) != 0
    }

    /// Check if a peripheral reset is done.
    pub fn is_reset_done(&self, peripheral: ResetPeripheral) -> bool {
        (self.reset_done & peripheral.done_bit()) != 0
    }

    /// Set watchdog select for a peripheral.
    pub fn set_wdsel(&mut self, peripheral: ResetPeripheral, enable: bool) {
        let wdsel_bit = match peripheral {
            ResetPeripheral::Usbctrl => wdsel::USBCTRL,
            ResetPeripheral::Uart0 => wdsel::UART0,
            ResetPeripheral::Uart1 => wdsel::UART1,
            ResetPeripheral::Spi0 => wdsel::SPI0,
            ResetPeripheral::Spi1 => wdsel::SPI1,
            ResetPeripheral::I2c0 => wdsel::I2C0,
            ResetPeripheral::I2c1 => wdsel::I2C1,
            ResetPeripheral::Adc => wdsel::ADC,
            ResetPeripheral::Pwm => wdsel::PWM,
            ResetPeripheral::Sio => wdsel::SIO,
            ResetPeripheral::Pio0 => wdsel::PIO0,
            ResetPeripheral::Pio1 => wdsel::PIO1,
            ResetPeripheral::Timer0 => wdsel::TIMER0,
            ResetPeripheral::Timer1 => wdsel::TIMER1,
            ResetPeripheral::Trng => wdsel::TRNG,
            ResetPeripheral::Tbman => wdsel::TBMAN,
            ResetPeripheral::PllUsb => wdsel::PLL_USB,
            ResetPeripheral::PllSys => wdsel::PLL_SYS,
            _ => 0,
        };

        if enable {
            self.wdsel |= wdsel_bit;
        } else {
            self.wdsel &= !wdsel_bit;
        }
    }

    /// Perform watchdog reset (resets all peripherals selected in WDSEL).
    pub fn watchdog_reset(&mut self) {
        self.reset |= self.wdsel;
        self.pending_reset = self.wdsel;
        self.reset_done = 0;
    }

    /// Tick the reset controller (process pending resets).
    pub fn tick(&mut self) {
        if self.pending_reset != 0 {
            self.reset_cycles += 1;
            if self.reset_cycles >= 2 {
                self.reset_done = 0xFFFFFFFF;
                self.reset_done_tbman = 1;
                self.pending_reset = 0;
                self.reset_cycles = 0;
            }
        }
    }
}

impl Device for Reset {
    fn id(&self) -> DeviceId {
        DeviceId::RESETS
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - RESETS_BASE;

        match offset {
            regs::RESET => Ok(self.reset),
            regs::WDSEL => Ok(self.wdsel),
            regs::RESET_DONE => Ok(self.reset_done),
            regs::RESET_DONE_TBMAN => Ok(self.reset_done_tbman),
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - RESETS_BASE;

        match offset {
            regs::RESET => {
                self.reset = value;
                if value != 0 {
                    self.pending_reset = value;
                    self.reset_done &= !value;
                    self.reset_cycles = 0;
                } else {
                    self.pending_reset = 0;
                    self.reset_done = 0xFFFFFFFF;
                }
            }
            regs::WDSEL => {
                self.wdsel = value;
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
    fn test_reset_creation() {
        let reset = Reset::new();
        assert_eq!(reset.reset, 0);
        assert_eq!(reset.reset_done, 0xFFFFFFFF);
    }

    #[test]
    fn test_reset_assert() {
        let mut reset = Reset::new();
        
        reset.assert_reset(ResetPeripheral::Uart0);
        assert!(reset.is_in_reset(ResetPeripheral::Uart0));
        assert!(!reset.is_reset_done(ResetPeripheral::Uart0));
    }

    #[test]
    fn test_reset_deassert() {
        let mut reset = Reset::new();
        
        reset.assert_reset(ResetPeripheral::Spi0);
        reset.deassert_reset(ResetPeripheral::Spi0);
        
        assert!(!reset.is_in_reset(ResetPeripheral::Spi0));
        assert!(reset.is_reset_done(ResetPeripheral::Spi0));
    }

    #[test]
    fn test_reset_device() {
        let mut reset = Reset::new();
        
        reset.reset_device(ResetPeripheral::I2c0);
        
        assert!(!reset.is_in_reset(ResetPeripheral::I2c0));
        assert!(reset.is_reset_done(ResetPeripheral::I2c0));
    }

    #[test]
    fn test_reset_register() {
        let mut reset = Reset::new();
        
        reset.write(RESETS_BASE + regs::RESET, reset::UART1).unwrap();
        assert_eq!(reset.read(RESETS_BASE + regs::RESET).unwrap(), reset::UART1);
        
        reset.write(RESETS_BASE + regs::RESET, 0).unwrap();
        assert_eq!(reset.read(RESETS_BASE + regs::RESET).unwrap(), 0);
    }

    #[test]
    fn test_wdsel_register() {
        let mut reset = Reset::new();
        
        reset.write(RESETS_BASE + regs::WDSEL, wdsel::PIO0 | wdsel::PIO1).unwrap();
        assert_eq!(reset.read(RESETS_BASE + regs::WDSEL).unwrap(), wdsel::PIO0 | wdsel::PIO1);
    }

    #[test]
    fn test_reset_done_status() {
        let mut reset = Reset::new();
        
        assert!(reset.is_reset_done(ResetPeripheral::Uart0));
        
        reset.assert_reset(ResetPeripheral::Uart0);
        assert!(!reset.is_reset_done(ResetPeripheral::Uart0));
        
        reset.deassert_reset(ResetPeripheral::Uart0);
        assert!(reset.is_reset_done(ResetPeripheral::Uart0));
    }

    #[test]
    fn test_watchdog_reset() {
        let mut reset = Reset::new();
        
        reset.set_wdsel(ResetPeripheral::Pio0, true);
        reset.set_wdsel(ResetPeripheral::Pio1, true);
        
        reset.watchdog_reset();
        
        assert!(reset.is_in_reset(ResetPeripheral::Pio0));
        assert!(reset.is_in_reset(ResetPeripheral::Pio1));
    }

    #[test]
    fn test_multiple_resets() {
        let mut reset = Reset::new();
        
        reset.assert_reset(ResetPeripheral::Uart0);
        reset.assert_reset(ResetPeripheral::Spi0);
        reset.assert_reset(ResetPeripheral::I2c0);
        
        assert!(reset.is_in_reset(ResetPeripheral::Uart0));
        assert!(reset.is_in_reset(ResetPeripheral::Spi0));
        assert!(reset.is_in_reset(ResetPeripheral::I2c0));
        
        assert!(!reset.is_in_reset(ResetPeripheral::Uart1));
        assert!(!reset.is_in_reset(ResetPeripheral::Spi1));
    }

    #[test]
    fn test_reset_tick() {
        let mut reset = Reset::new();
        
        reset.assert_reset(ResetPeripheral::Adc);
        assert!(!reset.is_reset_done(ResetPeripheral::Adc));
        
        reset.tick();
        assert!(!reset.is_reset_done(ResetPeripheral::Adc));
        
        reset.tick();
        assert!(reset.is_reset_done(ResetPeripheral::Adc));
    }

    #[test]
    fn test_reset_all_peripherals() {
        let mut reset = Reset::new();
        
        let all_resets = reset::UART0 | reset::UART1 | reset::SPI0 | reset::SPI1 
            | reset::I2C0 | reset::I2C1 | reset::ADC | reset::PWM
            | reset::PIO0 | reset::PIO1 | reset::TIMER0 | reset::TIMER1;
        
        reset.write(RESETS_BASE + regs::RESET, all_resets).unwrap();
        
        assert!(reset.is_in_reset(ResetPeripheral::Uart0));
        assert!(reset.is_in_reset(ResetPeripheral::Spi0));
        assert!(reset.is_in_reset(ResetPeripheral::I2c0));
    }

    #[test]
    fn test_reset_peripheral_bits() {
        assert_eq!(ResetPeripheral::Uart0.reset_bit(), reset::UART0);
        assert_eq!(ResetPeripheral::Spi1.reset_bit(), reset::SPI1);
        assert_eq!(ResetPeripheral::Pio0.reset_bit(), reset::PIO0);
    }
}