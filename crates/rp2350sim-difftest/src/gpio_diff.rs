//! GPIO diff utilities.

use crate::DiffResult;

/// GPIO state for diff comparison.
#[derive(Debug, Clone)]
pub struct GpioState {
    /// Pin values.
    pub pins: Vec<bool>,
    /// Pin directions.
    pub directions: Vec<bool>,
}

impl GpioState {
    /// Create a new GPIO state.
    pub fn new(pin_count: usize) -> Self {
        Self {
            pins: vec![false; pin_count],
            directions: vec![false; pin_count],
        }
    }
}

/// Compare GPIO states.
pub fn diff_gpio(a: &GpioState, b: &GpioState) -> Vec<(usize, DiffResult)> {
    let mut results = Vec::new();
    
    for i in 0..a.pins.len().min(b.pins.len()) {
        if a.pins[i] != b.pins[i] {
            results.push((i, DiffResult::Mismatch {
                expected: a.pins[i] as u64,
                actual: b.pins[i] as u64,
            }));
        }
    }
    
    results
}