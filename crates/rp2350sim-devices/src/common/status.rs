//! Status register utilities.

/// Status bit definitions.
pub struct StatusBits;

impl StatusBits {
    pub const BUSY: u32 = 1 << 0;
    pub const READY: u32 = 1 << 1;
    pub const ERROR: u32 = 1 << 2;
    pub const ENABLED: u32 = 1 << 3;
}