#![allow(dead_code)]
//! IRQ trace events.

use serde::{Deserialize, Serialize};

/// IRQ trace event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrqEvent {
    pub tick: u64,
    pub irq: u8,
    pub active: bool,
}