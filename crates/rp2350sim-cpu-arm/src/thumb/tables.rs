#![allow(dead_code)]

//! Instruction tables.

/// Condition code names.
pub const COND_NAMES: [&str; 16] = [
    "eq", "ne", "cs", "cc", "mi", "pl", "vs", "vc",
    "hi", "ls", "ge", "lt", "gt", "le", "al", "nv",
];

/// Check a condition code.
pub fn check_condition(cond: u8, n: bool, z: bool, c: bool, v: bool) -> bool {
    match cond {
        0b0000 => z,                    // EQ
        0b0001 => !z,                   // NE
        0b0010 => c,                    // CS/HS
        0b0011 => !c,                   // CC/LO
        0b0100 => n,                    // MI
        0b0101 => !n,                   // PL
        0b0110 => v,                    // VS
        0b0111 => !v,                   // VC
        0b1000 => c && !z,              // HI
        0b1001 => !c || z,              // LS
        0b1010 => n == v,               // GE
        0b1011 => n != v,               // LT
        0b1100 => !z && n == v,         // GT
        0b1101 => z || n != v,          // LE
        0b1110 => true,                 // AL
        0b1111 => false,                // NV
        _ => false,
    }
}