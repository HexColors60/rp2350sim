//! Peripheral tests for RP2350 Simulator.

#[cfg(test)]
mod peripheral_tests {
    use rp2350sim_devices::adc::Adc;
    use rp2350sim_devices::gpio::{Gpio, GpioFunction, GPIO_BASE};
    use rp2350sim_devices::i2c::I2c;
    use rp2350sim_devices::pio::Pio;
    use rp2350sim_devices::pwm::Pwm;
    use rp2350sim_devices::spi::Spi;
    use rp2350sim_devices::timer::Timer;
    use rp2350sim_devices::uart::{Uart, UART0_BASE};

    // ==================== GPIO Tests ====================

    #[test]
    fn test_gpio_creation() {
        let _gpio = Gpio::new();
        // GPIO should be created successfully
    }

    #[test]
    fn test_gpio_function() {
        // Test GpioFunction enum
        assert_eq!(GpioFunction::from(1), GpioFunction::Spi);
        assert_eq!(GpioFunction::from(2), GpioFunction::Uart);
        assert_eq!(GpioFunction::from(5), GpioFunction::Sio);
    }

    #[test]
    fn test_gpio_base_address() {
        assert_eq!(GPIO_BASE, 0x40014000);
    }

    // ==================== UART Tests ====================

    #[test]
    fn test_uart_creation() {
        let _uart = Uart::new(0);
        // UART should be created successfully
    }

    #[test]
    fn test_uart_base_address() {
        assert_eq!(UART0_BASE, 0x40034000);
    }

    // ==================== SPI Tests ====================

    #[test]
    fn test_spi_creation() {
        let _spi = Spi::new(0);
        // SPI should be created successfully
    }

    // ==================== I2C Tests ====================

    #[test]
    fn test_i2c_creation() {
        let _i2c = I2c::new(0);
        // I2C should be created successfully
    }

    // ==================== PWM Tests ====================

    #[test]
    fn test_pwm_creation() {
        let _pwm = Pwm::new();
        // PWM should be created successfully
    }

    // ==================== ADC Tests ====================

    #[test]
    fn test_adc_creation() {
        let _adc = Adc::new();
        // ADC should be created successfully
    }

    // ==================== Timer Tests ====================

    #[test]
    fn test_timer_creation() {
        let _timer = Timer::new();
        // Timer should be created successfully
    }

    // ==================== PIO Tests ====================

    #[test]
    fn test_pio_creation() {
        let _pio = Pio::new(0);
        // PIO should be created successfully
    }

    // ==================== NVIC Tests ====================

    mod nvic_tests {
        use rp2350sim_cpu_arm::nvic::Nvic;

        #[test]
        fn test_nvic_creation() {
            let nvic = Nvic::new();
            assert!(!nvic.is_enabled(0));
            assert!(!nvic.is_pending(0));
        }

        #[test]
        fn test_nvic_enable_disable() {
            let mut nvic = Nvic::new();

            nvic.enable(5);
            assert!(nvic.is_enabled(5));
            assert!(!nvic.is_enabled(4));

            nvic.disable(5);
            assert!(!nvic.is_enabled(5));
        }

        #[test]
        fn test_nvic_pending() {
            let mut nvic = Nvic::new();

            nvic.set_pending(10);
            assert!(nvic.is_pending(10));

            nvic.clear_pending(10);
            assert!(!nvic.is_pending(10));
        }

        #[test]
        fn test_nvic_set_irq() {
            let mut nvic = Nvic::new();

            nvic.set_irq(15, true);
            assert!(nvic.is_pending(15));

            nvic.set_irq(15, false);
            // Pending is still set even after IRQ line goes low
            assert!(nvic.is_pending(15));
        }

        #[test]
        fn test_nvic_priority() {
            let mut nvic = Nvic::new();

            nvic.set_priority(0, 10);
            nvic.set_priority(1, 5);
            nvic.set_priority(2, 15);

            assert_eq!(nvic.get_priority(0), 10);
            assert_eq!(nvic.get_priority(1), 5);
            assert_eq!(nvic.get_priority(2), 15);
        }

        #[test]
        fn test_nvic_highest_pending() {
            let mut nvic = Nvic::new();

            // No pending interrupts
            assert!(nvic.highest_pending().is_none());

            // Set some pending interrupts with different priorities
            nvic.set_pending(5);
            nvic.set_pending(10);
            nvic.set_pending(15);

            nvic.enable(5);
            nvic.enable(10);
            nvic.enable(15);

            nvic.set_priority(5, 10);
            nvic.set_priority(10, 5); // Highest priority (lowest number)
            nvic.set_priority(15, 20);

            let highest = nvic.highest_pending();
            assert_eq!(highest, Some(10)); // IRQ 10 has highest priority
        }

