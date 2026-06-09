//! Dual-core test
//! Tests dual-core CPU functionality

#[cfg(test)]
mod tests {
    use rp2350sim_core::ids::CoreId;
    use rp2350sim_cpu_common::cpu_backend::CpuState;

    #[test]
    fn test_core_id() {
        let core0 = CoreId::new(0);
        let core1 = CoreId::new(1);
        
        assert_eq!(core0.as_usize(), 0);
        assert_eq!(core1.as_usize(), 1);
        assert_ne!(core0, core1);
    }

    #[test]
    fn test_core_state_creation() {
        let state = CpuState::new(CoreId::new(0));
        assert_eq!(state.core_id().as_usize(), 0);
    }

    #[test]
    fn test_dual_core_independent_pc() {
        // Create states for both cores
        let mut state0 = CpuState::new(CoreId::new(0));
        let mut state1 = CpuState::new(CoreId::new(1));
        
        // Set different PCs
        state0.set_pc(0x10000000);
        state1.set_pc(0x20000000);
        
        // Verify they're independent
        assert_eq!(state0.pc(), 0x10000000);
        assert_eq!(state1.pc(), 0x20000000);
    }

    #[test]
    fn test_dual_core_independent_registers() {
        let mut state0 = CpuState::new(CoreId::new(0));
        let mut state1 = CpuState::new(CoreId::new(1));
        
        // Set different register values
        state0.write_reg(0, 0x11111111);
        state1.write_reg(0, 0x22222222);
        
        // Verify they're independent
        assert_eq!(state0.read_reg(0), 0x11111111);
        assert_eq!(state1.read_reg(0), 0x22222222);
    }

    #[test]
    fn test_core_run_state() {
        use rp2350sim_core::types::RunState;
        
        let mut state = CpuState::new(CoreId::new(0));
        
        // Initially halted
        assert_eq!(state.run_state(), RunState::Halted);
        
        // Start running
        state.set_run_state(RunState::Running);
        assert_eq!(state.run_state(), RunState::Running);
        
        // Pause
        state.set_run_state(RunState::Paused);
        assert_eq!(state.run_state(), RunState::Paused);
    }

    #[test]
    fn test_core_breakpoint() {
        let mut state = CpuState::new(CoreId::new(0));
        
        // Add breakpoint
        state.add_breakpoint(0x10000100);
        
        // Check breakpoint exists
        assert!(state.has_breakpoint(0x10000100));
        assert!(!state.has_breakpoint(0x10000200));
        
        // Remove breakpoint
        state.remove_breakpoint(0x10000100);
        assert!(!state.has_breakpoint(0x10000100));
    }
}