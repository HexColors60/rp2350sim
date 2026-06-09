//! OTP (One-Time Programmable) memory controller for RP2350.
//!
//! Implements the OTP memory for permanent configuration storage.

use rp2350sim_core::{Device, DeviceId, Result};

/// OTP base address.
pub const OTP_BASE: u32 = 0x5003_0000;

/// OTP data base address (for reading).
pub const OTP_DATA_BASE: u32 = 0x4003_0000;

/// OTP register offsets.
pub mod regs {
    pub const CTRL: u32 = 0x000;
    pub const CTRL_SET: u32 = 0x004;
    pub const CTRL_CLR: u32 = 0x008;
    pub const STATUS: u32 = 0x00C;
    pub const CE: u32 = 0x010;
    pub const CE_SET: u32 = 0x014;
    pub const CE_CLR: u32 = 0x018;
    pub const KEY: u32 = 0x01C;
    pub const KEY_SET: u32 = 0x020;
    pub const KEY_CLR: u32 = 0x024;
    pub const DATA: u32 = 0x030;
    pub const ADDR: u32 = 0x034;
    pub const DBG_LOCK: u32 = 0x038;
    pub const DBG_LOCK_SET: u32 = 0x03C;
    pub const DBG_LOCK_CLR: u32 = 0x040;
    pub const SW_LOCK0: u32 = 0x044;
    pub const SW_LOCK1: u32 = 0x048;
    pub const SW_LOCK2: u32 = 0x04C;
    pub const SW_LOCK3: u32 = 0x050;
    pub const SW_LOCK4: u32 = 0x054;
    pub const SW_LOCK5: u32 = 0x058;
    pub const SW_LOCK6: u32 = 0x05C;
    pub const SW_LOCK7: u32 = 0x060;
    pub const SW_LOCK8: u32 = 0x064;
    pub const SW_LOCK9: u32 = 0x068;
    pub const SW_LOCK10: u32 = 0x06C;
    pub const SW_LOCK11: u32 = 0x070;
    pub const BOOTDIS: u32 = 0x074;
}

/// CTRL register bits.
pub mod ctrl {
    pub const WRITABLE: u32 = 1 << 0;
    pub const READABLE: u32 = 1 << 1;
    pub const ECC: u32 = 1 << 2;
    pub const POR: u32 = 1 << 3;
}

/// STATUS register bits.
pub mod status {
    pub const BUSY: u32 = 1 << 0;
    pub const ECC_FAIL: u32 = 1 << 1;
    pub const KEY_VALID: u32 = 1 << 2;
    pub const LOCKED: u32 = 1 << 3;
}

/// OTP size in rows (32-bit words).
const OTP_ROWS: usize = 4096;

/// OTP page size in rows.
const PAGE_SIZE: usize = 64;

/// Number of pages.
const NUM_PAGES: usize = OTP_ROWS / PAGE_SIZE;

/// OTP memory controller.
#[derive(Debug)]
pub struct Otp {
    /// Control register.
    ctrl: u32,
    /// Status register.
    status: u32,
    /// Chip enable.
    ce: u32,
    /// Key register.
    key: u32,
    /// Data register.
    #[allow(dead_code)]
    data: u32,
    /// Address register.
    addr: u32,
    /// Debug lock.
    dbg_lock: u32,
    /// Software locks (one per page).
    sw_lock: [u32; NUM_PAGES],
    /// OTP data storage.
    data_storage: [u32; OTP_ROWS],
    /// Written flags (to simulate one-time programming).
    written: [bool; OTP_ROWS],
}

impl Default for Otp {
    fn default() -> Self {
        Self::new()
    }
}

impl Otp {
    /// Create a new OTP controller.
    pub fn new() -> Self {
        let mut otp = Self {
            ctrl: ctrl::READABLE,  // Readable by default
            status: 0,
            ce: 0,
            key: 0,
            data: 0,
            addr: 0,
            dbg_lock: 0,
            sw_lock: [0; NUM_PAGES],
            data_storage: [0; OTP_ROWS],
            written: [false; OTP_ROWS],
        };

        // Initialize with default values
        otp.init_defaults();
        otp
    }

