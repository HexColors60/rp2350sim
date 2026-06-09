//! Boot smoke test
//! Tests that the simulator can boot and run basic firmware

use std::time::Duration;

#[cfg(test)]
mod tests {
    use crate::common::harness::TestHarness;

    #[test]
    fn test_harness_creation() {
        let harness = TestHarness::new();
        assert_eq!(harness.max_cycles(), 10_000_000);
    }

    #[test]
    fn test_harness_timeout() {
        let mut harness = TestHarness::new()
            .with_timeout(Duration::from_millis(100));
        harness.start();
        std::thread::sleep(Duration::from_millis(150));
        assert!(harness.is_timed_out());
    }

    #[test]
    fn test_harness_no_timeout() {
        let mut harness = TestHarness::new()
            .with_timeout(Duration::from_secs(10));
        harness.start();
        std::thread::sleep(Duration::from_millis(50));
        assert!(!harness.is_timed_out());
    }
}