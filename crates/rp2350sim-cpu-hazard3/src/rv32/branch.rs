#![allow(dead_code)]

//! Branch instructions.

/// Calculate branch target.
pub fn calculate_branch_target(pc: u32, offset: i32) -> u32 {
    (pc as i32 + offset) as u32
}