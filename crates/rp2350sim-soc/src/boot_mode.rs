//! Boot mode.

use serde::{Deserialize, Serialize};

/// Boot mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BootMode {
    Flash,
    Sram,
    Usb,
}

impl Default for BootMode {
    fn default() -> Self {
        Self::Flash
    }
}