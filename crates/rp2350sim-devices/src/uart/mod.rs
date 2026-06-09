//! UART device for RP2350.
//!
//! Implements the UART peripheral with full register support.

use std::sync::Mutex;

use rp2350sim_core::{Device, DeviceId, Result};

/// UART base addresses.
pub const UART0_BASE: u32 = 0x4003_4000;
pub const UART1_BASE: u32 = 0x4003_8000;

/// UART register offsets.
pub mod regs {
    pub const UARTDR: u32 = 0x000;
    pub const UARTRSR: u32 = 0x004;
    pub const UARTFR: u32 = 0x018;
    pub const UARTILPR: u32 = 0x020;
    pub const UARTIBRD: u32 = 0x024;
    pub const UARTFBRD: u32 = 0x028;
    pub const UARTLCR_H: u32 = 0x02C;
    pub const UARTCR: u32 = 0x030;
    pub const UARTIFLS: u32 = 0x034;
    pub const UARTIMSC: u32 = 0x038;
    pub const UARTRIS: u32 = 0x03C;
    pub const UARTMIS: u32 = 0x040;
    pub const UARTICR: u32 = 0x044;
    pub const UARTDMACR: u32 = 0x048;
    pub const UARTPERIPHID0: u32 = 0xFE0;
    pub const UARTPERIPHID1: u32 = 0xFE4;
    pub const UARTPERIPHID2: u32 = 0xFE8;
    pub const UARTPERIPHID3: u32 = 0xFEC;
    pub const UARTPCELLID0: u32 = 0xFF0;
    pub const UARTPCELLID1: u32 = 0xFF4;
    pub const UARTPCELLID2: u32 = 0xFF8;
    pub const UARTPCELLID3: u32 = 0xFFC;
}

/// UART flag register bits.
pub mod fr {
    pub const CTS: u32 = 1 << 0;
    pub const DSR: u32 = 1 << 1;
    pub const DCD: u32 = 1 << 2;
    pub const BUSY: u32 = 1 << 3;
    pub const RXFE: u32 = 1 << 4;
    pub const TXFF: u32 = 1 << 5;
    pub const RXFF: u32 = 1 << 6;
    pub const TXFE: u32 = 1 << 7;
    pub const RI: u32 = 1 << 8;
}

/// UART control register bits.
pub mod cr {
    pub const UARTEN: u32 = 1 << 0;
    pub const SIREN: u32 = 1 << 1;
    pub const SIRLP: u32 = 1 << 2;
    pub const LBE: u32 = 1 << 7;
    pub const TXE: u32 = 1 << 8;
    pub const RXE: u32 = 1 << 9;
    pub const DTR: u32 = 1 << 10;
    pub const RTS: u32 = 1 << 11;
    pub const OUT1: u32 = 1 << 12;
    pub const OUT2: u32 = 1 << 13;
    pub const RTSEN: u32 = 1 << 14;
    pub const CTSEN: u32 = 1 << 15;
}

/// UART line control register bits.
pub mod lcr_h {
    pub const BRK: u32 = 1 << 0;
    pub const PEN: u32 = 1 << 1;
    pub const EPS: u32 = 1 << 2;
    pub const STP2: u32 = 1 << 3;
    pub const FEN: u32 = 1 << 4;
    pub const WLEN_5: u32 = 0 << 5;
    pub const WLEN_6: u32 = 1 << 5;
    pub const WLEN_7: u32 = 2 << 5;
    pub const WLEN_8: u32 = 3 << 5;
    pub const WLEN_MASK: u32 = 3 << 5;
    pub const SPS: u32 = 1 << 7;
}

/// UART interrupt bits.
pub mod int_bits {
    pub const RIRMIS: u32 = 1 << 0;
    pub const CTSMIS: u32 = 1 << 1;
    pub const DCDMIS: u32 = 1 << 2;
    pub const DSRMIS: u32 = 1 << 3;
    pub const RXIM: u32 = 1 << 4;
    pub const TXIM: u32 = 1 << 5;
    pub const RTIM: u32 = 1 << 6;
    pub const FEIM: u32 = 1 << 7;
    pub const PEIM: u32 = 1 << 8;
    pub const BEIM: u32 = 1 << 9;
    pub const OEIM: u32 = 1 << 10;
}

