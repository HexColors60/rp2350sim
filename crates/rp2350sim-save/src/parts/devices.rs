#![allow(dead_code)]

//! Device state for save/restore.

use serde::{Deserialize, Serialize};

/// Device state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DevicesState {
    // Placeholder for device state
}