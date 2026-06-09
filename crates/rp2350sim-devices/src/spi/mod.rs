//! SPI device for RP2350.
//!
//! Implements the SPI peripheral with full register support.

use rp2350sim_core::{Device, DeviceId, Result};
use std::collections::VecDeque;

/// SPI base addresses.
pub const SPI0_BASE: u32 = 0x4003_C000;
pub const SPI1_BASE: u32 = 0x4004_0000;

/// SPI register offsets.
pub mod regs {
    pub const SSPCR0: u32 = 0x000;
    pub const SSPCR1: u32 = 0x004;
    pub const SSPDR: u32 = 0x008;
    pub const SSPSR: u32 = 0x00C;
    pub const SSPCPSR: u32 = 0x010;
    pub const SSPIMSC: u32 = 0x014;
    pub const SSPRIS: u32 = 0x018;
    pub const SSPMIS: u32 = 0x01C;
    pub const SSPICR: u32 = 0x020;
    pub const SSPDMACR: u32 = 0x024;
    pub const SSPPERIPHID0: u32 = 0xFE0;
    pub const SSPPERIPHID1: u32 = 0xFE4;
    pub const SSPPERIPHID2: u32 = 0xFE8;
    pub const SSPPERIPHID3: u32 = 0xFEC;
    pub const SSPPCELLID0: u32 = 0xFF0;
    pub const SSPPCELLID1: u32 = 0xFF4;
    pub const SSPPCELLID2: u32 = 0xFF8;
    pub const SSPPCELLID3: u32 = 0xFFC;
}

/// SSPCR0 bits.
pub mod cr0 {
    pub const DSS_MASK: u32 = 0x0F;
    pub const DSS_4BIT: u32 = 0x03;
    pub const DSS_5BIT: u32 = 0x04;
    pub const DSS_6BIT: u32 = 0x05;
    pub const DSS_7BIT: u32 = 0x06;
    pub const DSS_8BIT: u32 = 0x07;
    pub const DSS_9BIT: u32 = 0x08;
    pub const DSS_10BIT: u32 = 0x09;
    pub const DSS_11BIT: u32 = 0x0A;
    pub const DSS_12BIT: u32 = 0x0B;
    pub const DSS_13BIT: u32 = 0x0C;
    pub const DSS_14BIT: u32 = 0x0D;
    pub const DSS_15BIT: u32 = 0x0E;
    pub const DSS_16BIT: u32 = 0x0F;
    pub const FRF_MASK: u32 = 0x03 << 4;
    pub const FRF_MOTOROLA: u32 = 0x00 << 4;
    pub const FRF_TI: u32 = 0x01 << 4;
    pub const FRF_MICROWIRE: u32 = 0x02 << 4;
    pub const SPO: u32 = 1 << 6;
    pub const SPH: u32 = 1 << 7;
    pub const SCR_MASK: u32 = 0xFF << 8;
}

/// SSPCR1 bits.
pub mod cr1 {
    pub const LBM: u32 = 1 << 0;
    pub const SSE: u32 = 1 << 1;
    pub const MS: u32 = 1 << 2;
    pub const SOD: u32 = 1 << 3;
}

/// SSPSR bits.
pub mod sr {
    pub const TFE: u32 = 1 << 0;  // Transmit FIFO empty
    pub const TNF: u32 = 1 << 1;  // Transmit FIFO not full
    pub const RNE: u32 = 1 << 2;  // Receive FIFO not empty
    pub const RFF: u32 = 1 << 3;  // Receive FIFO full
    pub const BSY: u32 = 1 << 4;  // Busy
}

/// SSPIMSC bits.
pub mod imsc {
    pub const RORIM: u32 = 1 << 0;
    pub const RTIM: u32 = 1 << 1;
    pub const RXIM: u32 = 1 << 2;
    pub const TXIM: u32 = 1 << 3;
}

/// FIFO depth.
const FIFO_SIZE: usize = 8;

/// SPI device.
pub struct Spi {
    /// Base address.
    base: u32,
    /// Control register 0.
    cr0: u32,
    /// Control register 1.
    cr1: u32,
    /// Status register.
    sr: u32,
    /// Clock prescale register.
    cpsr: u32,
    /// Interrupt mask.
    imsc: u32,
    /// Raw interrupt status.
    ris: u32,
    /// DMA control.
    dmacr: u32,
    /// TX FIFO.
    tx_fifo: VecDeque<u16>,
    /// RX FIFO.
    rx_fifo: VecDeque<u16>,
    /// Loopback mode.
    loopback: bool,
    /// Virtual slave device.
    slave: Option<Box<dyn SpiSlave + Send + Sync>>,
}

