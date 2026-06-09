//! RP2350 Differential Testing

mod compare;
mod diff;
mod gpio_diff;
mod memory_diff;
mod mmio_diff;
mod pio_diff;
mod register_diff;
mod report;
mod uart_diff;

pub use compare::Compare;
pub use diff::DiffTester;
pub use gpio_diff::{GpioState, diff_gpio};
pub use memory_diff::{MemoryState, diff_memory};
pub use mmio_diff::{MmioAccess, MmioState, diff_mmio};
pub use pio_diff::{PioSmState, PioState, diff_pio};
pub use register_diff::{RegisterState, diff_registers};
pub use report::DiffReport;
pub use uart_diff::{UartState, diff_uart};

/// Result of a diff comparison.
#[derive(Debug, Clone)]
pub enum DiffResult {
    /// Values match.
    Match,
    /// Values don't match.
    Mismatch {
        expected: u64,
        actual: u64,
    },
    /// Lengths don't match.
    LengthMismatch {
        expected: usize,
        actual: usize,
    },
    /// Element mismatch.
    ElementMismatch {
        index: usize,
        inner: Box<DiffResult>,
    },
    /// Data mismatch.
    DataMismatch {
        name: String,
        expected: Vec<u8>,
        actual: Vec<u8>,
    },
    /// Expected value is missing.
    Missing,
    /// Unexpected extra value.
    Extra,
}