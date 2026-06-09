//! I2S (Inter-IC Sound) controller for RP2350.
//!
//! Implements the I2S peripheral for audio communication.

use rp2350sim_core::{Device, DeviceId, Result};

/// I2S base addresses.
pub const I2S0_BASE: u32 = 0x5000_8000;
pub const I2S1_BASE: u32 = 0x5000_C000;

/// I2S register offsets.
pub mod regs {
    pub const CTRL: u32 = 0x000;
    pub const FMT: u32 = 0x004;
    pub const INTCR: u32 = 0x008;
    pub const INTSR: u32 = 0x00C;
    pub const CLR: u32 = 0x010;
    pub const DMA: u32 = 0x014;
    pub const TXDR: u32 = 0x018;
    pub const RXDR: u32 = 0x01C;
    pub const TXFIFOLR: u32 = 0x020;
    pub const RXFIFOLR: u32 = 0x024;
    pub const IER: u32 = 0x028;
    pub const IRER: u32 = 0x02C;
    pub const ITER: u32 = 0x030;
    pub const CER: u32 = 0x034;
    pub const CLOCK: u32 = 0x038;
    pub const RATIO: u32 = 0x03C;
}

/// CTRL register bits.
pub mod ctrl {
    pub const I2S_EN: u32 = 1 << 0;
    pub const DMA_TX_EN: u32 = 1 << 1;
    pub const DMA_RX_EN: u32 = 1 << 2;
    pub const TX_EN: u32 = 1 << 3;
    pub const RX_EN: u32 = 1 << 4;
    pub const TX_PAUSE: u32 = 1 << 5;
    pub const RX_PAUSE: u32 = 1 << 6;
    pub const TX_SLAVE: u32 = 1 << 7;
    pub const RX_SLAVE: u32 = 1 << 8;
}

/// FMT register bits.
pub mod fmt {
    pub const DATA_WIDTH_SHIFT: u32 = 0;
    pub const DATA_WIDTH_MASK: u32 = 0x7;
    pub const CHANNELS_SHIFT: u32 = 3;
    pub const CHANNELS_MASK: u32 = 0x3 << 3;
    pub const FORMAT_SHIFT: u32 = 5;
    pub const FORMAT_MASK: u32 = 0x3 << 5;
    pub const I2S: u32 = 0 << 5;
    pub const LEFT_JUSTIFIED: u32 = 1 << 5;
    pub const RIGHT_JUSTIFIED: u32 = 2 << 5;
}

/// FIFO depth.
const FIFO_DEPTH: usize = 32;

/// Data width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataWidth {
    Bits8 = 0,
    Bits16 = 1,
    Bits24 = 2,
    Bits32 = 3,
}

impl Default for DataWidth {
    fn default() -> Self {
        Self::Bits16
    }
}

/// Audio format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    I2S = 0,
    LeftJustified = 1,
    RightJustified = 2,
}

impl Default for AudioFormat {
    fn default() -> Self {
        Self::I2S
    }
}

/// I2S channel state.
#[derive(Debug, Clone)]
pub struct I2sChannel {
    /// TX FIFO.
    tx_fifo: Vec<u32>,
    /// RX FIFO.
    rx_fifo: Vec<u32>,
    /// TX enabled.
    tx_enabled: bool,
    /// RX enabled.
    rx_enabled: bool,
    /// TX pause.
    tx_pause: bool,
    /// RX pause.
    rx_pause: bool,
    /// TX count.
    tx_count: u64,
    /// RX count.
    rx_count: u64,
}

impl Default for I2sChannel {
    fn default() -> Self {
        Self {
            tx_fifo: Vec::with_capacity(FIFO_DEPTH),
            rx_fifo: Vec::with_capacity(FIFO_DEPTH),
            tx_enabled: false,
            rx_enabled: false,
            tx_pause: false,
            rx_pause: false,
            tx_count: 0,
            rx_count: 0,
        }
    }
}

