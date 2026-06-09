#![allow(dead_code)]

//! CPU state part.

use serde::{Deserialize, Serialize};

/// CPU state for saving.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuState {
    pub pc: u32,
    pub sp: u32,
    pub regs: [u32; 16],
}