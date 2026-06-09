//! Bus error types.

use thiserror::Error;

/// Bus error type.
#[derive(Error, Debug)]
pub enum BusError {
    #[error("Address 0x{0:08X} is not mapped")]
    UnmappedAddress(u32),

    #[error("Access violation at address 0x{addr:08X}: {reason}")]
    AccessViolation { addr: u32, reason: String },

    #[error("Misaligned access at address 0x{0:08X}")]
    MisalignedAccess(u32),

    #[error("Read from write-only region at 0x{0:08X}")]
    WriteOnlyRead(u32),

    #[error("Write to read-only region at 0x{0:08X}")]
    ReadOnlyWrite(u32),

    #[error("Bus timeout at address 0x{0:08X}")]
    Timeout(u32),

    #[error("Device error: {0}")]
    DeviceError(String),
}

/// Result type for bus operations.
pub type Result<T> = std::result::Result<T, BusError>;