        #[test]
        fn test_nvic_only_enabled_pending() {
            let mut nvic = Nvic::new();

            // Set pending but don't enable
            nvic.set_pending(5);
            assert!(nvic.highest_pending().is_none());

            // Enable it
            nvic.enable(5);
            assert_eq!(nvic.highest_pending(), Some(5));
        }

        #[test]
        fn test_nvic_reset() {
            let mut nvic = Nvic::new();

            nvic.enable(5);
            nvic.set_pending(10);
            nvic.set_priority(15, 100);

            nvic.reset();

            assert!(!nvic.is_enabled(5));
            assert!(!nvic.is_pending(10));
            assert_eq!(nvic.get_priority(15), 0);
        }

        #[test]
        fn test_nvic_multiple_irqs_same_reg() {
            let mut nvic = Nvic::new();

            // Test multiple IRQs in the same register
            nvic.enable(0);
            nvic.enable(1);
            nvic.enable(31);

            assert!(nvic.is_enabled(0));
            assert!(nvic.is_enabled(1));
            assert!(nvic.is_enabled(31));
            assert!(!nvic.is_enabled(32));
        }

        #[test]
        fn test_nvic_boundary_conditions() {
            let mut nvic = Nvic::new();

            // Test boundary conditions
            nvic.enable(0);
            nvic.enable(239);
            nvic.set_pending(0);
            nvic.set_pending(239);

            assert!(nvic.is_enabled(0));
            assert!(nvic.is_enabled(239));
            assert!(nvic.is_pending(0));
            assert!(nvic.is_pending(239));

            // Out of bounds should be silently ignored
            nvic.enable(240);
            nvic.set_pending(240);
            assert!(!nvic.is_enabled(240));
            assert!(!nvic.is_pending(240));
        }
    }

    // ==================== Exception Tests ====================

    mod exception_tests {
        use rp2350sim_cpu_common::{ExceptionKind, ExceptionState};

        #[test]
        fn test_exception_kind_numbers() {
            assert_eq!(ExceptionKind::Reset.exception_number(), 1);
            assert_eq!(ExceptionKind::Nmi.exception_number(), 2);
            assert_eq!(ExceptionKind::HardFault.exception_number(), 3);
            assert_eq!(ExceptionKind::MemManage.exception_number(), 4);
            assert_eq!(ExceptionKind::BusFault.exception_number(), 5);
            assert_eq!(ExceptionKind::UsageFault.exception_number(), 6);
            assert_eq!(ExceptionKind::SVCall.exception_number(), 11);
            assert_eq!(ExceptionKind::DebugMonitor.exception_number(), 12);
            assert_eq!(ExceptionKind::PendSV.exception_number(), 14);
            assert_eq!(ExceptionKind::SysTick.exception_number(), 15);
            assert_eq!(ExceptionKind::Interrupt(0).exception_number(), 16);
            assert_eq!(ExceptionKind::Interrupt(100).exception_number(), 116);
        }

        #[test]
        fn test_exception_priorities() {
            // Reset has highest priority (most negative)
            assert_eq!(ExceptionKind::Reset.priority(), -3);
            assert_eq!(ExceptionKind::Nmi.priority(), -2);
            assert_eq!(ExceptionKind::HardFault.priority(), -1);

            // Other exceptions have priority 0 (configurable in real hardware)
            assert_eq!(ExceptionKind::MemManage.priority(), 0);
            assert_eq!(ExceptionKind::BusFault.priority(), 0);
            assert_eq!(ExceptionKind::SVCall.priority(), 0);
            assert_eq!(ExceptionKind::SysTick.priority(), 0);
            assert_eq!(ExceptionKind::Interrupt(0).priority(), 0);
        }

        #[test]
        fn test_exception_state_pend() {
            let mut state = ExceptionState::new();

            assert!(!state.is_pending(ExceptionKind::HardFault));

            state.pend(ExceptionKind::HardFault);
            assert!(state.is_pending(ExceptionKind::HardFault));

            // Pend again should not duplicate
            state.pend(ExceptionKind::HardFault);
            assert_eq!(state.highest_pending(), Some(ExceptionKind::HardFault));
        }

        #[test]
        fn test_exception_state_activate() {
            let mut state = ExceptionState::new();

            state.pend(ExceptionKind::SVCall);
            assert!(state.is_pending(ExceptionKind::SVCall));
            assert!(!state.is_active(ExceptionKind::SVCall));

            state.activate(ExceptionKind::SVCall);
            assert!(!state.is_pending(ExceptionKind::SVCall));
            assert!(state.is_active(ExceptionKind::SVCall));
        }

