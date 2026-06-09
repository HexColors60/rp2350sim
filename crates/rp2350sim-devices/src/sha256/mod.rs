//! SHA-256 hardware accelerator for RP2350.
//!
//! Implements the SHA-256 cryptographic accelerator.

use rp2350sim_core::{Device, DeviceId, Result};

/// SHA-256 base address.
pub const SHA256_BASE: u32 = 0x5001_1000;

/// SHA-256 register offsets.
pub mod regs {
    pub const CTRL: u32 = 0x000;
    pub const STATUS: u32 = 0x004;
    pub const WDATA: u32 = 0x008;
    pub const SUM0: u32 = 0x00C;
    pub const SUM1: u32 = 0x010;
    pub const SUM2: u32 = 0x014;
    pub const SUM3: u32 = 0x018;
    pub const SUM4: u32 = 0x01C;
    pub const SUM5: u32 = 0x020;
    pub const SUM6: u32 = 0x024;
    pub const SUM7: u32 = 0x028;
}

/// CTRL register bits.
pub mod ctrl {
    pub const START: u32 = 1 << 0;
    pub const RESET: u32 = 1 << 1;
    pub const DOUBLE_SHA: u32 = 1 << 2;
    pub const BIG_ENDIAN: u32 = 1 << 3;
}

/// STATUS register bits.
pub mod status {
    pub const BUSY: u32 = 1 << 0;
    pub const ERROR: u32 = 1 << 1;
    pub const READY: u32 = 1 << 2;
}

/// SHA-256 constants (first 64 bits of fractional parts of cube roots of first 64 primes).
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// Initial hash values (first 32 bits of fractional parts of square roots of first 8 primes).
const H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// SHA-256 accelerator.
#[derive(Debug)]
pub struct Sha256 {
    /// Control register.
    ctrl: u32,
    /// Status register.
    status: u32,
    /// Input data buffer.
    input_buffer: Vec<u8>,
    /// Current hash state.
    hash: [u32; 8],
    /// Message schedule.
    w: [u32; 64],
    /// Total bits processed.
    total_bits: u64,
    /// Processing state.
    processing: bool,
    /// Double SHA mode.
    double_sha: bool,
    /// Big endian mode.
    big_endian: bool,
}

impl Default for Sha256 {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha256 {
    /// Create a new SHA-256 accelerator.
    pub fn new() -> Self {
        Self {
            ctrl: 0,
            status: status::READY,
            input_buffer: Vec::new(),
            hash: H0,
            w: [0; 64],
            total_bits: 0,
            processing: false,
            double_sha: false,
            big_endian: false,
        }
    }

    /// Check if busy.
    pub fn is_busy(&self) -> bool {
        (self.status & status::BUSY) != 0
    }

    /// Check if ready.
    pub fn is_ready(&self) -> bool {
        (self.status & status::READY) != 0
    }

    /// Reset the hash state.
    pub fn reset_hash(&mut self) {
        self.hash = H0;
        self.input_buffer.clear();
        self.total_bits = 0;
        self.status = status::READY;
        self.processing = false;
    }

    /// Write data to the accelerator.
    pub fn write_data(&mut self, data: u32) {
        if self.is_busy() {
            return;
        }

        // Convert to bytes based on endianness
        if self.big_endian {
            self.input_buffer.push(((data >> 24) & 0xFF) as u8);
            self.input_buffer.push(((data >> 16) & 0xFF) as u8);
            self.input_buffer.push(((data >> 8) & 0xFF) as u8);
            self.input_buffer.push((data & 0xFF) as u8);
        } else {
            self.input_buffer.push((data & 0xFF) as u8);
            self.input_buffer.push(((data >> 8) & 0xFF) as u8);
            self.input_buffer.push(((data >> 16) & 0xFF) as u8);
            self.input_buffer.push(((data >> 24) & 0xFF) as u8);
        }

        // Process complete blocks (512 bits = 64 bytes)
        while self.input_buffer.len() >= 64 {
            let block: [u8; 64] = self.input_buffer.drain(..64).collect::<Vec<_>>().try_into().unwrap();
            self.process_block(&block);
            self.total_bits += 512;
        }
    }

