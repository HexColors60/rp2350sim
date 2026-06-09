//! RV32 disassembler fallback.

/// Disassemble an RV32 instruction (fallback implementation).
pub fn disasm_rv32_fallback(opcode: u32, _pc: u32) -> String {
    format!("{:08x}\t???", opcode)
}