        #[test]
        fn test_exception_state_deactivate() {
            let mut state = ExceptionState::new();

            state.activate(ExceptionKind::PendSV);
            assert!(state.is_active(ExceptionKind::PendSV));

            state.deactivate(ExceptionKind::PendSV);
            assert!(!state.is_active(ExceptionKind::PendSV));
        }

        #[test]
        fn test_exception_state_highest_pending() {
            let mut state = ExceptionState::new();

            // No pending exceptions
            assert!(state.highest_pending().is_none());

            // Add multiple pending exceptions
            state.pend(ExceptionKind::SVCall);
            state.pend(ExceptionKind::SysTick);
            state.pend(ExceptionKind::PendSV);

            // All have same priority (0), so any could be returned
            let highest = state.highest_pending();
            assert!(highest.is_some());

            // Add HardFault which has higher priority
            state.pend(ExceptionKind::HardFault);
            assert_eq!(state.highest_pending(), Some(ExceptionKind::HardFault));
        }

        #[test]
        fn test_exception_state_reset() {
            let mut state = ExceptionState::new();

            state.pend(ExceptionKind::HardFault);
            state.activate(ExceptionKind::SVCall);

            state.reset();

            assert!(!state.is_pending(ExceptionKind::HardFault));
            assert!(!state.is_active(ExceptionKind::SVCall));
        }

        #[test]
        fn test_exception_interrupt_numbers() {
            // Test interrupt number calculation
            for i in 0..=239 {
                let irq = ExceptionKind::Interrupt(i);
                assert_eq!(irq.exception_number(), 16 + i);
            }
        }

        #[test]
        fn test_exception_equality() {
            assert_eq!(ExceptionKind::Reset, ExceptionKind::Reset);
            assert_eq!(ExceptionKind::Interrupt(5), ExceptionKind::Interrupt(5));
            assert_ne!(ExceptionKind::HardFault, ExceptionKind::BusFault);
            assert_ne!(ExceptionKind::Interrupt(5), ExceptionKind::Interrupt(6));
        }
    }

    // ==================== SysTick Tests ====================

    mod systick_tests {
        use rp2350sim_cpu_arm::systick::SysTick;

        #[test]
        fn test_systick_creation() {
            let systick = SysTick::new();
            assert!(!systick.is_enabled());
            assert!(!systick.is_interrupt_enabled());
            assert_eq!(systick.current_value(), 0);
            assert_eq!(systick.reload_value(), 0);
        }

        #[test]
        fn test_systick_enable() {
            let mut systick = SysTick::new();

            systick.write_csr(0x1); // Enable
            assert!(systick.is_enabled());

            systick.write_csr(0x0); // Disable
            assert!(!systick.is_enabled());
        }

        #[test]
        fn test_systick_interrupt_enable() {
            let mut systick = SysTick::new();

            systick.write_csr(0x3); // Enable + Tickint
            assert!(systick.is_enabled());
            assert!(systick.is_interrupt_enabled());
        }

        #[test]
        fn test_systick_reload() {
            let mut systick = SysTick::new();

            systick.write_rvr(1000);
            assert_eq!(systick.reload_value(), 1000);

            // Should be masked to 24 bits
            systick.write_rvr(0x1FFFFFF);
            assert_eq!(systick.reload_value(), 0xFFFFFF);
        }

        #[test]
        fn test_systick_wrap() {
            let mut systick = SysTick::new();

            systick.write_rvr(5);
            systick.write_csr(0x1); // Enable

            // First tick: CVR was 0, loads RVR=5
            systick.tick();
            assert_eq!(systick.current_value(), 5);

            // Count down: 5 -> 4 -> 3 -> 2 -> 1 -> 0
            systick.tick(); // 4
            systick.tick(); // 3
            systick.tick(); // 2
            systick.tick(); // 1
            systick.tick(); // 0
            assert_eq!(systick.current_value(), 0);

            // Next tick: CVR is 0, reloads RVR=5
            systick.tick();
            assert_eq!(systick.current_value(), 5); // Reloaded
        }

        #[test]
        fn test_systick_count_flag() {
            let mut systick = SysTick::new();

            systick.write_rvr(3);
            systick.write_csr(0x1); // Enable

            // First tick loads RVR into CVR and sets COUNTFLAG
            systick.tick(); // CVR = 3, COUNTFLAG set

            assert!(systick.count_flag());

            // Reading CSR clears COUNTFLAG
            let csr = systick.read_csr();
            assert!(csr & (1 << 16) != 0);
            assert!(!systick.count_flag());
        }

