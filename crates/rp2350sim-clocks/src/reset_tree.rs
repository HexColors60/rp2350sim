//! Reset tree.

use std::collections::HashMap;

/// Reset controller.
#[derive(Debug, Default)]
pub struct ResetTree {
    resets: HashMap<u32, bool>,
}

impl ResetTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self, id: u32) {
        self.resets.insert(id, true);
    }

    pub fn release(&mut self, id: u32) {
        self.resets.insert(id, false);
    }

    pub fn is_reset(&self, id: u32) -> bool {
        self.resets.get(&id).copied().unwrap_or(false)
    }
}