//! SPI loopback test
//! Tests SPI transmit and receive functionality

#[cfg(test)]
mod tests {
    use rp2350sim_devices::spi::{Spi, SPI0_BASE, SPI1_BASE};

    #[test]
    fn test_spi0_creation() {
        let spi = Spi::spi0();
        assert_eq!(spi.base(), SPI0_BASE);
    }

    #[test]
    fn test_spi1_creation() {
        let spi = Spi::spi1();
        assert_eq!(spi.base(), SPI1_BASE);
    }

    #[test]
    fn test_spi_loopback_single_byte() {
        let mut spi = Spi::spi0();
        spi.enable(true);
        
        // In loopback mode, TX data appears on RX
        spi.set_loopback(true);
        
        // Transfer a byte
        let tx_data = 0xAB;
        let rx_data = spi.transfer(tx_data);
        
        // In loopback, should receive what we sent
        assert_eq!(rx_data, Some(tx_data));
    }

    #[test]
    fn test_spi_loopback_multiple_bytes() {
        let mut spi = Spi::spi0();
        spi.enable(true);
        spi.set_loopback(true);
        
        let tx_data: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
        let mut rx_data: [u8; 4] = [0; 4];
        
        for (i, &tx) in tx_data.iter().enumerate() {
            rx_data[i] = spi.transfer(tx).unwrap_or(0);
        }
        
        assert_eq!(tx_data, rx_data);
    }

    #[test]
    fn test_spi_clock_polarity() {
        let mut spi = Spi::spi0();
        
        // Test CPOL = 0 (clock idle low)
        spi.set_clock_polarity(false);
        assert!(!spi.get_clock_polarity());
        
        // Test CPOL = 1 (clock idle high)
        spi.set_clock_polarity(true);
        assert!(spi.get_clock_polarity());
    }

    #[test]
    fn test_spi_clock_phase() {
        let mut spi = Spi::spi0();
        
        // Test CPHA = 0 (capture on first edge)
        spi.set_clock_phase(false);
        assert!(!spi.get_clock_phase());
        
        // Test CPHA = 1 (capture on second edge)
        spi.set_clock_phase(true);
        assert!(spi.get_clock_phase());
    }

    #[test]
    fn test_spi_chip_select() {
        let mut spi = Spi::spi0();
        spi.enable(true);
        
        // Assert CS
        spi.set_cs(0, false); // Active low
        assert!(!spi.get_cs(0));
        
        // Deassert CS
        spi.set_cs(0, true);
        assert!(spi.get_cs(0));
    }
}