//! GPIO binding component.

use serde::{Deserialize, Serialize};

/// GPIO binding component for connecting entities to GPIO pins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GpioBind {
    /// GPIO pin number.
    pub pin: u8,
    /// Whether this is an input (true) or output (false) binding.
    pub is_input: bool,
}

impl GpioBind {
    /// Create a new GPIO binding.
    pub fn new(pin: u8, is_input: bool) -> Self {
        Self { pin, is_input }
    }

    /// Create an input binding.
    pub fn input(pin: u8) -> Self {
        Self::new(pin, true)
    }

    /// Create an output binding.
    pub fn output(pin: u8) -> Self {
        Self::new(pin, false)
    }
}