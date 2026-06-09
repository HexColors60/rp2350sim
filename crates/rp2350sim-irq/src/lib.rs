//! RP2350 IRQ System

pub mod enable;
pub mod inject;
pub mod lines;
pub mod pending;
pub mod priority;
pub mod routing;
pub mod trace;

pub use lines::IrqLines;