    /// Process a 512-bit block.
    fn process_block(&mut self, block: &[u8; 64]) {
        // Prepare message schedule
        for i in 0..16 {
            self.w[i] = ((block[i * 4] as u32) << 24)
                | ((block[i * 4 + 1] as u32) << 16)
                | ((block[i * 4 + 2] as u32) << 8)
                | (block[i * 4 + 3] as u32);
        }

        for i in 16..64 {
            let s0 = self.w[i - 15].rotate_right(7)
                ^ self.w[i - 15].rotate_right(18)
                ^ (self.w[i - 15] >> 3);
            let s1 = self.w[i - 2].rotate_right(17)
                ^ self.w[i - 2].rotate_right(19)
                ^ (self.w[i - 2] >> 10);
            self.w[i] = self.w[i - 16].wrapping_add(s0)
                .wrapping_add(self.w[i - 7])
                .wrapping_add(s1);
        }

        // Initialize working variables
        let mut a = self.hash[0];
        let mut b = self.hash[1];
        let mut c = self.hash[2];
        let mut d = self.hash[3];
        let mut e = self.hash[4];
        let mut f = self.hash[5];
        let mut g = self.hash[6];
        let mut h = self.hash[7];

        // Main loop
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(self.w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        // Add compressed chunk to current hash value
        self.hash[0] = self.hash[0].wrapping_add(a);
        self.hash[1] = self.hash[1].wrapping_add(b);
        self.hash[2] = self.hash[2].wrapping_add(c);
        self.hash[3] = self.hash[3].wrapping_add(d);
        self.hash[4] = self.hash[4].wrapping_add(e);
        self.hash[5] = self.hash[5].wrapping_add(f);
        self.hash[6] = self.hash[6].wrapping_add(g);
        self.hash[7] = self.hash[7].wrapping_add(h);
    }

    /// Finalize the hash computation.
    pub fn finalize(&mut self) {
        // Pad the message
        let msg_len = self.input_buffer.len();
        self.total_bits += (msg_len as u64) * 8;

        // Append bit '1' to message
        self.input_buffer.push(0x80);

        // Append zeros until message length ≡ 448 (mod 512)
        while self.input_buffer.len() % 64 != 56 {
            self.input_buffer.push(0);
        }

        // Append original length in bits as 64-bit big-endian
        let len_bits = self.total_bits.to_be_bytes();
        for byte in len_bits {
            self.input_buffer.push(byte);
        }

        // Process remaining blocks
        while !self.input_buffer.is_empty() {
            let mut block = [0u8; 64];
            let take = self.input_buffer.len().min(64);
            block[..take].copy_from_slice(&self.input_buffer[..take]);
            self.input_buffer.drain(..take);
            self.process_block(&block);
        }

        // Handle double SHA
        if self.double_sha {
            let first_hash = self.hash;
            self.hash = H0;
            
            // Hash the first hash result
            let mut block = [0u8; 64];
            for i in 0..8 {
                let word = first_hash[i].to_be();
                block[i * 4] = (word >> 24) as u8;
                block[i * 4 + 1] = (word >> 16) as u8;
                block[i * 4 + 2] = (word >> 8) as u8;
                block[i * 4 + 3] = word as u8;
            }
            // Pad for final block
            block[32] = 0x80;
            let len_bits: u64 = 256;
            let len_bytes = len_bits.to_be_bytes();
            block[56..64].copy_from_slice(&len_bytes);
            
            self.process_block(&block);
        }

        self.status = status::READY;
        self.processing = false;
    }

    /// Get hash result.
    pub fn get_hash(&self) -> [u32; 8] {
        self.hash
    }
}

impl Device for Sha256 {
    fn id(&self) -> DeviceId {
        DeviceId::SHA256
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - SHA256_BASE;

        match offset {
            regs::CTRL => Ok(self.ctrl),
            regs::STATUS => Ok(self.status),
            regs::WDATA => Ok(0), // Write-only
            regs::SUM0 => Ok(self.hash[0]),
            regs::SUM1 => Ok(self.hash[1]),
            regs::SUM2 => Ok(self.hash[2]),
            regs::SUM3 => Ok(self.hash[3]),
            regs::SUM4 => Ok(self.hash[4]),
            regs::SUM5 => Ok(self.hash[5]),
            regs::SUM6 => Ok(self.hash[6]),
            regs::SUM7 => Ok(self.hash[7]),
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - SHA256_BASE;

        match offset {
            regs::CTRL => {
                self.ctrl = value;
                self.double_sha = (value & ctrl::DOUBLE_SHA) != 0;
                self.big_endian = (value & ctrl::BIG_ENDIAN) != 0;

                if (value & ctrl::RESET) != 0 {
                    self.reset_hash();
                }

                if (value & ctrl::START) != 0 {
                    self.status = status::BUSY;
                    self.processing = true;
                    self.finalize();
                }
            }
            regs::WDATA => {
                self.write_data(value);
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
    fn test_sha256_creation() {
        let sha = Sha256::new();
        assert!(!sha.is_busy());
        assert!(sha.is_ready());
    }

    #[test]
    fn test_sha256_reset() {
        let mut sha = Sha256::new();

        // Write some data
        sha.write_data(0x61626364); // "abcd"

        // Reset
        sha.reset_hash();

        // Check state is reset
        assert!(sha.is_ready());
        assert_eq!(sha.input_buffer.len(), 0);
    }

    #[test]
    fn test_sha256_empty_hash() {
        let mut sha = Sha256::new();

        // Hash empty message
        sha.write(SHA256_BASE + regs::CTRL, ctrl::START).unwrap();

        // Expected SHA256 of empty string:
        // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let expected: [u32; 8] = [
            0xe3b0c442, 0x98fc1c14, 0x9afbf4c8,
            0x996fb924, 0x27ae41e4, 0x649b934c,
            0xa495991b, 0x7852b855,
        ];

        let hash = sha.get_hash();
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_sha256_abc() {
        let mut sha = Sha256::new();

        // Write "abc"
        sha.write_data(0x61626300); // "abc\0" in little-endian

        // Clear the null byte by writing just 3 bytes worth
        sha.reset_hash();
        sha.input_buffer.extend_from_slice(b"abc");

        // Start hash
        sha.write(SHA256_BASE + regs::CTRL, ctrl::START).unwrap();

        // Expected SHA256 of "abc":
        // ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
        let expected: [u32; 8] = [
            0xba7816bf, 0x8f01cfea, 0x414140de,
            0x5dae2223, 0xb00361a3, 0x96177a9c,
            0xb410ff61, 0xf20015ad,
        ];

        let hash = sha.get_hash();
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_sha256_register_read_write() {
        let mut sha = Sha256::new();

        // Write and read CTRL
        sha.write(SHA256_BASE + regs::CTRL, ctrl::BIG_ENDIAN).unwrap();
        assert_eq!(sha.read(SHA256_BASE + regs::CTRL).unwrap(), ctrl::BIG_ENDIAN);

        // Check STATUS
        let status = sha.read(SHA256_BASE + regs::STATUS).unwrap();
        assert_eq!(status & status::READY, status::READY);
    }

    #[test]
    fn test_sha256_sum_registers() {
        let mut sha = Sha256::new();

        // After reset, SUM registers should contain initial hash values
        assert_eq!(sha.read(SHA256_BASE + regs::SUM0).unwrap(), H0[0]);
        assert_eq!(sha.read(SHA256_BASE + regs::SUM1).unwrap(), H0[1]);
        assert_eq!(sha.read(SHA256_BASE + regs::SUM2).unwrap(), H0[2]);
        assert_eq!(sha.read(SHA256_BASE + regs::SUM3).unwrap(), H0[3]);
        assert_eq!(sha.read(SHA256_BASE + regs::SUM4).unwrap(), H0[4]);
        assert_eq!(sha.read(SHA256_BASE + regs::SUM5).unwrap(), H0[5]);
        assert_eq!(sha.read(SHA256_BASE + regs::SUM6).unwrap(), H0[6]);
        assert_eq!(sha.read(SHA256_BASE + regs::SUM7).unwrap(), H0[7]);
    }

    #[test]
    fn test_sha256_control_reset() {
        let mut sha = Sha256::new();

        // Write some data
        sha.write_data(0x12345678);

        // Trigger reset via control register
        sha.write(SHA256_BASE + regs::CTRL, ctrl::RESET).unwrap();

        // Check state is reset
        assert!(sha.is_ready());
        assert_eq!(sha.input_buffer.len(), 0);
    }

    #[test]
    fn test_sha256_big_endian() {
        let mut sha = Sha256::new();

        // Enable big-endian mode
        sha.write(SHA256_BASE + regs::CTRL, ctrl::BIG_ENDIAN).unwrap();
        assert!(sha.big_endian);

        // Write data - should be stored in big-endian order
        sha.write_data(0x61626364);

        // Check buffer contains "abcd" in order
        assert_eq!(sha.input_buffer, vec![0x61, 0x62, 0x63, 0x64]);
    }

    #[test]
    fn test_sha256_double_sha() {
        let mut sha = Sha256::new();

        // Enable double SHA mode
        sha.write(SHA256_BASE + regs::CTRL, ctrl::DOUBLE_SHA).unwrap();
        assert!(sha.double_sha);
    }

    #[test]
    fn test_sha256_device_reset() {
        let mut sha = Sha256::new();

        // Modify state
        sha.write_data(0xDEADBEEF);
        sha.hash[0] = 0x12345678;

        // Reset via Device trait
        sha.reset();

        // Check state is reset
        assert!(sha.is_ready());
        assert_eq!(sha.input_buffer.len(), 0);
        assert_eq!(sha.hash, H0);
    }

    #[test]
    fn test_sha256_process_block() {
        let mut sha = Sha256::new();

        // Process a block of zeros
        let block = [0u8; 64];
        sha.process_block(&block);

        // Hash should have changed from initial values
        assert_ne!(sha.hash[0], H0[0]);
    }
}