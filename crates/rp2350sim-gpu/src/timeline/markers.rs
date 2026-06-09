#![allow(dead_code)]

//! Timeline markers.

/// A timeline marker.
#[derive(Debug, Clone)]
pub struct Marker {
    /// Time position.
    pub time: u64,
    /// Marker label.
    pub label: String,
    /// Marker color.
    pub color: [f32; 4],
}

/// Timeline markers for events.
#[derive(Debug, Default)]
pub struct TimelineMarkers {
    /// Markers.
    markers: Vec<Marker>,
}

impl TimelineMarkers {
    /// Create new timeline markers.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a marker.
    pub fn add(&mut self, time: u64, label: &str, color: [f32; 4]) {
        self.markers.push(Marker {
            time,
            label: label.to_string(),
            color,
        });
    }

    /// Remove a marker by index.
    pub fn remove(&mut self, index: usize) {
        if index < self.markers.len() {
            self.markers.remove(index);
        }
    }

    /// Get all markers.
    pub fn markers(&self) -> &[Marker] {
        &self.markers
    }

    /// Get marker at time (within tolerance).
    pub fn get_at_time(&self, time: u64, tolerance: u64) -> Option<&Marker> {
        self.markers.iter().find(|m| m.time.abs_diff(time) <= tolerance)
    }

    /// Clear all markers.
    pub fn clear(&mut self) {
        self.markers.clear();
    }

    /// Sort markers by time.
    pub fn sort(&mut self) {
        self.markers.sort_by_key(|m| m.time);
    }

    /// Render all markers.
    pub fn render(&self, time_scale: f32, scroll_x: f32, height: f32) {
        for marker in &self.markers {
            let x = (marker.time as f32 * time_scale) - scroll_x;
            
            // Only render if visible
            if x >= 0.0 {
                self.render_marker(marker, x, height);
            }
        }
    }

    /// Render a single marker.
    fn render_marker(&self, marker: &Marker, x: f32, height: f32) {
        // Marker rendering would use macroquad or wgpu
        // For now, this is a placeholder
        let _ = (marker.label.as_str(), marker.color, x, height);
    }
}