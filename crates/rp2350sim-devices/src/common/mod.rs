//! Common device utilities.

pub mod config;
pub mod fifo;
pub mod irq_flags;
pub mod reg32;
pub mod status;

pub use fifo::Fifo;
pub use reg32::Reg32;