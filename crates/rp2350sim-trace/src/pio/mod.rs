#![allow(dead_code)]
//! PIO trace events.

use serde::{Deserialize, Serialize};

/// PIO trace event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PioEvent {
    pub tick: u64,
    pub pio: u8,
    pub sm: u8,
    pub pc: u8,
    pub opcode: u16,
}