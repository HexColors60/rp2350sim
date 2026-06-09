//! UART echo test
//! Tests UART transmit and receive functionality

#[cfg(test)]
mod tests {
    use rp2350sim_devices::uart::{Uart, UART0_BASE, UART1_BASE, cr};

    #[test]
    fn test_uart0_creation() {
        let uart = Uart::uart0();
        assert_eq!(uart.base(), UART0_BASE);
    }

    #[test]
    fn test_uart1_creation() {
        let uart = Uart::uart1();
        assert_eq!(uart.base(), UART1_BASE);
    }

    #[test]
    fn test_uart_echo_single_byte() {
        let mut uart = Uart::uart0();
        uart.cr = cr::UARTEN | cr::TXE | cr::RXE;
        
        // Send a byte
        uart.push_rx(b'H');
        
        // Read it back (echo)
        let received = uart.read_byte();
        assert_eq!(received, Some(b'H'));
        
        // Transmit it
        uart.write_byte(b'H');
        
        // Check TX FIFO
        assert_eq!(uart.tx_fifo.len(), 1);
    }

    #[test]
    fn test_uart_echo_string() {
        let mut uart = Uart::uart0();
        uart.cr = cr::UARTEN | cr::TXE | cr::RXE;
        
        let test_string = b"Hello, RP2350!";
        
        // Send each byte
        for &byte in test_string {
            uart.push_rx(byte);
        }
        
        // Read and echo each byte
        let mut echoed = Vec::new();
        for _ in test_string {
            if let Some(byte) = uart.read_byte() {
                uart.write_byte(byte);
                echoed.push(byte);
            }
        }
        
        assert_eq!(&echoed[..], test_string);
    }

    #[test]
    fn test_uart_fifo_full() {
        let mut uart = Uart::uart0();
        uart.cr = cr::UARTEN | cr::RXE;
        
        // Fill RX FIFO (typically 32 bytes)
        for i in 0..32 {
            uart.push_rx(i as u8);
        }
        
        // FIFO should be full
        assert!(uart.rx_fifo.is_full() || uart.rx_fifo.len() == 32);
    }

    #[test]
    fn test_uart_baud_rate() {
        let mut uart = Uart::uart0();
        
        // Set baud rate divider
        uart.set_baud_divisor(9600, 48_000_000);
        
        // Baud rate should be approximately 9600
        // (exact calculation depends on implementation)
        assert!(uart.baud_rate() > 9000);
        assert!(uart.baud_rate() < 10000);
    }
}