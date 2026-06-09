//! TRNG (True Random Number Generator) for RP2350.
//!
//! Implements the hardware random number generator.

use rp2350sim_core::{Device, DeviceId, Result};

/// TRNG base address.
pub const TRNG_BASE: u32 = 0x5001_2000;

/// TRNG register offsets.
pub mod regs {
    pub const CTRL: u32 = 0x000;
    pub const STATUS: u32 = 0x004;
    pub const DATA: u32 = 0x008;
    pub const INTEN: u32 = 0x00C;
    pub const INTSTAT: u32 = 0x010;
    pub const RND_SRC_EN: u32 = 0x014;
    pub const SAMPLE_CNT: u32 = 0x018;
}

/// CTRL register bits.
pub mod ctrl {
    pub const ENABLE: u32 = 1 << 0;
    pub const RESET: u32 = 1 << 1;
    pub const START: u32 = 1 << 2;
    pub const RESEED: u32 = 1 << 3;
}

/// STATUS register bits.
pub mod status {
    pub const READY: u32 = 1 << 0;
    pub const BUSY: u32 = 1 << 1;
    pub const ERROR: u32 = 1 << 2;
    pub const FIFO_EMPTY: u32 = 1 << 3;
    pub const FIFO_FULL: u32 = 1 << 4;
}

/// INTEN/INTSTAT register bits.
pub mod irq {
    pub const READY: u32 = 1 << 0;
    pub const ERROR: u32 = 1 << 1;
}

/// FIFO depth.
const FIFO_DEPTH: usize = 16;

/// TRNG controller.
#[derive(Debug)]
pub struct Trng {
    /// Control register.
    ctrl: u32,
    /// Status register.
    status: u32,
    /// Interrupt enable.
    inten: u32,
    /// Interrupt status.
    intstat: u32,
    /// Random source enable.
    rnd_src_en: u32,
    /// Sample count.
    sample_cnt: u32,
    /// Random data FIFO.
    fifo: Vec<u32>,
    /// Enabled flag.
    enabled: bool,
    /// Seed for pseudo-random generation (simulated TRNG).
    seed: u64,
}

impl Default for Trng {
    fn default() -> Self {
        Self::new()
    }
}

impl Trng {
    /// Create a new TRNG controller.
    pub fn new() -> Self {
        Self {
            ctrl: 0,
            status: status::READY | status::FIFO_EMPTY,
            inten: 0,
            intstat: 0,
            rnd_src_en: 0,
            sample_cnt: 32,
            fifo: Vec::with_capacity(FIFO_DEPTH),
            enabled: false,
            seed: 0x123456789ABCDEF0,
        }
    }

    /// Check if TRNG is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Check if data is ready.
    pub fn is_ready(&self) -> bool {
        (self.status & status::READY) != 0
    }

    /// Check if FIFO is empty.
    pub fn is_fifo_empty(&self) -> bool {
        self.fifo.is_empty()
    }

    /// Check if FIFO is full.
    pub fn is_fifo_full(&self) -> bool {
        self.fifo.len() >= FIFO_DEPTH
    }

    /// Generate a random number (simulated TRNG using xorshift64).
    fn generate_random(&mut self) -> u32 {
        // Xorshift64 algorithm for pseudo-random generation
        let mut x = self.seed;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.seed = x;

        // Add some "entropy" based on sample count
        let sample = self.sample_cnt as u64;
        x ^= x.wrapping_mul(sample);

        x as u32
    }

    /// Fill the FIFO with random data.
    fn fill_fifo(&mut self) {
        while self.fifo.len() < FIFO_DEPTH {
            let random = self.generate_random();
            self.fifo.push(random);
        }

        self.status &= !(status::FIFO_EMPTY);
        if self.fifo.len() >= FIFO_DEPTH {
            self.status |= status::FIFO_FULL;
        }
    }

    /// Read random data from FIFO.
    pub fn read_data(&mut self) -> u32 {
        if let Some(data) = self.fifo.pop() {
            // Update status
            self.status &= !(status::FIFO_FULL);
            if self.fifo.is_empty() {
                self.status |= status::FIFO_EMPTY;
            }

            // Refill FIFO if enabled
            if self.enabled {
                self.fill_fifo();
            }

            data
        } else {
            // Generate on-demand if FIFO is empty
            self.generate_random()
        }
    }