/// TX/RX FIFO.
const FIFO_SIZE: usize = 32;

/// UART device.
pub struct Uart {
    /// Base address.
    base: u32,
    /// Data register (write to TX, read from RX).
    dr: u32,
    /// Receive status register / error clear.
    rsr: u32,
    /// Flag register.
    fr: u32,
    /// IrDA low-power counter.
    ilpr: u32,
    /// Integer baud rate divisor.
    ibrd: u32,
    /// Fractional baud rate divisor.
    fbrd: u32,
    /// Line control register.
    lcr_h: u32,
    /// Control register.
    cr: u32,
    /// Interrupt FIFO level select.
    ifls: u32,
    /// Interrupt mask set/clear.
    imsc: u32,
    /// Raw interrupt status.
    ris: u32,
    /// Masked interrupt status.
    mis: u32,
    /// DMA control.
    dmacr: u32,
    /// TX FIFO.
    tx_fifo: Vec<u8>,
    /// RX FIFO.
    rx_fifo: Vec<u8>,
    /// TX callback.
    tx_callback: Option<Mutex<Box<dyn FnMut(u8) + Send>>>,
    /// RX callback.
    rx_callback: Option<Mutex<Box<dyn FnMut() -> Option<u8> + Send>>>,
}

impl std::fmt::Debug for Uart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Uart")
            .field("base", &self.base)
            .field("dr", &self.dr)
            .field("fr", &self.fr)
            .field("tx_fifo_len", &self.tx_fifo.len())
            .field("rx_fifo_len", &self.rx_fifo.len())
            .field("tx_callback", &self.tx_callback.is_some())
            .field("rx_callback", &self.rx_callback.is_some())
            .finish()
    }
}

impl Default for Uart {
    fn default() -> Self {
        Self::new(UART0_BASE)
    }
}

impl Uart {
    /// Create a new UART device.
    pub fn new(base: u32) -> Self {
        Self {
            base,
            dr: 0,
            rsr: 0,
            fr: fr::TXFE | fr::RXFE, // TX empty, RX empty
            ilpr: 0,
            ibrd: 0,
            fbrd: 0,
            lcr_h: 0,
            cr: 0,
            ifls: 0x12, // Default FIFO levels
            imsc: 0,
            ris: 0,
            mis: 0,
            dmacr: 0,
            tx_fifo: Vec::with_capacity(FIFO_SIZE),
            rx_fifo: Vec::with_capacity(FIFO_SIZE),
            tx_callback: None,
            rx_callback: None,
        }
    }

    /// Create UART0.
    pub fn uart0() -> Self {
        Self::new(UART0_BASE)
    }

    /// Create UART1.
    pub fn uart1() -> Self {
        Self::new(UART1_BASE)
    }

    /// Check if UART is enabled.
    pub fn is_enabled(&self) -> bool {
        (self.cr & cr::UARTEN) != 0
    }

    /// Check if TX is enabled.
    pub fn is_tx_enabled(&self) -> bool {
        (self.cr & cr::TXE) != 0
    }

    /// Check if RX is enabled.
    pub fn is_rx_enabled(&self) -> bool {
        (self.cr & cr::RXE) != 0
    }

    /// Check if FIFOs are enabled.
    pub fn is_fifo_enabled(&self) -> bool {
        (self.lcr_h & lcr_h::FEN) != 0
    }

    /// Get word length (5-8 bits).
    pub fn get_word_length(&self) -> u8 {
        match self.lcr_h & lcr_h::WLEN_MASK {
            lcr_h::WLEN_5 => 5,
            lcr_h::WLEN_6 => 6,
            lcr_h::WLEN_7 => 7,
            lcr_h::WLEN_8 => 8,
            _ => 8,
        }
    }

    /// Check if parity is enabled.
    pub fn is_parity_enabled(&self) -> bool {
        (self.lcr_h & lcr_h::PEN) != 0
    }

    /// Check if parity is even.
    pub fn is_even_parity(&self) -> bool {
        (self.lcr_h & lcr_h::EPS) != 0
    }

    /// Check if two stop bits.
    pub fn is_two_stop_bits(&self) -> bool {
        (self.lcr_h & lcr_h::STP2) != 0
    }

