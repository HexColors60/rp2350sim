//! Thumb instruction decoder and executor.

mod alu;
mod branch;
pub mod decode;
pub mod execute;
mod load_store;
mod system;
mod tables;

pub use decode::{decode, disassemble, ThumbInstruction, ThumbKind};
pub use execute::{execute, MemoryAccess};