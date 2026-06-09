//! Tick counter type.

use serde::{Deserialize, Serialize};

/// Simulation tick counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Ticks(pub u64);

impl Ticks {
    pub const ZERO: Self = Self(0);

    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(&self) -> u64 {
        self.0
    }

    pub fn saturating_add(&self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    pub fn saturating_sub(&self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }

    pub fn wrapping_add(&self, other: Self) -> Self {
        Self(self.0.wrapping_add(other.0))
    }
}

impl Default for Ticks {
    fn default() -> Self {
        Self::ZERO
    }
}

impl std::ops::Add for Ticks {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl std::ops::Add<u64> for Ticks {
    type Output = Self;

    fn add(self, other: u64) -> Self::Output {
        Self(self.0 + other)
    }
}

impl std::ops::Sub for Ticks {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - other.0)
    }
}

impl std::ops::Sub<u64> for Ticks {
    type Output = Self;

    fn sub(self, other: u64) -> Self::Output {
        Self(self.0 - other)
    }
}