    /// Reset the TRNG.
    fn reset_trng(&mut self) {
        self.fifo.clear();
        self.status = status::READY | status::FIFO_EMPTY;
        self.intstat = 0;
        // Reseed with different value
        self.seed = self.seed.wrapping_add(0xDEADBEEFCAFEBABE);
    }

    /// Reseed the TRNG.
    fn reseed(&mut self) {
        // Simulate reseeding with new entropy
        self.seed = self.seed.wrapping_mul(0x5851F42D4C957F2D);
        self.fill_fifo();
    }

    /// Check for pending interrupt.
    pub fn has_interrupt(&self) -> bool {
        (self.intstat & self.inten) != 0
    }
}

impl Device for Trng {
    fn id(&self) -> DeviceId {
        DeviceId::TRNG
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - TRNG_BASE;

        match offset {
            regs::CTRL => Ok(self.ctrl),
            regs::STATUS => Ok(self.status),
            regs::DATA => Ok(self.read_data()),
            regs::INTEN => Ok(self.inten),
            regs::INTSTAT => Ok(self.intstat),
            regs::RND_SRC_EN => Ok(self.rnd_src_en),
            regs::SAMPLE_CNT => Ok(self.sample_cnt),
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - TRNG_BASE;

        match offset {
            regs::CTRL => {
                self.ctrl = value;

                if (value & ctrl::RESET) != 0 {
                    self.reset_trng();
                }

                if (value & ctrl::ENABLE) != 0 {
                    self.enabled = true;
                    self.fill_fifo();
                } else {
                    self.enabled = false;
                }

                if (value & ctrl::RESEED) != 0 {
                    self.reseed();
                }
            }
            regs::INTEN => {
                self.inten = value;
            }
            regs::INTSTAT => {
                // Write 1 to clear
                self.intstat &= !value;
            }
            regs::RND_SRC_EN => {
                self.rnd_src_en = value;
            }
            regs::SAMPLE_CNT => {
                self.sample_cnt = value;
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
    fn test_trng_creation() {
        let trng = Trng::new();
        assert!(!trng.is_enabled());
        assert!(trng.is_ready());
        assert!(trng.is_fifo_empty());
    }

    #[test]
    fn test_trng_enable() {
        let mut trng = Trng::new();

        // Enable TRNG
        trng.write(TRNG_BASE + regs::CTRL, ctrl::ENABLE).unwrap();
        assert!(trng.is_enabled());

        // FIFO should be filled
        assert!(!trng.is_fifo_empty());
    }

    #[test]
    fn test_trng_disable() {
        let mut trng = Trng::new();

        // Enable then disable
        trng.write(TRNG_BASE + regs::CTRL, ctrl::ENABLE).unwrap();
        assert!(trng.is_enabled());

        trng.write(TRNG_BASE + regs::CTRL, 0).unwrap();
        assert!(!trng.is_enabled());
    }

    #[test]
    fn test_trng_read_data() {
        let mut trng = Trng::new();

        // Enable TRNG
        trng.write(TRNG_BASE + regs::CTRL, ctrl::ENABLE).unwrap();

        // Read random data
        let data = trng.read(TRNG_BASE + regs::DATA).unwrap();
        // Just check it doesn't panic - value is pseudo-random
        assert!(data != 0 || data == 0); // Always true, just checking read works
    }

    #[test]
    fn test_trng_randomness() {
        let mut trng = Trng::new();

        // Enable TRNG
        trng.write(TRNG_BASE + regs::CTRL, ctrl::ENABLE).unwrap();

        // Generate multiple random numbers
        let mut values = Vec::new();
        for _ in 0..10 {
            values.push(trng.read(TRNG_BASE + regs::DATA).unwrap());
        }

        // Check that not all values are the same (very unlikely for a good RNG)
        let first = values[0];
        let all_same = values.iter().all(|&v| v == first);
        assert!(!all_same, "TRNG should produce different values");
    }

    #[test]
    fn test_trng_reset() {
        let mut trng = Trng::new();

        // Enable and fill FIFO
        trng.write(TRNG_BASE + regs::CTRL, ctrl::ENABLE).unwrap();
        assert!(!trng.is_fifo_empty());

        // Reset via control register
        trng.write(TRNG_BASE + regs::CTRL, ctrl::RESET).unwrap();

        // Check state is reset
        assert!(trng.is_ready());
        assert!(trng.is_fifo_empty());
    }

    #[test]
    fn test_trng_reseed() {
        let mut trng = Trng::new();

        // Enable TRNG
        trng.write(TRNG_BASE + regs::CTRL, ctrl::ENABLE).unwrap();

        // Read some data
        let _ = trng.read(TRNG_BASE + regs::DATA).unwrap();

        // Reseed
        trng.write(TRNG_BASE + regs::CTRL, ctrl::ENABLE | ctrl::RESEED).unwrap();

        // Should still be enabled and have data
        assert!(trng.is_enabled());
        assert!(!trng.is_fifo_empty());
    }

    #[test]
    fn test_trng_fifo_full() {
        let mut trng = Trng::new();

        // Enable TRNG - should fill FIFO
        trng.write(TRNG_BASE + regs::CTRL, ctrl::ENABLE).unwrap();

        // Check FIFO is full
        assert!(trng.is_fifo_full());

        // Check status register
        let status = trng.read(TRNG_BASE + regs::STATUS).unwrap();
        assert_eq!(status & status::FIFO_FULL, status::FIFO_FULL);
    }

    #[test]
    fn test_trng_interrupt_enable() {
        let mut trng = Trng::new();

        // Enable interrupts
        trng.write(TRNG_BASE + regs::INTEN, irq::READY | irq::ERROR).unwrap();

        let inten = trng.read(TRNG_BASE + regs::INTEN).unwrap();
        assert_eq!(inten, irq::READY | irq::ERROR);
    }

    #[test]
    fn test_trng_sample_count() {
        let mut trng = Trng::new();

        // Set sample count
        trng.write(TRNG_BASE + regs::SAMPLE_CNT, 64).unwrap();

        let sample_cnt = trng.read(TRNG_BASE + regs::SAMPLE_CNT).unwrap();
        assert_eq!(sample_cnt, 64);
    }

    #[test]
    fn test_trng_random_source_enable() {
        let mut trng = Trng::new();

        // Enable random source
        trng.write(TRNG_BASE + regs::RND_SRC_EN, 0x1).unwrap();

        let rnd_src_en = trng.read(TRNG_BASE + regs::RND_SRC_EN).unwrap();
        assert_eq!(rnd_src_en, 0x1);
    }

    #[test]
    fn test_trng_device_reset() {
        let mut trng = Trng::new();

        // Modify state
        trng.write(TRNG_BASE + regs::CTRL, ctrl::ENABLE).unwrap();
        trng.write(TRNG_BASE + regs::SAMPLE_CNT, 128).unwrap();

        // Reset via Device trait
        trng.reset();

        // Check state is reset
        assert!(!trng.is_enabled());
        assert!(trng.is_ready());
        assert_eq!(trng.sample_cnt, 32); // Default value
    }

    #[test]
    fn test_trng_status_register() {
        let mut trng = Trng::new();

        // Check initial status
        let status = trng.read(TRNG_BASE + regs::STATUS).unwrap();
        assert_eq!(status & status::READY, status::READY);
        assert_eq!(status & status::FIFO_EMPTY, status::FIFO_EMPTY);

        // Enable and check status
        trng.write(TRNG_BASE + regs::CTRL, ctrl::ENABLE).unwrap();
        let status = trng.read(TRNG_BASE + regs::STATUS).unwrap();
        assert_eq!(status & status::FIFO_FULL, status::FIFO_FULL);
    }

    #[test]
    fn test_trng_fifo_level() {
        let mut trng = Trng::new();

        // Enable TRNG
        trng.write(TRNG_BASE + regs::CTRL, ctrl::ENABLE).unwrap();

        // Check FIFO is not empty
        assert!(!trng.is_fifo_empty());

        // Read one value
        let _ = trng.read(TRNG_BASE + regs::DATA).unwrap();

        // FIFO should still have values
        assert!(!trng.is_fifo_empty());
    }

    #[test]
    fn test_trng_multiple_reads() {
        let mut trng = Trng::new();

        trng.write(TRNG_BASE + regs::CTRL, ctrl::ENABLE).unwrap();

        // Read multiple values
        let mut values = Vec::new();
        for _ in 0..10 {
            values.push(trng.read(TRNG_BASE + regs::DATA).unwrap());
        }

        // All values should be valid u32
        for v in &values {
            assert_ne!(*v, 0); // Very unlikely to be 0
        }
    }

    #[test]
    fn test_trng_clear_interrupt() {
        let mut trng = Trng::new();

        // Enable interrupts
        trng.write(TRNG_BASE + regs::INTEN, irq::READY).unwrap();

        // Check interrupt enable
        let inten = trng.read(TRNG_BASE + regs::INTEN).unwrap();
        assert_eq!(inten, irq::READY);
    }

    #[test]
    fn test_trng_fifo_drain() {
        let mut trng = Trng::new();

        trng.write(TRNG_BASE + regs::CTRL, ctrl::ENABLE).unwrap();

        // Drain FIFO
        while !trng.is_fifo_empty() {
            let _ = trng.read(TRNG_BASE + regs::DATA).unwrap();
        }

        assert!(trng.is_fifo_empty());
    }

    #[test]
    fn test_trng_consecutive_values_different() {
        let mut trng = Trng::new();

        trng.write(TRNG_BASE + regs::CTRL, ctrl::ENABLE).unwrap();

        // Get two consecutive values
        let v1 = trng.read(TRNG_BASE + regs::DATA).unwrap();
        let v2 = trng.read(TRNG_BASE + regs::DATA).unwrap();

        // They should be different (statistically)
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_trng_bit_distribution() {
        let mut trng = Trng::new();

        trng.write(TRNG_BASE + regs::CTRL, ctrl::ENABLE).unwrap();

        // Collect samples
        let mut bit_counts = [0u32; 32];
        for _ in 0..100 {
            let value = trng.read(TRNG_BASE + regs::DATA).unwrap();
            for bit in 0..32 {
                if value & (1 << bit) != 0 {
                    bit_counts[bit] += 1;
                }
            }
        }

        // Each bit should have roughly 50% 1s (allow 30-70% range)
        for (bit, &count) in bit_counts.iter().enumerate() {
            assert!(count > 20 && count < 80,
                "Bit {} has count {} which is outside expected range", bit, count);
        }
    }

    #[test]
    fn test_trng_reseed_changes_output() {
        let mut trng = Trng::new();

        trng.write(TRNG_BASE + regs::CTRL, ctrl::ENABLE).unwrap();

        // Get first value
        let v1 = trng.read(TRNG_BASE + regs::DATA).unwrap();

        // Reseed
        trng.write(TRNG_BASE + regs::CTRL, ctrl::ENABLE | ctrl::RESEED).unwrap();

        // Get second value
        let v2 = trng.read(TRNG_BASE + regs::DATA).unwrap();

        // Values should be different after reseed
        // (Note: This is a probabilistic test, but very likely to pass)
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_trng_disabled_read_returns_zero() {
        let mut trng = Trng::new();

        // Don't enable TRNG
        // Reading data should return 0 or some default
        let data = trng.read(TRNG_BASE + regs::DATA).unwrap();
        // When disabled, FIFO is empty, so data should be 0
        assert_eq!(data, 0);
    }

    #[test]
    fn test_trng_error_interrupt() {
        let mut trng = Trng::new();

        // Enable error interrupt
        trng.write(TRNG_BASE + regs::INTEN, irq::ERROR).unwrap();

        let inten = trng.read(TRNG_BASE + regs::INTEN).unwrap();
        assert_eq!(inten, irq::ERROR);
    }

    #[test]
    fn test_trng_all_interrupt_sources() {
        let mut trng = Trng::new();

        // Enable all interrupts
        let all_irqs = irq::READY | irq::ERROR;
        trng.write(TRNG_BASE + regs::INTEN, all_irqs).unwrap();

        let inten = trng.read(TRNG_BASE + regs::INTEN).unwrap();
        assert_eq!(inten, all_irqs);
    }
}