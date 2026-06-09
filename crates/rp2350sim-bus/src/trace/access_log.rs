//! Access log for bus operations.

use rp2350sim_core::AccessWidth;

/// Access log entry.
#[derive(Debug, Clone)]
pub struct AccessLogEntry {
    pub tick: u64,
    pub addr: u32,
    pub width: AccessWidth,
    pub is_write: bool,
    pub value: u64,
    pub device_name: Option<String>,
}

/// Access log.
#[derive(Debug, Default)]
pub struct AccessLog {
    entries: Vec<AccessLogEntry>,
    max_entries: usize,
}

impl AccessLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 10_000,
        }
    }

    pub fn log(&mut self, entry: AccessLogEntry) {
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &[AccessLogEntry] {
        &self.entries
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}