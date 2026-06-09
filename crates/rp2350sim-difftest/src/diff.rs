//! Differential tester.

use rp2350sim_core::Result;

/// Differential tester.
#[derive(Debug, Default)]
pub struct DiffTester {
    enabled: bool,
}

impl DiffTester {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn compare(&self, a: &[u8], b: &[u8]) -> Result<DiffReport> {
        Ok(DiffReport {
            matches: a == b,
            differences: Vec::new(),
        })
    }
}

/// Differential test report.
#[derive(Debug, Clone)]
pub struct DiffReport {
    pub matches: bool,
    pub differences: Vec<DiffEntry>,
}

/// Difference entry.
#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub kind: String,
    pub expected: u64,
    pub actual: u64,
}