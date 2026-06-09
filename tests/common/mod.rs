//! Common test utilities and fixtures

pub mod harness;
pub mod fixtures;

/// Test timeout in milliseconds
pub const TEST_TIMEOUT_MS: u64 = 5000;

/// Maximum cycles to run in a test
pub const MAX_TEST_CYCLES: u64 = 10_000_000;