//! Checkpoint management.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Checkpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub name: String,
    pub tick: u64,
    pub timestamp: u64,
    pub data: Vec<u8>,
}

impl Checkpoint {
    pub fn new(name: impl Into<String>, tick: u64, data: Vec<u8>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            name: name.into(),
            tick,
            timestamp,
            data,
        }
    }
}