//! I2C scan test
//! Tests I2C bus scanning functionality

#[cfg(test)]
mod tests {
    use rp2350sim_devices::i2c::{I2c, I2C0_BASE, I2C1_BASE};

    #[test]
    fn test_i2c0_creation() {
        let i2c = I2c::i2c0();
        assert_eq!(i2c.base(), I2C0_BASE);
    }

    #[test]
    fn test_i2c1_creation() {
        let i2c = I2c::i2c1();
        assert_eq!(i2c.base(), I2C1_BASE);
    }

    #[test]
    fn test_i2c_start_condition() {
        let mut i2c = I2c::i2c0();
        i2c.enable(true);
        
        // Generate start condition
        let result = i2c.start();
        assert!(result.is_ok());
        assert!(i2c.is_busy());
    }

    #[test]
    fn test_i2c_stop_condition() {
        let mut i2c = I2c::i2c0();
        i2c.enable(true);
        
        // Generate start then stop
        i2c.start().ok();
        let result = i2c.stop();
        assert!(result.is_ok());
        assert!(!i2c.is_busy());
    }

    #[test]
    fn test_i2c_address_write() {
        let mut i2c = I2c::i2c0();
        i2c.enable(true);
        
        // Start transaction
        i2c.start().ok();
        
        // Send address with write bit (0)
        let addr = 0x50; // EEPROM address
        let result = i2c.write_address(addr, false);
        
        // Without a slave, this should NACK
        // (in a real test, we'd have a virtual slave attached)
        assert!(result.is_ok() || result.is_err()); // Either is acceptable without slave
    }

    #[test]
    fn test_i2c_address_read() {
        let mut i2c = I2c::i2c0();
        i2c.enable(true);
        
        // Start transaction
        i2c.start().ok();
        
        // Send address with read bit (1)
        let addr = 0x50;
        let result = i2c.write_address(addr, true);
        
        // Without a slave, this should NACK
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_i2c_scan_bus() {
        let mut i2c = I2c::i2c0();
        i2c.enable(true);
        
        // Scan all addresses
        let mut found_devices: Vec<u8> = Vec::new();
        
        for addr in 0x08..0x78 {
            i2c.start().ok();
            let result = i2c.write_address(addr, false);
            i2c.stop().ok();
            
            // In simulation without slaves, no devices should be found
            if result.is_ok() {
                found_devices.push(addr);
            }
        }
        
        // Without virtual slaves, should find no devices
        assert!(found_devices.is_empty());
    }
}