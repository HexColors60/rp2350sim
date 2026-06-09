//! Access types.

use rp2350sim_core::AccessWidth;

/// Memory access record.
#[derive(Debug, Clone)]
pub struct AccessRecord {
    pub addr: u32,
    pub width: AccessWidth,
    pub is_write: bool,
    pub value: u64,
    pub tick: u64,
}

/// Access logger.
#[derive(Debug, Default)]
pub struct AccessLog {
    records: Vec<AccessRecord>,
    max_records: usize,
}

impl AccessLog {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            max_records: 100_000,
        }
    }

    pub fn log(&mut self, record: AccessRecord) {
        if self.records.len() >= self.max_records {
            self.records.remove(0);
        }
        self.records.push(record);
    }

    pub fn records(&self) -> &[AccessRecord] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}