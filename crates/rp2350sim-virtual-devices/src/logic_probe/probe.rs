//! Logic probe emulation.

use std::collections::VecDeque;

/// Logic probe for capturing digital signals.
#[derive(Debug)]
pub struct LogicProbe {
    /// Pin being probed.
    pin: u8,
    /// Sample buffer.
    buffer: VecDeque<bool>,
    /// Maximum buffer size.
    max_size: usize,
    /// Last sampled value.
    last_value: bool,
}

impl LogicProbe {
    /// Create a new logic probe.
    pub fn new(pin: u8, buffer_size: usize) -> Self {
        Self {
            pin,
            buffer: VecDeque::with_capacity(buffer_size),
            max_size: buffer_size,
            last_value: false,
        }
    }

    /// Get the probed pin.
    pub fn pin(&self) -> u8 {
        self.pin
    }

    /// Sample the current value.
    pub fn sample(&mut self, value: bool) {
        if self.buffer.len() >= self.max_size {
            self.buffer.pop_front();
        }
        self.buffer.push_back(value);
        self.last_value = value;
    }

    /// Get the last sampled value.
    pub fn last_value(&self) -> bool {
        self.last_value
    }

    /// Get the sample buffer.
    pub fn buffer(&self) -> &VecDeque<bool> {
        &self.buffer
    }

    /// Clear the buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}