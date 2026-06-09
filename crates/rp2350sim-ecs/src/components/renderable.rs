//! Renderable component.

use serde::{Deserialize, Serialize};

/// Renderable marker.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Renderable {
    /// Whether this entity is visible.
    pub visible: bool,
    /// Render layer.
    pub layer: u8,
    /// Width for displays.
    pub width: u32,
    /// Height for displays.
    pub height: u32,
    /// Whether this entity is animated.
    pub animated: bool,
}

impl Renderable {
    /// Create a new renderable.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a renderable for an LED.
    pub fn led() -> Self {
        Self { visible: true, layer: 1, width: 10, height: 10, animated: false }
    }

    /// Create a renderable for a button.
    pub fn button() -> Self {
        Self { visible: true, layer: 1, width: 20, height: 20, animated: false }
    }

    /// Create a renderable for a logic probe.
    pub fn probe() -> Self {
        Self { visible: true, layer: 1, width: 15, height: 15, animated: false }
    }

    /// Create a renderable for a display.
    pub fn display() -> Self {
        Self { visible: true, layer: 2, width: 128, height: 64, animated: false }
    }

    /// Create a renderable for a display with specific dimensions.
    pub fn display_with_size(width: u32, height: u32) -> Self {
        Self { visible: true, layer: 2, width, height, animated: false }
    }
}