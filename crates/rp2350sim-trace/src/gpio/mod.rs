#![allow(dead_code)]
//! GPIO trace events.

use serde::{Deserialize, Serialize};

/// GPIO trace event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpioEvent {
    pub tick: u64,
    pub pin: u8,
    pub value: bool,
}