//! Save/restore test
//! Tests checkpoint and state serialization functionality

#[cfg(test)]
mod tests {
    use rp2350sim_save::{SaveState, Checkpoint};
    use rp2350sim_core::ids::CoreId;
    use rp2350sim_cpu_common::cpu_backend::CpuState;

    #[test]
    fn test_checkpoint_creation() {
        let checkpoint = Checkpoint::new("test_checkpoint");
        assert_eq!(checkpoint.name(), "test_checkpoint");
    }

    #[test]
    fn test_save_state_creation() {
        let save_state = SaveState::new();
        assert!(save_state.is_empty());
    }

    #[test]
    fn test_save_state_cpu() {
        let mut save_state = SaveState::new();
        let mut cpu_state = CpuState::new(CoreId::new(0));
        
        // Set some state
        cpu_state.set_pc(0x10000100);
        cpu_state.write_reg(0, 0x12345678);
        cpu_state.write_reg(1, 0xDEADBEEF);
        
        // Save CPU state
        save_state.save_cpu(&cpu_state);
        
        assert!(!save_state.is_empty());
    }

    #[test]
    fn test_save_state_memory() {
        let mut save_state = SaveState::new();
        
        // Create some memory content
        let memory: Vec<u8> = (0..256).map(|i| i as u8).collect();
        
        // Save memory region
        save_state.save_memory(0x20000000, &memory);
        
        assert!(!save_state.is_empty());
    }

    #[test]
    fn test_save_restore_roundtrip() {
        let mut save_state = SaveState::new();
        
        // Create and save CPU state
        let mut cpu_state = CpuState::new(CoreId::new(0));
        cpu_state.set_pc(0x10000100);
        cpu_state.write_reg(0, 0x12345678);
        cpu_state.write_reg(1, 0xDEADBEEF);
        cpu_state.write_reg(2, 0xCAFEBABE);
        
        save_state.save_cpu(&cpu_state);
        
        // Create new CPU state and restore
        let mut restored_state = CpuState::new(CoreId::new(0));
        save_state.restore_cpu(&mut restored_state);
        
        // Verify state matches
        assert_eq!(restored_state.pc(), 0x10000100);
        assert_eq!(restored_state.read_reg(0), 0x12345678);
        assert_eq!(restored_state.read_reg(1), 0xDEADBEEF);
        assert_eq!(restored_state.read_reg(2), 0xCAFEBABE);
    }

    #[test]
    fn test_checkpoint_timestamp() {
        let checkpoint = Checkpoint::new("test");
        let ts = checkpoint.timestamp();
        
        // Timestamp should be recent
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        assert!(ts <= now);
        assert!(ts > now - 10); // Within last 10 seconds
    }

    #[test]
    fn test_save_state_serialization() {
        let mut save_state = SaveState::new();
        
        // Add some state
        let mut cpu_state = CpuState::new(CoreId::new(0));
        cpu_state.set_pc(0x10000100);
        save_state.save_cpu(&cpu_state);
        
        // Serialize
        let serialized = save_state.serialize();
        assert!(!serialized.is_empty());
        
        // Deserialize
        let restored = SaveState::deserialize(&serialized);
        assert!(restored.is_ok());
    }
}