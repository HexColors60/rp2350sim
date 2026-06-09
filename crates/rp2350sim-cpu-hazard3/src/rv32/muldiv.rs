#![allow(dead_code)]

//! MUL/DIV extension.

/// Multiply (low 32 bits).
pub fn mul(a: u32, b: u32) -> u32 {
    a.wrapping_mul(b)
}

/// Multiply (high 32 bits, signed).
pub fn mulh(a: u32, b: u32) -> u32 {
    ((a as i32 as i64) * (b as i32 as i64) >> 32) as u32
}

/// Multiply (high 32 bits, unsigned).
pub fn mulhu(a: u32, b: u32) -> u32 {
    ((a as u64) * (b as u64) >> 32) as u32
}

/// Multiply (high 32 bits, signed * unsigned).
pub fn mulhsu(a: u32, b: u32) -> u32 {
    ((a as i32 as i64) * (b as u64) as i64 >> 32) as u32
}

/// Divide (signed).
pub fn div(a: u32, b: u32) -> u32 {
    if b == 0 {
        u32::MAX
    } else if a == 0x80000000 && b == 0xFFFFFFFF {
        a // Overflow
    } else {
        (a as i32 / b as i32) as u32
    }
}

/// Divide (unsigned).
pub fn divu(a: u32, b: u32) -> u32 {
    if b == 0 {
        u32::MAX
    } else {
        a / b
    }
}

/// Remainder (signed).
pub fn rem(a: u32, b: u32) -> u32 {
    if b == 0 {
        a
    } else if a == 0x80000000 && b == 0xFFFFFFFF {
        0 // Overflow
    } else {
        (a as i32 % b as i32) as u32
    }
}

/// Remainder (unsigned).
pub fn remu(a: u32, b: u32) -> u32 {
    if b == 0 {
        a
    } else {
        a % b
    }
}