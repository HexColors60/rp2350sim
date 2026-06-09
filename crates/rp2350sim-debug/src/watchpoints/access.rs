//! Access watchpoint.

use serde::{Deserialize, Serialize};

/// Access watchpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessWatchpoint {
    pub addr: u64,
    pub size: usize,
    pub read: bool,
    pub write: bool,
    pub enabled: bool,
}