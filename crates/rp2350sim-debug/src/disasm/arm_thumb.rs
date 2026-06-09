//! ARM Thumb disassembler fallback.

/// Disassemble a Thumb instruction (fallback implementation).
pub fn disasm_arm_thumb_fallback(opcode: u16, _pc: u32) -> String {
    format!("{:04x}\t???", opcode)
}