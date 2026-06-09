//! Register diff utilities.

use crate::DiffResult;

/// CPU register state for diff comparison.
#[derive(Debug, Clone)]
pub struct RegisterState {
    /// General purpose registers (R0-R15 for ARM, X0-X31 for RISC-V).
    pub gp: [u32; 32],
    /// Program counter.
    pub pc: u32,
    /// Stack pointer.
    pub sp: u32,
    /// Flags/status register.
    pub flags: u32,
}

impl Default for RegisterState {
    fn default() -> Self {
        Self::new()
    }
}

impl RegisterState {
    /// Create a new register state.
    pub fn new() -> Self {
        Self {
            gp: [0; 32],
            pc: 0,
            sp: 0,
            flags: 0,
        }
    }
}

/// Compare register states.
pub fn diff_registers(a: &RegisterState, b: &RegisterState) -> Vec<(usize, DiffResult)> {
    let mut results = Vec::new();
    
    // Compare GP registers
    for (i, (ra, rb)) in a.gp.iter().zip(b.gp.iter()).enumerate() {
        if ra != rb {
            results.push((i, DiffResult::Mismatch {
                expected: *ra as u64,
                actual: *rb as u64,
            }));
        }
    }
    
    // Compare PC
    if a.pc != b.pc {
        results.push((32, DiffResult::Mismatch {
            expected: a.pc as u64,
            actual: b.pc as u64,
        }));
    }
    
    // Compare SP
    if a.sp != b.sp {
        results.push((33, DiffResult::Mismatch {
            expected: a.sp as u64,
            actual: b.sp as u64,
        }));
    }
    
    // Compare flags
    if a.flags != b.flags {
        results.push((34, DiffResult::Mismatch {
            expected: a.flags as u64,
            actual: b.flags as u64,
        }));
    }
    
    results
}