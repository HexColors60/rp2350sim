//! Vector table.


/// Vector table.
#[derive(Debug, Clone)]
pub struct VectorTable {
    base: u32,
}

impl Default for VectorTable {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorTable {
    pub fn new() -> Self {
        Self { base: 0 }
    }

    /// Set the vector table base address.
    pub fn set_base(&mut self, base: u32) {
        self.base = base & 0xFFFF_FF80;
    }

    /// Get the vector table base address.
    pub fn base(&self) -> u32 {
        self.base
    }

    /// Get the address of a vector.
    pub fn vector_address(&self, exception: u8) -> u32 {
        self.base + (exception as u32) * 4
    }

    /// Get the initial SP (vector 0).
    pub fn initial_sp_address(&self) -> u32 {
        self.base
    }

    /// Get the reset vector address (vector 1).
    pub fn reset_vector_address(&self) -> u32 {
        self.base + 4
    }
}