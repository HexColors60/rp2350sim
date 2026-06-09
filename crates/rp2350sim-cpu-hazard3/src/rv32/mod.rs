//! RV32 instruction decoder and executor.

mod alu;
mod branch;
pub mod decode;
pub mod execute;
mod load_store;
mod muldiv;
mod tables;

pub use decode::{decode, disassemble, Rv32Instruction, Rv32Kind};
pub use execute::{execute, MemoryAccess};