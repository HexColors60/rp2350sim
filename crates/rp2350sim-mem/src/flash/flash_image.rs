//! Flash image implementation.

use crate::memory::{MemoryBlock, MemoryRegion};
use rp2350sim_core::Result;

/// Flash memory image.
#[derive(Debug)]
pub struct FlashImage {
    block: MemoryBlock,
    sector_size: usize,
    page_size: usize,
}

impl Default for FlashImage {
    fn default() -> Self {
        Self::new(16 * 1024 * 1024) // 16 MB default
    }
}

impl FlashImage {
    pub fn new(size: usize) -> Self {
        Self {
            block: MemoryBlock::new("Flash", size),
            sector_size: 4096,
            page_size: 256,
        }
    }

    pub fn read(&self, offset: usize, data: &mut [u8]) -> Result<()> {
        self.block.read(offset as u64, data)
    }

    pub fn write(&mut self, offset: usize, data: &[u8]) -> Result<()> {
        self.block.write(offset as u64, data)
    }

    pub fn sector_size(&self) -> usize {
        self.sector_size
    }

    pub fn page_size(&self) -> usize {
        self.page_size
    }

    pub fn size(&self) -> usize {
        self.block.size()
    }

    pub fn erase_sector(&mut self, sector: usize) {
        let offset = sector * self.sector_size;
        let end = (offset + self.sector_size).min(self.block.size());
        for i in offset..end {
            self.block.write_byte(i, 0xFF);
        }
    }

    pub fn erase_all(&mut self) {
        self.block.data_mut().fill(0xFF);
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

    /// Load firmware data into flash at offset 0.
    pub fn load(&mut self, data: &[u8]) -> Result<()> {
        self.write(0, data)
    }

    /// Load firmware data at a specific address.
    pub fn load_at(&mut self, addr: u32, data: &[u8]) -> Result<()> {
        self.write(addr as usize, data)
    }

    /// Read a slice of data from flash.
    pub fn read_slice(&self, offset: usize, len: usize) -> Vec<u8> {
        let end = offset.saturating_add(len).min(self.block.size());
        let actual_len = end.saturating_sub(offset);
        let mut data = vec![0u8; actual_len];
        let _ = self.read(offset, &mut data);
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flash_new() {
        let flash = FlashImage::new(1024);
        assert_eq!(flash.size(), 1024);
    }

    #[test]
    fn test_flash_default() {
        let flash = FlashImage::default();
        assert_eq!(flash.size(), 16 * 1024 * 1024);
    }

    #[test]
    fn test_flash_sector_page_size() {
        let flash = FlashImage::new(4096);
        assert_eq!(flash.sector_size(), 4096);
        assert_eq!(flash.page_size(), 256);
    }

    #[test]
    fn test_flash_write_read() {
        let mut flash = FlashImage::new(1024);
        
        let data = vec![0x01, 0x02, 0x03, 0x04];
        flash.write(0, &data).unwrap();
        
        let mut read_data = vec![0u8; 4];
        flash.read(0, &mut read_data).unwrap();
        
        assert_eq!(read_data, data);
    }

    #[test]
    fn test_flash_load() {
        let mut flash = FlashImage::new(1024);
        
        let data = vec![0xAA, 0xBB, 0xCC, 0xDD];
        flash.load(&data).unwrap();
        
        assert_eq!(&flash.data()[0..4], &[0xAA, 0xBB, 0xCC, 0xDD]);
    }

    #[test]
    fn test_flash_erase_sector() {
        let mut flash = FlashImage::new(8192);
        
        flash.write(0, &[0x00, 0x00, 0x00, 0x00]).unwrap();
        flash.write(4096, &[0x11, 0x22, 0x33, 0x44]).unwrap();
        
        flash.erase_sector(0);
        
        assert_eq!(&flash.data()[0..4], &[0xFF, 0xFF, 0xFF, 0xFF]);
        assert_eq!(&flash.data()[4096..4100], &[0x11, 0x22, 0x33, 0x44]);
    }

    #[test]
    fn test_flash_erase_all() {
        let mut flash = FlashImage::new(1024);
        
        flash.write(0, &[0x00, 0x11, 0x22, 0x33]).unwrap();
        flash.erase_all();
        
        for byte in flash.data() {
            assert_eq!(*byte, 0xFF);
        }
    }

    #[test]
    fn test_flash_clear() {
        let mut flash = FlashImage::new(1024);
        
        flash.write(0, &[0xAA, 0xBB]).unwrap();
        flash.clear();
        
        assert_eq!(&flash.data()[0..2], &[0x00, 0x00]);
    }

    #[test]
    fn test_flash_read_slice() {
        let mut flash = FlashImage::new(1024);
        
        flash.write(100, &[0x01, 0x02, 0x03, 0x04]).unwrap();
        
        let data = flash.read_slice(100, 4);
        assert_eq!(data, vec![0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_flash_data_mut() {
        let mut flash = FlashImage::new(1024);
        
        let data = flash.data_mut();
        data[0] = 0x42;
        data[1] = 0x43;
        
        assert_eq!(flash.data()[0], 0x42);
        assert_eq!(flash.data()[1], 0x43);
    }
}