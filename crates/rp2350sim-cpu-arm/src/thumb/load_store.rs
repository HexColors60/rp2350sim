#![allow(dead_code)]

//! Load/store instructions.

/// Calculate memory address for load/store.
pub fn calculate_address(base: u32, offset: i32) -> u32 {
    (base as i32 + offset) as u32
}