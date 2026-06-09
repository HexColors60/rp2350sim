//! PIO disassembler.
#![allow(dead_code)]

/// Disassemble a PIO instruction.
pub fn disasm_pio(opcode: u16) -> String {
    format!("{:04x}\t???", opcode)
}