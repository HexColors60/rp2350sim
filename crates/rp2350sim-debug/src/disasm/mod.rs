//! Disassembler.

mod arm_thumb;
mod pio;
mod rv32;

// Re-export the real disassemblers from the CPU crates
pub use rp2350sim_cpu_arm::thumb::disassemble as disasm_arm_thumb;
pub use rp2350sim_cpu_hazard3::rv32::disassemble as disasm_rv32;

// Also keep the internal modules available if needed
pub use arm_thumb::disasm_arm_thumb_fallback;
pub use rv32::disasm_rv32_fallback;
pub use pio::disasm_pio;