    /// Initialize default OTP values.
    fn init_defaults(&mut self) {
        // Row 0: Boot configuration
        self.data_storage[0] = 0x00000000;  // Default boot config
        
        // Row 1-3: Unique ID (simulated)
        self.data_storage[1] = 0xDEADBEEF;
        self.data_storage[2] = 0xCAFEBABE;
        self.data_storage[3] = 0x12345678;
        
        // Mark these as "factory programmed"
        for i in 0..4 {
            self.written[i] = true;
        }
    }

    /// Check if address is valid.
    #[allow(dead_code)]
    fn is_valid_addr(&self, addr: u32) -> bool {
        (addr as usize) < OTP_ROWS
    }

    /// Check if page is locked.
    fn is_page_locked(&self, row: usize) -> bool {
        let page = row / PAGE_SIZE;
        if page < NUM_PAGES {
            self.sw_lock[page] != 0 || self.dbg_lock != 0
        } else {
            true
        }
    }

    /// Read OTP data at address.
    fn read_otp(&self, addr: u32) -> u32 {
        let row = addr as usize;
        if row < OTP_ROWS {
            self.data_storage[row]
        } else {
            0
        }
    }

    /// Write OTP data at address.
    fn write_otp(&mut self, addr: u32, value: u32) -> bool {
        let row = addr as usize;
        if row >= OTP_ROWS {
            return false;
        }

        // Check if already written (OTP can only be written once per bit)
        if self.written[row] {
            // Can only change 1 bits to 0 bits
            let new_value = self.data_storage[row] & value;
            self.data_storage[row] = new_value;
        } else {
            self.data_storage[row] = value;
            self.written[row] = true;
        }

        true
    }

    /// Check if key is valid for write operations.
    fn is_key_valid(&self) -> bool {
        self.key == 0xDEADBEEF  // Simple key check
    }

    /// Get current address.
    pub fn get_addr(&self) -> u32 {
        self.addr
    }

    /// Set address for read/write.
    pub fn set_addr(&mut self, addr: u32) {
        self.addr = addr & 0xFFF;  // 12-bit address
    }

    /// Read data at current address.
    pub fn read_data(&mut self) -> u32 {
        if (self.ctrl & ctrl::READABLE) != 0 {
            self.read_otp(self.addr)
        } else {
            0
        }
    }

    /// Write data at current address.
    pub fn write_data(&mut self, value: u32) -> bool {
        if (self.ctrl & ctrl::WRITABLE) == 0 {
            return false;
        }

        if self.is_page_locked(self.addr as usize) {
            return false;
        }

        if !self.is_key_valid() {
            return false;
        }

        self.write_otp(self.addr, value)
    }

    /// Lock a page.
    pub fn lock_page(&mut self, page: usize) {
        if page < NUM_PAGES {
            self.sw_lock[page] = 1;
        }
    }

    /// Unlock a page.
    pub fn unlock_page(&mut self, page: usize) {
        if page < NUM_PAGES {
            self.sw_lock[page] = 0;
        }
    }

    /// Check if busy.
    pub fn is_busy(&self) -> bool {
        (self.status & status::BUSY) != 0
    }
}

impl Device for Otp {
    fn id(&self) -> DeviceId {
        DeviceId::OTP
    }

    fn read(&mut self, addr: u32) -> Result<u32> {
        let offset = addr - OTP_BASE;

        match offset {
            regs::CTRL => Ok(self.ctrl),
            regs::STATUS => Ok(self.status),
            regs::CE => Ok(self.ce),
            regs::KEY => Ok(0),  // Key is write-only
            regs::DATA => Ok(self.read_data()),
            regs::ADDR => Ok(self.addr),
            regs::DBG_LOCK => Ok(self.dbg_lock),
            0x044..=0x070 => {
                let lock_idx = ((offset - 0x044) / 4) as usize;
                if lock_idx < NUM_PAGES {
                    Ok(self.sw_lock[lock_idx])
                } else {
                    Ok(0)
                }
            }
            _ => Ok(0),
        }
    }