        #[test]
        fn test_systick_interrupt_on_wrap() {
            let mut systick = SysTick::new();

            systick.write_rvr(3);
            systick.write_csr(0x3); // Enable + Tickint

            // First tick: CVR was 0, loads RVR, returns true
            let should_interrupt = systick.tick();

            assert!(should_interrupt);
        }

        #[test]
        fn test_systick_no_interrupt_when_disabled() {
            let mut systick = SysTick::new();

            systick.write_rvr(3);
            systick.write_csr(0x1); // Enable, no Tickint

            systick.tick();
            systick.tick();
            systick.tick();
            let should_interrupt = systick.tick();

            assert!(!should_interrupt);
        }

        #[test]
        fn test_systick_cvr_write_clears() {
            let mut systick = SysTick::new();

            systick.write_rvr(100);
            systick.write_csr(0x1); // Enable
            systick.tick(); // Load 100

            assert_eq!(systick.current_value(), 100);

            // Any write to CVR clears it to 0
            systick.write_cvr(50);
            assert_eq!(systick.current_value(), 0);
        }

        #[test]
        fn test_systick_reset() {
            let mut systick = SysTick::new();

            systick.write_rvr(1000);
            systick.write_csr(0x3);
            systick.tick();

            systick.reset();

            assert!(!systick.is_enabled());
            assert!(!systick.is_interrupt_enabled());
            assert_eq!(systick.reload_value(), 0);
            assert_eq!(systick.current_value(), 0);
        }

        #[test]
        fn test_systick_disabled_no_tick() {
            let mut systick = SysTick::new();

            systick.write_rvr(100);
            // Don't enable

            systick.tick();
            assert_eq!(systick.current_value(), 0); // Should not change
        }
    }

    // ==================== PSR Tests ====================

    mod psr_tests {
        use rp2350sim_cpu_arm::psr::Psr;

        #[test]
        fn test_psr_creation() {
            let psr = Psr::default();
            assert_eq!(psr.bits(), 0);
            assert!(!psr.n());
            assert!(!psr.z());
            assert!(!psr.c());
            assert!(!psr.v());
        }

        #[test]
        fn test_psr_flags() {
            let mut psr = Psr::default();

            // Test N flag
            psr.set_n(true);
            assert!(psr.n());
            assert_eq!(psr.bits() >> 31, 1);

            psr.set_n(false);
            assert!(!psr.n());

            // Test Z flag
            psr.set_z(true);
            assert!(psr.z());
            assert_eq!((psr.bits() >> 30) & 1, 1);

            // Test C flag
            psr.set_c(true);
            assert!(psr.c());
            assert_eq!((psr.bits() >> 29) & 1, 1);

            // Test V flag
            psr.set_v(true);
            assert!(psr.v());
            assert_eq!((psr.bits() >> 28) & 1, 1);
        }

        #[test]
        fn test_psr_multiple_flags() {
            let mut psr = Psr::default();

            psr.set_n(true);
            psr.set_z(true);
            psr.set_c(true);
            psr.set_v(true);

            assert!(psr.n());
            assert!(psr.z());
            assert!(psr.c());
            assert!(psr.v());

            // All condition code flags should be set
            assert_eq!(psr.bits() >> 28, 0b1111);
        }

        #[test]
        fn test_psr_thumb_bit() {
            let mut psr = Psr::default();

            assert!(!psr.t());

            psr.set_t(true);
            assert!(psr.t());

            psr.set_t(false);
            assert!(!psr.t());
        }

        #[test]
        fn test_psr_exception() {
            let mut psr = Psr::default();

            psr.set_exception(3); // HardFault
            assert_eq!(psr.exception(), 3);

            psr.set_exception(15); // SysTick
            assert_eq!(psr.exception(), 15);
        }

        #[test]
        fn test_psr_set_get() {
            let mut psr = Psr::default();

            psr.set(0x61000000); // N=0, Z=1, C=1, V=0 with some other bits
            assert!(!psr.n());
            assert!(psr.z());
            assert!(psr.c());
            assert!(!psr.v());
            assert_eq!(psr.get(), 0x61000000);
        }

        #[test]
        fn test_psr_new() {
            let psr = Psr::new(0xF0000000);
            assert!(psr.n());
            assert!(psr.z());
            assert!(psr.c());
            assert!(psr.v());
        }
    }

    // ==================== RISC-V CSR Tests ====================

    mod csr_tests {
        use rp2350sim_cpu_hazard3::csr::{
            CsrState, MCAUSE, MEPC, MIE, MIP, MISA, MSCRATCH, MSTATUS, MTVAL, MTVEC,
        };

        #[test]
        fn test_csr_creation() {
            let csr = CsrState::new();
            assert_eq!(csr.mstatus, 0);
            assert_eq!(csr.misa, 0x40101104); // RV32IMC
            assert_eq!(csr.mie, 0);
        }

