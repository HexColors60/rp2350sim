//! PWM basic test
//! Tests PWM channel functionality

#[cfg(test)]
mod tests {
    use rp2350sim_devices::pwm::{Pwm, PwmChannel, PwmConfig};

    #[test]
    fn test_pwm_creation() {
        let pwm = Pwm::new();
        assert_eq!(pwm.channel_count(), 24); // 12 slices * 2 channels
    }

    #[test]
    fn test_pwm_channel_enable() {
        let mut pwm = Pwm::new();
        
        // Enable channel 0
        pwm.enable_channel(0, true);
        assert!(pwm.is_channel_enabled(0));
        
        // Disable channel 0
        pwm.enable_channel(0, false);
        assert!(!pwm.is_channel_enabled(0));
    }

    #[test]
    fn test_pwm_duty_cycle() {
        let mut pwm = Pwm::new();
        pwm.enable_channel(0, true);
        
        // Set 50% duty cycle
        pwm.set_duty(0, 32768); // Out of 65536
        assert_eq!(pwm.get_duty(0), 32768);
        
        // Set 25% duty cycle
        pwm.set_duty(0, 16384);
        assert_eq!(pwm.get_duty(0), 16384);
        
        // Set 100% duty cycle
        pwm.set_duty(0, 65535);
        assert_eq!(pwm.get_duty(0), 65535);
    }

    #[test]
    fn test_pwm_divider() {
        let mut pwm = Pwm::new();
        
        // Set clock divider
        pwm.set_divider(0, 4.0);
        let div = pwm.get_divider(0);
        
        assert!(div > 3.9 && div < 4.1);
    }

    #[test]
    fn test_pwm_frequency() {
        let mut pwm = Pwm::new();
        pwm.enable_channel(0, true);
        
        // Set up for 1kHz output
        // With 150MHz clock and divider of 125, we get 1.2MHz
        // With wrap of 1200, we get 1kHz
        pwm.set_divider(0, 125.0);
        pwm.set_wrap(0, 1200);
        
        let freq = pwm.get_frequency(0);
        assert!(freq > 900 && freq < 1100, "Frequency should be ~1kHz, got {}", freq);
    }

    #[test]
    fn test_pwm_output_level() {
        let mut pwm = Pwm::new();
        pwm.enable_channel(0, true);
        
        // At 0% duty, output should be low
        pwm.set_duty(0, 0);
        assert!(!pwm.get_output_level(0));
        
        // At 100% duty, output should be high
        pwm.set_duty(0, 65535);
        assert!(pwm.get_output_level(0));
    }

    #[test]
    fn test_pwm_phase_correct() {
        let mut pwm = Pwm::new();
        
        // Enable phase-correct mode
        pwm.set_phase_correct(0, true);
        assert!(pwm.is_phase_correct(0));
        
        // Disable phase-correct mode
        pwm.set_phase_correct(0, false);
        assert!(!pwm.is_phase_correct(0));
    }
}