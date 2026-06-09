//! IRQ trace.

use serde::{Deserialize, Serialize};

/// IRQ trace event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrqTraceEvent {
    pub tick: u64,
    pub irq: u8,
    pub active: bool,
    pub core: u8,
}

/// IRQ trace buffer.
#[derive(Debug, Default)]
pub struct IrqTrace {
    events: Vec<IrqTraceEvent>,
    max_events: usize,
}

impl IrqTrace {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            max_events: 10_000,
        }
    }

    pub fn record(&mut self, event: IrqTraceEvent) {
        if self.events.len() >= self.max_events {
            self.events.remove(0);
        }
        self.events.push(event);
    }

    pub fn events(&self) -> &[IrqTraceEvent] {
        &self.events
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}