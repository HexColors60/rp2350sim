//! Signal source component.

use serde::{Deserialize, Serialize};

/// Signal source component for generating analog/digital signals.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SignalSource {
    /// Constant value.
    Constant(f32),
    /// Sine wave with frequency and amplitude.
    Sine { frequency: f32, amplitude: f32, phase: f32 },
    /// Square wave with frequency and duty cycle.
    Square { frequency: f32, duty_cycle: f32 },
    /// Ramp/sawtooth wave.
    Ramp { frequency: f32, rising: bool },
    /// Noise signal.
    Noise { amplitude: f32 },
}

impl Default for SignalSource {
    fn default() -> Self {
        Self::Constant(0.0)
    }
}

impl SignalSource {
    /// Create a constant signal source.
    pub fn constant(value: f32) -> Self {
        Self::Constant(value)
    }

    /// Create a sine wave signal source.
    pub fn sine(frequency: f32, amplitude: f32) -> Self {
        Self::Sine {
            frequency,
            amplitude,
            phase: 0.0,
        }
    }

    /// Create a square wave signal source.
    pub fn square(frequency: f32, duty_cycle: f32) -> Self {
        Self::Square {
            frequency,
            duty_cycle: duty_cycle.clamp(0.0, 1.0),
        }
    }

    /// Update the signal source (advance phase for oscillators).
    pub fn update(&mut self, delta_time: f32) {
        match self {
            Self::Sine { phase, frequency, .. } => {
                *phase += delta_time * *frequency * std::f32::consts::TAU;
                *phase %= std::f32::consts::TAU;
            }
            Self::Square { frequency, .. } => {
                let _ = frequency; // Phase tracking not needed for sample-based
            }
            Self::Ramp { frequency, .. } => {
                let _ = frequency; // Phase tracking not needed for sample-based
            }
            _ => {}
        }
    }

    /// Sample the signal at a given time.
    pub fn sample(&self, time: f64) -> f32 {
        match self {
            Self::Constant(v) => *v,
            Self::Sine { frequency, amplitude, phase } => {
                let t = time * (*frequency as f64) + (*phase as f64);
                (t.sin() as f32) * amplitude
            }
            Self::Square { frequency, duty_cycle } => {
                let period = 1.0 / *frequency as f64;
                let t = (time % period) / period;
                if t < *duty_cycle as f64 {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Ramp { frequency, rising } => {
                let period = 1.0 / *frequency as f64;
                let t = (time % period) / period;
                if *rising {
                    t as f32
                } else {
                    1.0 - t as f32
                }
            }
            Self::Noise { amplitude } => {
                // Simple pseudo-random noise
                let t = time.to_bits();
                let noise = ((t.wrapping_mul(1103515245).wrapping_add(12345)) >> 16) as i16;
                (noise as f32 / 32768.0) * amplitude
            }
        }
    }
}