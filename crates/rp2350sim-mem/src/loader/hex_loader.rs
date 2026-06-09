//! Intel HEX file loader.

use rp2350sim_core::Result;
use std::io::{BufRead, Read};

/// Intel HEX loader.
pub struct HexLoader;

impl HexLoader {
    /// Load an Intel HEX file into memory.
    pub fn load<R: Read>(reader: &mut R, memory: &mut [u8]) -> Result<usize> {
        let mut total_written = 0;
        let mut extended_addr = 0u32;
        let buf_reader = std::io::BufReader::new(reader);

        for line in buf_reader.lines() {
            let line = line.map_err(|e| {
                rp2350sim_core::Error::Io(e)
            })?;

            let line = line.trim();
            if line.is_empty() || !line.starts_with(':') {
                continue;
            }

            let record = Self::parse_record(&line[1..])?;
            
            match record.record_type {
                0x00 => {
                    // Data record
                    let addr = extended_addr | record.address as u32;
                    let end = addr as usize + record.data.len();
                    if end <= memory.len() {
                        memory[addr as usize..end].copy_from_slice(&record.data);
                        total_written += record.data.len();
                    }
                }
                0x01 => {
                    // End of file
                    break;
                }
                0x02 => {
                    // Extended segment address
                    if record.data.len() >= 2 {
                        extended_addr = ((record.data[0] as u32) << 12) |
                                        ((record.data[1] as u32) << 4);
                    }
                }
                0x04 => {
                    // Extended linear address
                    if record.data.len() >= 2 {
                        extended_addr = ((record.data[0] as u32) << 24) |
                                        ((record.data[1] as u32) << 16);
                    }
                }
                _ => {}
            }
        }

        Ok(total_written)
    }

    fn parse_record(line: &str) -> Result<HexRecord> {
        let bytes = (0..line.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&line[i..i + 2], 16))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| {
                rp2350sim_core::Error::Serialization(format!("Invalid hex: {}", e))
            })?;

        if bytes.len() < 5 {
            return Err(rp2350sim_core::Error::Serialization("Record too short".into()));
        }

        let byte_count = bytes[0] as usize;
        let address = u16::from_be_bytes([bytes[1], bytes[2]]);
        let record_type = bytes[3];
        let data = bytes[4..4 + byte_count].to_vec();
        let checksum = bytes[4 + byte_count];

        // Verify checksum
        let sum: u8 = bytes[..4 + byte_count].iter().sum();
        if sum.wrapping_add(checksum) != 0 {
            return Err(rp2350sim_core::Error::Serialization("Checksum mismatch".into()));
        }

        Ok(HexRecord {
            byte_count,
            address,
            record_type,
            data,
        })
    }
}

/// Intel HEX record.
struct HexRecord {
    #[allow(dead_code)]
    byte_count: usize,
    address: u16,
    record_type: u8,
    data: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_hex_load_data_record() {
        // :03 0000 00 01 23 45 [checksum]
        // sum = 03+00+00+00+01+23+45 = 6C, checksum = 94
        let hex = ":0300000001234594\n";
        let mut cursor = Cursor::new(hex.as_bytes());
        let mut memory = vec![0u8; 256];
        
        let result = HexLoader::load(&mut cursor, &mut memory);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(memory[0], 0x01);
        assert_eq!(memory[1], 0x23);
        assert_eq!(memory[2], 0x45);
    }

    #[test]
    fn test_hex_load_multiple_records() {
        // :02 0000 00 11 22 [checksum]
        // sum = 02+00+00+00+11+22 = 35, checksum = CB
        // :02 0002 00 33 44 [checksum]
        // sum = 02+00+02+00+33+44 = 7B, checksum = 85
        let hex = ":020000001122CB\n:02000200334485\n";
        let mut cursor = Cursor::new(hex.as_bytes());
        let mut memory = vec![0u8; 256];
        
        let result = HexLoader::load(&mut cursor, &mut memory);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 4);
        assert_eq!(&memory[0..4], &[0x11, 0x22, 0x33, 0x44]);
    }

    #[test]
    fn test_hex_load_with_eof() {
        // Data record followed by EOF
        let hex = ":0100000055AA\n:00000001FF\n";
        let mut cursor = Cursor::new(hex.as_bytes());
        let mut memory = vec![0u8; 256];
        
        let result = HexLoader::load(&mut cursor, &mut memory);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
        assert_eq!(memory[0], 0x55);
    }

    #[test]
    fn test_hex_load_empty() {
        let hex = ":00000001FF\n"; // EOF only
        let mut cursor = Cursor::new(hex.as_bytes());
        let mut memory = vec![0u8; 256];
        
        let result = HexLoader::load(&mut cursor, &mut memory);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_hex_skip_non_hex_lines() {
        // Lines without : prefix should be skipped
        let hex = "This is a comment\n:01000000AB54\n";
        let mut cursor = Cursor::new(hex.as_bytes());
        let mut memory = vec![0u8; 256];
        
        let result = HexLoader::load(&mut cursor, &mut memory);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
        assert_eq!(memory[0], 0xAB);
    }

    #[test]
    fn test_hex_invalid_checksum() {
        // Invalid checksum (should be C6, is C7)
        let hex = ":03000000012345C7\n";
        let mut cursor = Cursor::new(hex.as_bytes());
        let mut memory = vec![0u8; 256];
        
        let result = HexLoader::load(&mut cursor, &mut memory);
        assert!(result.is_err());
    }
}