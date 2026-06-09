//! Decode result.


/// Result of instruction decode.
#[derive(Debug, Clone)]
pub struct DecodeResult {
    /// Instruction length in bytes
    pub length: u8,
    /// Instruction name
    pub name: &'static str,
    /// Decoded operands
    pub operands: Vec<Operand>,
    /// Whether this is a branch instruction
    pub is_branch: bool,
    /// Whether this is a call instruction
    pub is_call: bool,
    /// Whether this is a return instruction
    pub is_return: bool,
    /// Whether this instruction reads memory
    pub reads_memory: bool,
    /// Whether this instruction writes memory
    pub writes_memory: bool,
}

impl Default for DecodeResult {
    fn default() -> Self {
        Self {
            length: 2,
            name: "unknown",
            operands: Vec::new(),
            is_branch: false,
            is_call: false,
            is_return: false,
            reads_memory: false,
            writes_memory: false,
        }
    }
}

/// Instruction operand.
#[derive(Debug, Clone)]
pub enum Operand {
    /// Register operand
    Register { index: u8, name: &'static str },
    /// Immediate operand
    Immediate { value: i64 },
    /// PC-relative offset
    PcRelative { offset: i32 },
    /// Memory operand
    Memory { base: u8, offset: i32, size: u8 },
    /// Shift amount
    Shift { amount: u8, kind: ShiftKind },
    /// Condition code
    Condition { code: u8 },
}

/// Shift kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShiftKind {
    Lsl,
    Lsr,
    Asr,
    Ror,
}