impl std::fmt::Debug for Spi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Spi")
            .field("base", &self.base)
            .field("cr0", &self.cr0)
            .field("cr1", &self.cr1)
            .field("slave", &self.slave.as_ref().map(|_| "..."))
            .finish()
    }
}

impl Default for Spi {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Spi {
    /// Create a new SPI device.
    pub fn new(id: u8) -> Self {
        Self {
            base: if id == 0 { SPI0_BASE } else { SPI1_BASE },
            cr0: 0,
            cr1: 0,
            sr: sr::TFE | sr::TNF, // TX empty, TX not full
            cpsr: 0,
            imsc: 0,
            ris: imsc::TXIM, // TX interrupt set since TX FIFO is empty
            dmacr: 0,
            tx_fifo: VecDeque::with_capacity(FIFO_SIZE),
            rx_fifo: VecDeque::with_capacity(FIFO_SIZE),
            loopback: false,
            slave: None,
        }
    }

    /// Create SPI0.
    pub fn spi0() -> Self {
        Self::new(0)
    }

    /// Create SPI1.
    pub fn spi1() -> Self {
        Self::new(1)
    }

    /// Get base address.
    pub fn base(&self) -> u32 {
        self.base
    }

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        (self.cr1 & cr1::SSE) != 0
    }

    /// Check if master mode.
    pub fn is_master(&self) -> bool {
        (self.cr1 & cr1::MS) == 0
    }

    /// Get clock polarity (CPOL).
    pub fn get_clock_polarity(&self) -> bool {
        (self.cr0 & cr0::SPO) != 0
    }

    /// Set clock polarity.
    pub fn set_clock_polarity(&mut self, value: bool) {
        if value {
            self.cr0 |= cr0::SPO;
        } else {
            self.cr0 &= !cr0::SPO;
        }
    }

    /// Get clock phase (CPHA).
    pub fn get_clock_phase(&self) -> bool {
        (self.cr0 & cr0::SPH) != 0
    }

    /// Set clock phase.
    pub fn set_clock_phase(&mut self, value: bool) {
        if value {
            self.cr0 |= cr0::SPH;
        } else {
            self.cr0 &= !cr0::SPH;
        }
    }

    /// Get data size (4-16 bits).
    pub fn get_data_size(&self) -> u8 {
        ((self.cr0 & cr0::DSS_MASK) as u8) + 1
    }

    /// Get clock divider.
    pub fn get_clock_divider(&self) -> u32 {
        self.cpsr
    }

    /// Set loopback mode.
    pub fn set_loopback(&mut self, value: bool) {
        self.loopback = value;
        if value {
            self.cr1 |= cr1::LBM;
        } else {
            self.cr1 &= !cr1::LBM;
        }
    }

    /// Set chip select.
    pub fn set_cs(&mut self, _cs: usize, _value: bool) {
        // CS handling would be done via GPIO
    }

    /// Get chip select.
    pub fn get_cs(&self, _cs: usize) -> bool {
        true // Default high (inactive)
    }

    /// Transfer a single byte/word.
    pub fn transfer(&mut self, tx_data: u16) -> Option<u16> {
        if !self.is_enabled() {
            return None;
        }

        // In loopback mode, return TX data
        if self.loopback {
            return Some(tx_data);
        }

        // Send to slave device
        if let Some(ref mut slave) = self.slave {
            let rx = slave.transfer(tx_data);
            return Some(rx);
        }

        // No slave - return 0
        Some(0)
    }

    /// Set slave device.
    pub fn set_slave<S: SpiSlave + Send + 'static>(&mut self, slave: S) {
        self.slave = Some(Box::new(slave));
    }

    /// Update status register.
    fn update_sr(&mut self) {
        self.sr = 0;
        if self.tx_fifo.is_empty() {
            self.sr |= sr::TFE;
        }
        if self.tx_fifo.len() < FIFO_SIZE {
            self.sr |= sr::TNF;
        }
        if !self.rx_fifo.is_empty() {
            self.sr |= sr::RNE;
        }
        if self.rx_fifo.len() >= FIFO_SIZE {
            self.sr |= sr::RFF;
        }
        if !self.tx_fifo.is_empty() || !self.rx_fifo.is_empty() {
            self.sr |= sr::BSY;
        }

        // Update raw interrupt status
        // TX interrupt: TX FIFO is half empty or less (<= 4 entries)
        if self.tx_fifo.len() <= FIFO_SIZE / 2 {
            self.ris |= imsc::TXIM;
        } else {
            self.ris &= !imsc::TXIM;
        }
        // RX interrupt: RX FIFO is half full or more (>= 4 entries)
        if self.rx_fifo.len() >= FIFO_SIZE / 2 {
            self.ris |= imsc::RXIM;
        } else {
            self.ris &= !imsc::RXIM;
        }
    }

    /// Update masked interrupt status.
    #[allow(dead_code)]
    fn update_mis(&mut self) {
        // mis = ris & imsc
    }

    /// Check if there's a pending interrupt.
    pub fn has_interrupt(&self) -> bool {
        (self.ris & self.imsc) != 0
    }
}

