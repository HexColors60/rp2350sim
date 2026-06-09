//! Diff report.

/// Differential test report.
#[derive(Debug, Clone, Default)]
pub struct DiffReport {
    pub matches: bool,
    pub entries: Vec<DiffEntry>,
}

/// Difference entry.
#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub kind: String,
    pub address: u64,
    pub expected: u64,
    pub actual: u64,
}