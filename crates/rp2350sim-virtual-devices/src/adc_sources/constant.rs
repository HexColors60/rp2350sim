//! Constant signal source.

use super::AdcSource;

/// A constant (DC) signal source.
#[derive(Debug, Clone, Copy)]
pub struct ConstantSource {
    value: f32,
}

impl ConstantSource {
    /// Create a new constant source.
    pub fn new(value: f32) -> Self {
        Self { value }
    }
}

impl AdcSource for ConstantSource {
    fn sample(&self, _time: f64) -> f32 {
        self.value
    }

    fn reset(&mut self) {
        // No state to reset
    }
}