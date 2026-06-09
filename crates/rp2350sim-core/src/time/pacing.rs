//! Simulation pacing control.

use std::time::{Duration, Instant};

/// Controls the pacing of simulation time vs wall-clock time.
#[derive(Debug, Clone)]
pub enum PacingMode {
    /// Run as fast as possible (no synchronization)
    Unlimited,
    /// Synchronize simulation time with wall-clock time
    Realtime {
        /// Target ratio of simulation time to wall-clock time (1.0 = realtime)
        ratio: f64,
    },
    /// Fixed number of cycles per step
    FixedStep {
        /// Number of cycles per step
        cycles_per_step: u64,
    },
}

impl Default for PacingMode {
    fn default() -> Self {
        Self::Unlimited
    }
}

/// Pacing controller.
#[derive(Debug)]
pub struct PacingController {
    mode: PacingMode,
    last_update: Option<Instant>,
    accumulated_error: Duration,
}

impl PacingController {
    pub fn new(mode: PacingMode) -> Self {
        Self {
            mode,
            last_update: None,
            accumulated_error: Duration::ZERO,
        }
    }

    pub fn set_mode(&mut self, mode: PacingMode) {
        self.mode = mode;
        self.last_update = None;
        self.accumulated_error = Duration::ZERO;
    }

    /// Calculate how many simulation cycles to run.
    pub fn calculate_cycles(&mut self, clock_hz: u64) -> u64 {
        match &self.mode {
            PacingMode::Unlimited => u64::MAX,
            PacingMode::Realtime { ratio } => {
                let now = Instant::now();
                if let Some(last) = self.last_update {
                    let elapsed = now - last + self.accumulated_error;
                    let target_cycles = (elapsed.as_nanos() as f64 * clock_hz as f64 * ratio / 1e9) as u64;
                    self.accumulated_error = elapsed - Duration::from_nanos(
                        (target_cycles as f64 * 1e9 / (clock_hz as f64 * ratio)) as u64
                    );
                    self.last_update = Some(now);
                    target_cycles
                } else {
                    self.last_update = Some(now);
                    0
                }
            }
            PacingMode::FixedStep { cycles_per_step } => *cycles_per_step,
        }
    }

    /// Reset the pacing controller.
    pub fn reset(&mut self) {
        self.last_update = None;
        self.accumulated_error = Duration::ZERO;
    }
}