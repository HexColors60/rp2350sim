#![allow(dead_code)]
//! USB trace events.

use serde::{Deserialize, Serialize};

/// USB trace event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbEvent {
    pub tick: u64,
    pub endpoint: u8,
    pub direction: bool,
    pub data: Vec<u8>,
}