impl Device for Spi {
    fn id(&self) -> DeviceId {
        DeviceId::SPI
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - self.base;

        match offset {
            regs::SSPCR0 => Ok(self.cr0),
            regs::SSPCR1 => Ok(self.cr1),
            regs::SSPDR => {
                let data = self.rx_fifo.pop_front().unwrap_or(0);
                self.update_sr();
                Ok(data as u32)
            }
            regs::SSPSR => Ok(self.sr),
            regs::SSPCPSR => Ok(self.cpsr),
            regs::SSPIMSC => Ok(self.imsc),
            regs::SSPRIS => Ok(self.ris),
            regs::SSPMIS => Ok(self.ris & self.imsc),
            regs::SSPDMACR => Ok(self.dmacr),
            regs::SSPPERIPHID0 => Ok(0x22),
            regs::SSPPERIPHID1 => Ok(0x10),
            regs::SSPPERIPHID2 => Ok(0x34),
            regs::SSPPERIPHID3 => Ok(0x00),
            regs::SSPPCELLID0 => Ok(0x0D),
            regs::SSPPCELLID1 => Ok(0xF0),
            regs::SSPPCELLID2 => Ok(0x05),
            regs::SSPPCELLID3 => Ok(0xB1),
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - self.base;

        match offset {
            regs::SSPCR0 => {
                self.cr0 = value;
            }
            regs::SSPCR1 => {
                self.cr1 = value & 0x0F;
                self.loopback = (value & cr1::LBM) != 0;
            }
            regs::SSPDR => {
                if self.tx_fifo.len() < FIFO_SIZE {
                    let tx_data = value as u16;
                    self.tx_fifo.push_back(tx_data);

                    // Perform transfer
                    if let Some(rx_data) = self.transfer(tx_data) {
                        self.rx_fifo.push_back(rx_data);
                    }

                    self.update_sr();
                }
            }
            regs::SSPCPSR => {
                self.cpsr = value & 0xFE;
            }
            regs::SSPIMSC => {
                self.imsc = value & 0x0F;
            }
            regs::SSPICR => {
                // Write to clear interrupts
                self.ris &= !value;
            }
            regs::SSPDMACR => {
                self.dmacr = value & 0x03;
            }
            _ => {}
        }

        Ok(())
    }

    fn reset(&mut self) {
        let base = self.base;
        *self = Self {
            base,
            ..Self::new(if base == SPI0_BASE { 0 } else { 1 })
        };
    }
}

/// SPI slave device trait.
pub trait SpiSlave: Send + Sync {
    /// Transfer data (TX -> RX).
    fn transfer(&mut self, tx_data: u16) -> u16;
}

/// Loopback slave (returns TX data).
pub struct LoopbackSlave;

impl SpiSlave for LoopbackSlave {
    fn transfer(&mut self, tx_data: u16) -> u16 {
        tx_data
    }
}

/// SPI flash slave.
pub struct SpiFlash {
    data: Vec<u8>,
    address: u32,
    command: u8,
    state: u8,
}

impl SpiFlash {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0xFF; size],
            address: 0,
            command: 0,
            state: 0,
        }
    }

    pub fn load(&mut self, offset: usize, data: &[u8]) {
        let end = std::cmp::min(offset + data.len(), self.data.len());
        self.data[offset..end].copy_from_slice(&data[..end - offset]);
    }
}

