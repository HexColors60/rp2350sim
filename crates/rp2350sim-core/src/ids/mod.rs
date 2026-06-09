//! Identifier types for various simulator components.

use serde::{Deserialize, Serialize};
use std::fmt;

/// CPU core identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CoreId(pub u8);

impl CoreId {
    pub const CORE0: Self = Self(0);
    pub const CORE1: Self = Self(1);

    pub const fn new(id: u8) -> Self {
        Self(id)
    }

    pub const fn index(&self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for CoreId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Core{}", self.0)
    }
}

impl Default for CoreId {
    fn default() -> Self {
        Self::CORE0
    }
}

/// Device identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId(pub u16);

impl DeviceId {
    pub const GPIO: Self = Self(0);
    pub const UART: Self = Self(1);
    pub const SPI: Self = Self(2);
    pub const I2C: Self = Self(3);
    pub const PWM: Self = Self(4);
    pub const ADC: Self = Self(5);
    pub const USB: Self = Self(6);
    pub const PIO: Self = Self(7);
    pub const TIMER: Self = Self(8);
    pub const WATCHDOG: Self = Self(9);
    pub const CLOCKS: Self = Self(10);
    pub const RESETS: Self = Self(11);
    pub const WLAN: Self = Self(12);
    pub const DMA: Self = Self(13);
    pub const XIP: Self = Self(14);
    pub const I2S: Self = Self(15);
    pub const RTC: Self = Self(16);
    pub const PLL: Self = Self(17);
    pub const SHA256: Self = Self(18);
    pub const TRNG: Self = Self(19);
    pub const INTERP: Self = Self(20);
    pub const OTP: Self = Self(21);
    pub const HSTX: Self = Self(22);
    pub const POWMAN: Self = Self(23);
    pub const BOOTRAM: Self = Self(24);
    pub const SYSINFO: Self = Self(25);
    pub const SYSTICK: Self = Self(26);
    pub const CORESIGHT: Self = Self(27);
    pub const BUSCTRL: Self = Self(28);
    pub const MPU: Self = Self(29);
    pub const NVIC: Self = Self(30);
    pub const PLIC: Self = Self(31);

    pub const fn index(&self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::GPIO => write!(f, "GPIO"),
            Self::UART => write!(f, "UART"),
            Self::SPI => write!(f, "SPI"),
            Self::I2C => write!(f, "I2C"),
            Self::PWM => write!(f, "PWM"),
            Self::ADC => write!(f, "ADC"),
            Self::USB => write!(f, "USB"),
            Self::PIO => write!(f, "PIO"),
            Self::TIMER => write!(f, "TIMER"),
            Self::WATCHDOG => write!(f, "WATCHDOG"),
            Self::CLOCKS => write!(f, "CLOCKS"),
            Self::RESETS => write!(f, "RESETS"),
            Self::WLAN => write!(f, "WLAN"),
            Self::DMA => write!(f, "DMA"),
            Self::XIP => write!(f, "XIP"),
            Self::I2S => write!(f, "I2S"),
            Self::RTC => write!(f, "RTC"),
            Self::PLL => write!(f, "PLL"),
            Self::SHA256 => write!(f, "SHA256"),
            Self::TRNG => write!(f, "TRNG"),
            Self::INTERP => write!(f, "INTERP"),
            Self::OTP => write!(f, "OTP"),
            Self::HSTX => write!(f, "HSTX"),
            Self::POWMAN => write!(f, "POWMAN"),
            Self::BOOTRAM => write!(f, "BOOTRAM"),
            Self::SYSINFO => write!(f, "SYSINFO"),
            Self::SYSTICK => write!(f, "SYSTICK"),
            Self::CORESIGHT => write!(f, "CORESIGHT"),
            Self::BUSCTRL => write!(f, "BUSCTRL"),
            Self::MPU => write!(f, "MPU"),
            Self::NVIC => write!(f, "NVIC"),
            Self::PLIC => write!(f, "PLIC"),
            _ => write!(f, "Device{}", self.0),
        }
    }
}

/// Interrupt request identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IrqId(pub u8);

impl IrqId {
    // Timer interrupts
    pub const TIMER_0: Self = Self(0);
    pub const TIMER_1: Self = Self(1);
    pub const TIMER_2: Self = Self(2);
    pub const TIMER_3: Self = Self(3);

    // UART interrupts
    pub const UART0: Self = Self(20);
    pub const UART1: Self = Self(21);

    // GPIO interrupts
    pub const GPIO0: Self = Self(4);
    pub const GPIO1: Self = Self(5);
    pub const GPIO2: Self = Self(6);
    pub const GPIO3: Self = Self(7);

    // SPI interrupts
    pub const SPI0: Self = Self(18);
    pub const SPI1: Self = Self(19);

    // I2C interrupts
    pub const I2C0: Self = Self(23);
    pub const I2C1: Self = Self(24);

    // USB interrupts
    pub const USBCTRL: Self = Self(13);

    // PIO interrupts
    pub const PIO0_0: Self = Self(9);
    pub const PIO0_1: Self = Self(10);
    pub const PIO1_0: Self = Self(11);
    pub const PIO1_1: Self = Self(12);

    // ADC interrupts
    pub const ADC_FIFO: Self = Self(22);

    // PWM interrupts
    pub const PWM_WRAP: Self = Self(8);

    pub const fn index(&self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for IrqId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IRQ{}", self.0)
    }
}

/// PIO state machine identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PioSmId {
    /// PIO instance (0 or 1)
    pub pio: u8,
    /// State machine number (0-5)
    pub sm: u8,
}

impl PioSmId {
    pub const fn new(pio: u8, sm: u8) -> Self {
        Self { pio, sm }
    }

    pub const fn index(&self) -> usize {
        (self.pio as usize) * 6 + (self.sm as usize)
    }
}

impl fmt::Display for PioSmId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PIO{}.SM{}", self.pio, self.sm)
    }
}

/// GPIO pin identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PinId(pub u8);

impl PinId {
    pub const fn index(&self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for PinId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GPIO{}", self.0)
    }
}

/// UART identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UartId {
    Uart0,
    Uart1,
}

impl fmt::Display for UartId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uart0 => write!(f, "UART0"),
            Self::Uart1 => write!(f, "UART1"),
        }
    }
}

/// SPI identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpiId {
    Spi0,
    Spi1,
}

impl fmt::Display for SpiId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spi0 => write!(f, "SPI0"),
            Self::Spi1 => write!(f, "SPI1"),
        }
    }
}

/// I2C identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum I2cId {
    I2c0,
    I2c1,
}

impl fmt::Display for I2cId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::I2c0 => write!(f, "I2C0"),
            Self::I2c1 => write!(f, "I2C1"),
        }
    }
}