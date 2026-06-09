#![allow(dead_code)]
//! ALU operations.

/// Perform addition.
pub fn add(a: u32, b: u32) -> u32 {
    a.wrapping_add(b)
}

/// Perform subtraction.
pub fn sub(a: u32, b: u32) -> u32 {
    a.wrapping_sub(b)
}

/// Logical shift left.
pub fn sll(a: u32, b: u32) -> u32 {
    a << (b & 0x1F)
}

/// Logical shift right.
pub fn srl(a: u32, b: u32) -> u32 {
    a >> (b & 0x1F)
}

/// Arithmetic shift right.
pub fn sra(a: u32, b: u32) -> u32 {
    ((a as i32) >> (b & 0x1F)) as u32
}

/// Set less than (signed).
pub fn slt(a: u32, b: u32) -> u32 {
    if (a as i32) < (b as i32) { 1 } else { 0 }
}

/// Set less than (unsigned).
pub fn sltu(a: u32, b: u32) -> u32 {
    if a < b { 1 } else { 0 }
}

/// XOR.
pub fn xor(a: u32, b: u32) -> u32 {
    a ^ b
}

/// OR.
pub fn or(a: u32, b: u32) -> u32 {
    a | b
}

/// AND.
pub fn and(a: u32, b: u32) -> u32 {
    a & b
}