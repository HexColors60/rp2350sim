//! RP2350 ARM Cortex-M33 CPU Backend
//!
//! This crate provides an ARM Cortex-M33 CPU backend for the simulator.

pub mod backend;
pub mod core;
pub mod exception;
pub mod nvic;
pub mod psr;
pub mod regs;
pub mod state;
pub mod systick;
pub mod thumb;
pub mod vector_table;

pub use backend::ArmBackend;
pub use core::ArmCore;
pub use state::ArmState;
pub use thumb::{decode, disassemble, ThumbInstruction, ThumbKind};