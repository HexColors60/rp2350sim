//! UF2 file loader.

use rp2350sim_core::Result;
use std::io::Read;

/// UF2 block header.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Uf2Block {
    pub magic_start0: u32,
    pub magic_start1: u32,
    pub flags: u32,
    pub target_addr: u32,
    pub payload_size: u32,
    pub block_no: u32,
    pub num_blocks: u32,
    pub file_size: u32,
    pub data: [u8; 476],
    pub magic_end: u32,
}

const UF2_MAGIC_START0: u32 = 0x0A324655;
const UF2_MAGIC_START1: u32 = 0x9E5D5157;
const UF2_MAGIC_END: u32 = 0x0AB16F30;

/// UF2 loader.
pub struct Uf2Loader;

impl Uf2Loader {
    /// Load a UF2 file into a buffer.
    pub fn load<R: Read>(reader: &mut R, flash: &mut [u8]) -> Result<usize> {
        let mut total_written = 0;
        let mut block = Uf2Block {
            magic_start0: 0,
            magic_start1: 0,
            flags: 0,
            target_addr: 0,
            payload_size: 0,
            block_no: 0,
            num_blocks: 0,
            file_size: 0,
            data: [0; 476],
            magic_end: 0,
        };

        loop {
            let block_bytes = unsafe {
                std::slice::from_raw_parts_mut(
                    &mut block as *mut Uf2Block as *mut u8,
                    std::mem::size_of::<Uf2Block>(),
                )
            };

            match reader.read_exact(block_bytes) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            }

            // Validate magic numbers
            if block.magic_start0 != UF2_MAGIC_START0 ||
               block.magic_start1 != UF2_MAGIC_START1 ||
               block.magic_end != UF2_MAGIC_END {
                return Err(rp2350sim_core::Error::Serialization("Invalid UF2 block magic".into()));
            }

            // Write payload to flash
            let addr = block.target_addr as usize;
            let size = block.payload_size as usize;

            if addr + size <= flash.len() {
                flash[addr..addr + size].copy_from_slice(&block.data[..size]);
                total_written += size;
            }
        }

        Ok(total_written)
    }

    /// Check if data looks like a UF2 file.
    pub fn is_uf2(data: &[u8]) -> bool {
        if data.len() < 8 {
            return false;
        }
        let magic0 = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let magic1 = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        magic0 == UF2_MAGIC_START0 && magic1 == UF2_MAGIC_START1
    }
}