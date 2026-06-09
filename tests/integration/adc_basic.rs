//! ADC basic test
//! Tests ADC channel functionality

#[cfg(test)]
mod tests {
    use rp2350sim_devices::adc::{Adc, AdcChannel, AdcSource};

    #[test]
    fn test_adc_creation() {
        let adc = Adc::new();
        assert_eq!(adc.channel_count(), 4);
    }

    #[test]
    fn test_adc_channel_selection() {
        let mut adc = Adc::new();
        
        // Select channel 0
        adc.select_channel(0);
        assert_eq!(adc.get_selected_channel(), 0);
        
        // Select channel 3
        adc.select_channel(3);
        assert_eq!(adc.get_selected_channel(), 3);
    }

    #[test]
    fn test_adc_read_constant() {
        let mut adc = Adc::new();
        adc.enable(true);
        
        // Set channel 0 to constant value (50% = 2048)
        adc.set_input_value(0, 2048);
        adc.select_channel(0);
        
        // Read the value
        let value = adc.read();
        assert_eq!(value, Some(2048));
    }

    #[test]
    fn test_adc_read_full_scale() {
        let mut adc = Adc::new();
        adc.enable(true);
        
        // Test minimum (0)
        adc.set_input_value(0, 0);
        adc.select_channel(0);
        assert_eq!(adc.read(), Some(0));
        
        // Test maximum (4095 for 12-bit ADC)
        adc.set_input_value(0, 4095);
        assert_eq!(adc.read(), Some(4095));
    }

    #[test]
    fn test_adc_temperature_sensor() {
        let mut adc = Adc::new();
        adc.enable(true);
        
        // Enable temperature sensor (channel 4 on RP2350)
        adc.enable_temperature_sensor(true);
        
        // Read temperature sensor
        adc.select_channel(4);
        let temp_raw = adc.read();
        
        // Should get some value (typically around 0.7V at 27°C)
        assert!(temp_raw.is_some());
        let value = temp_raw.unwrap();
        assert!(value > 500 && value < 3500);
    }

    #[test]
    fn test_adc_fifo() {
        let mut adc = Adc::new();
        adc.enable(true);
        
        // Enable FIFO
        adc.set_fifo_enabled(true);
        
        // Do multiple conversions
        adc.set_input_value(0, 1000);
        adc.select_channel(0);
        
        for _ in 0..4 {
            adc.start_conversion();
            adc.wait_conversion();
        }
        
        // FIFO should have 4 entries
        assert_eq!(adc.fifo_len(), 4);
        
        // Drain FIFO
        let mut values = Vec::new();
        while let Some(v) = adc.fifo_pop() {
            values.push(v);
        }
        
        assert_eq!(values.len(), 4);
        for v in values {
            assert!(v >= 900 && v <= 1100);
        }
    }

    #[test]
    fn test_adc_round_robin() {
        let mut adc = Adc::new();
        adc.enable(true);
        
        // Set different values on each channel
        adc.set_input_value(0, 100);
        adc.set_input_value(1, 200);
        adc.set_input_value(2, 300);
        adc.set_input_value(3, 400);
        
        // Enable round-robin on channels 0-3
        adc.set_round_robin(true, 0b1111);
        
        // Do 4 conversions
        for _ in 0..4 {
            adc.start_conversion();
            adc.wait_conversion();
        }
        
        // Should have read each channel once
        assert_eq!(adc.fifo_len(), 4);
    }
}