    /// Calculate baud rate.
    pub fn calculate_baud_rate(&self, clock_freq: u32) -> u32 {
        if self.ibrd == 0 {
            return 0;
        }
        let divisor = (self.ibrd << 6) | (self.fbrd & 0x3F);
        clock_freq / (16 * divisor / 64)
    }

    /// Enable the UART.
    pub fn enable(&mut self) {
        self.cr |= cr::UARTEN;
    }

    /// Enable TX.
    pub fn enable_tx(&mut self) {
        self.cr |= cr::UARTEN | cr::TXE;
    }

    /// Enable RX.
    pub fn enable_rx(&mut self) {
        self.cr |= cr::UARTEN | cr::RXE;
    }

    /// Set TX callback.
    pub fn set_tx_callback<F: FnMut(u8) + Send + 'static>(&mut self, callback: F) {
        self.tx_callback = Some(Mutex::new(Box::new(callback)));
    }

    /// Set RX callback.
    pub fn set_rx_callback<F: FnMut() -> Option<u8> + Send + 'static>(&mut self, callback: F) {
        self.rx_callback = Some(Mutex::new(Box::new(callback)));
    }

    /// Write a byte to the TX FIFO.
    pub fn write_byte(&mut self, byte: u8) {
        if !self.is_enabled() || !self.is_tx_enabled() {
            return;
        }

        if self.tx_fifo.len() < FIFO_SIZE {
            self.tx_fifo.push(byte);
            self.update_fr();

            // Trigger callback
            if let Some(ref callback) = self.tx_callback {
                if let Ok(mut cb) = callback.lock() {
                    cb(byte);
                }
            }

            // Set TX interrupt if enabled
            if (self.imsc & int_bits::TXIM) != 0 {
                self.ris |= int_bits::TXIM;
                self.update_mis();
            }
        }
    }

    /// Read a byte from the RX FIFO.
    pub fn read_byte(&mut self) -> Option<u8> {
        if !self.is_enabled() || !self.is_rx_enabled() {
            return None;
        }

        let byte = self.rx_fifo.pop();
        self.update_fr();

        // Set RX interrupt if enabled
        if (self.imsc & int_bits::RXIM) != 0 && !self.rx_fifo.is_empty() {
            self.ris |= int_bits::RXIM;
            self.update_mis();
        }

        byte
    }

    /// Push a byte into the RX FIFO (external input).
    pub fn push_rx(&mut self, byte: u8) {
        if !self.is_enabled() || !self.is_rx_enabled() {
            return;
        }

        if self.rx_fifo.len() < FIFO_SIZE {
            self.rx_fifo.push(byte);
            self.update_fr();

            // Set RX interrupt if enabled
            if (self.imsc & int_bits::RXIM) != 0 {
                self.ris |= int_bits::RXIM;
                self.update_mis();
            }
        } else {
            // Overrun error
            self.ris |= int_bits::OEIM;
            self.update_mis();
        }
    }

    /// Pop a byte from the TX FIFO (for external reading of transmitted data).
    pub fn pop_tx(&mut self) -> Option<u8> {
        if self.tx_fifo.is_empty() {
            return None;
        }
        let byte = self.tx_fifo.remove(0);
        self.update_fr();
        Some(byte)
    }

    /// Check if TX FIFO has data.
    pub fn has_tx_data(&self) -> bool {
        !self.tx_fifo.is_empty()
    }

    /// Get TX FIFO length.
    pub fn tx_len(&self) -> usize {
        self.tx_fifo.len()
    }

    /// Update flag register.
    fn update_fr(&mut self) {
        self.fr &= !(fr::TXFE | fr::TXFF | fr::RXFE | fr::RXFF | fr::BUSY);

        if self.tx_fifo.is_empty() {
            self.fr |= fr::TXFE;
        }
        if self.tx_fifo.len() >= FIFO_SIZE {
            self.fr |= fr::TXFF;
        }
        if self.rx_fifo.is_empty() {
            self.fr |= fr::RXFE;
        }
        if self.rx_fifo.len() >= FIFO_SIZE {
            self.fr |= fr::RXFF;
        }
        if !self.tx_fifo.is_empty() || !self.rx_fifo.is_empty() {
            self.fr |= fr::BUSY;
        }
    }

    /// Update masked interrupt status.
    fn update_mis(&mut self) {
        self.mis = self.ris & self.imsc;
    }

    /// Check if there's a pending interrupt.
    pub fn has_interrupt(&self) -> bool {
        self.mis != 0
    }

    /// Get the base address.
    pub fn base(&self) -> u32 {
        self.base
    }
}

