//! Display framebuffer.

/// A simple framebuffer for display devices.
#[derive(Debug, Clone)]
pub struct Framebuffer {
    /// Width in pixels.
    width: u32,
    /// Height in pixels.
    height: u32,
    /// Pixel data (RGB565 or RGB888).
    data: Vec<u8>,
    /// Bytes per pixel.
    #[allow(dead_code)]
    bpp: u8,
}

impl Framebuffer {
    /// Create a new framebuffer.
    pub fn new(width: u32, height: u32) -> Self {
        let bpp = 2; // RGB565
        Self {
            width,
            height,
            data: vec![0; (width * height * bpp as u32) as usize],
            bpp,
        }
    }

    /// Get the dimensions.
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get the raw data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get a mutable reference to the raw data.
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Set a pixel at (x, y) to a 16-bit RGB565 value.
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u16) {
        if x < self.width && y < self.height {
            let offset = ((y * self.width + x) * 2) as usize;
            if offset + 1 < self.data.len() {
                self.data[offset] = (color >> 8) as u8;
                self.data[offset + 1] = color as u8;
            }
        }
    }

    /// Get a pixel at (x, y).
    pub fn get_pixel(&self, x: u32, y: u32) -> u16 {
        if x < self.width && y < self.height {
            let offset = ((y * self.width + x) * 2) as usize;
            if offset + 1 < self.data.len() {
                return ((self.data[offset] as u16) << 8) | (self.data[offset + 1] as u16);
            }
        }
        0
    }

    /// Clear the framebuffer.
    pub fn clear(&mut self, color: u16) {
        let hi = (color >> 8) as u8;
        let lo = color as u8;
        for chunk in self.data.chunks_exact_mut(2) {
            chunk[0] = hi;
            chunk[1] = lo;
        }
    }
}