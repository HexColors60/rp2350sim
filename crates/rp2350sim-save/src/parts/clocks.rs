#![allow(dead_code)]

//! Clocks state part.

use serde::{Deserialize, Serialize};

/// Clocks state for saving.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClocksState {
    pub tick: u64,
}