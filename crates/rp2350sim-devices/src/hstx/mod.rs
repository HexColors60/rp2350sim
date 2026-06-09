//! HSTX (High-Speed Transmitter) for RP2350.
//!
//! Implements the high-speed serial transmitter for display output.

use rp2350sim_core::{Device, DeviceId, Result};

/// HSTX base address.
pub const HSTX_BASE: u32 = 0x5004_0000;

/// HSTX register offsets.
pub mod regs {
    pub const CS: u32 = 0x000;
    pub const CS_SET: u32 = 0x004;
    pub const CS_CLR: u32 = 0x008;
    pub const IRQ: u32 = 0x00C;
    pub const IRQ_SET: u32 = 0x010;
    pub const IRQ_CLR: u32 = 0x014;
    pub const IRQE: u32 = 0x018;
    pub const IRQE_SET: u32 = 0x01C;
    pub const IRQE_CLR: u32 = 0x020;
    pub const FIFOCFG: u32 = 0x024;
    pub const FIFOSTAT: u32 = 0x028;
    pub const FIFO: u32 = 0x02C;
    pub const BIT: u32 = 0x030;
    pub const BIT_SET: u32 = 0x034;
    pub const BIT_CLR: u32 = 0x038;
    pub const SLOW: u32 = 0x03C;
    pub const SLOW_SET: u32 = 0x040;
    pub const SLOW_CLR: u32 = 0x044;
    pub const EXPAND: u32 = 0x048;
    pub const EXPAND_SET: u32 = 0x04C;
    pub const EXPAND_CLR: u32 = 0x050;
    pub const VSYNC: u32 = 0x054;
    pub const VSYNC_SET: u32 = 0x058;
    pub const VSYNC_CLR: u32 = 0x05C;
    pub const HSYNC: u32 = 0x060;
    pub const HSYNC_SET: u32 = 0x064;
    pub const HSYNC_CLR: u32 = 0x068;
    pub const LINE: u32 = 0x06C;
    pub const LINE_SET: u32 = 0x070;
    pub const LINE_CLR: u32 = 0x074;
    pub const LINE_VSYNC: u32 = 0x078;
    pub const LINE_VSYNC_SET: u32 = 0x07C;
    pub const LINE_VSYNC_CLR: u32 = 0x080;
    pub const LINE_HSYNC: u32 = 0x084;
    pub const LINE_HSYNC_SET: u32 = 0x088;
    pub const LINE_HSYNC_CLR: u32 = 0x08C;
    pub const LINE_FIRST: u32 = 0x090;
    pub const LINE_FIRST_SET: u32 = 0x094;
    pub const LINE_FIRST_CLR: u32 = 0x098;
    pub const LINE_LAST: u32 = 0x09C;
    pub const LINE_LAST_SET: u32 = 0x0A0;
    pub const LINE_LAST_CLR: u32 = 0x0A4;
    pub const SLOWEST: u32 = 0x0A8;
    pub const SLOWEST_SET: u32 = 0x0AC;
    pub const SLOWEST_CLR: u32 = 0x0B0;
    pub const GPOA: u32 = 0x0B4;
    pub const GPOA_SET: u32 = 0x0B8;
    pub const GPOA_CLR: u32 = 0x0BC;
    pub const GPOB: u32 = 0x0C0;
    pub const GPOB_SET: u32 = 0x0C4;
    pub const GPOB_CLR: u32 = 0x0C8;
}

/// CS register bits.
pub mod cs {
    pub const EN: u32 = 1 << 0;
    pub const SFR: u32 = 1 << 1;
    pub const SFT: u32 = 1 << 2;
    pub const SFV: u32 = 1 << 3;
    pub const SFH: u32 = 1 << 4;
    pub const SFM: u32 = 1 << 5;
    pub const SFQ: u32 = 1 << 6;
    pub const SFC: u32 = 1 << 7;
    pub const SFE: u32 = 1 << 8;
    pub const SFP: u32 = 1 << 9;
    pub const SFN: u32 = 1 << 10;
}

/// IRQ register bits.
pub mod irq {
    pub const FIFO: u32 = 1 << 0;
    pub const LINE: u32 = 1 << 1;
    pub const VSYNC: u32 = 1 << 2;
    pub const HSYNC: u32 = 1 << 3;
}

