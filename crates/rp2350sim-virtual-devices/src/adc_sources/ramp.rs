//! Ramp/sawtooth wave signal source.

use super::AdcSource;

/// A ramp/sawtooth wave signal source.
#[derive(Debug, Clone, Copy)]
pub struct RampSource {
    frequency: f32,
    amplitude: f32,
    rising: bool,
}

impl RampSource {
    /// Create a new ramp wave source.
    pub fn new(frequency: f32) -> Self {
        Self {
            frequency,
            amplitude: 1.0,
            rising: true,
        }
    }

    /// Set the amplitude.
    pub fn with_amplitude(mut self, amplitude: f32) -> Self {
        self.amplitude = amplitude;
        self
    }

    /// Set the direction (rising or falling).
    pub fn with_direction(mut self, rising: bool) -> Self {
        self.rising = rising;
        self
    }
}

impl AdcSource for RampSource {
    fn sample(&self, time: f64) -> f32 {
        let period = 1.0 / self.frequency as f64;
        let t = (time % period) / period;
        if self.rising {
            (t as f32) * self.amplitude
        } else {
            (1.0 - t as f32) * self.amplitude
        }
    }

    fn reset(&mut self) {
        // No state to reset
    }
}