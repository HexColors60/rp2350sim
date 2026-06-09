//! Dual-core support tests for RP2350 Simulator.

#[cfg(test)]
mod dual_core_tests {
    use rp2350sim_core::CoreId;

    #[test]
    fn test_core_id_creation() {
        let core0 = CoreId::new(0);
        let core1 = CoreId::new(1);
        
        assert_eq!(core0.index(), 0);
        assert_eq!(core1.index(), 1);
    }

    #[test]
    fn test_core_id_constants() {
        assert_eq!(CoreId::CORE0.index(), 0);
        assert_eq!(CoreId::CORE1.index(), 1);
    }

    #[test]
    fn test_core_id_equality() {
        let core0a = CoreId::new(0);
        let core0b = CoreId::new(0);
        let core1 = CoreId::new(1);
        
        assert_eq!(core0a, core0b);
        assert_ne!(core0a, core1);
    }

    #[test]
    fn test_core_id_display() {
        let core0 = CoreId::new(0);
        let core1 = CoreId::new(1);
        
        assert_eq!(format!("{}", core0), "Core0");
        assert_eq!(format!("{}", core1), "Core1");
    }

    #[test]
    fn test_core_id_default() {
        let core = CoreId::default();
        assert_eq!(core.index(), 0);
    }
}

#[cfg(test)]
mod cpu_arch_tests {
    use rp2350sim_core::CpuArch;

    #[test]
    fn test_cpu_arch_arm() {
        let arch = CpuArch::Arm;
        assert_eq!(format!("{:?}", arch), "Arm");
    }

    #[test]
    fn test_cpu_arch_hazard3() {
        let arch = CpuArch::Hazard3;
        assert_eq!(format!("{:?}", arch), "Hazard3");
    }

    #[test]
    fn test_cpu_arch_equality() {
        assert_eq!(CpuArch::Arm, CpuArch::Arm);
        assert_eq!(CpuArch::Hazard3, CpuArch::Hazard3);
        assert_ne!(CpuArch::Arm, CpuArch::Hazard3);
    }

    #[test]
    fn test_cpu_arch_clone() {
        let arch = CpuArch::Arm;
        let cloned = arch.clone();
        assert_eq!(arch, cloned);
    }

    #[test]
    fn test_cpu_arch_copy() {
        let arch = CpuArch::Hazard3;
        let copied = arch;
        assert_eq!(arch, copied);
    }
}
#[cfg(test)]
mod dual_core_arm_tests {
    use rp2350sim_cpu_arm::ArmBackend;
    use rp2350sim_cpu_common::CpuBackend;
    use rp2350sim_core::CoreId;

    #[test]
    fn test_arm_backend_core_count() {
        let backend = ArmBackend::new();
        assert_eq!(backend.core_count(), 2);
    }

    #[test]
    fn test_arm_backend_dual_pc() {
        let mut backend = ArmBackend::new();
        
        // Set different PCs for each core using trait methods
        CpuBackend::set_pc(&mut backend, CoreId::CORE0, 0x1000);
        CpuBackend::set_pc(&mut backend, CoreId::CORE1, 0x2000);
        
        assert_eq!(CpuBackend::pc(&backend, CoreId::CORE0), 0x1000u64);
        assert_eq!(CpuBackend::pc(&backend, CoreId::CORE1), 0x2000u64);
    }

    #[test]
    fn test_arm_backend_dual_registers() {
        let mut backend = ArmBackend::new();
        
        // Set different register values for each core
        CpuBackend::write_reg(&mut backend, CoreId::CORE0, 0, 0x11111111);
        CpuBackend::write_reg(&mut backend, CoreId::CORE1, 0, 0x22222222);
        
        assert_eq!(CpuBackend::read_reg(&backend, CoreId::CORE0, 0), 0x11111111u64);
        assert_eq!(CpuBackend::read_reg(&backend, CoreId::CORE1, 0), 0x22222222u64);
    }

    #[test]
    fn test_arm_backend_halt_resume() {
        let mut backend = ArmBackend::new();
        
        // Both cores start halted, resume them first
        backend.resume(CoreId::CORE0);
        backend.resume(CoreId::CORE1);
        
        // Halt core 1 while core 0 continues
        backend.halt(CoreId::CORE1);
        
        assert!(!backend.is_halted(CoreId::CORE0));
        assert!(backend.is_halted(CoreId::CORE1));
        
        // Resume core 1
        backend.resume(CoreId::CORE1);
        
        assert!(!backend.is_halted(CoreId::CORE1));
    }

    #[test]
    fn test_arm_backend_reset() {
        let mut backend = ArmBackend::new();
        
        // Set some state
        CpuBackend::set_pc(&mut backend, CoreId::CORE0, 0x1000);
        CpuBackend::set_pc(&mut backend, CoreId::CORE1, 0x2000);
        
        // Reset
        backend.reset();
        
        // Core count should remain 2
        assert_eq!(backend.core_count(), 2);
    }
}

#[cfg(test)]
mod dual_core_hazard3_tests {
    use rp2350sim_cpu_hazard3::Hazard3Backend;
    use rp2350sim_cpu_common::CpuBackend;
    use rp2350sim_core::CoreId;

    #[test]
    fn test_hazard3_backend_core_count() {
        let backend = Hazard3Backend::new();
        assert_eq!(backend.core_count(), 2);
    }

    #[test]
    fn test_hazard3_backend_dual_pc() {
        let mut backend = Hazard3Backend::new();
        
        // Set different PCs for each core using trait methods
        CpuBackend::set_pc(&mut backend, CoreId::CORE0, 0x1000);
        CpuBackend::set_pc(&mut backend, CoreId::CORE1, 0x2000);
        
        assert_eq!(CpuBackend::pc(&backend, CoreId::CORE0), 0x1000u64);
        assert_eq!(CpuBackend::pc(&backend, CoreId::CORE1), 0x2000u64);
    }

    #[test]
    fn test_hazard3_backend_dual_registers() {
        let mut backend = Hazard3Backend::new();
        
        // Set different register values for each core
        CpuBackend::write_reg(&mut backend, CoreId::CORE0, 1, 0x11111111);
        CpuBackend::write_reg(&mut backend, CoreId::CORE1, 1, 0x22222222);
        
        assert_eq!(CpuBackend::read_reg(&backend, CoreId::CORE0, 1), 0x11111111u64);
        assert_eq!(CpuBackend::read_reg(&backend, CoreId::CORE1, 1), 0x22222222u64);
    }

    #[test]
    fn test_hazard3_backend_halt_resume() {
        let mut backend = Hazard3Backend::new();
        
        // Both cores start halted, resume them first
        backend.resume(CoreId::CORE0);
        backend.resume(CoreId::CORE1);
        
        // Halt core 1 while core 0 continues
        backend.halt(CoreId::CORE1);
        
        assert!(!backend.is_halted(CoreId::CORE0));
        assert!(backend.is_halted(CoreId::CORE1));
        
        // Resume core 1
        backend.resume(CoreId::CORE1);
        
        assert!(!backend.is_halted(CoreId::CORE1));
    }
}