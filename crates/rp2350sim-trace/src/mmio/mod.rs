#![allow(dead_code)]
//! MMIO trace events.

use serde::{Deserialize, Serialize};

/// MMIO trace event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmioEvent {
    pub tick: u64,
    pub addr: u32,
    pub value: u32,
    pub is_write: bool,
    pub width: u8,
}