/// FIFO configuration bits.
pub mod fifocfg {
    pub const THRESH_SHIFT: u32 = 0;
    pub const THRESH_MASK: u32 = 0x3F;
    pub const DREQ_SHIFT: u32 = 8;
    pub const DREQ_MASK: u32 = 0x3F << 8;
}

/// FIFO status bits.
pub mod fifostat {
    pub const LEVEL_SHIFT: u32 = 0;
    pub const LEVEL_MASK: u32 = 0x3F;
    pub const EMPTY: u32 = 1 << 8;
    pub const FULL: u32 = 1 << 9;
    pub const OVER: u32 = 1 << 10;
    pub const UNDER: u32 = 1 << 11;
}

/// HSTX FIFO depth.
const FIFO_DEPTH: usize = 64;

/// HSTX mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HstxMode {
    #[default]
    Disabled,
    Dvi,
    Custom,
}

/// HSTX channel configuration.
#[derive(Debug, Clone, Copy, Default)]
pub struct HstxChannel {
    /// Enable flag.
    pub enabled: bool,
    /// Invert output.
    pub invert: bool,
    /// Clock divider.
    pub clock_div: u16,
    /// Bit mode (8-bit or 10-bit).
    pub bit_mode: bool,
}

/// HSTX (High-Speed Transmitter) peripheral.
#[derive(Debug)]
pub struct Hstx {
    /// Control/status register.
    cs: u32,
    /// IRQ status.
    irq: u32,
    /// IRQ enable.
    irqe: u32,
    /// FIFO configuration.
    fifocfg: u32,
    /// FIFO status.
    fifostat: u32,
    /// FIFO data.
    fifo: [u32; FIFO_DEPTH],
    /// FIFO write pointer.
    fifo_wr: usize,
    /// FIFO read pointer.
    fifo_rd: usize,
    /// FIFO count.
    fifo_count: usize,
    /// Bit configuration.
    bit: u32,
    /// Slow clock configuration.
    slow: u32,
    /// Expand configuration.
    expand: u32,
    /// VSYNC configuration.
    vsync: u32,
    /// HSYNC configuration.
    hsync: u32,
    /// Line counter.
    line: u32,
    /// Line VSYNC.
    line_vsync: u32,
    /// Line HSYNC.
    line_hsync: u32,
    /// First line.
    line_first: u32,
    /// Last line.
    line_last: u32,
    /// Slowest configuration.
    slowest: u32,
    /// GPIO A output.
    gpoa: u32,
    /// GPIO B output.
    gpob: u32,
    /// Current mode.
    mode: HstxMode,
    /// Channels.
    #[allow(dead_code)]
    channels: [HstxChannel; 4],
    /// Current line number.
    current_line: u32,
    /// Current pixel in line.
    current_pixel: u32,
    /// Total lines per frame.
    total_lines: u32,
    /// Total pixels per line.
    total_pixels: u32,
}

impl Default for Hstx {
    fn default() -> Self {
        Self::new()
    }
}

