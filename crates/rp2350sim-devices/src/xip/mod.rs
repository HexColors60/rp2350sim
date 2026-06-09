//! XIP (Execute In Place) controller for RP2350.
//!
//! Implements the XIP controller for flash memory access.

use rp2350sim_core::{Device, DeviceId, Result};

/// XIP base address.
pub const XIP_BASE: u32 = 0x4000_0000;

/// XIP register offsets.
pub mod regs {
    pub const CTRL: u32 = 0x000;
    pub const FLUSH: u32 = 0x004;
    pub const STAT: u32 = 0x008;
    pub const CTR_HIT: u32 = 0x00C;
    pub const CTR_ACC: u32 = 0x010;
    pub const STREAM_ADDR: u32 = 0x014;
    pub const STREAM_CTR: u32 = 0x018;
    pub const STREAM_FIFO: u32 = 0x01C;
}

/// CTRL register bits.
pub mod ctrl {
    pub const EN: u32 = 1 << 0;
    pub const INVALIDATE: u32 = 1 << 1;
    pub const POWER_DOWN: u32 = 1 << 2;
    pub const ZERO_BACKGROUND: u32 = 1 << 3;
    pub const QUAD: u32 = 1 << 4;
    pub const PAGE_SIZE_SHIFT: u32 = 5;
    pub const PAGE_SIZE_MASK: u32 = 0x7 << 5;
    pub const CACHE_BYPASS: u32 = 1 << 8;
    pub const CACHE_ENABLE: u32 = 1 << 9;
    pub const PREPARE: u32 = 1 << 10;
}

/// STAT register bits.
pub mod stat {
    pub const FIFO_EMPTY: u32 = 1 << 0;
    pub const FIFO_FULL: u32 = 1 << 1;
    pub const FIFO_LEVEL_SHIFT: u32 = 2;
    pub const FIFO_LEVEL_MASK: u32 = 0xF << 2;
    pub const FLUSH_READY: u32 = 1 << 6;
    pub const STREAMING: u32 = 1 << 7;
    pub const POWER_DOWN: u32 = 1 << 8;
    pub const QUAD: u32 = 1 << 9;
    pub const CACHE_ENABLED: u32 = 1 << 10;
    pub const CACHE_BYPASS: u32 = 1 << 11;
}

/// XIP cache line size.
const CACHE_LINE_SIZE: usize = 64;

/// Number of cache lines.
const NUM_CACHE_LINES: usize = 256;

/// Cache line state.
#[derive(Debug, Clone, Copy)]
pub struct CacheLine {
    /// Tag (address bits 31:12).
    pub tag: u32,
    /// Valid flag.
    pub valid: bool,
    /// Dirty flag.
    pub dirty: bool,
    /// Data.
    pub data: [u8; CACHE_LINE_SIZE],
}

impl Default for CacheLine {
    fn default() -> Self {
        Self {
            tag: 0,
            valid: false,
            dirty: false,
            data: [0; CACHE_LINE_SIZE],
        }
    }
}

/// XIP controller.
#[derive(Debug)]
pub struct Xip {
    /// Control register.
    ctrl: u32,
    /// Status register.
    stat: u32,
    /// Cache hit counter.
    ctr_hit: u32,
    /// Cache access counter.
    ctr_acc: u32,
    /// Stream address.
    stream_addr: u32,
    /// Stream control.
    stream_ctr: u32,
    /// Stream FIFO.
    stream_fifo: Vec<u32>,
    /// Cache lines.
    cache: [CacheLine; NUM_CACHE_LINES],
    /// Flash memory (simulated).
    flash: Vec<u8>,
    /// Flash size (default 16MB).
    flash_size: usize,
}

impl Default for Xip {
    fn default() -> Self {
        Self::new()
    }
}

impl Xip {
    /// Create a new XIP controller.
    pub fn new() -> Self {
        Self {
            ctrl: ctrl::EN | ctrl::CACHE_ENABLE,
            stat: stat::FLUSH_READY,
            ctr_hit: 0,
            ctr_acc: 0,
            stream_addr: 0,
            stream_ctr: 0,
            stream_fifo: Vec::with_capacity(16),
            cache: [CacheLine::default(); NUM_CACHE_LINES],
            flash: vec![0xFF; 16 * 1024 * 1024], // 16MB default
            flash_size: 16 * 1024 * 1024,
        }
    }

    /// Create XIP with custom flash size.
    pub fn with_flash_size(flash_size: usize) -> Self {
        let mut xip = Self::new();
        xip.flash = vec![0xFF; flash_size];
        xip.flash_size = flash_size;
        xip
    }

