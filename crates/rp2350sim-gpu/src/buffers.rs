#![allow(dead_code)]

//! GPU buffers.

/// A GPU buffer for vertex data.
#[derive(Debug)]
pub struct GpuBuffer {
    /// Buffer data.
    data: Vec<u8>,
    /// Buffer size in bytes.
    size: usize,
}

impl GpuBuffer {
    /// Create a new GPU buffer.
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
            size,
        }
    }

    /// Get the buffer size.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Write data to the buffer.
    pub fn write(&mut self, offset: usize, data: &[u8]) {
        let end = (offset + data.len()).min(self.size);
        if offset < end {
            self.data[offset..end].copy_from_slice(&data[..end - offset]);
        }
    }

    /// Read data from the buffer.
    pub fn read(&self, offset: usize, data: &mut [u8]) {
        let end = (offset + data.len()).min(self.size);
        if offset < end {
            data[..end - offset].copy_from_slice(&self.data[offset..end]);
        }
    }
}