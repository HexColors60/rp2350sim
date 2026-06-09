//! Pin renderer.
#![allow(dead_code)]

use super::board_renderer::PinState;

/// Renders GPIO pins on the board.
#[derive(Debug)]
pub struct PinRenderer {
    /// Pin radius.
    radius: f32,
    /// Colors for different pin states.
    color_low: [f32; 4],
    color_high: [f32; 4],
    color_input: [f32; 4],
    color_output: [f32; 4],
}

impl Default for PinRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl PinRenderer {
    pub fn new() -> Self {
        Self {
            radius: 6.0,
            color_low: [0.3, 0.3, 0.35, 1.0],
            color_high: [0.2, 0.8, 0.3, 1.0],
            color_input: [0.3, 0.5, 0.8, 1.0],
            color_output: [0.8, 0.5, 0.3, 1.0],
        }
    }

    /// Render a pin.
    pub fn render(&self, pin: &PinState) {
        // Determine color based on state
        let color = if pin.value {
            self.color_high
        } else {
            self.color_low
        };

        // Pin rendering would use macroquad or wgpu
        // For now, this is a placeholder
        let _ = (pin.x, pin.y, color, self.radius);
    }

    /// Check if a point is over a pin.
    pub fn hit_test(&self, pin: &PinState, x: f32, y: f32) -> bool {
        let dx = x - pin.x;
        let dy = y - pin.y;
        (dx * dx + dy * dy) <= self.radius * self.radius
    }

    /// Set pin radius.
    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }
}