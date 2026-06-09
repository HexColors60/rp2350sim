#![allow(dead_code)]

//! GPU mesh utilities.

/// A simple mesh for rendering.
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Vertex positions (x, y, z).
    pub positions: Vec<[f32; 3]>,
    /// Vertex colors (r, g, b, a).
    pub colors: Vec<[f32; 4]>,
    /// Indices for triangles.
    pub indices: Vec<u32>,
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}

impl Mesh {
    /// Create a new empty mesh.
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            colors: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Add a vertex.
    pub fn add_vertex(&mut self, pos: [f32; 3], color: [f32; 4]) -> u32 {
        let index = self.positions.len() as u32;
        self.positions.push(pos);
        self.colors.push(color);
        index
    }

    /// Add a triangle.
    pub fn add_triangle(&mut self, i0: u32, i1: u32, i2: u32) {
        self.indices.push(i0);
        self.indices.push(i1);
        self.indices.push(i2);
    }

    /// Create a quad mesh.
    pub fn quad(x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) -> Self {
        let mut mesh = Self::new();
        let v0 = mesh.add_vertex([x, y, 0.0], color);
        let v1 = mesh.add_vertex([x + w, y, 0.0], color);
        let v2 = mesh.add_vertex([x + w, y + h, 0.0], color);
        let v3 = mesh.add_vertex([x, y + h, 0.0], color);
        mesh.add_triangle(v0, v1, v2);
        mesh.add_triangle(v0, v2, v3);
        mesh
    }
}