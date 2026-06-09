//! Error types for the simulator core.

use thiserror::Error;

/// Main error type for the simulator.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Bus error: {0}")]
    Bus(String),

    #[error("Memory access error at address 0x{0:08X}")]
    MemoryAccess(u32),

    #[error("Invalid memory region: {0}")]
    InvalidRegion(String),

    #[error("CPU error: {0}")]
    Cpu(String),

    #[error("Invalid core index: {0}")]
    InvalidCore(usize),

    #[error("Peripheral error: {0}")]
    Peripheral(String),

    #[error("Invalid instruction at PC 0x{0:08X}")]
    InvalidInstruction(u32),

    #[error("Breakpoint hit at 0x{0:08X}")]
    Breakpoint(u32),

    #[error("Watchpoint hit at 0x{0:08X}")]
    Watchpoint(u32),

    #[error("Unimplemented feature: {0}")]
    Unimplemented(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid save format: {0}")]
    InvalidSaveFormat(String),

    #[error("Timeout: {0}")]
    Timeout(String),
}

/// Result type alias for simulator operations.
pub type Result<T> = std::result::Result<T, Error>;