    fn write(&mut self, addr: u32, value: u32) -> Result<()> {
        let offset = addr - OTP_BASE;

        match offset {
            regs::CTRL | regs::CTRL_SET => {
                self.ctrl |= value;
            }
            regs::CTRL_CLR => {
                self.ctrl &= !value;
            }
            regs::CE | regs::CE_SET => {
                self.ce |= value;
            }
            regs::CE_CLR => {
                self.ce &= !value;
            }
            regs::KEY | regs::KEY_SET => {
                self.key = value;
                if value != 0 {
                    self.status |= status::KEY_VALID;
                }
            }
            regs::KEY_CLR => {
                self.key = 0;
                self.status &= !status::KEY_VALID;
            }
            regs::DATA => {
                self.write_data(value);
            }
            regs::ADDR => {
                self.set_addr(value);
            }
            regs::DBG_LOCK | regs::DBG_LOCK_SET => {
                self.dbg_lock |= value;
                if value != 0 {
                    self.status |= status::LOCKED;
                }
            }
            regs::DBG_LOCK_CLR => {
                self.dbg_lock &= !value;
                if self.dbg_lock == 0 {
                    self.status &= !status::LOCKED;
                }
            }
            0x044..=0x070 => {
                let lock_idx = ((offset - 0x044) / 4) as usize;
                if lock_idx < NUM_PAGES {
                    self.sw_lock[lock_idx] = value;
                }
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

    const OTP_BASE: u32 = super::OTP_BASE;

    #[test]
    fn test_otp_creation() {
        let otp = Otp::new();

        // Should be readable by default
        assert_eq!(otp.ctrl & ctrl::READABLE, ctrl::READABLE);
        assert!(!otp.is_busy());
    }

    #[test]
    fn test_otp_default() {
        let otp = Otp::default();
        assert_eq!(otp.ctrl & ctrl::READABLE, ctrl::READABLE);
    }

    #[test]
    fn test_otp_ctrl_register() {
        let mut otp = Otp::new();

        // Set via CTRL_SET
        otp.write(OTP_BASE + regs::CTRL_SET, ctrl::WRITABLE).unwrap();
        assert_eq!(otp.ctrl & ctrl::WRITABLE, ctrl::WRITABLE);

        // Clear via CTRL_CLR
        otp.write(OTP_BASE + regs::CTRL_CLR, ctrl::WRITABLE).unwrap();
        assert_eq!(otp.ctrl & ctrl::WRITABLE, 0);
    }

    #[test]
    fn test_otp_ce_register() {
        let mut otp = Otp::new();

        otp.write(OTP_BASE + regs::CE_SET, 1).unwrap();
        assert_eq!(otp.ce, 1);

        otp.write(OTP_BASE + regs::CE_CLR, 1).unwrap();
        assert_eq!(otp.ce, 0);
    }

    #[test]
    fn test_otp_key_register() {
        let mut otp = Otp::new();

        // Key is write-only, read returns 0
        otp.write(OTP_BASE + regs::KEY_SET, 0xDEADBEEF).unwrap();
        assert_eq!(otp.read(OTP_BASE + regs::KEY).unwrap(), 0);
        assert_eq!(otp.key, 0xDEADBEEF);

        // Status should show KEY_VALID
        assert_eq!(otp.read(OTP_BASE + regs::STATUS).unwrap() & status::KEY_VALID, status::KEY_VALID);

        // Clear key
        otp.write(OTP_BASE + regs::KEY_CLR, 1).unwrap();
        assert_eq!(otp.key, 0);
    }

    #[test]
    fn test_otp_addr_register() {
        let mut otp = Otp::new();

        otp.write(OTP_BASE + regs::ADDR, 0x123).unwrap();
        assert_eq!(otp.get_addr(), 0x123);

        // Only 12 bits valid
        otp.write(OTP_BASE + regs::ADDR, 0x1FFF).unwrap();
        assert_eq!(otp.get_addr(), 0xFFF);
    }

    #[test]
    fn test_otp_read_factory_data() {
        let otp = Otp::new();

        // Should have factory-programmed unique ID
        assert_eq!(otp.data_storage[1], 0xDEADBEEF);
        assert_eq!(otp.data_storage[2], 0xCAFEBABE);
        assert_eq!(otp.data_storage[3], 0x12345678);
    }

    #[test]
    fn test_otp_read_data() {
        let mut otp = Otp::new();

        otp.set_addr(1);
        let data = otp.read_data();
        assert_eq!(data, 0xDEADBEEF);
    }

    #[test]
    fn test_otp_write_needs_key() {
        let mut otp = Otp::new();

        otp.write(OTP_BASE + regs::CTRL_SET, ctrl::WRITABLE | ctrl::READABLE).unwrap();
        otp.write(OTP_BASE + regs::ADDR, 0x100).unwrap();

        // Without key, write should fail
        assert!(!otp.write_data(0x12345678));
        assert_eq!(otp.read_otp(0x100), 0);
    }

    #[test]
    fn test_otp_write_needs_writable() {
        let mut otp = Otp::new();

        otp.write(OTP_BASE + regs::KEY_SET, 0xDEADBEEF).unwrap();
        otp.write(OTP_BASE + regs::ADDR, 0x100).unwrap();

        // Without WRITABLE flag, write should fail
        assert!(!otp.write_data(0x12345678));
    }

    #[test]
    fn test_otp_write_success() {
        let mut otp = Otp::new();

        // Enable write and set key
        otp.write(OTP_BASE + regs::CTRL_SET, ctrl::WRITABLE | ctrl::READABLE).unwrap();
        otp.write(OTP_BASE + regs::KEY_SET, 0xDEADBEEF).unwrap();
        otp.write(OTP_BASE + regs::ADDR, 0x100).unwrap();

        // Write should succeed
        assert!(otp.write_data(0x12345678));
        assert_eq!(otp.read_otp(0x100), 0x12345678);
    }

    #[test]
    fn test_otp_write_once() {
        let mut otp = Otp::new();

        otp.write(OTP_BASE + regs::CTRL_SET, ctrl::WRITABLE).unwrap();
        otp.write(OTP_BASE + regs::KEY_SET, 0xDEADBEEF).unwrap();
        otp.write(OTP_BASE + regs::ADDR, 0x100).unwrap();

        // First write
        assert!(otp.write_data(0xFFFFFFFF));

        // Try to write again - OTP can only change 1->0
        assert!(otp.write_data(0xAAAAAAAA));
        // Result should be AND of old and new
        assert_eq!(otp.read_otp(0x100), 0xAAAAAAAA);
    }

    #[test]
    fn test_otp_page_lock() {
        let mut otp = Otp::new();

        otp.write(OTP_BASE + regs::CTRL_SET, ctrl::WRITABLE).unwrap();
        otp.write(OTP_BASE + regs::KEY_SET, 0xDEADBEEF).unwrap();

        // Lock page 4 (rows 256-319)
        otp.lock_page(4);

        // Try to write to locked page
        otp.write(OTP_BASE + regs::ADDR, 0x100).unwrap(); // Row 256 is in page 4
        assert!(!otp.write_data(0x12345678));

        // Unlock and try again
        otp.unlock_page(4);
        assert!(otp.write_data(0x12345678));
    }

    #[test]
    fn test_otp_debug_lock() {
        let mut otp = Otp::new();

        // Enable write and set key
        otp.write(OTP_BASE + regs::CTRL_SET, ctrl::WRITABLE).unwrap();
        otp.write(OTP_BASE + regs::KEY_SET, 0xDEADBEEF).unwrap();

        // Lock via debug lock
        otp.write(OTP_BASE + regs::DBG_LOCK_SET, 1).unwrap();
        assert_eq!(otp.read(OTP_BASE + regs::STATUS).unwrap() & status::LOCKED, status::LOCKED);

        // Write should fail
        otp.write(OTP_BASE + regs::ADDR, 0x100).unwrap();
        assert!(!otp.write_data(0x12345678));

        // Unlock
        otp.write(OTP_BASE + regs::DBG_LOCK_CLR, 1).unwrap();
        assert_eq!(otp.read(OTP_BASE + regs::STATUS).unwrap() & status::LOCKED, 0);
    }

    #[test]
    fn test_otp_sw_lock_registers() {
        let mut otp = Otp::new();

        // Write to SW_LOCK0
        otp.write(OTP_BASE + regs::SW_LOCK0, 1).unwrap();
        assert_eq!(otp.read(OTP_BASE + regs::SW_LOCK0).unwrap(), 1);

        // Write to SW_LOCK1
        otp.write(OTP_BASE + regs::SW_LOCK1, 1).unwrap();
        assert_eq!(otp.read(OTP_BASE + regs::SW_LOCK1).unwrap(), 1);
    }

    #[test]
    fn test_otp_invalid_address() {
        let mut otp = Otp::new();

        // Invalid read offset
        let result = otp.read(OTP_BASE + 0x1000).unwrap();
        assert_eq!(result, 0);

        // Invalid write offset should be ignored
        otp.write(OTP_BASE + 0x1000, 0x12345678).unwrap();
    }

    #[test]
    fn test_otp_reset() {
        let mut otp = Otp::new();

        // Configure and write
        otp.write(OTP_BASE + regs::CTRL_SET, ctrl::WRITABLE).unwrap();
        otp.write(OTP_BASE + regs::KEY_SET, 0xDEADBEEF).unwrap();
        otp.write(OTP_BASE + regs::ADDR, 0x100).unwrap();
        otp.write(OTP_BASE + regs::DATA, 0x12345678).unwrap();

        // Reset
        otp.reset();

        // Check state is reset (except factory data)
        assert_eq!(otp.ctrl & ctrl::READABLE, ctrl::READABLE);
        assert_eq!(otp.key, 0);
        // Factory data preserved
        assert_eq!(otp.data_storage[1], 0xDEADBEEF);
        // Written data cleared
        assert_eq!(otp.read_otp(0x100), 0);
        assert!(!otp.written[0x100]);
    }

    #[test]
    fn test_otp_device_id() {
        let otp = Otp::new();
        assert_eq!(otp.id(), DeviceId::OTP);
    }

    #[test]
    fn test_otp_data_write_via_register() {
        let mut otp = Otp::new();

        otp.write(OTP_BASE + regs::CTRL_SET, ctrl::WRITABLE).unwrap();
        otp.write(OTP_BASE + regs::KEY_SET, 0xDEADBEEF).unwrap();
        otp.write(OTP_BASE + regs::ADDR, 0x200).unwrap();
        otp.write(OTP_BASE + regs::DATA, 0xDEADC0DE).unwrap();

        // Read back via DATA register
        let data = otp.read(OTP_BASE + regs::DATA).unwrap();
        assert_eq!(data, 0xDEADC0DE);
    }

    #[test]
    fn test_otp_not_readable() {
        let mut otp = Otp::new();

        // Clear readable flag
        otp.write(OTP_BASE + regs::CTRL_CLR, ctrl::READABLE).unwrap();

        otp.set_addr(1);
        let data = otp.read_data();
        // Should return 0 when not readable
        assert_eq!(data, 0);
    }

    #[test]
    fn test_otp_key_valid_status() {
        let mut otp = Otp::new();

        // Initially no key
        assert_eq!(otp.read(OTP_BASE + regs::STATUS).unwrap() & status::KEY_VALID, 0);

        // Set key
        otp.write(OTP_BASE + regs::KEY_SET, 0xDEADBEEF).unwrap();
        assert_eq!(otp.read(OTP_BASE + regs::STATUS).unwrap() & status::KEY_VALID, status::KEY_VALID);

        // Wrong key
        otp.write(OTP_BASE + regs::KEY_SET, 0xBADC0DE).unwrap();
        // KEY_VALID still set (any non-zero key sets it)
        assert_eq!(otp.read(OTP_BASE + regs::STATUS).unwrap() & status::KEY_VALID, status::KEY_VALID);
    }

    #[test]
    fn test_otp_page_boundaries() {
        let mut otp = Otp::new();

        otp.write(OTP_BASE + regs::CTRL_SET, ctrl::WRITABLE).unwrap();
        otp.write(OTP_BASE + regs::KEY_SET, 0xDEADBEEF).unwrap();

        // Lock page 0
        otp.lock_page(0);

        // Row 0 is in page 0, should be locked
        otp.write(OTP_BASE + regs::ADDR, 0).unwrap();
        assert!(!otp.write_data(0x12345678));

        // Row 64 is in page 1, should be writable
        otp.write(OTP_BASE + regs::ADDR, 64).unwrap();
        assert!(otp.write_data(0x12345678));
    }

    #[test]
    fn test_otp_write_enable() {
        let mut otp = Otp::new();

        // Initially not writable
        otp.write(OTP_BASE + regs::ADDR, 0x100).unwrap();
        otp.write(OTP_BASE + regs::KEY_SET, 0xDEADBEEF).unwrap();
        assert!(!otp.write_data(0x12345678));

        // Enable write
        otp.write(OTP_BASE + regs::CTRL_SET, ctrl::WRITABLE).unwrap();
        assert!(otp.write_data(0x12345678));

        // Disable write via CTRL_CLR
        otp.write(OTP_BASE + regs::ADDR, 0x110).unwrap();
        otp.write(OTP_BASE + regs::CTRL_CLR, ctrl::WRITABLE).unwrap();
        assert!(!otp.write_data(0xABCDEF00));

        // Re-enable via CTRL_SET
        otp.write(OTP_BASE + regs::CTRL_SET, ctrl::WRITABLE).unwrap();
        assert!(otp.write_data(0xABCDEF00));
    }

    #[test]
    fn test_otp_page_erase() {
        let mut otp = Otp::new();

        // Setup: enable write and key, write to a row
        otp.write(OTP_BASE + regs::CTRL_SET, ctrl::WRITABLE).unwrap();
        otp.write(OTP_BASE + regs::KEY_SET, 0xDEADBEEF).unwrap();
        otp.write(OTP_BASE + regs::ADDR, 0x100).unwrap();
        assert!(otp.write_data(0xFFFFFFFF));

        // Note: RP2350 OTP does not support page erase
        // This test verifies that written data persists (OTP behavior)
        // CE register controls chip enable, not erase
        otp.write(OTP_BASE + regs::CE_SET, 0xFF).unwrap();
        assert_eq!(otp.read_otp(0x100), 0xFFFFFFFF);

        // Clear CE and verify data still persists
        otp.write(OTP_BASE + regs::CE_CLR, 0xFF).unwrap();
        assert_eq!(otp.read_otp(0x100), 0xFFFFFFFF);
    }

    #[test]
    fn test_otp_read_row() {
        let mut otp = Otp::new();

        // Read factory rows (pre-programmed)
        assert_eq!(otp.read_otp(0), 0x00000000);
        assert_eq!(otp.read_otp(1), 0xDEADBEEF);
        assert_eq!(otp.read_otp(2), 0xCAFEBABE);
        assert_eq!(otp.read_otp(3), 0x12345678);

        // Read unwritten user area (should be 0)
        assert_eq!(otp.read_otp(4), 0);
        assert_eq!(otp.read_otp(0x100), 0);
        assert_eq!(otp.read_otp(0xFFF), 0); // Last valid row

        // Write to user area and read back
        otp.write(OTP_BASE + regs::CTRL_SET, ctrl::WRITABLE).unwrap();
        otp.write(OTP_BASE + regs::KEY_SET, 0xDEADBEEF).unwrap();
        otp.write(OTP_BASE + regs::ADDR, 0x200).unwrap();
        otp.write_data(0xAABBCCDD);
        assert_eq!(otp.read_otp(0x200), 0xAABBCCDD);
    }

    #[test]
    fn test_otp_key_write() {
        let mut otp = Otp::new();

        // Set key to correct value
        otp.write(OTP_BASE + regs::KEY_SET, 0xDEADBEEF).unwrap();
        assert!(otp.is_key_valid());
        assert_eq!(otp.key, 0xDEADBEEF);

        // KEY register is write-only (reads as 0)
        assert_eq!(otp.read(OTP_BASE + regs::KEY).unwrap(), 0);

        // Write with correct key should succeed
        otp.write(OTP_BASE + regs::CTRL_SET, ctrl::WRITABLE).unwrap();
        otp.write(OTP_BASE + regs::ADDR, 0x150).unwrap();
        assert!(otp.write_data(0x11223344));

        // Clear key via KEY_CLR
        otp.write(OTP_BASE + regs::KEY_CLR, 1).unwrap();
        assert_eq!(otp.key, 0);
        assert!(!otp.is_key_valid());

        // Write should fail without key
        otp.write(OTP_BASE + regs::ADDR, 0x160).unwrap();
        assert!(!otp.write_data(0x55667788));

        // Set wrong key - write should fail (key check is 0xDEADBEEF)
        otp.write(OTP_BASE + regs::KEY_SET, 0xBADC0DE).unwrap();
        otp.write(OTP_BASE + regs::ADDR, 0x170).unwrap();
        assert!(!otp.write_data(0x99AABBCC));
    }

    #[test]
    fn test_otp_factory_data() {
        let otp = Otp::new();

        // Factory data area: rows 0-3
        // Row 0: boot config (default 0)
        assert_eq!(otp.read_otp(0), 0x00000000);

        // Rows 1-3: unique ID (factory programmed)
        assert_eq!(otp.read_otp(1), 0xDEADBEEF);
        assert_eq!(otp.read_otp(2), 0xCAFEBABE);
        assert_eq!(otp.read_otp(3), 0x12345678);

        // Factory rows are marked as written
        assert!(otp.written[0]);
        assert!(otp.written[1]);
        assert!(otp.written[2]);
        assert!(otp.written[3]);

        // User area starts at row 4, not written yet
        assert!(!otp.written[4]);
    }

    #[test]
    fn test_otp_user_area() {
        let mut otp = Otp::new();

        // User area: rows 4 to 4095
        // Verify initial state is all zeros and unwritten
        for row in [4usize, 100, 500, 1000, 4000, 4095] {
            assert_eq!(otp.read_otp(row as u32), 0);
            assert!(!otp.written[row]);
        }

        // Write to multiple user area rows
        otp.write(OTP_BASE + regs::CTRL_SET, ctrl::WRITABLE).unwrap();
        otp.write(OTP_BASE + regs::KEY_SET, 0xDEADBEEF).unwrap();

        for (row, value) in [(4usize, 0x11111111u32), (100, 0x22222222), (500, 0x33333333)] {
            otp.write(OTP_BASE + regs::ADDR, row as u32).unwrap();
            assert!(otp.write_data(value));
            assert_eq!(otp.read_otp(row as u32), value);
            assert!(otp.written[row]);
        }

        // Verify OTP behavior: can only change 1->0 bits
        otp.write(OTP_BASE + regs::ADDR, 4).unwrap();
        assert!(otp.write_data(0x10101010));
        assert_eq!(otp.read_otp(4), 0x10101010); // 0x11111111 & 0x10101010
    }

    #[test]
    fn test_otp_lock() {
        let mut otp = Otp::new();

        // Setup for writes
        otp.write(OTP_BASE + regs::CTRL_SET, ctrl::WRITABLE).unwrap();
        otp.write(OTP_BASE + regs::KEY_SET, 0xDEADBEEF).unwrap();

        // Test software lock via lock_page()
        otp.lock_page(2); // Page 2 covers rows 128-191
        otp.write(OTP_BASE + regs::ADDR, 130).unwrap();
        assert!(!otp.write_data(0xDEADBEEF)); // Should fail - page locked

        // Unlock page
        otp.unlock_page(2);
        assert!(otp.write_data(0xDEADBEEF)); // Should succeed now

        // Test software lock via register write
        otp.write(OTP_BASE + regs::SW_LOCK3, 1).unwrap(); // Page 3: rows 192-255
        otp.write(OTP_BASE + regs::ADDR, 200).unwrap();
        assert!(!otp.write_data(0xCAFEBABE));
        assert_eq!(otp.read(OTP_BASE + regs::SW_LOCK3).unwrap(), 1);

        // Clear software lock via register
        otp.write(OTP_BASE + regs::SW_LOCK3, 0).unwrap();
        assert!(otp.write_data(0xCAFEBABE));

        // Test debug lock
        otp.write(OTP_BASE + regs::ADDR, 300).unwrap();
        otp.write(OTP_BASE + regs::DBG_LOCK_SET, 1).unwrap();
        assert!(!otp.write_data(0x12345678));
        assert_eq!(otp.read(OTP_BASE + regs::STATUS).unwrap() & status::LOCKED, status::LOCKED);

        // Clear debug lock
        otp.write(OTP_BASE + regs::DBG_LOCK_CLR, 0xFF).unwrap();
        assert!(otp.write_data(0x12345678));
        assert_eq!(otp.read(OTP_BASE + regs::STATUS).unwrap() & status::LOCKED, 0);
    }

    #[test]
    fn test_otp_out_of_bounds() {
        let mut otp = Otp::new();

        // Out of bounds read returns 0
        assert_eq!(otp.read_otp(OTP_ROWS as u32), 0);
        assert_eq!(otp.read_otp(0xFFFFFFFF), 0);

        // Out of bounds write returns false
        assert!(!otp.write_otp(OTP_ROWS as u32, 0x12345678));
        assert!(!otp.write_otp(0xFFFFFFFF, 0xDEADBEEF));

        // is_valid_addr checks
        assert!(otp.is_valid_addr(0));
        assert!(otp.is_valid_addr((OTP_ROWS - 1) as u32));
        assert!(!otp.is_valid_addr(OTP_ROWS as u32));
        assert!(!otp.is_valid_addr(0xFFFFFFFF));
    }

    #[test]
    fn test_otp_write_when_disabled() {
        let mut otp = Otp::new();

        // Test all failure modes for write_data
        otp.write(OTP_BASE + regs::ADDR, 0x100).unwrap();

        // 1. No writable flag - should fail
        otp.write(OTP_BASE + regs::KEY_SET, 0xDEADBEEF).unwrap();
        assert!(!otp.write_data(0x11111111));

        // 2. Page locked - should fail
        otp.write(OTP_BASE + regs::CTRL_SET, ctrl::WRITABLE).unwrap();
        otp.lock_page(4); // Row 0x100 is in page 4
        assert!(!otp.write_data(0x22222222));
        otp.unlock_page(4);

        // 3. Invalid key - should fail
        otp.write(OTP_BASE + regs::KEY_CLR, 1).unwrap();
        assert!(!otp.write_data(0x33333333));

        // 4. Debug locked - should fail
        otp.write(OTP_BASE + regs::KEY_SET, 0xDEADBEEF).unwrap();
        otp.write(OTP_BASE + regs::DBG_LOCK_SET, 1).unwrap();
        assert!(!otp.write_data(0x44444444));

        // 5. All conditions met - should succeed
        otp.write(OTP_BASE + regs::DBG_LOCK_CLR, 0xFF).unwrap();
        assert!(otp.write_data(0x55555555));
    }
}