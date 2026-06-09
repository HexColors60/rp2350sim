//! Clock domain management.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Clock domain identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClockDomainId(pub u8);

impl ClockDomainId {
    pub const SYSTEM: Self = Self(0);
    pub const PERIPHERAL: Self = Self(1);
    pub const USB: Self = Self(2);
    pub const ADC: Self = Self(3);
    pub const RTC: Self = Self(4);
}

/// Clock domain configuration and state.
#[derive(Debug, Clone)]
pub struct ClockDomain {
    /// Domain identifier
    pub id: ClockDomainId,
    /// Base frequency in Hz
    pub base_freq_hz: u64,
    /// Current divider value
    pub divider: u32,
    /// Whether the clock is enabled
    pub enabled: bool,
    /// Current tick count
    pub ticks: u64,
}

impl ClockDomain {
    pub fn new(id: ClockDomainId, base_freq_hz: u64) -> Self {
        Self {
            id,
            base_freq_hz,
            divider: 1,
            enabled: true,
            ticks: 0,
        }
    }

    /// Get the effective frequency after division.
    pub fn effective_freq_hz(&self) -> u64 {
        if self.enabled && self.divider > 0 {
            self.base_freq_hz / self.divider as u64
        } else {
            0
        }
    }

    /// Convert ticks to wall-clock duration.
    pub fn ticks_to_duration(&self, ticks: u64) -> Duration {
        let freq = self.effective_freq_hz();
        if freq == 0 {
            return Duration::ZERO;
        }
        let secs = ticks / freq;
        let nanos = ((ticks % freq) * 1_000_000_000 / freq) as u32;
        Duration::new(secs, nanos)
    }

    /// Convert wall-clock duration to ticks.
    pub fn duration_to_ticks(&self, duration: Duration) -> u64 {
        let freq = self.effective_freq_hz();
        let nanos = duration.as_nanos() as u64;
        nanos * freq / 1_000_000_000
    }

    /// Advance the clock by a number of ticks.
    pub fn advance(&mut self, ticks: u64) {
        self.ticks += ticks;
    }

    /// Reset the clock.
    pub fn reset(&mut self) {
        self.ticks = 0;
    }
}