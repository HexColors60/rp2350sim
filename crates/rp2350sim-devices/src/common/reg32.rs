//! 32-bit register abstraction.

use std::ops::{BitAnd, BitOr, Not};

/// 32-bit register.
#[derive(Debug, Clone, Copy, Default)]
pub struct Reg32(pub u32);

impl Reg32 {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn get(&self) -> u32 {
        self.0
    }

    pub fn set(&mut self, value: u32) {
        self.0 = value;
    }

    pub fn read_field(&self, start: u32, len: u32) -> u32 {
        (self.0 >> start) & ((1 << len) - 1)
    }

    pub fn write_field(&mut self, start: u32, len: u32, value: u32) {
        let mask = ((1 << len) - 1) << start;
        self.0 = (self.0 & !mask) | ((value << start) & mask);
    }
}

impl BitAnd for Reg32 {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}

impl BitOr for Reg32 {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl Not for Reg32 {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}