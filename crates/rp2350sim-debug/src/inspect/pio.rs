//! PIO inspector.
#![allow(dead_code)]

/// PIO inspector.
pub struct PioInspector {
    /// PIO instance (0 or 1).
    pub instance: u8,
}

impl PioInspector {
    /// Create a new PIO inspector.
    pub fn new(instance: u8) -> Self {
        Self { instance }
    }
}