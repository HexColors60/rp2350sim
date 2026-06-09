//! Waveform line rendering.

/// Lines for rendering a waveform.
#[derive(Debug, Clone)]
pub struct WaveformLines {
    /// Line vertices.
    vertices: Vec<[f32; 2]>,
}

impl Default for WaveformLines {
    fn default() -> Self {
        Self::new()
    }
}

impl WaveformLines {
    /// Create new waveform lines.
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
        }
    }

    /// Add a vertex.
    pub fn add_vertex(&mut self, x: f32, y: f32) {
        self.vertices.push([x, y]);
    }

    /// Add a line segment (two vertices).
    pub fn add_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.vertices.push([x1, y1]);
        self.vertices.push([x2, y2]);
    }

    /// Get the vertices.
    pub fn vertices(&self) -> &[[f32; 2]] {
        &self.vertices
    }
}