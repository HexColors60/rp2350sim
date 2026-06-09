//! Bootram for RP2350.
//!
//! Implements the boot RAM used during the boot process.

use rp2350sim_core::{Device, DeviceId, Result};

/// Bootram base address.
pub const BOOTRAM_BASE: u32 = 0x4000_0000;

/// Bootram size in bytes (8 KB).
pub const BOOTRAM_SIZE: usize = 8 * 1024;

/// Bootram register offsets.
pub mod regs {
    pub const DATA_START: u32 = 0x000;
    pub const DATA_END: u32 = 0x1FFC;
    pub const CTRL: u32 = 0x2000;
    pub const STATUS: u32 = 0x2004;
    pub const LOCK: u32 = 0x2008;
}

/// CTRL register bits.
pub mod ctrl {
    pub const ENABLE: u32 = 1 << 0;
    pub const CLEAR: u32 = 1 << 1;
    pub const PROTECT: u32 = 1 << 2;
}

/// STATUS register bits.
pub mod status {
    pub const ENABLED: u32 = 1 << 0;
    pub const LOCKED: u32 = 1 << 1;
    pub const PROTECTED: u32 = 1 << 2;
}

/// Bootram peripheral.
#[derive(Debug)]
pub struct Bootram {
    /// RAM data.
    data: [u32; BOOTRAM_SIZE / 4],
    /// Control register.
    ctrl: u32,
    /// Status register.
    status: u32,
    /// Lock register.
    lock: u32,
    /// Write protection enabled.
    write_protected: bool,
}

impl Default for Bootram {
    fn default() -> Self {
        Self::new()
    }
}

impl Bootram {
    /// Create a new Bootram instance.
    pub fn new() -> Self {
        Self {
            data: [0; BOOTRAM_SIZE / 4],
            ctrl: ctrl::ENABLE,
            status: status::ENABLED,
            lock: 0,
            write_protected: false,
        }
    }

    /// Get base address.
    pub fn base(&self) -> u32 {
        BOOTRAM_BASE
    }

    /// Get size.
    pub fn size(&self) -> usize {
        BOOTRAM_SIZE
    }

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        (self.ctrl & ctrl::ENABLE) != 0
    }

    /// Check if locked.
    pub fn is_locked(&self) -> bool {
        self.lock != 0
    }

    /// Check if write protected.
    pub fn is_write_protected(&self) -> bool {
        self.write_protected || (self.ctrl & ctrl::PROTECT) != 0
    }

    /// Read word at offset.
    pub fn read_word(&self, offset: u32) -> u32 {
        if !self.is_enabled() {
            return 0;
        }

        let index = (offset >> 2) as usize;
        if index < self.data.len() {
            self.data[index]
        } else {
            0
        }
    }

    /// Write word at offset.
    pub fn write_word(&mut self, offset: u32, value: u32) -> bool {
        if !self.is_enabled() || self.is_write_protected() || self.is_locked() {
            return false;
        }

        let index = (offset >> 2) as usize;
        if index < self.data.len() {
            self.data[index] = value;
            true
        } else {
            false
        }
    }

    /// Clear all data.
    pub fn clear(&mut self) {
        if !self.is_locked() {
            self.data.fill(0);
        }
    }

    /// Lock the bootram.
    pub fn lock(&mut self, key: u32) {
        if key == 0xDEADBEEF {
            self.lock = 1;
            self.status |= status::LOCKED;
        }
    }

    /// Unlock the bootram.
    pub fn unlock(&mut self, key: u32) {
        if key == 0xDEADBEEF {
            self.lock = 0;
            self.status &= !status::LOCKED;
        }
    }

    /// Load boot code.
    pub fn load_boot_code(&mut self, code: &[u8]) -> bool {
        if self.is_locked() {
            return false;
        }

        // Clear existing data
        self.data.fill(0);

        // Copy code as words
        for (i, chunk) in code.chunks(4).enumerate() {
            if i >= self.data.len() {
                break;
            }
            let mut word = 0u32;
            for (j, &byte) in chunk.iter().enumerate() {
                word |= (byte as u32) << (j * 8);
            }
            self.data[i] = word;
        }

        true
    }

    /// Get boot code.
    pub fn get_boot_code(&self) -> Vec<u8> {
        let mut code = Vec::with_capacity(self.data.len() * 4);
        for &word in &self.data {
            code.push((word & 0xFF) as u8);
            code.push(((word >> 8) & 0xFF) as u8);
            code.push(((word >> 16) & 0xFF) as u8);
            code.push(((word >> 24) & 0xFF) as u8);
        }
        // Trim trailing zeros
        while let Some(&0) = code.last() {
            code.pop();
        }
        code
    }
}

