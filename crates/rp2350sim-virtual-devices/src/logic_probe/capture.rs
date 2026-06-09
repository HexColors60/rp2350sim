//! Signal capture buffer.

use std::collections::VecDeque;

/// A capture buffer for storing signal samples with timestamps.
#[derive(Debug, Clone)]
pub struct CaptureBuffer {
    /// Samples with timestamps.
    samples: VecDeque<(f64, bool)>,
    /// Maximum buffer size.
    max_size: usize,
}

impl CaptureBuffer {
    /// Create a new capture buffer.
    pub fn new(max_size: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// Add a sample.
    pub fn push(&mut self, time: f64, value: bool) {
        if self.samples.len() >= self.max_size {
            self.samples.pop_front();
        }
        self.samples.push_back((time, value));
    }

    /// Get all samples.
    pub fn samples(&self) -> &VecDeque<(f64, bool)> {
        &self.samples
    }

    /// Clear the buffer.
    pub fn clear(&mut self) {
        self.samples.clear();
    }

    /// Get the number of samples.
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
}