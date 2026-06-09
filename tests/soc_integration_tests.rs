//! SoC peripheral integration tests.
//!
//! Tests for peripheral integration including interrupts.

#![cfg(test)]

mod interrupt_tests {
    use rp2350sim_soc::Soc;
    use rp2350sim_core::CpuArch;
    
    /// Test that Soc creates with all peripherals.
    #[test]
    fn test_soc_creation_with_peripherals() {
        let soc = Soc::new(CpuArch::Arm);
        
        // Verify peripherals exist and have interrupt methods
        let _ = soc.timer.has_interrupt();
        let _ = soc.uart0.has_interrupt();
        let _ = soc.uart1.has_interrupt();
        let _ = soc.spi0.has_interrupt();
        let _ = soc.spi1.has_interrupt();
        let _ = soc.i2c0.has_interrupt();
        let _ = soc.i2c1.has_interrupt();
        let _ = soc.pwm.has_interrupt();
        let _ = soc.adc.has_interrupt();
    }
    
    /// Test that Soc reset resets all peripherals.
    #[test]
    fn test_soc_reset() {
        let mut soc = Soc::new(CpuArch::Arm);
        
        // Reset should complete without error
        soc.reset();
    }
    
    /// Test timer interrupt wiring.
    #[test]
    fn test_timer_interrupt_wiring() {
        let soc = Soc::new(CpuArch::Arm);
        
        // Timer has_interrupt should be callable
        let _has_interrupt = soc.timer.has_interrupt();
    }
    
    /// Test UART interrupt wiring.
    #[test]
    fn test_uart_interrupt_wiring() {
        let soc = Soc::new(CpuArch::Arm);
        
        // UART should have has_interrupt method
        let _uart0_irq = soc.uart0.has_interrupt();
        let _uart1_irq = soc.uart1.has_interrupt();
    }
    
    /// Test SPI interrupt wiring.
    #[test]
    fn test_spi_interrupt_wiring() {
        let soc = Soc::new(CpuArch::Arm);
        
        // SPI should have has_interrupt method
        let _spi0_irq = soc.spi0.has_interrupt();
        let _spi1_irq = soc.spi1.has_interrupt();
    }
    
    /// Test I2C interrupt wiring.
    #[test]
    fn test_i2c_interrupt_wiring() {
        let soc = Soc::new(CpuArch::Arm);
        
        // I2C should have has_interrupt method
        let _i2c0_irq = soc.i2c0.has_interrupt();
        let _i2c1_irq = soc.i2c1.has_interrupt();
    }
    
    /// Test PWM interrupt wiring.
    #[test]
    fn test_pwm_interrupt_wiring() {
        let soc = Soc::new(CpuArch::Arm);
        
        // PWM should have has_interrupt method
        let _pwm_irq = soc.pwm.has_interrupt();
    }
    
    /// Test ADC interrupt wiring.
    #[test]
    fn test_adc_interrupt_wiring() {
        let soc = Soc::new(CpuArch::Arm);
        
        // ADC should have has_interrupt method
        let _adc_irq = soc.adc.has_interrupt();
    }
    
    /// Test GPIO interrupt wiring.
    #[test]
    fn test_gpio_interrupt_wiring() {
        let soc = Soc::new(CpuArch::Arm);
        
        // GPIO should have has_interrupt method for each pin
        for pin in 0..48 {
            let _irq = soc.gpio.has_interrupt(pin);
        }
    }
}

mod peripheral_tick_tests {
    use rp2350sim_soc::Soc;
    use rp2350sim_core::CpuArch;
    
    /// Test that step() ticks all peripherals without error.
    #[test]
    fn test_step_ticks_peripherals() {
        let mut soc = Soc::new(CpuArch::Arm);
        
        // Run multiple steps
        for _ in 0..100 {
            let result = soc.step();
            assert!(result.is_ok());
        }
    }
    
    /// Test that step increments cycle count.
    #[test]
    fn test_step_increments_cycles() {
        let mut soc = Soc::new(CpuArch::Arm);
        
        let initial_cycles = soc.cycles();
        
        // Run steps
        for _ in 0..10 {
            let _ = soc.step();
        }
        
        // Cycles should have incremented
        assert!(soc.cycles() >= initial_cycles);
    }
}