impl Device for Bootram {
    fn id(&self) -> DeviceId {
        DeviceId::BOOTRAM
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - BOOTRAM_BASE;

        match offset {
            regs::CTRL => Ok(self.ctrl),
            regs::STATUS => Ok(self.status),
            regs::LOCK => Ok(self.lock),
            _ => {
                if offset < BOOTRAM_SIZE as u32 {
                    Ok(self.read_word(offset))
                } else {
                    Ok(0)
                }
            }
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - BOOTRAM_BASE;

        match offset {
            regs::CTRL => {
                if !self.is_locked() {
                    self.ctrl = value & (ctrl::ENABLE | ctrl::CLEAR | ctrl::PROTECT);
                    
                    // Update status
                    if (self.ctrl & ctrl::ENABLE) != 0 {
                        self.status |= status::ENABLED;
                    } else {
                        self.status &= !status::ENABLED;
                    }
                    
                    if (self.ctrl & ctrl::PROTECT) != 0 {
                        self.status |= status::PROTECTED;
                    } else {
                        self.status &= !status::PROTECTED;
                    }
                    
                    // Handle clear
                    if (self.ctrl & ctrl::CLEAR) != 0 {
                        self.clear();
                        self.ctrl &= !ctrl::CLEAR;
                    }
                }
            }
            regs::LOCK => {
                if value == 0xDEADBEEF {
                    self.lock(0xDEADBEEF);
                } else if value == 0xCAFEBABE {
                    self.unlock(0xDEADBEEF);
                }
            }
            _ => {
                if offset < BOOTRAM_SIZE as u32 {
                    self.write_word(offset, value);
                }
            }
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

    const BASE: u32 = BOOTRAM_BASE;

    // ==================== Basic Creation Tests ====================

    #[test]
    fn test_bootram_creation() {
        let br = Bootram::new();

        assert!(br.is_enabled());
        assert!(!br.is_locked());
        assert!(!br.is_write_protected());
        assert_eq!(br.size(), 8 * 1024);
        assert_eq!(br.base(), 0x4000_0000);
    }

    #[test]
    fn test_bootram_default() {
        let br = Bootram::default();
        assert!(br.is_enabled());
    }

    // ==================== Read/Write Tests ====================

    #[test]
    fn test_read_write_word() {
        let mut br = Bootram::new();

        assert!(br.write_word(0, 0x12345678));
        assert_eq!(br.read_word(0), 0x12345678);

        assert!(br.write_word(4, 0xDEADBEEF));
        assert_eq!(br.read_word(4), 0xDEADBEEF);
    }

    #[test]
    fn test_read_write_multiple_words() {
        let mut br = Bootram::new();

        for i in 0..100 {
            let offset = (i * 4) as u32;
            let value = i as u32 * 0x11111111;
            assert!(br.write_word(offset, value));
            assert_eq!(br.read_word(offset), value);
        }
    }

    #[test]
    fn test_read_write_last_word() {
        let mut br = Bootram::new();

        // Last word in 8KB
        let last_offset = (BOOTRAM_SIZE - 4) as u32;
        assert!(br.write_word(last_offset, 0xFFFFFFFF));
        assert_eq!(br.read_word(last_offset), 0xFFFFFFFF);
    }

    #[test]
    fn test_read_write_out_of_bounds() {
        let mut br = Bootram::new();

        // Out of bounds read returns 0
        assert_eq!(br.read_word(BOOTRAM_SIZE as u32), 0);

        // Out of bounds write returns false
        assert!(!br.write_word(BOOTRAM_SIZE as u32, 0x12345678));
    }

    // ==================== Enable/Disable Tests ====================

    #[test]
    fn test_disable_enable() {
        let mut br = Bootram::new();

        br.write_word(0, 0x12345678);
        assert_eq!(br.read_word(0), 0x12345678);

        // Disable
        br.ctrl = 0;
        assert!(!br.is_enabled());

        // Read returns 0 when disabled
        assert_eq!(br.read_word(0), 0);

        // Write returns false when disabled
        assert!(!br.write_word(0, 0xDEADBEEF));

        // Re-enable
        br.ctrl = ctrl::ENABLE;
        assert!(br.is_enabled());

        // Original data is still there
        assert_eq!(br.read_word(0), 0x12345678);
    }

    // ==================== Lock/Unlock Tests ====================

    #[test]
    fn test_lock() {
        let mut br = Bootram::new();

        assert!(!br.is_locked());

        // Lock with correct key
        br.lock(0xDEADBEEF);
        assert!(br.is_locked());
        assert_eq!(br.status & status::LOCKED, status::LOCKED);
    }

    #[test]
    fn test_lock_wrong_key() {
        let mut br = Bootram::new();

        // Wrong key should not lock
        br.lock(0x12345678);
        assert!(!br.is_locked());
    }

    #[test]
    fn test_unlock() {
        let mut br = Bootram::new();

        br.lock(0xDEADBEEF);
        assert!(br.is_locked());

        // Unlock with correct key
        br.unlock(0xDEADBEEF);
        assert!(!br.is_locked());
        assert_eq!(br.status & status::LOCKED, 0);
    }

    #[test]
    fn test_unlock_wrong_key() {
        let mut br = Bootram::new();

        br.lock(0xDEADBEEF);
        assert!(br.is_locked());

        // Wrong key should not unlock
        br.unlock(0x12345678);
        assert!(br.is_locked());
    }

    #[test]
    fn test_locked_prevents_write() {
        let mut br = Bootram::new();

        br.write_word(0, 0x12345678);
        br.lock(0xDEADBEEF);

        // Write should fail when locked
        assert!(!br.write_word(0, 0xDEADBEEF));
        assert_eq!(br.read_word(0), 0x12345678); // Original preserved
    }

    #[test]
    fn test_locked_prevents_clear() {
        let mut br = Bootram::new();

        br.write_word(0, 0x12345678);
        br.lock(0xDEADBEEF);

        br.clear();

        // Data should be preserved when locked
        assert_eq!(br.read_word(0), 0x12345678);
    }

    // ==================== Write Protection Tests ====================

    #[test]
    fn test_write_protect() {
        let mut br = Bootram::new();

        br.write_word(0, 0x12345678);

        // Enable write protection
        br.ctrl |= ctrl::PROTECT;
        assert!(br.is_write_protected());

        // Write should fail
        assert!(!br.write_word(0, 0xDEADBEEF));
        assert_eq!(br.read_word(0), 0x12345678);
    }

    #[test]
    fn test_write_protected_flag() {
        let mut br = Bootram::new();

        assert!(!br.is_write_protected());

        br.write_protected = true;
        assert!(br.is_write_protected());
    }

    // ==================== Clear Tests ====================

    #[test]
    fn test_clear() {
        let mut br = Bootram::new();

        for i in 0..10 {
            br.write_word((i * 4) as u32, 0xFFFFFFFF);
        }

        br.clear();

        for i in 0..10 {
            assert_eq!(br.read_word((i * 4) as u32), 0);
        }
    }

    // ==================== Boot Code Tests ====================

    #[test]
    fn test_load_boot_code() {
        let mut br = Bootram::new();

        let code: &[u8] = &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
        assert!(br.load_boot_code(code));

        // First word: 0x04030201 (little endian)
        assert_eq!(br.read_word(0), 0x04030201);
        // Second word: 0x00000605
        assert_eq!(br.read_word(4), 0x00000605);
    }

    #[test]
    fn test_load_boot_code_empty() {
        let mut br = Bootram::new();

        assert!(br.load_boot_code(&[]));
    }

    #[test]
    fn test_load_boot_code_locked() {
        let mut br = Bootram::new();
        br.lock(0xDEADBEEF);

        assert!(!br.load_boot_code(&[0x01, 0x02, 0x03]));
    }

    #[test]
    fn test_get_boot_code() {
        let mut br = Bootram::new();

        br.write_word(0, 0x04030201);
        br.write_word(4, 0x00000605);

        let code = br.get_boot_code();
        assert_eq!(code, vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);
    }

    // ==================== Register Read Tests ====================

    #[test]
    fn test_read_ctrl() {
        let mut br = Bootram::new();
        br.ctrl = 0x12345678;
        assert_eq!(br.read(BASE + regs::CTRL).unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_status() {
        let mut br = Bootram::new();
        br.status = 0xABCDEF00;
        assert_eq!(br.read(BASE + regs::STATUS).unwrap(), 0xABCDEF00);
    }

    #[test]
    fn test_read_lock() {
        let mut br = Bootram::new();
        br.lock = 1;
        assert_eq!(br.read(BASE + regs::LOCK).unwrap(), 1);
    }

    #[test]
    fn test_read_data() {
        let mut br = Bootram::new();

        br.write_word(0, 0x12345678);
        br.write_word(0x100, 0xDEADBEEF);

        assert_eq!(br.read(BASE + 0).unwrap(), 0x12345678);
        assert_eq!(br.read(BASE + 0x100).unwrap(), 0xDEADBEEF);
    }

    // ==================== Register Write Tests ====================

    #[test]
    fn test_write_ctrl_enable() {
        let mut br = Bootram::new();

        br.ctrl = 0;
        br.write(BASE + regs::CTRL, ctrl::ENABLE).unwrap();

        assert!(br.is_enabled());
        assert_eq!(br.status & status::ENABLED, status::ENABLED);
    }

    #[test]
    fn test_write_ctrl_clear() {
        let mut br = Bootram::new();

        br.write_word(0, 0x12345678);
        br.write(BASE + regs::CTRL, ctrl::ENABLE | ctrl::CLEAR).unwrap();

        // Data should be cleared
        assert_eq!(br.read_word(0), 0);
        // CLEAR bit should be auto-cleared
        assert_eq!(br.ctrl & ctrl::CLEAR, 0);
    }

    #[test]
    fn test_write_ctrl_protect() {
        let mut br = Bootram::new();

        br.write(BASE + regs::CTRL, ctrl::ENABLE | ctrl::PROTECT).unwrap();

        assert!(br.is_write_protected());
        assert_eq!(br.status & status::PROTECTED, status::PROTECTED);
    }

    #[test]
    fn test_write_lock() {
        let mut br = Bootram::new();

        // Lock with correct key
        br.write(BASE + regs::LOCK, 0xDEADBEEF).unwrap();
        assert!(br.is_locked());

        // Unlock with unlock key
        br.write(BASE + regs::LOCK, 0xCAFEBABE).unwrap();
        assert!(!br.is_locked());
    }

    #[test]
    fn test_write_data() {
        let mut br = Bootram::new();

        br.write(BASE + 0, 0x12345678).unwrap();
        assert_eq!(br.read_word(0), 0x12345678);

        br.write(BASE + 0x100, 0xDEADBEEF).unwrap();
        assert_eq!(br.read_word(0x100), 0xDEADBEEF);
    }

    #[test]
    fn test_write_ctrl_locked() {
        let mut br = Bootram::new();

        br.lock(0xDEADBEEF);
        br.write(BASE + regs::CTRL, 0).unwrap();

        // CTRL should not change when locked
        assert_eq!(br.ctrl & ctrl::ENABLE, ctrl::ENABLE);
    }

    // ==================== Device Trait Tests ====================

    #[test]
    fn test_device_id() {
        let br = Bootram::new();
        assert_eq!(br.id(), DeviceId::BOOTRAM);
    }

    #[test]
    fn test_device_reset() {
        let mut br = Bootram::new();

        br.write_word(0, 0x12345678);
        br.lock(0xDEADBEEF);
        br.ctrl = 0;

        br.reset();

        assert!(br.is_enabled());
        assert!(!br.is_locked());
        assert_eq!(br.read_word(0), 0);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_read_invalid_offset() {
        let mut br = Bootram::new();
        assert_eq!(br.read(BASE + 0x3000).unwrap(), 0);
    }

    #[test]
    fn test_write_invalid_offset() {
        let mut br = Bootram::new();
        // Should not panic
        br.write(BASE + 0x3000, 0x12345678).unwrap();
    }
}