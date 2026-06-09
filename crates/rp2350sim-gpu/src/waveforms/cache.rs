#![allow(dead_code)]

//! Waveform render cache.

/// Cache for waveform rendering.
#[derive(Debug)]
pub struct WaveformCache {
    /// Whether the cache is valid.
    valid: bool,
    /// Cached vertices.
    cached_vertices: Vec<[f32; 2]>,
}

impl Default for WaveformCache {
    fn default() -> Self {
        Self::new()
    }
}

impl WaveformCache {
    /// Create a new cache.
    pub fn new() -> Self {
        Self {
            valid: false,
            cached_vertices: Vec::new(),
        }
    }

    /// Check if the cache is valid.
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Invalidate the cache.
    pub fn invalidate(&mut self) {
        self.valid = false;
    }

    /// Update the cache.
    pub fn update(&mut self, vertices: Vec<[f32; 2]>) {
        self.cached_vertices = vertices;
        self.valid = true;
    }

    /// Get the cached vertices.
    pub fn vertices(&self) -> &[[f32; 2]] {
        &self.cached_vertices
    }
}