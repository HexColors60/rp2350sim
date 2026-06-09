//! PIO WS2812 test
//! Tests PIO state machine functionality for WS2812 LED driving

#[cfg(test)]
mod tests {
    use rp2350sim_devices::pio::{Pio, StateMachine, PioInstruction};

    #[test]
    fn test_pio_creation() {
        let pio = Pio::pio0();
        assert_eq!(pio.state_machine_count(), 4);
    }

    #[test]
    fn test_pio_instruction_memory() {
        let mut pio = Pio::pio0();
        
        // Load a simple program
        let program: [u16; 4] = [
            0xE080, // SET X, 0
            0x0085, // JMP X-- label
            0x0000, // JMP 0
            0x0000, // NOP
        ];
        
        pio.load_program(0, &program);
        
        // Verify program loaded
        for (i, &instr) in program.iter().enumerate() {
            assert_eq!(pio.get_instruction(i as u8), Some(instr));
        }
    }

    #[test]
    fn test_pio_state_machine_enable() {
        let mut pio = Pio::pio0();
        
        // Enable state machine 0
        pio.enable_sm(0, true);
        assert!(pio.is_sm_enabled(0));
        
        // Disable state machine 0
        pio.enable_sm(0, false);
        assert!(!pio.is_sm_enabled(0));
    }

    #[test]
    fn test_pio_fifo_tx() {
        let mut pio = Pio::pio0();
        
        // Push data to TX FIFO
        pio.push_tx(0, 0x12345678);
        pio.push_tx(0, 0xABCDEF00);
        
        // Check FIFO depth
        assert_eq!(pio.tx_fifo_len(0), 2);
        
        // Pull from FIFO
        let value = pio.pull_tx(0);
        assert_eq!(value, Some(0x12345678));
    }

    #[test]
    fn test_pio_fifo_rx() {
        let mut pio = Pio::pio0();
        
        // Simulate RX data
        pio.push_rx(0, 0xDEADBEEF);
        
        // Check RX FIFO
        assert_eq!(pio.rx_fifo_len(0), 1);
        
        // Pull from RX FIFO
        let value = pio.pull_rx(0);
        assert_eq!(value, Some(0xDEADBEEF));
    }

    #[test]
    fn test_pio_clock_divider() {
        let mut pio = Pio::pio0();
        
        // Set clock divider
        pio.set_clkdiv(0, 2.5);
        
        let div = pio.get_clkdiv(0);
        assert!((div - 2.5).abs() < 0.01);
    }

    #[test]
    fn test_pio_pin_mapping() {
        let mut pio = Pio::pio0();
        
        // Set base pin for state machine 0
        pio.set_out_base(0, 5);
        pio.set_in_base(0, 5);
        pio.set_set_base(0, 5);
        pio.set_sideset_base(0, 5);
        
        assert_eq!(pio.get_out_base(0), 5);
        assert_eq!(pio.get_in_base(0), 5);
        assert_eq!(pio.get_set_base(0), 5);
        assert_eq!(pio.get_sideset_base(0), 5);
    }

    #[test]
    fn test_pio_ws2812_timing() {
        let mut pio = Pio::pio0();
        
        // WS2812 requires specific timing:
        // T0H: 0.4us, T0L: 0.85us
        // T1H: 0.8us, T1L: 0.45us
        // Total bit time: ~1.25us
        
        // With 125MHz PIO clock, 1 cycle = 8ns
        // T0H = 50 cycles, T0L = 106 cycles
        // T1H = 100 cycles, T1L = 56 cycles
        
        // Set clock divider for correct timing
        // For 800kHz data rate, need ~1.25us per bit
        // With div=1, each instruction cycle is 8ns
        // Need ~156 cycles per bit
        
        pio.set_clkdiv(0, 1.0);
        
        // Verify timing is approximately correct
        let cycles_per_bit = 156;
        let clock_mhz = 125.0 / pio.get_clkdiv(0);
        let us_per_bit = cycles_per_bit as f64 / clock_mhz;
        
        assert!(us_per_bit > 1.0 && us_per_bit < 1.5);
    }
}