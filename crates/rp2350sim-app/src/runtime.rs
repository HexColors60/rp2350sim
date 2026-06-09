#![allow(dead_code)]

//! Runtime management.


/// Runtime state.
pub struct Runtime {
    tick: u64,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        Self { tick: 0 }
    }

    pub fn tick(&mut self) {
        self.tick += 1;
    }

    pub fn current_tick(&self) -> u64 {
        self.tick
    }
}