        #[test]
        fn test_csr_read_write() {
            let mut csr = CsrState::new();

            // Test MSTATUS
            csr.write(MSTATUS, 0x12345678);
            assert_eq!(csr.read(MSTATUS), 0x12345678);

            // Test MIE
            csr.write(MIE, 0xFFFF);
            assert_eq!(csr.read(MIE), 0xFFFF);

            // Test MSCRATCH
            csr.write(MSCRATCH, 0xDEADBEEF);
            assert_eq!(csr.read(MSCRATCH), 0xDEADBEEF);
        }

        #[test]
        fn test_csr_mtvec_alignment() {
            let mut csr = CsrState::new();

            // MTVEC has MODE field in low 2 bits
            // The implementation masks off bit 1: value & !0x2
            csr.write(MTVEC, 0x12345677);
            assert_eq!(csr.read(MTVEC), 0x12345675); // Bit 1 cleared
        }

        #[test]
        fn test_csr_mepc_alignment() {
            let mut csr = CsrState::new();

            // MEPC should have low bit cleared (for RISC-V, instructions are at least 16-bit aligned)
            csr.write(MEPC, 0x12345679);
            assert_eq!(csr.read(MEPC), 0x12345678); // Low bit cleared
        }

        #[test]
        fn test_csr_mcause() {
            let mut csr = CsrState::new();

            // Test exception cause
            csr.write(MCAUSE, 0x8000000B); // Machine-level interrupt, cause 11 (external)
            assert_eq!(csr.read(MCAUSE), 0x8000000B);
        }

        #[test]
        fn test_csr_mip_mie_interrupts() {
            let mut csr = CsrState::new();

            // Set up interrupt enable and pending
            csr.write(MIE, 0x00000888); // Enable all external interrupts
            csr.write(MIP, 0x00000888); // Set pending

            assert_eq!(csr.read(MIE), 0x00000888);
            assert_eq!(csr.read(MIP), 0x00000888);
        }

        #[test]
        fn test_csr_reset() {
            let mut csr = CsrState::new();

            csr.write(MSTATUS, 0xFFFFFFFF);
            csr.write(MIE, 0xFFFF);
            csr.write(MTVEC, 0x12345678);

            csr.reset();

            assert_eq!(csr.mstatus, 0);
            assert_eq!(csr.misa, 0x40101104);
            assert_eq!(csr.mie, 0);
            assert_eq!(csr.mtvec, 0);
        }

        #[test]
        fn test_csr_unknown_address() {
            let mut csr = CsrState::new();

            // Unknown CSR addresses should return 0 on read
            assert_eq!(csr.read(0x999), 0);

            // Unknown CSR addresses should be silently ignored on write
            csr.write(0x999, 0xDEADBEEF);
            assert_eq!(csr.read(0x999), 0);
        }

        #[test]
        fn test_csr_misa_value() {
            let csr = CsrState::new();

            // MISA should indicate RV32IMC
            // Bit 2 = C extension (compressed)
            // Bit 8 = I extension (base integer)
            // Bit 12 = M extension (multiply/divide)
            // XLEN field indicates 32-bit
            let misa = csr.read(MISA);
            assert_ne!(misa, 0); // Should have some extensions set
        }

        #[test]
        fn test_csr_mtval() {
            let mut csr = CsrState::new();

            // MTVAL can hold trap value (bad address, faulting instruction, etc.)
            csr.write(MTVAL, 0xBADCAFE);
            assert_eq!(csr.read(MTVAL), 0xBADCAFE);
        }
    }

    // ==================== Vector Table Tests ====================

    mod vector_table_tests {
        use rp2350sim_cpu_arm::vector_table::VectorTable;

        #[test]
        fn test_vector_table_creation() {
            let vt = VectorTable::new();
            assert_eq!(vt.base(), 0);
        }

        #[test]
        fn test_vector_table_set_base() {
            let mut vt = VectorTable::new();

            vt.set_base(0x20000000);
            assert_eq!(vt.base(), 0x20000000);
        }

        #[test]
        fn test_vector_table_base_alignment() {
            let mut vt = VectorTable::new();

            // Vector table base must be aligned to 128 bytes (7 bits)
            vt.set_base(0x2000007F);
            assert_eq!(vt.base(), 0x20000000); // Low 7 bits masked off

            vt.set_base(0x20000100);
            assert_eq!(vt.base(), 0x20000100);
        }

        #[test]
        fn test_vector_table_vector_address() {
            let mut vt = VectorTable::new();
            vt.set_base(0x20000000);

            // Vector 0 = Initial SP
            assert_eq!(vt.initial_sp_address(), 0x20000000);

            // Vector 1 = Reset
            assert_eq!(vt.reset_vector_address(), 0x20000004);

            // Generic vector address
            assert_eq!(vt.vector_address(2), 0x20000008); // NMI
            assert_eq!(vt.vector_address(3), 0x2000000C); // HardFault
        }

