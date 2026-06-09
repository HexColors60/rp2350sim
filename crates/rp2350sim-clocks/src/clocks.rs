//! Clock management.

use crate::domains::ClockDomains;

/// Clock system.
#[derive(Debug)]
pub struct Clocks {
    domains: ClockDomains,
    tick: u64,
}

impl Default for Clocks {
    fn default() -> Self {
        Self::new()
    }
}

impl Clocks {
    pub fn new() -> Self {
        Self {
            domains: ClockDomains::new(),
            tick: 0,
        }
    }

    pub fn tick(&mut self) {
        self.tick += 1;
    }

    pub fn current_tick(&self) -> u64 {
        self.tick
    }

    pub fn reset(&mut self) {
        self.domains.reset();
        self.tick = 0;
    }
}