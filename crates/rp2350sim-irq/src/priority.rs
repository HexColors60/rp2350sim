//! IRQ priority.

use std::collections::HashMap;

/// IRQ priority state.
#[derive(Debug, Clone, Default)]
pub struct PriorityState {
    priorities: HashMap<u8, u8>,
}

impl PriorityState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, irq: u8, priority: u8) {
        self.priorities.insert(irq, priority);
    }

    pub fn get(&self, irq: u8) -> u8 {
        self.priorities.get(&irq).copied().unwrap_or(0)
    }
}