#![allow(dead_code)]

//! Memory state part.

use serde::{Deserialize, Serialize};

/// Memory state for saving.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryState {
    pub sram: Vec<u8>,
    pub flash: Vec<u8>,
}