        #[test]
        fn test_vector_table_default() {
            let vt = VectorTable::default();
            assert_eq!(vt.base(), 0);
        }
    }

    // ==================== Timer Tests ====================

    mod timer_tests {
        use rp2350sim_core::Device;
        use rp2350sim_devices::timer::Timer;

        #[test]
        fn test_timer_creation() {
            let timer = Timer::new();
            assert_eq!(timer.get_time(), 0);
        }

        #[test]
        fn test_timer_tick() {
            let mut timer = Timer::new();
            timer.tick();
            assert_eq!(timer.get_time(), 1);

            timer.tick();
            timer.tick();
            assert_eq!(timer.get_time(), 3);
        }

        #[test]
        fn test_timer_advance() {
            let mut timer = Timer::new();
            timer.advance(100);
            assert_eq!(timer.get_time(), 100);

            timer.advance(50);
            assert_eq!(timer.get_time(), 150);
        }

        #[test]
        fn test_timer_alarm() {
            let mut timer = Timer::new();
            timer.set_alarm(0, 100);
            assert!(timer.is_armed(0));

            timer.advance(99);
            assert!(timer.is_armed(0)); // Not yet triggered

            timer.advance(1);
            assert!(!timer.is_armed(0)); // Triggered
            assert!(timer.has_interrupt());
        }

        #[test]
        fn test_timer_multiple_alarms() {
            let mut timer = Timer::new();
            timer.set_alarm(0, 100);
            timer.set_alarm(1, 200);
            timer.set_alarm(2, 300);
            timer.set_alarm(3, 400);

            assert!(timer.is_armed(0));
            assert!(timer.is_armed(1));
            assert!(timer.is_armed(2));
            assert!(timer.is_armed(3));

            timer.advance(100);
            assert!(!timer.is_armed(0));
            assert!(timer.is_armed(1));

            timer.advance(100);
            assert!(!timer.is_armed(1));
            assert!(timer.is_armed(2));
        }

        #[test]
        fn test_timer_reset() {
            let mut timer = Timer::new();
            timer.advance(1000);
            timer.set_alarm(0, 500);

            timer.reset();

            assert_eq!(timer.get_time(), 0);
            assert!(!timer.is_armed(0));
        }
    }

    // ==================== RISC-V Instruction Tests ====================

    mod rv32_instruction_tests {
        use rp2350sim_core::{CoreId, Result};
        use rp2350sim_cpu_hazard3::rv32::{
            decode,
            execute::{execute, MemoryAccess},
        };
        use rp2350sim_cpu_hazard3::state::Hazard3CoreState;

        struct MockMemory;

        impl MemoryAccess for MockMemory {
            fn read_byte(&self, _addr: u32) -> Result<u8> {
                Ok(0)
            }
            fn read_half(&self, _addr: u32) -> Result<u16> {
                Ok(0)
            }
            fn read_word(&self, _addr: u32) -> Result<u32> {
                Ok(0)
            }
            fn write_byte(&mut self, _addr: u32, _value: u8) -> Result<()> {
                Ok(())
            }
            fn write_half(&mut self, _addr: u32, _value: u16) -> Result<()> {
                Ok(())
            }
            fn write_word(&mut self, _addr: u32, _value: u32) -> Result<()> {
                Ok(())
            }
        }

        fn create_core() -> Hazard3CoreState {
            Hazard3CoreState::new(CoreId::new(0))
        }

