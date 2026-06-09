#![allow(dead_code)]

//! Zoom controller for waveforms.

/// Zoom controller.
#[derive(Debug, Clone, Copy)]
pub struct ZoomController {
    /// Zoom level (samples per pixel).
    zoom: f32,
    /// Minimum zoom.
    min_zoom: f32,
    /// Maximum zoom.
    max_zoom: f32,
}

impl Default for ZoomController {
    fn default() -> Self {
        Self::new()
    }
}

impl ZoomController {
    /// Create a new zoom controller.
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            min_zoom: 0.01,
            max_zoom: 100.0,
        }
    }

    /// Get the current zoom level.
    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    /// Set the zoom level.
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(self.min_zoom, self.max_zoom);
    }

    /// Zoom in.
    pub fn zoom_in(&mut self, factor: f32) {
        self.set_zoom(self.zoom * factor);
    }

    /// Zoom out.
    pub fn zoom_out(&mut self, factor: f32) {
        self.set_zoom(self.zoom / factor);
    }
}