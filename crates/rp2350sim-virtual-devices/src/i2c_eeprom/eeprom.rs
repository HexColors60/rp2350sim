//! I2C EEPROM emulation.

/// I2C EEPROM device.
#[derive(Debug)]
pub struct I2cEeprom {
    /// Device address (7-bit).
    address: u8,
    /// Memory contents.
    memory: Vec<u8>,
    /// Current address pointer.
    addr_ptr: usize,
    /// Size in bytes.
    size: usize,
    /// Write state (receiving address or data).
    receiving_addr: bool,
}

impl I2cEeprom {
    /// Create a new EEPROM with the given size.
    pub fn new(address: u8, size_kb: usize) -> Self {
        let size = size_kb * 1024;
        Self {
            address,
            memory: vec![0xFF; size],
            addr_ptr: 0,
            size,
            receiving_addr: true,
        }
    }

    /// Create a 24LC256 (32KB) EEPROM.
    pub fn at24c256(address: u8) -> Self {
        Self::new(address, 32)
    }

    /// Get the device address.
    pub fn address(&self) -> u8 {
        self.address
    }

    /// Read a byte from memory.
    pub fn read_byte(&self, addr: usize) -> u8 {
        let addr = addr % self.size;
        self.memory[addr]
    }

    /// Write a byte to memory.
    pub fn write_byte(&mut self, addr: usize, data: u8) {
        let addr = addr % self.size;
        self.memory[addr] = data;
    }
}

impl rp2350sim_devices::i2c::I2cSlave for I2cEeprom {
    fn match_address(&self, addr: u8) -> bool {
        self.address == addr
    }

    fn write(&mut self, data: u8) {
        if self.receiving_addr {
            // First byte is the address pointer (high byte for larger EEPROMs)
            self.addr_ptr = (self.addr_ptr & 0x00FF) | ((data as usize) << 8);
            self.receiving_addr = false;
        } else {
            // Check if this is the second address byte or data
            if self.addr_ptr < 0x100 && self.memory.len() > 256 {
                // This is the low byte of address
                self.addr_ptr = (self.addr_ptr & 0xFF00) | (data as usize);
            } else {
                // This is data
                self.write_byte(self.addr_ptr, data);
                self.addr_ptr = self.addr_ptr.wrapping_add(1);
            }
        }
    }

    fn read(&mut self, _ack: bool) -> u8 {
        let byte = self.read_byte(self.addr_ptr);
        self.addr_ptr = self.addr_ptr.wrapping_add(1);
        byte
    }
}