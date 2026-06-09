//! Square wave signal source.

use super::AdcSource;

/// A square wave signal source.
#[derive(Debug, Clone, Copy)]
pub struct SquareSource {
    frequency: f32,
    duty_cycle: f32,
    amplitude: f32,
}

impl SquareSource {
    /// Create a new square wave source.
    pub fn new(frequency: f32, duty_cycle: f32) -> Self {
        Self {
            frequency,
            duty_cycle: duty_cycle.clamp(0.0, 1.0),
            amplitude: 1.0,
        }
    }

    /// Set the amplitude.
    pub fn with_amplitude(mut self, amplitude: f32) -> Self {
        self.amplitude = amplitude;
        self
    }
}

impl AdcSource for SquareSource {
    fn sample(&self, time: f64) -> f32 {
        let period = 1.0 / self.frequency as f64;
        let t = (time % period) / period;
        if t < self.duty_cycle as f64 {
            self.amplitude
        } else {
            0.0
        }
    }

    fn reset(&mut self) {
        // No state to reset
    }
}