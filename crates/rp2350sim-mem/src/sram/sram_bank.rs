//! SRAM bank implementation.

use crate::memory::MemoryBlock;

/// SRAM bank size (16 KB per bank).
pub const SRAM_BANK_SIZE: usize = 0x4000;

/// Number of SRAM banks (33 banks to cover 520 KB + some extra).
pub const SRAM_BANK_COUNT: usize = 33;

/// SRAM bank.
#[derive(Debug)]
pub struct SramBank {
    block: MemoryBlock,
    bank_id: u8,
}

impl SramBank {
    pub fn new(bank_id: u8) -> Self {
        Self {
            block: MemoryBlock::new(format!("SRAM Bank {}", bank_id), SRAM_BANK_SIZE),
            bank_id,
        }
    }

    pub fn bank_id(&self) -> u8 {
        self.bank_id
    }

    pub fn read(&self, offset: usize) -> u8 {
        self.block.read_byte(offset)
    }

    pub fn write(&mut self, offset: usize, value: u8) {
        self.block.write_byte(offset, value);
    }

    pub fn read_half(&self, offset: usize) -> u16 {
        self.block.read_half(offset)
    }

    pub fn write_half(&mut self, offset: usize, value: u16) {
        self.block.write_half(offset, value);
    }

    pub fn read_word(&self, offset: usize) -> u32 {
        self.block.read_word(offset)
    }

    pub fn write_word(&mut self, offset: usize, value: u32) {
        self.block.write_word(offset, value);
    }

    pub fn clear(&mut self) {
        self.block.clear();
    }

    pub fn data(&self) -> &[u8] {
        self.block.data()
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        self.block.data_mut()
    }
}