impl Device for Uart {
    fn id(&self) -> DeviceId {
        DeviceId::UART
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - self.base;

        match offset {
            regs::UARTDR => {
                if let Some(byte) = self.read_byte() {
                    Ok(byte as u32)
                } else {
                    Ok(0)
                }
            }
            regs::UARTRSR => Ok(self.rsr),
            regs::UARTFR => Ok(self.fr),
            regs::UARTILPR => Ok(self.ilpr),
            regs::UARTIBRD => Ok(self.ibrd),
            regs::UARTFBRD => Ok(self.fbrd),
            regs::UARTLCR_H => Ok(self.lcr_h),
            regs::UARTCR => Ok(self.cr),
            regs::UARTIFLS => Ok(self.ifls),
            regs::UARTIMSC => Ok(self.imsc),
            regs::UARTRIS => Ok(self.ris),
            regs::UARTMIS => Ok(self.mis),
            regs::UARTDMACR => Ok(self.dmacr),
            regs::UARTPERIPHID0 => Ok(0x11),
            regs::UARTPERIPHID1 => Ok(0x10),
            regs::UARTPERIPHID2 => Ok(0x14),
            regs::UARTPERIPHID3 => Ok(0x00),
            regs::UARTPCELLID0 => Ok(0x0D),
            regs::UARTPCELLID1 => Ok(0xF0),
            regs::UARTPCELLID2 => Ok(0x05),
            regs::UARTPCELLID3 => Ok(0xB1),
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - self.base;

        match offset {
            regs::UARTDR => {
                self.write_byte(value as u8);
            }
            regs::UARTRSR => {
                // Write to clear errors
                self.rsr = 0;
            }
            regs::UARTILPR => {
                self.ilpr = value & 0xFF;
            }
            regs::UARTIBRD => {
                self.ibrd = value & 0xFFFF;
            }
            regs::UARTFBRD => {
                self.fbrd = value & 0x3F;
            }
            regs::UARTLCR_H => {
                self.lcr_h = value & 0xFF;
                // Writing to LCR_H clears the FIFOs
                self.tx_fifo.clear();
                self.rx_fifo.clear();
                self.update_fr();
            }
            regs::UARTCR => {
                self.cr = value & 0xFFFF;
            }
            regs::UARTIFLS => {
                self.ifls = value & 0x3F;
            }
            regs::UARTIMSC => {
                self.imsc = value & 0x7FF;
                self.update_mis();
            }
            regs::UARTICR => {
                // Write to clear interrupts
                self.ris &= !value;
                self.update_mis();
            }
            regs::UARTDMACR => {
                self.dmacr = value & 0x07;
            }
            _ => {}
        }

        Ok(())
    }

    fn reset(&mut self) {
        let base = self.base;
        *self = Self::new(base);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uart_creation() {
        let uart = Uart::uart0();
        assert_eq!(uart.base(), UART0_BASE);
    }

    #[test]
    fn test_uart_enable() {
        let mut uart = Uart::uart0();
        assert!(!uart.is_enabled());
        uart.cr = cr::UARTEN;
        assert!(uart.is_enabled());
    }

    #[test]
    fn test_uart_tx() {
        let mut uart = Uart::uart0();
        uart.cr = cr::UARTEN | cr::TXE;

        // Test TX FIFO directly
        uart.write_byte(0x41);
        assert_eq!(uart.tx_fifo.len(), 1);
        
        uart.write_byte(0x42);
        assert_eq!(uart.tx_fifo.len(), 2);
    }

    #[test]
    fn test_uart_rx() {
        let mut uart = Uart::uart0();
        uart.cr = cr::UARTEN | cr::RXE;

        uart.push_rx(0x42);
        assert_eq!(uart.rx_fifo.len(), 1);

        let byte = uart.read_byte();
        assert_eq!(byte, Some(0x42));
    }

    #[test]
    fn test_uart_baud_rate() {
        let mut uart = Uart::uart0();
        uart.ibrd = 26;
        uart.fbrd = 3;

        // With 48MHz clock, this should give ~115200 baud
        let baud = uart.calculate_baud_rate(48_000_000);
        assert!(baud > 115000 && baud < 115400);
    }

    #[test]
    fn test_uart_word_length() {
        let mut uart = Uart::uart0();

        uart.lcr_h = lcr_h::WLEN_8;
        assert_eq!(uart.get_word_length(), 8);

        uart.lcr_h = lcr_h::WLEN_7;
        assert_eq!(uart.get_word_length(), 7);
    }

    #[test]
    fn test_uart_register_read_write() {
        let mut uart = Uart::uart0();

        // Test UARTIBRD register (integer baud rate)
        uart.write(UART0_BASE + regs::UARTIBRD, 26).unwrap();
        assert_eq!(uart.read(UART0_BASE + regs::UARTIBRD).unwrap(), 26);

        // Test UARTFBRD register (fractional baud rate)
        uart.write(UART0_BASE + regs::UARTFBRD, 3).unwrap();
        assert_eq!(uart.read(UART0_BASE + regs::UARTFBRD).unwrap(), 3);

        // Test UARTLCR_H register
        uart.write(UART0_BASE + regs::UARTLCR_H, lcr_h::WLEN_8).unwrap();
        assert_eq!(uart.read(UART0_BASE + regs::UARTLCR_H).unwrap(), lcr_h::WLEN_8);
    }

    #[test]
    fn test_uart_flag_register() {
        let mut uart = Uart::uart0();

        // Check initial flag status
        let fr = uart.read(UART0_BASE + regs::UARTFR).unwrap();
        // TX FIFO should be empty
        assert_eq!(fr & fr::TXFE, fr::TXFE);
        // RX FIFO should be empty
        assert_eq!(fr & fr::RXFE, fr::RXFE);
        // TX FIFO should not be full
        assert_eq!(fr & fr::TXFF, 0);
    }

    #[test]
    fn test_uart_interrupt_mask() {
        let mut uart = Uart::uart0();

        // Enable RX interrupt
        uart.write(UART0_BASE + regs::UARTIMSC, 0x10).unwrap(); // RXIM bit
        assert_eq!(uart.read(UART0_BASE + regs::UARTIMSC).unwrap(), 0x10);

        // Enable TX interrupt
        uart.write(UART0_BASE + regs::UARTIMSC, 0x20).unwrap(); // TXIM bit
        assert_eq!(uart.read(UART0_BASE + regs::UARTIMSC).unwrap(), 0x20);
    }

    #[test]
    fn test_uart_fifo_status() {
        let mut uart = Uart::uart0();
        uart.cr = cr::UARTEN | cr::TXE | cr::RXE;

        // Check TX FIFO status via flag register
        let fr = uart.read(UART0_BASE + regs::UARTFR).unwrap();
        assert_eq!(fr & fr::TXFE, fr::TXFE); // TX empty

        // Fill TX FIFO (32 entries)
        for i in 0..32 {
            uart.write_byte(i);
        }

        // Check TX FIFO is now full
        let fr = uart.read(UART0_BASE + regs::UARTFR).unwrap();
        assert_eq!(fr & fr::TXFF, fr::TXFF); // TX full

        // Check TX has data
        assert!(uart.has_tx_data());
    }

    #[test]
    fn test_uart_parity() {
        let mut uart = Uart::uart0();

        // Enable even parity
        uart.lcr_h = lcr_h::PEN | lcr_h::EPS;
        assert!(uart.is_parity_enabled());
        assert!(uart.is_even_parity());

        // Enable odd parity
        uart.lcr_h = lcr_h::PEN;
        assert!(uart.is_parity_enabled());
        assert!(!uart.is_even_parity());

        // Disable parity
        uart.lcr_h = 0;
        assert!(!uart.is_parity_enabled());
    }

    #[test]
    fn test_uart_stop_bits() {
        let mut uart = Uart::uart0();

        // 1 stop bit
        uart.lcr_h = 0;
        assert!(!uart.is_two_stop_bits());

        // 2 stop bits
        uart.lcr_h = lcr_h::STP2;
        assert!(uart.is_two_stop_bits());
    }
}