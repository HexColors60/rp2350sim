//! MMIO diff utilities.

use crate::DiffResult;
use std::collections::HashMap;

/// MMIO access record.
#[derive(Debug, Clone)]
pub struct MmioAccess {
    /// Address.
    pub addr: u32,
    /// Value.
    pub value: u32,
    /// Whether it was a write.
    pub is_write: bool,
    /// Timestamp.
    pub time: u64,
}

/// MMIO state for diff comparison.
#[derive(Debug, Clone, Default)]
pub struct MmioState {
    /// Register values.
    pub registers: HashMap<u32, u32>,
    /// Access log.
    pub accesses: Vec<MmioAccess>,
}

impl MmioState {
    /// Create a new MMIO state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a read.
    pub fn record_read(&mut self, addr: u32, value: u32, time: u64) {
        self.accesses.push(MmioAccess {
            addr,
            value,
            is_write: false,
            time,
        });
    }

    /// Record a write.
    pub fn record_write(&mut self, addr: u32, value: u32, time: u64) {
        self.registers.insert(addr, value);
        self.accesses.push(MmioAccess {
            addr,
            value,
            is_write: true,
            time,
        });
    }
}

/// Compare MMIO states.
pub fn diff_mmio(a: &MmioState, b: &MmioState) -> Vec<(u32, DiffResult)> {
    let mut results = Vec::new();
    
    // Compare registers
    for (addr, value_a) in &a.registers {
        match b.registers.get(addr) {
            Some(value_b) if value_a == value_b => continue,
            Some(value_b) => results.push((*addr, DiffResult::Mismatch {
                expected: *value_a as u64,
                actual: *value_b as u64,
            })),
            None => results.push((*addr, DiffResult::Missing)),
        }
    }
    
    // Check for registers only in b
    for addr in b.registers.keys() {
        if !a.registers.contains_key(addr) {
            results.push((*addr, DiffResult::Extra));
        }
    }
    
    results
}