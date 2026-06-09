//! Test harness utilities

use std::path::Path;
use std::time::{Duration, Instant};

/// Test harness for running simulation tests
pub struct TestHarness {
    /// Maximum cycles to run
    max_cycles: u64,
    /// Timeout duration
    timeout: Duration,
    /// Start time
    start: Option<Instant>,
}

impl TestHarness {
    /// Create a new test harness
    pub fn new() -> Self {
        Self {
            max_cycles: 10_000_000,
            timeout: Duration::from_secs(5),
            start: None,
        }
    }

    /// Set maximum cycles
    pub fn with_max_cycles(mut self, cycles: u64) -> Self {
        self.max_cycles = cycles;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Start the harness timer
    pub fn start(&mut self) {
        self.start = Some(Instant::now());
    }

    /// Check if timed out
    pub fn is_timed_out(&self) -> bool {
        if let Some(start) = self.start {
            start.elapsed() > self.timeout
        } else {
            false
        }
    }

    /// Get maximum cycles
    pub fn max_cycles(&self) -> u64 {
        self.max_cycles
    }
}

impl Default for TestHarness {
    fn default() -> Self {
        Self::new()
    }
}

/// Load a firmware file for testing
pub fn load_firmware(path: &Path) -> Vec<u8> {
    std::fs::read(path).expect("Failed to load firmware file")
}

/// Find the project root directory
pub fn find_project_root() -> std::path::PathBuf {
    let mut path = std::env::current_dir().expect("Failed to get current directory");
    while !path.join("Cargo.toml").exists() {
        path = path.parent().expect("Failed to find project root").to_path_buf();
    }
    path
}