impl Hstx {
    /// Create a new HSTX instance.
    pub fn new() -> Self {
        Self {
            cs: 0,
            irq: 0,
            irqe: 0,
            fifocfg: 32,  // Default threshold
            fifostat: fifostat::EMPTY,
            fifo: [0; FIFO_DEPTH],
            fifo_wr: 0,
            fifo_rd: 0,
            fifo_count: 0,
            bit: 0,
            slow: 0,
            expand: 0,
            vsync: 0,
            hsync: 0,
            line: 0,
            line_vsync: 0,
            line_hsync: 0,
            line_first: 0,
            line_last: 0,
            slowest: 0,
            gpoa: 0,
            gpob: 0,
            mode: HstxMode::Disabled,
            channels: [HstxChannel::default(); 4],
            current_line: 0,
            current_pixel: 0,
            total_lines: 525,   // Default VGA timing
            total_pixels: 800,  // Default VGA timing
        }
    }

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        (self.cs & cs::EN) != 0
    }

    /// Get FIFO threshold.
    pub fn fifo_threshold(&self) -> u32 {
        (self.fifocfg >> fifocfg::THRESH_SHIFT) & fifocfg::THRESH_MASK
    }

    /// Get FIFO level.
    pub fn fifo_level(&self) -> usize {
        self.fifo_count
    }

    /// Check if FIFO is empty.
    pub fn fifo_empty(&self) -> bool {
        self.fifo_count == 0
    }

    /// Check if FIFO is full.
    pub fn fifo_full(&self) -> bool {
        self.fifo_count >= FIFO_DEPTH
    }

    /// Push data to FIFO.
    pub fn fifo_push(&mut self, data: u32) -> bool {
        if self.fifo_full() {
            self.fifostat |= fifostat::OVER;
            return false;
        }

        self.fifo[self.fifo_wr] = data;
        self.fifo_wr = (self.fifo_wr + 1) % FIFO_DEPTH;
        self.fifo_count += 1;

        self.update_fifostat();
        true
    }

    /// Pop data from FIFO.
    pub fn fifo_pop(&mut self) -> Option<u32> {
        if self.fifo_empty() {
            self.fifostat |= fifostat::UNDER;
            return None;
        }

        let data = self.fifo[self.fifo_rd];
        self.fifo_rd = (self.fifo_rd + 1) % FIFO_DEPTH;
        self.fifo_count -= 1;

        self.update_fifostat();
        Some(data)
    }

    /// Update FIFO status.
    fn update_fifostat(&mut self) {
        self.fifostat &= !(fifostat::EMPTY | fifostat::FULL);
        self.fifostat |= (self.fifo_level() as u32) << fifostat::LEVEL_SHIFT;
        
        if self.fifo_empty() {
            self.fifostat |= fifostat::EMPTY;
        }
        if self.fifo_full() {
            self.fifostat |= fifostat::FULL;
        }

        // Check threshold IRQ
        if self.fifo_level() as u32 <= self.fifo_threshold() {
            self.irq |= irq::FIFO;
        }
    }

    /// Process one pixel.
    pub fn process_pixel(&mut self) {
        if !self.is_enabled() {
            return;
        }

        // Pop pixel from FIFO
        if let Some(_pixel) = self.fifo_pop() {
            self.current_pixel += 1;

            // Check end of line
            if self.current_pixel >= self.total_pixels {
                self.current_pixel = 0;
                self.current_line += 1;

                // Generate line IRQ
                self.irq |= irq::LINE;

                // Check end of frame
                if self.current_line >= self.total_lines {
                    self.current_line = 0;

                    // Generate VSYNC IRQ
                    self.irq |= irq::VSYNC;
                }

                // Generate HSYNC
                self.irq |= irq::HSYNC;
            }
        }
    }

    /// Get current line.
    pub fn get_current_line(&self) -> u32 {
        self.current_line
    }

    /// Get current pixel.
    pub fn get_current_pixel(&self) -> u32 {
        self.current_pixel
    }

    /// Set display timing.
    pub fn set_timing(&mut self, lines: u32, pixels: u32) {
        self.total_lines = lines;
        self.total_pixels = pixels;
    }
}

