//! PIO diff utilities.

use crate::DiffResult;

/// PIO state machine state.
#[derive(Debug, Clone)]
pub struct PioSmState {
    /// Program counter.
    pub pc: u8,
    /// X register.
    pub x: u32,
    /// Y register.
    pub y: u32,
    /// Input shift register.
    pub isr: u32,
    /// Output shift register.
    pub osr: u32,
}

/// PIO state for diff comparison.
#[derive(Debug, Clone)]
pub struct PioState {
    /// State machine states.
    pub state_machines: [PioSmState; 4],
    /// Instruction memory.
    pub instruction_mem: Vec<u16>,
}

impl PioState {
    /// Create a new PIO state.
    pub fn new() -> Self {
        Self {
            state_machines: std::array::from_fn(|_| PioSmState {
                pc: 0,
                x: 0,
                y: 0,
                isr: 0,
                osr: 0,
            }),
            instruction_mem: vec![0; 32],
        }
    }
}

impl Default for PioState {
    fn default() -> Self {
        Self::new()
    }
}

/// Compare PIO states.
pub fn diff_pio(a: &PioState, b: &PioState) -> Vec<(usize, DiffResult)> {
    let mut results = Vec::new();
    
    for (i, (sm_a, sm_b)) in a.state_machines.iter().zip(b.state_machines.iter()).enumerate() {
        if sm_a.pc != sm_b.pc {
            results.push((i, DiffResult::Mismatch {
                expected: sm_a.pc as u64,
                actual: sm_b.pc as u64,
            }));
        }
    }
    
    results
}