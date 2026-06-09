//! Sine wave signal source.

use super::AdcSource;

/// A sine wave signal source.
#[derive(Debug, Clone, Copy)]
pub struct SineSource {
    frequency: f32,
    amplitude: f32,
    phase: f32,
}

impl SineSource {
    /// Create a new sine wave source.
    pub fn new(frequency: f32, amplitude: f32) -> Self {
        Self {
            frequency,
            amplitude,
            phase: 0.0,
        }
    }

    /// Set the phase offset.
    pub fn with_phase(mut self, phase: f32) -> Self {
        self.phase = phase;
        self
    }
}

impl AdcSource for SineSource {
    fn sample(&self, time: f64) -> f32 {
        let t = time * (self.frequency as f64) + (self.phase as f64);
        (t.sin() as f32) * self.amplitude
    }

    fn reset(&mut self) {
        self.phase = 0.0;
    }
}