impl Device for Hstx {
    fn id(&self) -> DeviceId {
        DeviceId::HSTX
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - HSTX_BASE;

        match offset {
            regs::CS => Ok(self.cs),
            regs::IRQ => Ok(self.irq),
            regs::IRQE => Ok(self.irqe),
            regs::FIFOCFG => Ok(self.fifocfg),
            regs::FIFOSTAT => Ok(self.fifostat),
            regs::FIFO => {
                // Reading FIFO pops data
                Ok(self.fifo_pop().unwrap_or(0))
            }
            regs::BIT => Ok(self.bit),
            regs::SLOW => Ok(self.slow),
            regs::EXPAND => Ok(self.expand),
            regs::VSYNC => Ok(self.vsync),
            regs::HSYNC => Ok(self.hsync),
            regs::LINE => Ok(self.line),
            regs::LINE_VSYNC => Ok(self.line_vsync),
            regs::LINE_HSYNC => Ok(self.line_hsync),
            regs::LINE_FIRST => Ok(self.line_first),
            regs::LINE_LAST => Ok(self.line_last),
            regs::SLOWEST => Ok(self.slowest),
            regs::GPOA => Ok(self.gpoa),
            regs::GPOB => Ok(self.gpob),
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - HSTX_BASE;

        match offset {
            regs::CS | regs::CS_SET => {
                self.cs |= value;
                if (value & cs::EN) != 0 {
                    self.mode = HstxMode::Dvi;
                }
            }
            regs::CS_CLR => {
                self.cs &= !value;
                if (value & cs::EN) != 0 {
                    self.mode = HstxMode::Disabled;
                }
            }
            regs::IRQ => {
                self.irq = value;
            }
            regs::IRQ_SET => {
                self.irq |= value;
            }
            regs::IRQ_CLR => {
                self.irq &= !value;
            }
            regs::IRQE | regs::IRQE_SET => {
                self.irqe |= value;
            }
            regs::IRQE_CLR => {
                self.irqe &= !value;
            }
            regs::FIFOCFG => {
                self.fifocfg = value;
            }
            regs::FIFO => {
                self.fifo_push(value);
            }
            regs::BIT | regs::BIT_SET => {
                self.bit |= value;
            }
            regs::BIT_CLR => {
                self.bit &= !value;
            }
            regs::SLOW | regs::SLOW_SET => {
                self.slow |= value;
            }
            regs::SLOW_CLR => {
                self.slow &= !value;
            }
            regs::EXPAND => {
                self.expand = value;
            }
            regs::EXPAND_SET => {
                self.expand |= value;
            }
            regs::EXPAND_CLR => {
                self.expand &= !value;
            }
            regs::VSYNC | regs::VSYNC_SET => {
                self.vsync |= value;
            }
            regs::VSYNC_CLR => {
                self.vsync &= !value;
            }
            regs::HSYNC | regs::HSYNC_SET => {
                self.hsync |= value;
            }
            regs::HSYNC_CLR => {
                self.hsync &= !value;
            }
            regs::LINE => {
                self.line = value;
            }
            regs::LINE_SET => {
                self.line |= value;
            }
            regs::LINE_CLR => {
                self.line &= !value;
            }
            regs::LINE_VSYNC | regs::LINE_VSYNC_SET => {
                self.line_vsync |= value;
            }
            regs::LINE_VSYNC_CLR => {
                self.line_vsync &= !value;
            }
            regs::LINE_HSYNC | regs::LINE_HSYNC_SET => {
                self.line_hsync |= value;
            }
            regs::LINE_HSYNC_CLR => {
                self.line_hsync &= !value;
            }
            regs::LINE_FIRST => {
                self.line_first = value;
            }
            regs::LINE_LAST => {
                self.line_last = value;
            }
            regs::SLOWEST | regs::SLOWEST_SET => {
                self.slowest |= value;
            }
            regs::SLOWEST_CLR => {
                self.slowest &= !value;
            }
            regs::GPOA | regs::GPOA_SET => {
                self.gpoa |= value;
            }
            regs::GPOA_CLR => {
                self.gpoa &= !value;
            }
            regs::GPOB | regs::GPOB_SET => {
                self.gpob |= value;
            }
            regs::GPOB_CLR => {
                self.gpob &= !value;
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

    const HSTX_BASE: u32 = super::HSTX_BASE;

    #[test]
    fn test_hstx_creation() {
        let hstx = Hstx::new();
        assert!(!hstx.is_enabled());
        assert_eq!(hstx.mode, HstxMode::Disabled);
        assert!(hstx.fifo_empty());
    }

    #[test]
    fn test_hstx_enable_disable() {
        let mut hstx = Hstx::new();

        hstx.write(HSTX_BASE + regs::CS_SET, cs::EN).unwrap();
        assert!(hstx.is_enabled());

        hstx.write(HSTX_BASE + regs::CS_CLR, cs::EN).unwrap();
        assert!(!hstx.is_enabled());
    }

    #[test]
    fn test_fifo_push_pop() {
        let mut hstx = Hstx::new();

        assert!(hstx.fifo_push(0x12345678));
        assert_eq!(hstx.fifo_level(), 1);

        let data = hstx.fifo_pop().unwrap();
        assert_eq!(data, 0x12345678);
        assert!(hstx.fifo_empty());
    }

    #[test]
    fn test_fifo_full() {
        let mut hstx = Hstx::new();

        for i in 0..64 {
            assert!(hstx.fifo_push(i));
        }
        assert!(hstx.fifo_full());
        assert!(!hstx.fifo_push(999));
    }

    #[test]
    fn test_fifo_underflow() {
        let mut hstx = Hstx::new();
        assert!(hstx.fifo_pop().is_none());

        let stat = hstx.read(HSTX_BASE + regs::FIFOSTAT).unwrap();
        assert_eq!(stat & fifostat::UNDER, fifostat::UNDER);
    }

    #[test]
    fn test_fifo_status() {
        let mut hstx = Hstx::new();

        let stat = hstx.read(HSTX_BASE + regs::FIFOSTAT).unwrap();
        assert_eq!(stat & fifostat::EMPTY, fifostat::EMPTY);

        hstx.fifo_push(0x12345678);
        let stat = hstx.read(HSTX_BASE + regs::FIFOSTAT).unwrap();
        assert_eq!(stat & fifostat::EMPTY, 0);
    }

    #[test]
    fn test_irq_set_clear() {
        let mut hstx = Hstx::new();

        hstx.write(HSTX_BASE + regs::IRQ_SET, irq::FIFO | irq::LINE).unwrap();
        let irq_val = hstx.read(HSTX_BASE + regs::IRQ).unwrap();
        assert_eq!(irq_val & irq::FIFO, irq::FIFO);

        hstx.write(HSTX_BASE + regs::IRQ_CLR, irq::FIFO).unwrap();
        let irq_val = hstx.read(HSTX_BASE + regs::IRQ).unwrap();
        assert_eq!(irq_val & irq::FIFO, 0);
    }

    #[test]
    fn test_irq_enable() {
        let mut hstx = Hstx::new();

        hstx.write(HSTX_BASE + regs::IRQE_SET, irq::VSYNC).unwrap();
        assert_eq!(hstx.read(HSTX_BASE + regs::IRQE).unwrap() & irq::VSYNC, irq::VSYNC);

        hstx.write(HSTX_BASE + regs::IRQE_CLR, irq::VSYNC).unwrap();
        assert_eq!(hstx.read(HSTX_BASE + regs::IRQE).unwrap() & irq::VSYNC, 0);
    }

    #[test]
    fn test_config_registers() {
        let mut hstx = Hstx::new();

        hstx.write(HSTX_BASE + regs::BIT, 0x12345678).unwrap();
        assert_eq!(hstx.read(HSTX_BASE + regs::BIT).unwrap(), 0x12345678);

        hstx.write(HSTX_BASE + regs::SLOW, 0x11111111).unwrap();
        assert_eq!(hstx.read(HSTX_BASE + regs::SLOW).unwrap(), 0x11111111);

        hstx.write(HSTX_BASE + regs::EXPAND, 0x22222222).unwrap();
        assert_eq!(hstx.read(HSTX_BASE + regs::EXPAND).unwrap(), 0x22222222);
    }

    #[test]
    fn test_sync_config() {
        let mut hstx = Hstx::new();

        hstx.write(HSTX_BASE + regs::VSYNC, 0x1000).unwrap();
        hstx.write(HSTX_BASE + regs::HSYNC, 0x800).unwrap();

        assert_eq!(hstx.read(HSTX_BASE + regs::VSYNC).unwrap(), 0x1000);
        assert_eq!(hstx.read(HSTX_BASE + regs::HSYNC).unwrap(), 0x800);
    }

    #[test]
    fn test_line_config() {
        let mut hstx = Hstx::new();

        hstx.write(HSTX_BASE + regs::LINE, 100).unwrap();
        hstx.write(HSTX_BASE + regs::LINE_FIRST, 10).unwrap();
        hstx.write(HSTX_BASE + regs::LINE_LAST, 515).unwrap();

        assert_eq!(hstx.read(HSTX_BASE + regs::LINE).unwrap(), 100);
        assert_eq!(hstx.read(HSTX_BASE + regs::LINE_FIRST).unwrap(), 10);
        assert_eq!(hstx.read(HSTX_BASE + regs::LINE_LAST).unwrap(), 515);
    }

    #[test]
    fn test_gpo_output() {
        let mut hstx = Hstx::new();

        hstx.write(HSTX_BASE + regs::GPOA, 0x12345678).unwrap();
        hstx.write(HSTX_BASE + regs::GPOB, 0xABCDEF00).unwrap();

        assert_eq!(hstx.read(HSTX_BASE + regs::GPOA).unwrap(), 0x12345678);
        assert_eq!(hstx.read(HSTX_BASE + regs::GPOB).unwrap(), 0xABCDEF00);
    }

    #[test]
    fn test_process_pixel() {
        let mut hstx = Hstx::new();

        // Disabled - no processing
        hstx.fifo_push(0x12345678);
        hstx.process_pixel();
        assert_eq!(hstx.get_current_pixel(), 0);

        // Enabled - process
        hstx.write(HSTX_BASE + regs::CS_SET, cs::EN).unwrap();
        hstx.process_pixel();
        assert_eq!(hstx.get_current_pixel(), 1);
        assert!(hstx.fifo_empty());
    }

    #[test]
    fn test_set_timing() {
        let mut hstx = Hstx::new();
        hstx.set_timing(1080, 1920);

        assert_eq!(hstx.total_lines, 1080);
        assert_eq!(hstx.total_pixels, 1920);
    }

    #[test]
    fn test_hstx_reset() {
        let mut hstx = Hstx::new();

        hstx.write(HSTX_BASE + regs::CS_SET, cs::EN).unwrap();
        hstx.write(HSTX_BASE + regs::VSYNC, 1000).unwrap();
        hstx.fifo_push(0x12345678);

        hstx.reset();

        assert!(!hstx.is_enabled());
        assert!(hstx.fifo_empty());
        assert_eq!(hstx.read(HSTX_BASE + regs::VSYNC).unwrap(), 0);
    }

    #[test]
    fn test_device_id() {
        let hstx = Hstx::new();
        assert_eq!(hstx.id(), DeviceId::HSTX);
    }

    #[test]
    fn test_set_clear_operations() {
        let mut hstx = Hstx::new();

        hstx.write(HSTX_BASE + regs::BIT_SET, 0xFF).unwrap();
        assert_eq!(hstx.bit, 0xFF);

        hstx.write(HSTX_BASE + regs::BIT_CLR, 0x0F).unwrap();
        assert_eq!(hstx.bit, 0xF0);

        hstx.write(HSTX_BASE + regs::SLOW_SET, 0xFF).unwrap();
        assert_eq!(hstx.slow, 0xFF);

        hstx.write(HSTX_BASE + regs::SLOW_CLR, 0x0F).unwrap();
        assert_eq!(hstx.slow, 0xF0);
    }

    #[test]
    fn test_vsync_hsync_set_clear() {
        let mut hstx = Hstx::new();

        hstx.write(HSTX_BASE + regs::VSYNC_SET, 0xFF).unwrap();
        assert_eq!(hstx.vsync, 0xFF);

        hstx.write(HSTX_BASE + regs::VSYNC_CLR, 0x0F).unwrap();
        assert_eq!(hstx.vsync, 0xF0);

        hstx.write(HSTX_BASE + regs::HSYNC_SET, 0xFF).unwrap();
        assert_eq!(hstx.hsync, 0xFF);

        hstx.write(HSTX_BASE + regs::HSYNC_CLR, 0x0F).unwrap();
        assert_eq!(hstx.hsync, 0xF0);
    }

    #[test]
    fn test_line_irq_generation() {
        let mut hstx = Hstx::new();

        hstx.write(HSTX_BASE + regs::CS_SET, cs::EN).unwrap();
        hstx.set_timing(2, 2);

        hstx.fifo_push(1);
        hstx.process_pixel();
        hstx.fifo_push(2);
        hstx.process_pixel();

        let irq_val = hstx.read(HSTX_BASE + regs::IRQ).unwrap();
        assert_eq!(irq_val & irq::LINE, irq::LINE);
        assert_eq!(irq_val & irq::HSYNC, irq::HSYNC);
    }
}