#![allow(dead_code)]

//! Waveform visualization.

use super::cache::WaveformCache;
use super::lines::WaveformLines;
use super::zoom::ZoomController;

/// Waveform renderer.
#[derive(Debug)]
pub struct Waveform {
    /// Waveform name.
    name: String,
    /// Sample data.
    samples: Vec<f32>,
    /// Time stamps for each sample.
    times: Vec<u64>,
    /// Minimum value.
    min_value: f32,
    /// Maximum value.
    max_value: f32,
    /// Zoom controller.
    zoom: ZoomController,
    /// Render cache.
    cache: WaveformCache,
    /// Maximum samples to keep.
    max_samples: usize,
}

impl Waveform {
    /// Create a new waveform.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            samples: Vec::new(),
            times: Vec::new(),
            min_value: f32::MAX,
            max_value: f32::MIN,
            zoom: ZoomController::new(),
            cache: WaveformCache::new(),
            max_samples: 100_000,
        }
    }

    /// Add a sample.
    pub fn add_sample(&mut self, time: u64, value: f32) {
        // Update min/max
        self.min_value = self.min_value.min(value);
        self.max_value = self.max_value.max(value);
        
        // Add sample
        self.samples.push(value);
        self.times.push(time);
        
        // Trim old samples if needed
        if self.samples.len() > self.max_samples {
            let trim = self.samples.len() - self.max_samples;
            self.samples.drain(0..trim);
            self.times.drain(0..trim);
        }
        
        self.cache.invalidate();
    }

    /// Add multiple samples.
    pub fn add_samples(&mut self, samples: &[(u64, f32)]) {
        for (time, value) in samples {
            self.add_sample(*time, *value);
        }
    }

    /// Get the sample count.
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Get value range.
    pub fn value_range(&self) -> (f32, f32) {
        if self.samples.is_empty() {
            (0.0, 1.0)
        } else {
            (self.min_value, self.max_value)
        }
    }

    /// Get time range.
    pub fn time_range(&self) -> (u64, u64) {
        if self.times.is_empty() {
            (0, 0)
        } else {
            (*self.times.first().unwrap(), *self.times.last().unwrap())
        }
    }

    /// Clear all samples.
    pub fn clear(&mut self) {
        self.samples.clear();
        self.times.clear();
        self.min_value = f32::MAX;
        self.max_value = f32::MIN;
        self.cache.invalidate();
    }

    /// Render the waveform.
    pub fn render(&self, width: f32, height: f32, time_scale: f32, scroll_x: f32) -> WaveformLines {
        let mut lines = WaveformLines::new();
        
        if self.samples.is_empty() {
            return lines;
        }
        
        // Calculate visible time range
        let start_time = (scroll_x / time_scale) as u64;
        let end_time = ((scroll_x + width) / time_scale) as u64;
        
        // Find samples in visible range
        let (min_val, max_val) = self.value_range();
        let value_range = (max_val - min_val).max(1.0);
        
        let mut prev_x = None;
        let mut prev_y = None;
        
        for (i, (&time, &value)) in self.times.iter().zip(self.samples.iter()).enumerate() {
            if time < start_time {
                continue;
            }
            if time > end_time {
                break;
            }
            
            let x = (time as f32 * time_scale) - scroll_x;
            let y = height - ((value - min_val) / value_range * height);
            
            if let (Some(px), Some(py)) = (prev_x, prev_y) {
                lines.add_line(px, py, x, y);
            }
            
            prev_x = Some(x);
            prev_y = Some(y);
            
            // Limit number of points rendered
            if i > 10000 {
                break;
            }
        }
        
        lines
    }

    /// Get the name.
    pub fn name(&self) -> &str {
        &self.name
    }
}