        fn encode_r_type(
            opcode: u32,
            rd: u32,
            funct3: u32,
            rs1: u32,
            rs2: u32,
            funct7: u32,
        ) -> u32 {
            (funct7 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode
        }

        #[test]
        fn test_rv32_add() {
            let mut core = create_core();
            core.x[1] = 100;
            core.x[2] = 50;

            // ADD x3, x1, x2
            let opcode = encode_r_type(0x33, 3, 0, 1, 2, 0);
            let instr = decode(opcode);
            let mut mem = MockMemory;
            execute(&mut core, &instr, &mut mem).unwrap();

            assert_eq!(core.x[3], 150);
        }

        #[test]
        fn test_rv32_sub() {
            let mut core = create_core();
            core.x[1] = 100;
            core.x[2] = 30;

            // SUB x3, x1, x2 (funct7 = 0x20)
            let opcode = encode_r_type(0x33, 3, 0, 1, 2, 0x20);
            let instr = decode(opcode);
            let mut mem = MockMemory;
            execute(&mut core, &instr, &mut mem).unwrap();

            assert_eq!(core.x[3], 70);
        }

        #[test]
        fn test_rv32_and() {
            let mut core = create_core();
            core.x[1] = 0xFF;
            core.x[2] = 0x0F;

            // AND x3, x1, x2 (funct3 = 7)
            let opcode = encode_r_type(0x33, 3, 7, 1, 2, 0);
            let instr = decode(opcode);
            let mut mem = MockMemory;
            execute(&mut core, &instr, &mut mem).unwrap();

            assert_eq!(core.x[3], 0x0F);
        }

        #[test]
        fn test_rv32_or() {
            let mut core = create_core();
            core.x[1] = 0xF0;
            core.x[2] = 0x0F;

            // OR x3, x1, x2 (funct3 = 6)
            let opcode = encode_r_type(0x33, 3, 6, 1, 2, 0);
            let instr = decode(opcode);
            let mut mem = MockMemory;
            execute(&mut core, &instr, &mut mem).unwrap();

            assert_eq!(core.x[3], 0xFF);
        }

        #[test]
        fn test_rv32_xor() {
            let mut core = create_core();
            core.x[1] = 0xFF;
            core.x[2] = 0x0F;

            // XOR x3, x1, x2 (funct3 = 4)
            let opcode = encode_r_type(0x33, 3, 4, 1, 2, 0);
            let instr = decode(opcode);
            let mut mem = MockMemory;
            execute(&mut core, &instr, &mut mem).unwrap();

            assert_eq!(core.x[3], 0xF0);
        }

        #[test]
        fn test_rv32_sll() {
            let mut core = create_core();
            core.x[1] = 1;
            core.x[2] = 4;

            // SLL x3, x1, x2 (funct3 = 1)
            let opcode = encode_r_type(0x33, 3, 1, 1, 2, 0);
            let instr = decode(opcode);
            let mut mem = MockMemory;
            execute(&mut core, &instr, &mut mem).unwrap();

            assert_eq!(core.x[3], 16);
        }

        #[test]
        fn test_rv32_srl() {
            let mut core = create_core();
            core.x[1] = 16;
            core.x[2] = 2;

            // SRL x3, x1, x2 (funct3 = 5)
            let opcode = encode_r_type(0x33, 3, 5, 1, 2, 0);
            let instr = decode(opcode);
            let mut mem = MockMemory;
            execute(&mut core, &instr, &mut mem).unwrap();

            assert_eq!(core.x[3], 4);
        }

        #[test]
        fn test_rv32_sra() {
            let mut core = create_core();
            core.x[1] = 0xFFFFFFFC; // -4 in 2's complement
            core.x[2] = 1;

            // SRA x3, x1, x2 (funct3 = 5, funct7 = 0x20)
            let opcode = encode_r_type(0x33, 3, 5, 1, 2, 0x20);
            let instr = decode(opcode);
            let mut mem = MockMemory;
            execute(&mut core, &instr, &mut mem).unwrap();

            assert_eq!(core.x[3] as i32, -2);
        }

        #[test]
        fn test_rv32_slt() {
            let mut core = create_core();
            core.x[1] = 5;
            core.x[2] = 10;

            // SLT x3, x1, x2 (funct3 = 2)
            let opcode = encode_r_type(0x33, 3, 2, 1, 2, 0);
            let instr = decode(opcode);
            let mut mem = MockMemory;
            execute(&mut core, &instr, &mut mem).unwrap();

            assert_eq!(core.x[3], 1); // 5 < 10

            core.x[1] = 10;
            core.x[2] = 5;
            execute(&mut core, &instr, &mut mem).unwrap();
            assert_eq!(core.x[3], 0); // 10 < 5 is false
        }

        #[test]
        fn test_rv32_sltu() {
            let mut core = create_core();
            core.x[1] = 5;
            core.x[2] = 10;

            // SLTU x3, x1, x2 (funct3 = 3)
            let opcode = encode_r_type(0x33, 3, 3, 1, 2, 0);
            let instr = decode(opcode);
            let mut mem = MockMemory;
            execute(&mut core, &instr, &mut mem).unwrap();

            assert_eq!(core.x[3], 1); // 5 < 10 (unsigned)
        }

        #[test]
        fn test_rv32_mul() {
            let mut core = create_core();
            core.x[1] = 7;
            core.x[2] = 6;

            // MUL x3, x1, x2 (funct3 = 0, funct7 = 1)
            let opcode = encode_r_type(0x33, 3, 0, 1, 2, 1);
            let instr = decode(opcode);
            let mut mem = MockMemory;
            execute(&mut core, &instr, &mut mem).unwrap();

            assert_eq!(core.x[3], 42);
        }

        #[test]
        fn test_rv32_divu() {
            let mut core = create_core();
            core.x[1] = 100;
            core.x[2] = 7;

            // DIVU x3, x1, x2 (funct3 = 5, funct7 = 1)
            let opcode = encode_r_type(0x33, 3, 5, 1, 2, 1);
            let instr = decode(opcode);
            let mut mem = MockMemory;
            execute(&mut core, &instr, &mut mem).unwrap();

            assert_eq!(core.x[3], 14); // 100 / 7 = 14
        }

        #[test]
        fn test_rv32_divu_by_zero() {
            let mut core = create_core();
            core.x[1] = 100;
            core.x[2] = 0;

            // DIVU x3, x1, x2 (funct3 = 5, funct7 = 1)
            let opcode = encode_r_type(0x33, 3, 5, 1, 2, 1);
            let instr = decode(opcode);
            let mut mem = MockMemory;
            execute(&mut core, &instr, &mut mem).unwrap();

            assert_eq!(core.x[3], u32::MAX); // Division by zero returns all 1s
        }
    }
    // ==================== DMA Tests ====================

