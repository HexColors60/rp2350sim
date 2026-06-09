//! ARM registers.

/// Register names.
pub const REG_NAMES: [&str; 16] = [
    "r0", "r1", "r2", "r3", "r4", "r5", "r6", "r7",
    "r8", "r9", "r10", "r11", "r12", "sp", "lr", "pc",
];

/// Get register name.
pub const fn reg_name(reg: usize) -> &'static str {
    if reg < 16 {
        REG_NAMES[reg]
    } else {
        "unknown"
    }
}

/// Special register indices.
pub const SP: usize = 13;
pub const LR: usize = 14;
pub const PC: usize = 15;