/// I2S controller.
#[derive(Debug)]
pub struct I2s {
    /// I2S instance ID (0 or 1).
    pub id: u8,
    /// Base address.
    base: u32,
    /// Control register.
    ctrl: u32,
    /// Format register.
    fmt: u32,
    /// Interrupt control.
    intcr: u32,
    /// Interrupt status.
    intsr: u32,
    /// DMA control.
    dma: u32,
    /// Clock enable.
    clock_enabled: bool,
    /// Clock ratio.
    ratio: u32,
    /// Data width.
    data_width: DataWidth,
    /// Audio format.
    audio_format: AudioFormat,
    /// Number of channels.
    channels: u8,
    /// Channel state.
    channel: I2sChannel,
    /// Sample rate (Hz).
    sample_rate: u32,
    /// MCLK frequency.
    mclk_freq: u32,
}

impl Default for I2s {
    fn default() -> Self {
        Self::new(0)
    }
}

impl I2s {
    /// Create a new I2S controller.
    pub fn new(id: u8) -> Self {
        Self {
            id,
            base: if id == 0 { I2S0_BASE } else { I2S1_BASE },
            ctrl: 0,
            fmt: fmt::I2S | (1 << fmt::CHANNELS_SHIFT), // I2S format, 2 channels
            intcr: 0,
            intsr: 0,
            dma: 0,
            clock_enabled: false,
            ratio: 256, // MCLK = 256 * sample_rate
            data_width: DataWidth::Bits16,
            audio_format: AudioFormat::I2S,
            channels: 2,
            channel: I2sChannel::default(),
            sample_rate: 48000,
            mclk_freq: 12_288_000, // 256 * 48000
        }
    }

    /// Get base address.
    pub fn base(&self) -> u32 {
        self.base
    }

    /// Check if I2S is enabled.
    pub fn is_enabled(&self) -> bool {
        (self.ctrl & ctrl::I2S_EN) != 0
    }

    /// Check if TX is enabled.
    pub fn is_tx_enabled(&self) -> bool {
        (self.ctrl & ctrl::TX_EN) != 0 && self.channel.tx_enabled
    }

    /// Check if RX is enabled.
    pub fn is_rx_enabled(&self) -> bool {
        (self.ctrl & ctrl::RX_EN) != 0 && self.channel.rx_enabled
    }

    /// Get data width.
    pub fn get_data_width(&self) -> DataWidth {
        self.data_width
    }

    /// Get audio format.
    pub fn get_audio_format(&self) -> AudioFormat {
        self.audio_format
    }

    /// Get sample rate.
    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Set sample rate.
    pub fn set_sample_rate(&mut self, rate: u32) {
        self.sample_rate = rate;
        self.mclk_freq = self.ratio * rate;
    }

    /// Get TX FIFO level.
    pub fn tx_fifo_level(&self) -> usize {
        self.channel.tx_fifo.len()
    }

    /// Get RX FIFO level.
    pub fn rx_fifo_level(&self) -> usize {
        self.channel.rx_fifo.len()
    }

    /// Write to TX FIFO.
    pub fn write_tx_fifo(&mut self, data: u32) -> bool {
        if self.channel.tx_fifo.len() < FIFO_DEPTH {
            self.channel.tx_fifo.push(data);
            self.channel.tx_count += 1;
            true
        } else {
            false
        }
    }

    /// Read from RX FIFO.
    pub fn read_rx_fifo(&mut self) -> Option<u32> {
        if let Some(data) = self.channel.rx_fifo.pop() {
            self.channel.rx_count += 1;
            Some(data)
        } else {
            None
        }
    }

    /// Push audio sample to RX FIFO (simulated input).
    pub fn push_rx_sample(&mut self, left: i16, right: i16) {
        if self.channel.rx_fifo.len() < FIFO_DEPTH {
            let sample = ((left as u32) & 0xFFFF) | ((right as u32) << 16);
            self.channel.rx_fifo.push(sample);
        }
    }

    /// Pop audio sample from TX FIFO (for output).
    pub fn pop_tx_sample(&mut self) -> Option<(i16, i16)> {
        self.channel.tx_fifo.pop().map(|sample| {
            let left = (sample & 0xFFFF) as i16;
            let right = ((sample >> 16) & 0xFFFF) as i16;
            (left, right)
        })
    }

