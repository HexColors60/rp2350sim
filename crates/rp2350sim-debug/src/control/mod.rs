//! Debug controller.

use std::collections::HashSet;

/// Debug controller.
#[derive(Debug, Default)]
pub struct DebugController {
    halted: bool,
    stepping: bool,
    step_count: u64,
    breakpoints: HashSet<u32>,
}

impl DebugController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn halt(&mut self) {
        self.halted = true;
    }

    pub fn resume(&mut self) {
        self.halted = false;
        self.stepping = false;
    }

    pub fn step(&mut self, count: u64) {
        self.halted = false;
        self.stepping = true;
        self.step_count = count;
    }

    pub fn is_halted(&self) -> bool {
        self.halted
    }

    /// Add a breakpoint at the given address.
    pub fn add_breakpoint(&mut self, addr: u32) {
        self.breakpoints.insert(addr);
    }

    /// Remove a breakpoint at the given address.
    pub fn remove_breakpoint(&mut self, addr: u32) {
        self.breakpoints.remove(&addr);
    }

    /// Check if there's a breakpoint at the given address.
    pub fn has_breakpoint(&self, addr: u32) -> bool {
        self.breakpoints.contains(&addr)
    }

    /// Get all breakpoints.
    pub fn breakpoints(&self) -> &HashSet<u32> {
        &self.breakpoints
    }

    /// Clear all breakpoints.
    pub fn clear_breakpoints(&mut self) {
        self.breakpoints.clear();
    }
}