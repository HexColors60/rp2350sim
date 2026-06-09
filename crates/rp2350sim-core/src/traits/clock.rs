//! Clock trait.

use crate::Ticks;

/// Clock source trait.
pub trait ClockSource: Send + Sync {
    /// Get the current tick count.
    fn ticks(&self) -> Ticks;

    /// Get the clock frequency in Hz.
    fn frequency_hz(&self) -> u64;

    /// Advance the clock by a number of ticks.
    fn advance(&mut self, ticks: u64);

    /// Reset the clock.
    fn reset(&mut self);
}

/// Clock consumer trait.
pub trait ClockConsumer: Send + Sync {
    /// Called when the clock ticks.
    fn on_tick(&mut self, ticks: Ticks);
}