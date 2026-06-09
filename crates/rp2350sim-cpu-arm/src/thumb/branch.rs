#![allow(dead_code)]

//! Branch instructions.

/// Calculate branch target.
pub fn calculate_branch_target(pc: u32, offset: i32) -> u32 {
    (pc as i32 + offset) as u32
}

/// Calculate branch target for Thumb BL.
pub fn calculate_bl_target(pc: u32, offset: i32) -> u32 {
    ((pc as i32 + offset) as u32) | 1 // Set Thumb bit
}