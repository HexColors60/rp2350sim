//! GPIO blink test
//! Tests GPIO output functionality

#[cfg(test)]
mod tests {
    use rp2350sim_devices::gpio::{Gpio, GpioFunction};

    #[test]
    fn test_gpio_blink_sequence() {
        let mut gpio = Gpio::new();
        
        // Set GPIO 25 (LED pin) as output
        gpio.set_dir(25, true);
        
        // Turn LED on
        gpio.set_output(25, true);
        assert!(gpio.get_value(25), "LED should be on");
        
        // Turn LED off
        gpio.set_output(25, false);
        assert!(!gpio.get_value(25), "LED should be off");
        
        // Turn LED on again
        gpio.set_output(25, true);
        assert!(gpio.get_value(25), "LED should be on again");
    }

    #[test]
    fn test_gpio_multiple_pins() {
        let mut gpio = Gpio::new();
        
        // Set multiple pins as outputs
        for pin in 0..8 {
            gpio.set_dir(pin, true);
        }
        
        // Set alternating pattern
        for pin in (0..8).step_by(2) {
            gpio.set_output(pin, true);
        }
        
        // Verify pattern
        for pin in 0..8 {
            let expected = pin % 2 == 0;
            assert_eq!(gpio.get_value(pin), expected, "Pin {} should be {}", pin, expected);
        }
    }

    #[test]
    fn test_gpio_input_output() {
        let mut gpio = Gpio::new();
        
        // Set pin 0 as output, pin 1 as input
        gpio.set_dir(0, true);
        gpio.set_dir(1, false);
        
        // Set output on pin 0
        gpio.set_output(0, true);
        
        // Simulate external connection: connect pin 0 output to pin 1 input
        let value = gpio.get_value(0);
        gpio.set_input(1, value);
        
        assert_eq!(gpio.get_value(1), true, "Input should match output");
    }

    #[test]
    fn test_gpio_function_mux() {
        let mut gpio = Gpio::new();
        
        // Test function selection
        gpio.get_pin_mut(0).unwrap().function = GpioFunction::Spi;
        assert_eq!(gpio.get_pin(0).unwrap().function, GpioFunction::Spi);
        
        gpio.get_pin_mut(1).unwrap().function = GpioFunction::Uart;
        assert_eq!(gpio.get_pin(1).unwrap().function, GpioFunction::Uart);
        
        gpio.get_pin_mut(2).unwrap().function = GpioFunction::I2c;
        assert_eq!(gpio.get_pin(2).unwrap().function, GpioFunction::I2c);
    }
}