impl SpiSlave for SpiFlash {
    fn transfer(&mut self, tx_data: u16) -> u16 {
        let byte = tx_data as u8;

        match self.state {
            0 => {
                self.command = byte;
                self.state = 1;
                self.address = 0;
            }
            1..=4 => {
                // Address bytes
                self.address = (self.address << 8) | (byte as u32);
                self.state += 1;
            }
            _ => {
                // Data phase
            }
        }

        // Return data from flash
        let addr = self.address as usize;
        if addr < self.data.len() {
            self.address = self.address.wrapping_add(1);
            self.data[addr] as u16
        } else {
            0xFF
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spi_creation() {
        let spi = Spi::spi0();
        assert_eq!(spi.base(), SPI0_BASE);
    }

    #[test]
    fn test_spi_enable() {
        let mut spi = Spi::spi0();
        assert!(!spi.is_enabled());
        spi.cr1 = cr1::SSE;
        assert!(spi.is_enabled());
    }

    #[test]
    fn test_spi_loopback() {
        let mut spi = Spi::spi0();
        spi.cr1 = cr1::SSE;
        spi.set_loopback(true);

        let rx = spi.transfer(0xAB);
        assert_eq!(rx, Some(0xAB));
    }

    #[test]
    fn test_spi_clock_polarity() {
        let mut spi = Spi::spi0();
        spi.set_clock_polarity(true);
        assert!(spi.get_clock_polarity());

        spi.set_clock_polarity(false);
        assert!(!spi.get_clock_polarity());
    }

    #[test]
    fn test_spi_clock_phase() {
        let mut spi = Spi::spi0();
        spi.set_clock_phase(true);
        assert!(spi.get_clock_phase());

        spi.set_clock_phase(false);
        assert!(!spi.get_clock_phase());
    }

    #[test]
    fn test_spi_fifo_operations() {
        let mut spi = Spi::spi0();
        spi.cr1 = cr1::SSE; // Enable SPI

        // Check initial status - TX FIFO should be empty
        let sr = spi.read(SPI0_BASE + regs::SSPSR).unwrap();
        assert_eq!(sr & sr::TFE, sr::TFE); // TX FIFO empty
        assert_eq!(sr & sr::TNF, sr::TNF); // TX FIFO not full

        // Write to TX FIFO via DR register
        spi.write(SPI0_BASE + regs::SSPDR, 0x55).unwrap();

        // Check status after write
        let sr = spi.read(SPI0_BASE + regs::SSPSR).unwrap();
        assert_eq!(sr & sr::TFE, 0); // TX FIFO not empty
    }

    #[test]
    fn test_spi_transfer_with_slave() {
        let mut spi = Spi::spi0();
        spi.cr1 = cr1::SSE; // Enable SPI

        // Create a simple slave that echoes back the received byte + 1
        struct EchoSlave;
        impl SpiSlave for EchoSlave {
            fn transfer(&mut self, data: u16) -> u16 {
                data.wrapping_add(1)
            }
        }

        spi.set_slave(EchoSlave);

        // Transfer should return slave's response
        let rx = spi.transfer(0x42);
        assert_eq!(rx, Some(0x43));
    }

    #[test]
    fn test_spi_register_read_write() {
        let mut spi = Spi::spi0();

        // Test CR0 register
        spi.write(SPI0_BASE + regs::SSPCR0, 0x0007).unwrap(); // 8-bit data
        assert_eq!(spi.read(SPI0_BASE + regs::SSPCR0).unwrap(), 0x0007);

        // Test CR1 register
        spi.write(SPI0_BASE + regs::SSPCR1, cr1::SSE).unwrap();
        assert_eq!(spi.read(SPI0_BASE + regs::SSPCR1).unwrap(), cr1::SSE);

        // Test CPSR register
        spi.write(SPI0_BASE + regs::SSPCPSR, 128).unwrap();
        assert_eq!(spi.read(SPI0_BASE + regs::SSPCPSR).unwrap(), 128);
    }

    #[test]
    fn test_spi_status_register() {
        let spi = Spi::spi0();

        // Check initial status (read doesn't require mutable)
        let sr = spi.sr;
        assert_eq!(sr & sr::TFE, sr::TFE); // TX FIFO empty
        assert_eq!(sr & sr::TNF, sr::TNF); // TX FIFO not full
        assert_eq!(sr & sr::RNE, 0);       // RX FIFO empty
    }

    #[test]
    fn test_spi_interrupt_status() {
        let mut spi = Spi::spi0();

        // Set interrupt mask
        spi.write(SPI0_BASE + regs::SSPIMSC, imsc::TXIM).unwrap();
        assert_eq!(spi.read(SPI0_BASE + regs::SSPIMSC).unwrap(), imsc::TXIM);

        // Check raw interrupt status
        let ris = spi.read(SPI0_BASE + regs::SSPRIS).unwrap();
        // TX interrupt should be set since TX FIFO is empty
        assert_eq!(ris & imsc::TXIM, imsc::TXIM);
    }
}