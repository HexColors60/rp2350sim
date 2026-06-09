//! Program Status Register.

use serde::{Deserialize, Serialize};

/// Program Status Register.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Psr(pub u32);

impl Psr {
    /// Condition code flags
    pub const N: u32 = 1 << 31; // Negative
    pub const Z: u32 = 1 << 30; // Zero
    pub const C: u32 = 1 << 29; // Carry
    pub const V: u32 = 1 << 28; // Overflow
    pub const Q: u32 = 1 << 27; // Saturation
    pub const GE: u32 = 0xF << 16; // Greater than or Equal
    pub const IT: u32 = 0xFF << 8; // If-Then
    pub const T: u32 = 1 << 24; // Thumb state
    pub const EXCEPTION: u32 = 0x1FF; // Exception number

    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn get(&self) -> u32 {
        self.0
    }

    pub fn bits(&self) -> u32 {
        self.0
    }

    pub fn set(&mut self, value: u32) {
        self.0 = value;
    }

    pub fn n(&self) -> bool {
        (self.0 & Self::N) != 0
    }

    pub fn set_n(&mut self, value: bool) {
        if value {
            self.0 |= Self::N;
        } else {
            self.0 &= !Self::N;
        }
    }

    pub fn z(&self) -> bool {
        (self.0 & Self::Z) != 0
    }

    pub fn set_z(&mut self, value: bool) {
        if value {
            self.0 |= Self::Z;
        } else {
            self.0 &= !Self::Z;
        }
    }

    pub fn c(&self) -> bool {
        (self.0 & Self::C) != 0
    }

    pub fn set_c(&mut self, value: bool) {
        if value {
            self.0 |= Self::C;
        } else {
            self.0 &= !Self::C;
        }
    }

    pub fn v(&self) -> bool {
        (self.0 & Self::V) != 0
    }

    pub fn set_v(&mut self, value: bool) {
        if value {
            self.0 |= Self::V;
        } else {
            self.0 &= !Self::V;
        }
    }

    pub fn t(&self) -> bool {
        (self.0 & Self::T) != 0
    }

    pub fn set_t(&mut self, value: bool) {
        if value {
            self.0 |= Self::T;
        } else {
            self.0 &= !Self::T;
        }
    }

    pub fn exception(&self) -> u8 {
        (self.0 & Self::EXCEPTION) as u8
    }

    pub fn set_exception(&mut self, value: u8) {
        self.0 = (self.0 & !Self::EXCEPTION) | (value as u32);
    }
}