    /// Load data into flash memory.
    pub fn load_flash(&mut self, offset: usize, data: &[u8]) {
        let end = (offset + data.len()).min(self.flash_size);
        for i in offset..end {
            self.flash[i] = data[i - offset];
        }
    }

    /// Get flash size.
    pub fn flash_size(&self) -> usize {
        self.flash_size
    }

    /// Check if XIP is enabled.
    pub fn is_enabled(&self) -> bool {
        (self.ctrl & ctrl::EN) != 0
    }

    /// Check if cache is enabled.
    pub fn is_cache_enabled(&self) -> bool {
        (self.ctrl & ctrl::CACHE_ENABLE) != 0
    }

    /// Check if cache bypass is enabled.
    pub fn is_cache_bypass(&self) -> bool {
        (self.ctrl & ctrl::CACHE_BYPASS) != 0
    }

    /// Check if quad mode is enabled.
    pub fn is_quad_mode(&self) -> bool {
        (self.ctrl & ctrl::QUAD) != 0
    }

    /// Get page size (in bytes).
    pub fn page_size(&self) -> u32 {
        let size_bits = (self.ctrl >> ctrl::PAGE_SIZE_SHIFT) & 0x7;
        256 << size_bits // 256, 512, 1024, 2048, 4096, ...
    }

    /// Calculate cache index from address.
    fn cache_index(&self, addr: u32) -> usize {
        ((addr >> 6) as usize) % NUM_CACHE_LINES
    }

    /// Calculate cache tag from address.
    fn cache_tag(&self, addr: u32) -> u32 {
        addr >> 12
    }

    /// Read from cache.
    pub fn read_cached(&mut self, addr: u32) -> Option<u32> {
        if !self.is_enabled() {
            return None;
        }

        self.ctr_acc = self.ctr_acc.wrapping_add(1);

        let index = self.cache_index(addr);
        let tag = self.cache_tag(addr);
        let line = &self.cache[index];

        if line.valid && line.tag == tag {
            // Cache hit
            self.ctr_hit = self.ctr_hit.wrapping_add(1);
            let offset = (addr as usize) % CACHE_LINE_SIZE;
            let mut data = [0u8; 4];
            for i in 0..4 {
                data[i] = line.data[offset + i];
            }
            Some(u32::from_le_bytes(data))
        } else {
            // Cache miss - load from flash
            None
        }
    }

    /// Read from flash (bypassing cache).
    pub fn read_flash(&self, addr: u32) -> u32 {
        let offset = addr as usize;
        if offset + 4 <= self.flash_size {
            let mut data = [0u8; 4];
            for i in 0..4 {
                data[i] = self.flash[offset + i];
            }
            u32::from_le_bytes(data)
        } else {
            0xFFFFFFFF
        }
    }

    /// Write to flash.
    pub fn write_flash(&mut self, addr: u32, value: u32) {
        let offset = addr as usize;
        if offset + 4 <= self.flash_size {
            let data = value.to_le_bytes();
            for i in 0..4 {
                self.flash[offset + i] = data[i];
            }
        }
    }

    /// Fill cache line from flash.
    #[allow(dead_code)]
    fn fill_cache_line(&mut self, addr: u32) {
        let index = self.cache_index(addr);
        let tag = self.cache_tag(addr);
        let line = &mut self.cache[index];

        line.tag = tag;
        line.valid = true;
        line.dirty = false;

        // Load data from flash
        let base_addr = (addr as usize) & !(CACHE_LINE_SIZE - 1);
        for i in 0..CACHE_LINE_SIZE {
            let flash_addr = base_addr + i;
            line.data[i] = if flash_addr < self.flash_size {
                self.flash[flash_addr]
            } else {
                0xFF
            };
        }
    }

    /// Invalidate cache.
    pub fn invalidate_cache(&mut self) {
        for line in &mut self.cache {
            line.valid = false;
        }
    }

    /// Flush cache.
    pub fn flush_cache(&mut self) {
        // In a real implementation, this would write dirty lines back to flash
        // For simulation, we just mark all lines as not dirty
        for line in &mut self.cache {
            line.dirty = false;
        }
        self.stat |= stat::FLUSH_READY;
    }

    /// Start streaming read.
    pub fn start_stream(&mut self, addr: u32, count: u32) {
        self.stream_addr = addr;
        self.stream_ctr = count;
        self.stream_fifo.clear();
        self.stat &= !stat::FIFO_EMPTY;
    }

