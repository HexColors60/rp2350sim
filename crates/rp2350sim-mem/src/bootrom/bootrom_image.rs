//! Boot ROM image.

use rp2350sim_core::consts::{BOOTROM_BASE, BOOTROM_SIZE};

/// Boot ROM image.
#[derive(Debug)]
pub struct BootRomImage {
    data: Vec<u8>,
}

impl Default for BootRomImage {
    fn default() -> Self {
        Self::new()
    }
}

impl BootRomImage {
    pub fn new() -> Self {
        Self {
            data: vec![0; BOOTROM_SIZE],
        }
    }

    /// Load boot ROM from data.
    pub fn load(&mut self, data: &[u8]) {
        let len = data.len().min(self.data.len());
        self.data[..len].copy_from_slice(&data[..len]);
    }

    /// Read from boot ROM.
    pub fn read(&self, offset: usize) -> u8 {
        self.data.get(offset).copied().unwrap_or(0)
    }

    /// Read a halfword from boot ROM.
    pub fn read_half(&self, offset: usize) -> u16 {
        let lo = self.read(offset);
        let hi = self.read(offset + 1);
        u16::from_le_bytes([lo, hi])
    }

    /// Read a word from boot ROM.
    pub fn read_word(&self, offset: usize) -> u32 {
        let lo = self.read_half(offset);
        let hi = self.read_half(offset + 2);
        u32::from_le_bytes([
            lo as u8,
            (lo >> 8) as u8,
            hi as u8,
            (hi >> 8) as u8,
        ])
    }

    /// Get the data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the size.
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Get the base address.
    pub fn base(&self) -> u32 {
        BOOTROM_BASE
    }
}