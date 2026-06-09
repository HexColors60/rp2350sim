#![allow(dead_code)]
//! Instruction trace events.

use serde::{Deserialize, Serialize};

/// Instruction trace event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionEvent {
    pub tick: u64,
    pub core: u8,
    pub pc: u32,
    pub opcode: u32,
}