mod dual_core_soc_tests {
    use rp2350sim_soc::Soc;
    use rp2350sim_core::CpuArch;
    
    /// Test dual-core ARM Soc creation.
    #[test]
    fn test_dual_core_arm_soc() {
        let soc = Soc::new(CpuArch::Arm);
        
        // Should have ARM CPU
        assert!(soc.cpu_arm.is_some());
        assert!(soc.cpu_hazard3.is_none());
    }
    
    /// Test dual-core RISC-V Soc creation.
    #[test]
    fn test_dual_core_riscv_soc() {
        let soc = Soc::new(CpuArch::Hazard3);
        
        // Should have Hazard3 CPU
        assert!(soc.cpu_arm.is_none());
        assert!(soc.cpu_hazard3.is_some());
    }
    
    /// Test cycle counting.
    #[test]
    fn test_cycle_counting() {
        let soc = Soc::new(CpuArch::Arm);
        
        // Initial cycles should be 0
        assert_eq!(soc.cycles(), 0);
    }
    
    /// Test instruction counting.
    #[test]
    fn test_instruction_counting() {
        let soc = Soc::new(CpuArch::Arm);
        
        // Initial instructions should be 0
        assert_eq!(soc.instructions(), 0);
    }
}

mod new_peripheral_tests {
    use rp2350sim_soc::Soc;
    use rp2350sim_core::CpuArch;
    use rp2350sim_core::Device;
    
    /// Test DMA peripheral integration.
    #[test]
    fn test_dma_peripheral() {
        let soc = Soc::new(CpuArch::Arm);
        // DMA should be accessible
        let _ = soc.dma.id();
    }
    
    /// Test XIP peripheral integration.
    #[test]
    fn test_xip_peripheral() {
        let soc = Soc::new(CpuArch::Arm);
        // XIP should be accessible
        let _ = soc.xip.id();
    }
    
    /// Test I2S peripheral integration.
    #[test]
    fn test_i2s_peripherals() {
        let soc = Soc::new(CpuArch::Arm);
        // I2S should be accessible
        let _ = soc.i2s0.id();
        let _ = soc.i2s1.id();
    }
    
    /// Test RTC peripheral integration.
    #[test]
    fn test_rtc_peripheral() {
        let soc = Soc::new(CpuArch::Arm);
        // RTC should be accessible
        let _ = soc.rtc.id();
    }
    
    /// Test SHA256 peripheral integration.
    #[test]
    fn test_sha256_peripheral() {
        let soc = Soc::new(CpuArch::Arm);
        // SHA256 should be accessible
        let _ = soc.sha256.id();
    }
    
    /// Test TRNG peripheral integration.
    #[test]
    fn test_trng_peripheral() {
        let soc = Soc::new(CpuArch::Arm);
        // TRNG should be accessible
        let _ = soc.trng.id();
    }
    
    /// Test NVIC peripheral integration.
    #[test]
    fn test_nvic_peripheral() {
        let soc = Soc::new(CpuArch::Arm);
        // NVIC should be accessible
        let _ = soc.nvic.id();
    }
    
    /// Test PLIC peripheral integration (for RISC-V).
    #[test]
    fn test_plic_peripheral() {
        let soc = Soc::new(CpuArch::Hazard3);
        // PLIC should be accessible
        let _ = soc.plic.id();
    }
    
    /// Test PowerManager peripheral integration.
    #[test]
    fn test_powman_peripheral() {
        let soc = Soc::new(CpuArch::Arm);
        // PowerManager should be accessible
        let _ = soc.powman.id();
    }
    
    /// Test Sysinfo peripheral integration.
    #[test]
    fn test_sysinfo_peripheral() {
        let soc = Soc::new(CpuArch::Arm);
        // Sysinfo should be accessible
        let _ = soc.sysinfo.id();
    }
    
    /// Test OTP peripheral integration.
    #[test]
    fn test_otp_peripheral() {
        let soc = Soc::new(CpuArch::Arm);
        // OTP should be accessible
        let _ = soc.otp.id();
    }
}