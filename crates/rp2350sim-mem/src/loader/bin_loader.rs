//! Binary file loader.

use rp2350sim_core::Result;
use std::io::Read;

/// Binary loader.
pub struct BinLoader;

impl BinLoader {
    /// Load a binary file into memory at the specified address.
    pub fn load<R: Read>(reader: &mut R, memory: &mut [u8], offset: usize) -> Result<usize> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        let len = data.len().min(memory.len() - offset);
        memory[offset..offset + len].copy_from_slice(&data[..len]);

        Ok(len)
    }

    /// Load a binary file into flash.
    pub fn load_to_flash<R: Read>(reader: &mut R, flash: &mut [u8], offset: usize) -> Result<usize> {
        Self::load(reader, flash, offset)
    }

    /// Load a binary file into SRAM.
    pub fn load_to_sram<R: Read>(reader: &mut R, sram: &mut [u8], offset: usize) -> Result<usize> {
        Self::load(reader, sram, offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_bin_load_basic() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let mut cursor = Cursor::new(&data);
        let mut memory = vec![0u8; 16];
        
        let result = BinLoader::load(&mut cursor, &mut memory, 0);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 4);
        assert_eq!(&memory[0..4], &[0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_bin_load_with_offset() {
        let data = vec![0xAA, 0xBB, 0xCC];
        let mut cursor = Cursor::new(&data);
        let mut memory = vec![0u8; 16];
        
        let result = BinLoader::load(&mut cursor, &mut memory, 8);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(&memory[8..11], &[0xAA, 0xBB, 0xCC]);
        assert_eq!(&memory[0..8], &[0u8; 8]); // Should still be zeros
    }

    #[test]
    fn test_bin_load_truncated() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let mut cursor = Cursor::new(&data);
        let mut memory = vec![0u8; 4]; // Too small
        
        let result = BinLoader::load(&mut cursor, &mut memory, 0);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 4); // Only loaded what fit
        assert_eq!(&memory[0..4], &[0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_bin_load_empty() {
        let data: Vec<u8> = Vec::new();
        let mut cursor = Cursor::new(&data);
        let mut memory = vec![0u8; 16];
        
        let result = BinLoader::load(&mut cursor, &mut memory, 0);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_bin_load_to_flash() {
        let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let mut cursor = Cursor::new(&data);
        let mut flash = vec![0u8; 1024];
        
        let result = BinLoader::load_to_flash(&mut cursor, &mut flash, 0x100);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 4);
        assert_eq!(&flash[0x100..0x104], &[0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_bin_load_to_sram() {
        let data = vec![0x12, 0x34, 0x56, 0x78];
        let mut cursor = Cursor::new(&data);
        let mut sram = vec![0u8; 256];
        
        let result = BinLoader::load_to_sram(&mut cursor, &mut sram, 0);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 4);
        assert_eq!(&sram[0..4], &[0x12, 0x34, 0x56, 0x78]);
    }
}