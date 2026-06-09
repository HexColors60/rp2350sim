//! IRQ flag utilities.

use bitflags::bitflags;

bitflags! {
    /// IRQ flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct IrqFlags: u32 {
        const RX_NOT_EMPTY = 1 << 0;
        const RX_FULL = 1 << 1;
        const TX_NOT_FULL = 1 << 2;
        const TX_EMPTY = 1 << 3;
        const ERROR = 1 << 4;
        const OVERFLOW = 1 << 5;
        const UNDERFLOW = 1 << 6;
    }
}