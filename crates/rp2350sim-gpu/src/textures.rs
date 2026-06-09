#![allow(dead_code)]

//! GPU texture utilities.

/// A simple texture.
#[derive(Debug)]
pub struct Texture {
    /// Texture width.
    pub width: u32,
    /// Texture height.
    pub height: u32,
    /// Pixel data (RGBA).
    pub data: Vec<u8>,
}

impl Texture {
    /// Create a new texture.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0; (width * height * 4) as usize],
        }
    }

    /// Create a solid color texture.
    pub fn solid(width: u32, height: u32, r: u8, g: u8, b: u8, a: u8) -> Self {
        let mut texture = Self::new(width, height);
        for pixel in texture.data.chunks_exact_mut(4) {
            pixel[0] = r;
            pixel[1] = g;
            pixel[2] = b;
            pixel[3] = a;
        }
        texture
    }

    /// Set a pixel.
    pub fn set_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
        if x < self.width && y < self.height {
            let offset = ((y * self.width + x) * 4) as usize;
            if offset + 3 < self.data.len() {
                self.data[offset] = r;
                self.data[offset + 1] = g;
                self.data[offset + 2] = b;
                self.data[offset + 3] = a;
            }
        }
    }

    /// Get a pixel.
    pub fn get_pixel(&self, x: u32, y: u32) -> (u8, u8, u8, u8) {
        if x < self.width && y < self.height {
            let offset = ((y * self.width + x) * 4) as usize;
            if offset + 3 < self.data.len() {
                return (self.data[offset], self.data[offset + 1], self.data[offset + 2], self.data[offset + 3]);
            }
        }
        (0, 0, 0, 0)
    }
}