#![allow(dead_code)]

//! ALU operations.

/// Perform addition with carry.
pub fn add_with_carry(a: u32, b: u32, c: bool) -> (u32, bool, bool) {
    let result = a.wrapping_add(b).wrapping_add(if c { 1 } else { 0 });
    let carry = (a as u64 + b as u64 + if c { 1 } else { 0 }) > 0xFFFFFFFF;
    let overflow = (a >> 31 == b >> 31) && (a >> 31 != result >> 31);
    (result, carry, overflow)
}

/// Perform subtraction with carry.
pub fn sub_with_carry(a: u32, b: u32, c: bool) -> (u32, bool, bool) {
    let result = a.wrapping_sub(b).wrapping_sub(if c { 0 } else { 1 });
    let carry = (a as i64 - b as i64 - if c { 0 } else { 1 }) >= 0;
    let overflow = (a >> 31 != b >> 31) && (a >> 31 != result >> 31);
    (result, carry, overflow)
}

/// Logical shift left.
pub fn lsl(value: u32, shift: u32) -> (u32, bool) {
    if shift == 0 {
        (value, false)
    } else {
        let carry = (value >> (32 - shift)) & 1 != 0;
        (value << shift, carry)
    }
}

/// Logical shift right.
pub fn lsr(value: u32, shift: u32) -> (u32, bool) {
    if shift == 0 {
        (0, (value >> 31) != 0)
    } else {
        let carry = (value >> (shift - 1)) & 1 != 0;
        (value >> shift, carry)
    }
}

/// Arithmetic shift right.
pub fn asr(value: u32, shift: u32) -> (u32, bool) {
    if shift == 0 {
        let carry = (value >> 31) != 0;
        (if value & 0x80000000 != 0 { 0xFFFFFFFF } else { 0 }, carry)
    } else {
        let carry = (value >> (shift - 1)) & 1 != 0;
        (((value as i32) >> shift) as u32, carry)
    }
}

/// Rotate right.
pub fn ror(value: u32, shift: u32) -> (u32, bool) {
    if shift == 0 {
        (value, false)
    } else {
        let shift = shift & 31;
        let carry = (value >> (shift - 1)) & 1 != 0;
        ((value >> shift) | (value << (32 - shift)), carry)
    }
}