    /// Clear FIFOs.
    pub fn clear_fifos(&mut self) {
        self.channel.tx_fifo.clear();
        self.channel.rx_fifo.clear();
    }

    /// Tick the I2S controller (simulate audio processing).
    pub fn tick(&mut self) {
        if !self.is_enabled() || !self.clock_enabled {
            return;
        }

        // Simulate audio sample processing
        if self.is_tx_enabled() && !self.channel.tx_pause {
            // TX would output samples here
        }

        if self.is_rx_enabled() && !self.channel.rx_pause {
            // RX would receive samples here
        }
    }

    /// Check if there's a pending interrupt.
    pub fn has_interrupt(&self) -> bool {
        (self.intsr & self.intcr) != 0
    }

    /// Update format from register.
    fn update_format(&mut self) {
        let width_bits = (self.fmt >> fmt::DATA_WIDTH_SHIFT) & fmt::DATA_WIDTH_MASK;
        self.data_width = match width_bits {
            0 => DataWidth::Bits8,
            1 => DataWidth::Bits16,
            2 => DataWidth::Bits24,
            3 => DataWidth::Bits32,
            _ => DataWidth::Bits16,
        };

        let format_bits = (self.fmt >> fmt::FORMAT_SHIFT) & 0x3;
        self.audio_format = match format_bits {
            0 => AudioFormat::I2S,
            1 => AudioFormat::LeftJustified,
            2 => AudioFormat::RightJustified,
            _ => AudioFormat::I2S,
        };

        let ch_bits = (self.fmt >> fmt::CHANNELS_SHIFT) & 0x3;
        self.channels = match ch_bits {
            0 => 1,
            1 => 2,
            2 => 4,
            3 => 8,
            _ => 2,
        };
    }
}

