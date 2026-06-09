//! CPU trace.

use serde::{Deserialize, Serialize};

/// Trace event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuTraceEvent {
    pub tick: u64,
    pub core: u8,
    pub pc: u32,
    pub opcode: u32,
    pub disasm: Option<String>,
}

/// CPU trace buffer.
#[derive(Debug, Default)]
pub struct CpuTrace {
    events: Vec<CpuTraceEvent>,
    max_events: usize,
    enabled: bool,
}

impl CpuTrace {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            max_events: 100_000,
            enabled: false,
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn record(&mut self, event: CpuTraceEvent) {
        if !self.enabled {
            return;
        }
        if self.events.len() >= self.max_events {
            self.events.remove(0);
        }
        self.events.push(event);
    }

    pub fn events(&self) -> &[CpuTraceEvent] {
        &self.events
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}