    mod dma_tests {
        use rp2350sim_devices::dma::{Dma, DmaChannel, DataSize, DMA_BASE};
        use rp2350sim_core::Device;

        #[test]
        fn test_dma_creation() {
            let _dma = Dma::new();
        }

        #[test]
        fn test_dma_base_address() {
            assert_eq!(DMA_BASE, 0x50000000);
        }

        #[test]
        fn test_dma_channel_default() {
            let channel = DmaChannel::default();
            assert_eq!(channel.read_addr, 0);
            assert_eq!(channel.write_addr, 0);
            assert_eq!(channel.trans_count, 0);
            assert!(!channel.enabled);
            assert!(!channel.busy);
        }

        #[test]
        fn test_dma_data_size() {
            assert_eq!(DataSize::Byte as u8, 0);
            assert_eq!(DataSize::HalfWord as u8, 1);
            assert_eq!(DataSize::Word as u8, 2);
        }

        #[test]
        fn test_dma_reset() {
            let mut dma = Dma::new();
            dma.reset();
        }
    }

    // ==================== XIP Tests ====================

    mod xip_tests {
        use rp2350sim_devices::xip::{Xip, XIP_BASE};
        use rp2350sim_core::Device;

        #[test]
        fn test_xip_creation() {
            let xip = Xip::new();
            assert_eq!(xip.flash_size(), 16 * 1024 * 1024);
        }

        #[test]
        fn test_xip_base_address() {
            assert_eq!(XIP_BASE, 0x40000000);
        }

        #[test]
        fn test_xip_load_flash() {
            let mut xip = Xip::new();
            let data = vec![0x01, 0x02, 0x03, 0x04];
            xip.load_flash(0x100, &data);
        }

        #[test]
        fn test_xip_reset() {
            let mut xip = Xip::new();
            xip.reset();
        }

        #[test]
        fn test_xip_custom_flash_size() {
            let xip = Xip::with_flash_size(8 * 1024 * 1024);
            assert_eq!(xip.flash_size(), 8 * 1024 * 1024);
        }
    }

    // ==================== I2S Tests ====================

    mod i2s_tests {
        use rp2350sim_devices::i2s::I2s;
        use rp2350sim_core::Device;

        #[test]
        fn test_i2s_creation() {
            let _i2s = I2s::new(0);
        }

        #[test]
        fn test_i2s_reset() {
            let mut i2s = I2s::new(0);
            i2s.reset();
        }
    }

    // ==================== RTC Tests ====================

    mod rtc_tests {
        use rp2350sim_devices::rtc::Rtc;
        use rp2350sim_core::Device;

        #[test]
        fn test_rtc_creation() {
            let _rtc = Rtc::new();
        }

        #[test]
        fn test_rtc_reset() {
            let mut rtc = Rtc::new();
            rtc.reset();
        }
    }

    // ==================== SHA256 Tests ====================

    mod sha256_tests {
        use rp2350sim_devices::sha256::Sha256;
        use rp2350sim_core::Device;

        #[test]
        fn test_sha256_creation() {
            let _sha = Sha256::new();
        }

        #[test]
        fn test_sha256_reset() {
            let mut sha = Sha256::new();
            sha.reset();
        }
    }

    // ==================== TRNG Tests ====================

    mod trng_tests {
        use rp2350sim_devices::trng::Trng;
        use rp2350sim_core::Device;

        #[test]
        fn test_trng_creation() {
            let _trng = Trng::new();
        }

        #[test]
        fn test_trng_reset() {
            let mut trng = Trng::new();
            trng.reset();
        }
    }
}