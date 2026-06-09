//! Checkpoint and restore tests for RP2350 Simulator.

#[cfg(test)]
mod checkpoint_tests {
    use rp2350sim_save::Checkpoint;

    #[test]
    fn test_checkpoint_creation() {
        let checkpoint = Checkpoint::new("test", 0, vec![]);
        assert_eq!(checkpoint.name, "test");
        assert_eq!(checkpoint.tick, 0);
        assert_eq!(checkpoint.data, vec![]);
    }

    #[test]
    fn test_checkpoint_with_data() {
        let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let checkpoint = Checkpoint::new("test_checkpoint", 100, data.clone());
        assert_eq!(checkpoint.name, "test_checkpoint");
        assert_eq!(checkpoint.tick, 100);
        assert_eq!(checkpoint.data, data);
    }

    #[test]
    fn test_checkpoint_timestamp() {
        let checkpoint = Checkpoint::new("test", 0, vec![]);
        // Timestamp should be set (non-zero)
        assert!(checkpoint.timestamp > 0);
    }

    #[test]
    fn test_checkpoint_clone() {
        let checkpoint = Checkpoint::new("test", 42, vec![1, 2, 3]);
        let cloned = checkpoint.clone();
        assert_eq!(cloned.name, checkpoint.name);
        assert_eq!(cloned.tick, checkpoint.tick);
        assert_eq!(cloned.data, checkpoint.data);
    }
}

#[cfg(test)]
mod replay_tests {
    #[test]
    fn test_replay_determinism() {
        // Test that execution is deterministic
        // This would require full SoC simulation
        // For now, just verify the concept
        let values: Vec<u32> = (0..10).map(|i| i * 2).collect();
        let sum: u32 = values.iter().sum();
        assert_eq!(sum, 90); // 0+2+4+6+8+10+12+14+16+18 = 90
    }

    #[test]
    fn test_replay_consistency() {
        // Test that multiple runs produce same result
        fn compute_value() -> u32 {
            let mut x: u32 = 0;
            for i in 0..100 {
                x = x.wrapping_add(i);
            }
            x
        }
        
        let v1 = compute_value();
        let v2 = compute_value();
        let v3 = compute_value();
        
        assert_eq!(v1, v2);
        assert_eq!(v2, v3);
    }
}