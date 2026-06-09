//! Memory diff utilities.

use crate::DiffResult;

/// Memory state for diff comparison.
#[derive(Debug, Clone)]
pub struct MemoryState {
    /// Memory contents.
    pub data: Vec<u8>,
    /// Base address.
    pub base: u32,
}

impl MemoryState {
    /// Create a new memory state.
    pub fn new(size: usize, base: u32) -> Self {
        Self {
            data: vec![0; size],
            base,
        }
    }
}

/// Compare memory states.
pub fn diff_memory(a: &MemoryState, b: &MemoryState) -> Vec<(u32, DiffResult)> {
    let mut results = Vec::new();
    
    let min_len = a.data.len().min(b.data.len());
    for i in 0..min_len {
        if a.data[i] != b.data[i] {
            results.push((a.base + i as u32, DiffResult::Mismatch {
                expected: a.data[i] as u64,
                actual: b.data[i] as u64,
            }));
        }
    }
    
    if a.data.len() != b.data.len() {
        results.push((a.base + min_len as u32, DiffResult::LengthMismatch {
            expected: a.data.len(),
            actual: b.data.len(),
        }));
    }
    
    results
}