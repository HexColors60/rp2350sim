//! SRAM state management.

use crate::sram::{SramBank, SRAM_BANK_COUNT, SRAM_BANK_SIZE};
use rp2350sim_core::consts::SRAM_BASE;

/// SRAM state.
#[derive(Debug)]
pub struct SramState {
    banks: Vec<SramBank>,
}

impl Default for SramState {
    fn default() -> Self {
        Self::new()
    }
}

impl SramState {
    pub fn new() -> Self {
        let banks = (0..SRAM_BANK_COUNT)
            .map(|i| SramBank::new(i as u8))
            .collect();
        Self { banks }
    }

    /// Get the bank containing an address.
    pub fn get_bank(&self, addr: u32) -> Option<&SramBank> {
        let offset = addr.wrapping_sub(SRAM_BASE);
        if offset >= SRAM_BASE + (SRAM_BANK_COUNT * SRAM_BANK_SIZE) as u32 {
            return None;
        }
        let bank_idx = offset / SRAM_BANK_SIZE as u32;
        self.banks.get(bank_idx as usize)
    }

    /// Get the bank containing an address (mutable).
    pub fn get_bank_mut(&mut self, addr: u32) -> Option<&mut SramBank> {
        let offset = addr.wrapping_sub(SRAM_BASE);
        if offset >= SRAM_BASE + (SRAM_BANK_COUNT * SRAM_BANK_SIZE) as u32 {
            return None;
        }
        let bank_idx = offset / SRAM_BANK_SIZE as u32;
        self.banks.get_mut(bank_idx as usize)
    }

    /// Read a byte from SRAM.
    pub fn read_byte(&self, addr: u32) -> u8 {
        let offset = addr.wrapping_sub(SRAM_BASE);
        let bank_idx = (offset / SRAM_BANK_SIZE as u32) as usize;
        let bank_offset = (offset % SRAM_BANK_SIZE as u32) as usize;
        self.banks.get(bank_idx).map(|b| b.read(bank_offset)).unwrap_or(0)
    }

    /// Write a byte to SRAM.
    pub fn write_byte(&mut self, addr: u32, value: u8) {
        let offset = addr.wrapping_sub(SRAM_BASE);
        let bank_idx = (offset / SRAM_BANK_SIZE as u32) as usize;
        let bank_offset = (offset % SRAM_BANK_SIZE as u32) as usize;
        if let Some(bank) = self.banks.get_mut(bank_idx) {
            bank.write(bank_offset, value);
        }
    }

    /// Read a halfword from SRAM.
    pub fn read_half(&self, addr: u32) -> u16 {
        let lo = self.read_byte(addr);
        let hi = self.read_byte(addr + 1);
        u16::from_le_bytes([lo, hi])
    }

    /// Write a halfword to SRAM.
    pub fn write_half(&mut self, addr: u32, value: u16) {
        let bytes = value.to_le_bytes();
        self.write_byte(addr, bytes[0]);
        self.write_byte(addr + 1, bytes[1]);
    }

    /// Read a word from SRAM.
    pub fn read_word(&self, addr: u32) -> u32 {
        let lo = self.read_half(addr);
        let hi = self.read_half(addr + 2);
        u32::from_le_bytes([
            lo as u8,
            (lo >> 8) as u8,
            hi as u8,
            (hi >> 8) as u8,
        ])
    }

    /// Write a word to SRAM.
    pub fn write_word(&mut self, addr: u32, value: u32) {
        let bytes = value.to_le_bytes();
        self.write_byte(addr, bytes[0]);
        self.write_byte(addr + 1, bytes[1]);
        self.write_byte(addr + 2, bytes[2]);
        self.write_byte(addr + 3, bytes[3]);
    }

    /// Clear all SRAM.
    pub fn clear(&mut self) {
        for bank in &mut self.banks {
            bank.clear();
        }
    }

