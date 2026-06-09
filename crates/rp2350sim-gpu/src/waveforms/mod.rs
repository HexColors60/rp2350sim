//! Waveform rendering module.
#![allow(dead_code)]

mod waveform;
mod lines;
mod zoom;
mod cache;

pub use waveform::Waveform;

/// Waveform renderer for multiple signals.
#[derive(Debug)]
pub struct WaveformRenderer {
    /// Waveforms.
    waveforms: Vec<Waveform>,
    /// Width.
    width: f32,
    /// Height.
    height: f32,
    /// Time scale.
    time_scale: f32,
    /// Scroll offset.
    scroll_x: f32,
    /// Zoom level.
    zoom: f32,
}

impl WaveformRenderer {
    /// Create a new waveform renderer.
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            waveforms: Vec::new(),
            width,
            height,
            time_scale: 1.0,
            scroll_x: 0.0,
            zoom: 1.0,
        }
    }

    /// Add a waveform.
    pub fn add_waveform(&mut self, name: &str) -> usize {
        let idx = self.waveforms.len();
        self.waveforms.push(Waveform::new(name));
        idx
    }

    /// Get a waveform by index.
    pub fn get_waveform(&self, index: usize) -> Option<&Waveform> {
        self.waveforms.get(index)
    }

    /// Get a waveform by index (mutable).
    pub fn get_waveform_mut(&mut self, index: usize) -> Option<&mut Waveform> {
        self.waveforms.get_mut(index)
    }

    /// Add a sample to a waveform.
    pub fn add_sample(&mut self, waveform: usize, time: u64, value: f32) {
        if let Some(w) = self.waveforms.get_mut(waveform) {
            w.add_sample(time, value);
        }
    }

    /// Set zoom level.
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.1, 100.0);
        self.update_time_scale();
    }

    /// Set scroll offset.
    pub fn set_scroll(&mut self, scroll_x: f32) {
        self.scroll_x = scroll_x.max(0.0);
    }

    /// Update time scale.
    fn update_time_scale(&mut self) {
        self.time_scale = self.zoom;
    }

    /// Render all waveforms.
    pub fn render(&self) {
        #[allow(unused_variables)]
        let waveform_height = self.height / self.waveforms.len().max(1) as f32;
        #[allow(unused_variables)]
        let y = 0.0;
        
        for waveform in &self.waveforms {
            waveform.render(self.width, waveform_height, self.time_scale, self.scroll_x);
            _ = waveform_height;
        }
    }

    /// Resize the renderer.
    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    /// Clear all waveforms.
    pub fn clear(&mut self) {
        for waveform in &mut self.waveforms {
            waveform.clear();
        }
    }

    /// Get the number of waveforms.
    pub fn count(&self) -> usize {
        self.waveforms.len()
    }
}