impl Device for I2s {
    fn id(&self) -> DeviceId {
        DeviceId::I2S
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - self.base;

        match offset {
            regs::CTRL => Ok(self.ctrl),
            regs::FMT => Ok(self.fmt),
            regs::INTCR => Ok(self.intcr),
            regs::INTSR => Ok(self.intsr),
            regs::DMA => Ok(self.dma),
            regs::TXDR => Ok(0), // Write-only
            regs::RXDR => Ok(self.read_rx_fifo().unwrap_or(0)),
            regs::TXFIFOLR => Ok(self.channel.tx_fifo.len() as u32),
            regs::RXFIFOLR => Ok(self.channel.rx_fifo.len() as u32),
            regs::IER => Ok(if self.is_enabled() { 1 } else { 0 }),
            regs::IRER => Ok(if self.is_rx_enabled() { 1 } else { 0 }),
            regs::ITER => Ok(if self.is_tx_enabled() { 1 } else { 0 }),
            regs::CER => Ok(if self.clock_enabled { 1 } else { 0 }),
            regs::CLOCK => Ok(self.sample_rate),
            regs::RATIO => Ok(self.ratio),
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - self.base;

        match offset {
            regs::CTRL => {
                self.ctrl = value;
                self.channel.tx_enabled = (value & ctrl::TX_EN) != 0;
                self.channel.rx_enabled = (value & ctrl::RX_EN) != 0;
                self.channel.tx_pause = (value & ctrl::TX_PAUSE) != 0;
                self.channel.rx_pause = (value & ctrl::RX_PAUSE) != 0;
            }
            regs::FMT => {
                self.fmt = value;
                self.update_format();
            }
            regs::INTCR => {
                self.intcr = value;
            }
            regs::CLR => {
                // Clear FIFOs
                self.clear_fifos();
            }
            regs::DMA => {
                self.dma = value;
            }
            regs::TXDR => {
                self.write_tx_fifo(value);
            }
            regs::IER => {
                // Enable/disable I2S
                if value & 1 != 0 {
                    self.ctrl |= ctrl::I2S_EN;
                } else {
                    self.ctrl &= !ctrl::I2S_EN;
                }
            }
            regs::IRER => {
                self.channel.rx_enabled = value & 1 != 0;
            }
            regs::ITER => {
                self.channel.tx_enabled = value & 1 != 0;
            }
            regs::CER => {
                self.clock_enabled = value & 1 != 0;
            }
            regs::CLOCK => {
                self.set_sample_rate(value);
            }
            regs::RATIO => {
                self.ratio = value;
                self.mclk_freq = self.ratio * self.sample_rate;
            }
            _ => {}
        }

        Ok(())
    }

    fn reset(&mut self) {
        let id = self.id;
        let base = self.base;
        *self = Self {
            id,
            base,
            ..Self::new(id)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const I2S0_BASE: u32 = super::I2S0_BASE;
    const I2S1_BASE: u32 = super::I2S1_BASE;

    // ==================== Basic Tests ====================

    #[test]
    fn test_i2s_creation() {
        let i2s0 = I2s::new(0);
        let i2s1 = I2s::new(1);

        assert_eq!(i2s0.id, 0);
        assert_eq!(i2s1.id, 1);
        assert_eq!(i2s0.base(), I2S0_BASE);
        assert_eq!(i2s1.base(), I2S1_BASE);
    }

    #[test]
    fn test_i2s_default() {
        let i2s = I2s::default();
        assert_eq!(i2s.id, 0);
        assert_eq!(i2s.base(), I2S0_BASE);
        assert_eq!(i2s.get_sample_rate(), 48000);
        assert_eq!(i2s.get_data_width(), DataWidth::Bits16);
        assert_eq!(i2s.get_audio_format(), AudioFormat::I2S);
    }

    #[test]
    fn test_i2s_enable_disable() {
        let mut i2s = I2s::new(0);

        assert!(!i2s.is_enabled());

        // Enable via IER register
        i2s.write(I2S0_BASE + regs::IER, 1).unwrap();
        assert!(i2s.is_enabled());

        // Disable
        i2s.write(I2S0_BASE + regs::IER, 0).unwrap();
        assert!(!i2s.is_enabled());
    }

    #[test]
    fn test_ctrl_register() {
        let mut i2s = I2s::new(0);

        // Write control register
        i2s.write(I2S0_BASE + regs::CTRL, ctrl::I2S_EN | ctrl::TX_EN | ctrl::RX_EN).unwrap();

        let ctrl_val = i2s.read(I2S0_BASE + regs::CTRL).unwrap();
        assert_eq!(ctrl_val & ctrl::I2S_EN, ctrl::I2S_EN);
        assert_eq!(ctrl_val & ctrl::TX_EN, ctrl::TX_EN);
        assert_eq!(ctrl_val & ctrl::RX_EN, ctrl::RX_EN);
    }

    // ==================== TX/RX Tests ====================

    #[test]
    fn test_tx_enable() {
        let mut i2s = I2s::new(0);

        i2s.write(I2S0_BASE + regs::CTRL, ctrl::TX_EN).unwrap();
        assert!(i2s.channel.tx_enabled);

        // Verify via ITER register
        assert_eq!(i2s.read(I2S0_BASE + regs::ITER).unwrap(), 1);
    }

    #[test]
    fn test_rx_enable() {
        let mut i2s = I2s::new(0);

        i2s.write(I2S0_BASE + regs::CTRL, ctrl::RX_EN).unwrap();
        assert!(i2s.channel.rx_enabled);

        // Verify via IRER register
        assert_eq!(i2s.read(I2S0_BASE + regs::IRER).unwrap(), 1);
    }

    #[test]
    fn test_tx_pause() {
        let mut i2s = I2s::new(0);

        i2s.write(I2S0_BASE + regs::CTRL, ctrl::TX_EN | ctrl::TX_PAUSE).unwrap();

        assert!(i2s.channel.tx_enabled);
        assert!(i2s.channel.tx_pause);
    }

    #[test]
    fn test_rx_pause() {
        let mut i2s = I2s::new(0);

        i2s.write(I2S0_BASE + regs::CTRL, ctrl::RX_EN | ctrl::RX_PAUSE).unwrap();

        assert!(i2s.channel.rx_enabled);
        assert!(i2s.channel.rx_pause);
    }

    // ==================== FIFO Tests ====================

    #[test]
    fn test_tx_fifo_write() {
        let mut i2s = I2s::new(0);

        // Write to TX FIFO
        i2s.write(I2S0_BASE + regs::TXDR, 0x12345678).unwrap();

        assert_eq!(i2s.tx_fifo_level(), 1);

        // Pop sample
        let sample = i2s.pop_tx_sample().unwrap();
        assert_eq!(sample.0 as u16, 0x5678); // Left
        assert_eq!(sample.1 as u16, 0x1234); // Right
    }

    #[test]
    fn test_rx_fifo_read() {
        let mut i2s = I2s::new(0);

        // Push sample to RX FIFO
        i2s.push_rx_sample(-1000, 2000);

        assert_eq!(i2s.rx_fifo_level(), 1);

        // Read via register
        let data = i2s.read(I2S0_BASE + regs::RXDR).unwrap();
        assert_eq!(data & 0xFFFF, (-1000i16 as u16) as u32);
        assert_eq!((data >> 16) & 0xFFFF, 2000 as u32);
    }

    #[test]
    fn test_fifo_clear() {
        let mut i2s = I2s::new(0);

        // Add data to both FIFOs
        i2s.write(I2S0_BASE + regs::TXDR, 0x12345678).unwrap();
        i2s.push_rx_sample(100, 200);

        assert_eq!(i2s.tx_fifo_level(), 1);
        assert_eq!(i2s.rx_fifo_level(), 1);

        // Clear
        i2s.write(I2S0_BASE + regs::CLR, 1).unwrap();

        assert_eq!(i2s.tx_fifo_level(), 0);
        assert_eq!(i2s.rx_fifo_level(), 0);
    }

    #[test]
    fn test_fifo_levels() {
        let mut i2s = I2s::new(0);

        // Fill TX FIFO partially
        for i in 0..10 {
            i2s.write(I2S0_BASE + regs::TXDR, i).unwrap();
        }

        let level = i2s.read(I2S0_BASE + regs::TXFIFOLR).unwrap();
        assert_eq!(level, 10);
    }

    #[test]
    fn test_fifo_full() {
        let mut i2s = I2s::new(0);

        // Fill TX FIFO to capacity (32 entries)
        for i in 0..32 {
            assert!(i2s.write_tx_fifo(i));
        }

        // Next write should fail
        assert!(!i2s.write_tx_fifo(999));
        assert_eq!(i2s.tx_fifo_level(), 32);
    }

    // ==================== Format Tests ====================

    #[test]
    fn test_data_width_8bit() {
        let mut i2s = I2s::new(0);

        let fmt = 0 << fmt::DATA_WIDTH_SHIFT;
        i2s.write(I2S0_BASE + regs::FMT, fmt).unwrap();

        assert_eq!(i2s.get_data_width(), DataWidth::Bits8);
    }

    #[test]
    fn test_data_width_16bit() {
        let mut i2s = I2s::new(0);

        let fmt = 1 << fmt::DATA_WIDTH_SHIFT;
        i2s.write(I2S0_BASE + regs::FMT, fmt).unwrap();

        assert_eq!(i2s.get_data_width(), DataWidth::Bits16);
    }

    #[test]
    fn test_data_width_24bit() {
        let mut i2s = I2s::new(0);

        let fmt = 2 << fmt::DATA_WIDTH_SHIFT;
        i2s.write(I2S0_BASE + regs::FMT, fmt).unwrap();

        assert_eq!(i2s.get_data_width(), DataWidth::Bits24);
    }

    #[test]
    fn test_data_width_32bit() {
        let mut i2s = I2s::new(0);

        let fmt = 3 << fmt::DATA_WIDTH_SHIFT;
        i2s.write(I2S0_BASE + regs::FMT, fmt).unwrap();

        assert_eq!(i2s.get_data_width(), DataWidth::Bits32);
    }

    #[test]
    fn test_audio_format_i2s() {
        let mut i2s = I2s::new(0);

        i2s.write(I2S0_BASE + regs::FMT, fmt::I2S).unwrap();
        assert_eq!(i2s.get_audio_format(), AudioFormat::I2S);
    }

    #[test]
    fn test_audio_format_left_justified() {
        let mut i2s = I2s::new(0);

        i2s.write(I2S0_BASE + regs::FMT, fmt::LEFT_JUSTIFIED).unwrap();
        assert_eq!(i2s.get_audio_format(), AudioFormat::LeftJustified);
    }

    #[test]
    fn test_audio_format_right_justified() {
        let mut i2s = I2s::new(0);

        i2s.write(I2S0_BASE + regs::FMT, fmt::RIGHT_JUSTIFIED).unwrap();
        assert_eq!(i2s.get_audio_format(), AudioFormat::RightJustified);
    }

    #[test]
    fn test_channels() {
        let mut i2s = I2s::new(0);

        // 1 channel
        let fmt = 0 << fmt::CHANNELS_SHIFT;
        i2s.write(I2S0_BASE + regs::FMT, fmt).unwrap();
        assert_eq!(i2s.channels, 1);

        // 2 channels
        let fmt = 1 << fmt::CHANNELS_SHIFT;
        i2s.write(I2S0_BASE + regs::FMT, fmt).unwrap();
        assert_eq!(i2s.channels, 2);

        // 4 channels
        let fmt = 2 << fmt::CHANNELS_SHIFT;
        i2s.write(I2S0_BASE + regs::FMT, fmt).unwrap();
        assert_eq!(i2s.channels, 4);

        // 8 channels
        let fmt = 3 << fmt::CHANNELS_SHIFT;
        i2s.write(I2S0_BASE + regs::FMT, fmt).unwrap();
        assert_eq!(i2s.channels, 8);
    }

    // ==================== Clock Tests ====================

    #[test]
    fn test_clock_enable() {
        let mut i2s = I2s::new(0);

        assert!(!i2s.clock_enabled);

        i2s.write(I2S0_BASE + regs::CER, 1).unwrap();
        assert!(i2s.clock_enabled);

        // Verify via CER register
        assert_eq!(i2s.read(I2S0_BASE + regs::CER).unwrap(), 1);
    }

    #[test]
    fn test_sample_rate() {
        let mut i2s = I2s::new(0);

        i2s.write(I2S0_BASE + regs::CLOCK, 44100).unwrap();

        assert_eq!(i2s.get_sample_rate(), 44100);
        assert_eq!(i2s.mclk_freq, 44100 * 256);
    }

    #[test]
    fn test_clock_ratio() {
        let mut i2s = I2s::new(0);

        i2s.write(I2S0_BASE + regs::RATIO, 512).unwrap();

        assert_eq!(i2s.ratio, 512);
        assert_eq!(i2s.mclk_freq, i2s.sample_rate * 512);
    }

    // ==================== DMA Tests ====================

    #[test]
    fn test_dma_control() {
        let mut i2s = I2s::new(0);

        i2s.write(I2S0_BASE + regs::DMA, 0x03).unwrap();
        assert_eq!(i2s.read(I2S0_BASE + regs::DMA).unwrap(), 0x03);
    }

    #[test]
    fn test_dma_tx_enable() {
        let mut i2s = I2s::new(0);

        i2s.write(I2S0_BASE + regs::CTRL, ctrl::DMA_TX_EN).unwrap();

        let ctrl = i2s.read(I2S0_BASE + regs::CTRL).unwrap();
        assert_eq!(ctrl & ctrl::DMA_TX_EN, ctrl::DMA_TX_EN);
    }

    // ==================== Interrupt Tests ====================

    #[test]
    fn test_interrupt_control() {
        let mut i2s = I2s::new(0);

        i2s.write(I2S0_BASE + regs::INTCR, 0xFF).unwrap();
        assert_eq!(i2s.read(I2S0_BASE + regs::INTCR).unwrap(), 0xFF);
    }

    #[test]
    fn test_interrupt_status() {
        let mut i2s = I2s::new(0);

        i2s.intsr = 0x0F;
        assert_eq!(i2s.read(I2S0_BASE + regs::INTSR).unwrap(), 0x0F);
    }

    #[test]
    fn test_has_interrupt() {
        let mut i2s = I2s::new(0);

        assert!(!i2s.has_interrupt());

        i2s.intcr = 0x01;
        i2s.intsr = 0x01;

        assert!(i2s.has_interrupt());
    }

    // ==================== Reset Test ====================

    #[test]
    fn test_i2s_reset() {
        let mut i2s = I2s::new(0);

        // Set various values
        i2s.write(I2S0_BASE + regs::IER, 1).unwrap();
        i2s.write(I2S0_BASE + regs::CTRL, ctrl::TX_EN | ctrl::RX_EN).unwrap();
        i2s.write(I2S0_BASE + regs::TXDR, 0x12345678).unwrap();
        i2s.write(I2S0_BASE + regs::CLOCK, 96000).unwrap();

        // Reset
        i2s.reset();

        // Check values are reset
        assert!(!i2s.is_enabled());
        assert_eq!(i2s.tx_fifo_level(), 0);
        assert_eq!(i2s.rx_fifo_level(), 0);
        assert_eq!(i2s.get_sample_rate(), 48000); // Default

        // Base address should be preserved
        assert_eq!(i2s.base(), I2S0_BASE);
    }

    // ==================== I2S1 Independence Tests ====================

    #[test]
    fn test_i2s1_independence() {
        let mut i2s0 = I2s::new(0);
        let mut i2s1 = I2s::new(1);

        // Write to I2S0
        i2s0.write(I2S0_BASE + regs::TXDR, 0x11111111).unwrap();

        // Write to I2S1
        i2s1.write(I2S1_BASE + regs::TXDR, 0x22222222).unwrap();

        // Verify they are independent
        assert_eq!(i2s0.tx_fifo_level(), 1);
        assert_eq!(i2s1.tx_fifo_level(), 1);

        let sample0 = i2s0.pop_tx_sample().unwrap();
        let sample1 = i2s1.pop_tx_sample().unwrap();

        assert_ne!(sample0, sample1);
    }

    // ==================== Device Trait Tests ====================

    #[test]
    fn test_device_id() {
        let i2s = I2s::new(0);
        assert_eq!(i2s.id(), DeviceId::I2S);
    }

    #[test]
    fn test_invalid_register() {
        let mut i2s = I2s::new(0);

        // Read from invalid offset should return 0
        let result = i2s.read(I2S0_BASE + 0x1000).unwrap();
        assert_eq!(result, 0);

        // Write to invalid offset should be ignored
        i2s.write(I2S0_BASE + 0x1000, 0x12345678).unwrap();
    }

    // ==================== Audio Sample Tests ====================

    #[test]
    fn test_stereo_sample_conversion() {
        let mut i2s = I2s::new(0);

        // Push stereo sample
        i2s.push_rx_sample(-32768, 32767); // Min and max 16-bit values

        // Pop via register
        let data = i2s.read(I2S0_BASE + regs::RXDR).unwrap();
        assert_eq!(data & 0xFFFF, (-32768i16 as u16) as u32);
        assert_eq!((data >> 16) & 0xFFFF, 32767 as u32);
    }

    #[test]
    fn test_sample_count() {
        let mut i2s = I2s::new(0);

        // Write multiple samples
        for _ in 0..5 {
            i2s.write_tx_fifo(0x12345678);
        }

        assert_eq!(i2s.channel.tx_count, 5);
    }

    #[test]
    fn test_tick_no_crash() {
        let mut i2s = I2s::new(0);

        // Tick should not crash even when disabled
        i2s.tick();

        // Enable and tick
        i2s.write(I2S0_BASE + regs::IER, 1).unwrap();
        i2s.write(I2S0_BASE + regs::CER, 1).unwrap();
        i2s.tick();
    }
}