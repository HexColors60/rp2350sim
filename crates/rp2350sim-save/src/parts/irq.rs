#![allow(dead_code)]

//! IRQ state for save/restore.

use serde::{Deserialize, Serialize};

/// IRQ state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IrqState {
    // Placeholder for IRQ state
}