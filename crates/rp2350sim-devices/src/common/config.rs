//! Configuration utilities.

/// Configuration bit definitions.
pub struct ConfigBits;

impl ConfigBits {
    pub const ENABLE: u32 = 1 << 0;
    pub const RESET: u32 = 1 << 1;
    pub const IRQ_ENABLE: u32 = 1 << 2;
}