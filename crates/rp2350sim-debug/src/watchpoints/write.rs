//! Write watchpoints.
#![allow(dead_code)]

use std::collections::HashSet;

/// Write watchpoint manager.
#[derive(Debug, Default)]
pub struct WriteWatchpoints {
    /// Addresses being watched.
    addresses: HashSet<u32>,
}

impl WriteWatchpoints {
    /// Create a new write watchpoint manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a watchpoint.
    pub fn add(&mut self, addr: u32) {
        self.addresses.insert(addr);
    }

    /// Remove a watchpoint.
    pub fn remove(&mut self, addr: u32) {
        self.addresses.remove(&addr);
    }

    /// Check if an address is being watched.
    pub fn is_watched(&self, addr: u32) -> bool {
        self.addresses.contains(&addr)
    }

    /// Get all watched addresses.
    pub fn addresses(&self) -> &HashSet<u32> {
        &self.addresses
    }
}