    /// Load data into SRAM.
    pub fn load(&mut self, addr: u32, data: &[u8]) {
        for (i, &byte) in data.iter().enumerate() {
            self.write_byte(addr + i as u32, byte);
        }
    }

    /// Get total SRAM size.
    pub fn total_size(&self) -> usize {
        self.banks.len() * SRAM_BANK_SIZE
    }

    /// Get the number of banks.
    pub fn bank_count(&self) -> usize {
        self.banks.len()
    }

    /// Read a slice of data from SRAM.
    pub fn read(&self, offset: usize, len: usize) -> Vec<u8> {
        let addr = SRAM_BASE + offset as u32;
        let mut data = Vec::with_capacity(len);
        for i in 0..len {
            data.push(self.read_byte(addr + i as u32));
        }
        data
    }

    /// Write a slice of data to SRAM.
    pub fn write(&mut self, offset: usize, data: &[u8]) {
        let addr = SRAM_BASE + offset as u32;
        for (i, &byte) in data.iter().enumerate() {
            self.write_byte(addr + i as u32, byte);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sram_new() {
        let sram = SramState::new();
        assert!(sram.bank_count() > 0);
    }

    #[test]
    fn test_sram_total_size() {
        let sram = SramState::new();
        let size = sram.total_size();
        assert!(size > 0);
        assert_eq!(size, sram.bank_count() * SRAM_BANK_SIZE);
    }

    #[test]
    fn test_sram_write_read_byte() {
        let mut sram = SramState::new();
        
        sram.write_byte(SRAM_BASE, 0x42);
        sram.write_byte(SRAM_BASE + 1, 0xAA);
        
        assert_eq!(sram.read_byte(SRAM_BASE), 0x42);
        assert_eq!(sram.read_byte(SRAM_BASE + 1), 0xAA);
    }

    #[test]
    fn test_sram_write_read_half() {
        let mut sram = SramState::new();
        
        sram.write_half(SRAM_BASE, 0x1234);
        
        assert_eq!(sram.read_half(SRAM_BASE), 0x1234);
        assert_eq!(sram.read_byte(SRAM_BASE), 0x34);
        assert_eq!(sram.read_byte(SRAM_BASE + 1), 0x12);
    }

    #[test]
    fn test_sram_write_read_word() {
        let mut sram = SramState::new();
        
        sram.write_word(SRAM_BASE, 0xDEADBEEF);
        
        assert_eq!(sram.read_word(SRAM_BASE), 0xDEADBEEF);
        assert_eq!(sram.read_half(SRAM_BASE), 0xBEEF);
        assert_eq!(sram.read_half(SRAM_BASE + 2), 0xDEAD);
    }

    #[test]
    fn test_sram_clear() {
        let mut sram = SramState::new();
        
        sram.write_word(SRAM_BASE, 0x12345678);
        sram.clear();
        
        assert_eq!(sram.read_word(SRAM_BASE), 0);
    }

    #[test]
    fn test_sram_load_read() {
        let mut sram = SramState::new();
        
        let data = vec![0x01, 0x02, 0x03, 0x04];
        sram.load(SRAM_BASE + 0x100, &data);
        
        let read_data = sram.read(0x100, 4);
        assert_eq!(read_data, data);
    }

    #[test]
    fn test_sram_write_slice() {
        let mut sram = SramState::new();
        
        let data = vec![0xAA, 0xBB, 0xCC, 0xDD];
        sram.write(0x200, &data);
        
        assert_eq!(sram.read(0x200, 4), data);
    }

    #[test]
    fn test_sram_get_bank() {
        let sram = SramState::new();
        
        let bank = sram.get_bank(SRAM_BASE);
        assert!(bank.is_some());
        
        let bank = sram.get_bank(SRAM_BASE + SRAM_BANK_SIZE as u32);
        assert!(bank.is_some());
    }

    #[test]
    fn test_sram_out_of_bounds() {
        let sram = SramState::new();
        assert_eq!(sram.read_byte(0x10000000), 0);
    }
}