    /// Tick the XIP controller.
    pub fn tick(&mut self) {
        // Process streaming
        if self.stream_ctr > 0 && self.stream_fifo.len() < 16 {
            let value = self.read_flash(self.stream_addr);
            self.stream_fifo.push(value);
            self.stream_addr += 4;
            self.stream_ctr -= 1;

            if self.stream_ctr == 0 {
                self.stat &= !stat::STREAMING;
            }
        }

        // Update FIFO status
        if self.stream_fifo.is_empty() {
            self.stat |= stat::FIFO_EMPTY;
        } else {
            self.stat &= !stat::FIFO_EMPTY;
        }

        if self.stream_fifo.len() >= 16 {
            self.stat |= stat::FIFO_FULL;
        } else {
            self.stat &= !stat::FIFO_FULL;
        }

        // Update FIFO level
        self.stat = (self.stat & !stat::FIFO_LEVEL_MASK) | ((self.stream_fifo.len() as u32) << stat::FIFO_LEVEL_SHIFT);
    }

    /// Read from stream FIFO.
    pub fn read_stream_fifo(&mut self) -> u32 {
        self.stream_fifo.pop().unwrap_or(0)
    }
}

impl Device for Xip {
    fn id(&self) -> DeviceId {
        DeviceId::XIP
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - XIP_BASE;

        match offset {
            regs::CTRL => Ok(self.ctrl),
            regs::FLUSH => Ok(0),
            regs::STAT => Ok(self.stat),
            regs::CTR_HIT => Ok(self.ctr_hit),
            regs::CTR_ACC => Ok(self.ctr_acc),
            regs::STREAM_ADDR => Ok(self.stream_addr),
            regs::STREAM_CTR => Ok(self.stream_ctr),
            regs::STREAM_FIFO => Ok(self.read_stream_fifo()),
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - XIP_BASE;

        match offset {
            regs::CTRL => {
                self.ctrl = value & 0x0FFF;
                // Update status bits
                if (value & ctrl::POWER_DOWN) != 0 {
                    self.stat |= stat::POWER_DOWN;
                } else {
                    self.stat &= !stat::POWER_DOWN;
                }
                if (value & ctrl::QUAD) != 0 {
                    self.stat |= stat::QUAD;
                } else {
                    self.stat &= !stat::QUAD;
                }
                if (value & ctrl::CACHE_ENABLE) != 0 {
                    self.stat |= stat::CACHE_ENABLED;
                } else {
                    self.stat &= !stat::CACHE_ENABLED;
                }
                if (value & ctrl::CACHE_BYPASS) != 0 {
                    self.stat |= stat::CACHE_BYPASS;
                } else {
                    self.stat &= !stat::CACHE_BYPASS;
                }
            }
            regs::FLUSH => {
                self.flush_cache();
            }
            regs::STREAM_ADDR => {
                self.stream_addr = value;
            }
            regs::STREAM_CTR => {
                self.start_stream(self.stream_addr, value);
                self.stat |= stat::STREAMING;
            }
            _ => {}
        }

        Ok(())
    }

    fn reset(&mut self) {
        let flash_size = self.flash_size;
        *self = Self::with_flash_size(flash_size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xip_creation() {
        let xip = Xip::new();
        assert!(xip.is_enabled());
        assert!(xip.is_cache_enabled());
        assert!(!xip.is_cache_bypass());
        assert_eq!(xip.flash_size(), 16 * 1024 * 1024);
    }

    #[test]
    fn test_xip_register_read_write() {
        let mut xip = Xip::new();

        // Write and read CTRL register
        xip.write(XIP_BASE + regs::CTRL, ctrl::EN | ctrl::QUAD).unwrap();
        assert_eq!(xip.read(XIP_BASE + regs::CTRL).unwrap(), ctrl::EN | ctrl::QUAD);

        // Check status reflects quad mode
        let stat = xip.read(XIP_BASE + regs::STAT).unwrap();
        assert_eq!(stat & stat::QUAD, stat::QUAD);
    }

    #[test]
    fn test_xip_flash_access() {
        let mut xip = Xip::new();

        // Load data into flash
        let test_data: [u8; 16] = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
                                    0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        xip.load_flash(0x1000, &test_data);

        // Read from flash
        let value = xip.read_flash(0x1000);
        assert_eq!(value, 0x33221100);

        let value = xip.read_flash(0x1004);
        assert_eq!(value, 0x77665544);
    }

    #[test]
    fn test_xip_cache_enable_disable() {
        let mut xip = Xip::new();

        // Cache should be enabled by default
        assert!(xip.is_cache_enabled());

        // Disable cache
        xip.write(XIP_BASE + regs::CTRL, ctrl::EN).unwrap();
        assert!(!xip.is_cache_enabled());

        // Re-enable cache
        xip.write(XIP_BASE + regs::CTRL, ctrl::EN | ctrl::CACHE_ENABLE).unwrap();
        assert!(xip.is_cache_enabled());
    }

    #[test]
    fn test_xip_cache_bypass() {
        let mut xip = Xip::new();

        // Cache bypass should be disabled by default
        assert!(!xip.is_cache_bypass());

        // Enable cache bypass
        xip.write(XIP_BASE + regs::CTRL, ctrl::EN | ctrl::CACHE_ENABLE | ctrl::CACHE_BYPASS).unwrap();
        assert!(xip.is_cache_bypass());

        // Check status reflects cache bypass
        let stat = xip.read(XIP_BASE + regs::STAT).unwrap();
        assert_eq!(stat & stat::CACHE_BYPASS, stat::CACHE_BYPASS);
    }

    #[test]
    fn test_xip_flush() {
        let mut xip = Xip::new();

        // Flush should set FLUSH_READY bit
        xip.write(XIP_BASE + regs::FLUSH, 1).unwrap();
        let stat = xip.read(XIP_BASE + regs::STAT).unwrap();
        assert_eq!(stat & stat::FLUSH_READY, stat::FLUSH_READY);
    }

    #[test]
    fn test_xip_counters() {
        let mut xip = Xip::new();

        // Load data into flash
        let test_data: [u8; 16] = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
                                    0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        xip.load_flash(0x1000, &test_data);

        // Initial counters should be 0
        assert_eq!(xip.read(XIP_BASE + regs::CTR_HIT).unwrap(), 0);
        assert_eq!(xip.read(XIP_BASE + regs::CTR_ACC).unwrap(), 0);
    }

    #[test]
    fn test_xip_streaming() {
        let mut xip = Xip::new();

        // Load data into flash
        let test_data: [u8; 16] = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
                                    0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        xip.load_flash(0x1000, &test_data);

        // Set stream address
        xip.write(XIP_BASE + regs::STREAM_ADDR, 0x1000).unwrap();

        // Start streaming 4 words
        xip.write(XIP_BASE + regs::STREAM_CTR, 4).unwrap();

        // Check streaming status
        let stat = xip.read(XIP_BASE + regs::STAT).unwrap();
        assert_eq!(stat & stat::STREAMING, stat::STREAMING);

        // Tick to fill FIFO
        for _ in 0..10 {
            xip.tick();
        }

        // Check stream counter is 0 (all words transferred)
        let ctr = xip.read(XIP_BASE + regs::STREAM_CTR).unwrap();
        assert_eq!(ctr, 0);

        // Read from stream FIFO - should have data
        let _value = xip.read(XIP_BASE + regs::STREAM_FIFO).unwrap();
        // Just verify we can read from the FIFO
    }

    #[test]
    fn test_xip_page_size() {
        let mut xip = Xip::new();

        // Default page size (size_bits = 0) = 256
        assert_eq!(xip.page_size(), 256);

        // Set page size to 512 (size_bits = 1)
        xip.write(XIP_BASE + regs::CTRL, ctrl::EN | ctrl::CACHE_ENABLE | (1 << ctrl::PAGE_SIZE_SHIFT)).unwrap();
        assert_eq!(xip.page_size(), 512);

        // Set page size to 4096 (size_bits = 4)
        xip.write(XIP_BASE + regs::CTRL, ctrl::EN | ctrl::CACHE_ENABLE | (4 << ctrl::PAGE_SIZE_SHIFT)).unwrap();
        assert_eq!(xip.page_size(), 4096);
    }

    #[test]
    fn test_xip_power_down() {
        let mut xip = Xip::new();

        // Power down the XIP
        xip.write(XIP_BASE + regs::CTRL, ctrl::POWER_DOWN).unwrap();

        // Check status reflects power down
        let stat = xip.read(XIP_BASE + regs::STAT).unwrap();
        assert_eq!(stat & stat::POWER_DOWN, stat::POWER_DOWN);
    }

    #[test]
    fn test_xip_reset() {
        let mut xip = Xip::new();

        // Modify state
        xip.write(XIP_BASE + regs::CTRL, 0).unwrap();
        xip.load_flash(0x1000, &[0x42; 16]);

        // Reset
        xip.reset();

        // Check state is reset
        assert!(xip.is_enabled());
        assert!(xip.is_cache_enabled());
    }
}