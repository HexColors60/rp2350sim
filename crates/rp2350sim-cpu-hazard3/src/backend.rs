//! Hazard3 RISC-V backend.

use crate::state::Hazard3State;
use crate::rv32::decode;
use crate::rv32::execute::{execute, MemoryAccess};
use rp2350sim_core::{CoreId, CpuStepResult, Result};
use rp2350sim_cpu_common::CpuBackend;

/// Hazard3 RISC-V CPU backend.
pub struct Hazard3Backend {
    state: Hazard3State,
}

impl Default for Hazard3Backend {
    fn default() -> Self {
        Self::new()
    }
}

impl Hazard3Backend {
    pub fn new() -> Self {
        Self {
            state: Hazard3State::new(2),
        }
    }

    pub fn state(&self) -> &Hazard3State {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut Hazard3State {
        &mut self.state
    }

    /// Get PC (core 0).
    pub fn pc(&self) -> u32 {
        self.state.core(CoreId::new(0)).map(|c| c.pc).unwrap_or(0)
    }

    /// Set PC (core 0).
    pub fn set_pc(&mut self, value: u32) {
        if let Some(c) = self.state.core_mut(CoreId::new(0)) {
            c.pc = value;
        }
    }

    /// Read register (core 0).
    pub fn read_reg(&self, reg: usize) -> u32 {
        self.state.core(CoreId::new(0)).map(|c| c.get_reg(reg)).unwrap_or(0)
    }

    /// Write register (core 0).
    pub fn write_reg(&mut self, reg: usize, value: u32) {
        if let Some(c) = self.state.core_mut(CoreId::new(0)) {
            c.set_reg(reg, value);
        }
    }

    /// Get flags (mstatus).
    pub fn flags(&self) -> u32 {
        self.state.core(CoreId::new(0)).map(|c| c.csr.mstatus).unwrap_or(0)
    }

    /// Execute a single step with memory access.
    pub fn step_with_memory(&mut self, memory: &mut impl MemoryAccess) -> Result<CpuStepResult> {
        let mut result = CpuStepResult::default();

        // Get core 0
        let core = match self.state.core_mut(CoreId::new(0)) {
            Some(c) => c,
            None => return Ok(result),
        };

        // Auto-resume if halted (for testing convenience)
        if core.run_state == rp2350sim_cpu_common::CpuRunState::Halted {
            core.run_state = rp2350sim_cpu_common::CpuRunState::Running;
        }

        // Fetch instruction at PC
        // First, check if it's a compressed instruction (lowest 2 bits != 11)
        let pc = core.pc;
        let first_half = memory.read_half(pc)?;

        let (opcode, _length) = if first_half & 0x3 != 0x3 {
            // Compressed instruction (16-bit)
            (first_half as u32, 2)
        } else {
            // 32-bit instruction
            let second_half = memory.read_half(pc + 2)?;
            ((first_half as u32) | ((second_half as u32) << 16), 4)
        };

        // Decode instruction
        let instr = decode(opcode);

        // Execute instruction
        let cycles = execute(core, &instr, memory)?;

        result.cycles = cycles as u64;

        Ok(result)
    }
}

impl CpuBackend for Hazard3Backend {
    fn reset(&mut self) {
        self.state.reset();
    }

    fn step(&mut self) -> Result<CpuStepResult> {
        let mut result = CpuStepResult::default();
        result.cycles = 1;
        Ok(result)
    }

    fn run_for_cycles(&mut self, cycles: u64) -> Result<CpuStepResult> {
        let mut result = CpuStepResult::default();
        result.cycles = cycles;
        Ok(result)
    }

    fn set_irq(&mut self, line: usize, level: bool) {
        if let Some(core) = self.state.core_mut(CoreId::new(0)) {
            if level {
                core.csr.mip |= 1 << (line as u32);
            } else {
                core.csr.mip &= !(1 << (line as u32));
            }
        }
    }

    fn read_reg(&self, core: CoreId, reg: usize) -> u64 {
        self.state.core(core).map(|c| c.get_reg(reg) as u64).unwrap_or(0)
    }

    fn write_reg(&mut self, core: CoreId, reg: usize, value: u64) {
        if let Some(c) = self.state.core_mut(core) {
            c.set_reg(reg, value as u32);
        }
    }

    fn pc(&self, core: CoreId) -> u64 {
        self.state.core(core).map(|c| c.pc as u64).unwrap_or(0)
    }

    fn set_pc(&mut self, core: CoreId, value: u64) {
        if let Some(c) = self.state.core_mut(core) {
            c.pc = value as u32;
        }
    }

    fn is_halted(&self, core: CoreId) -> bool {
        self.state.core(core).map(|c| c.run_state == rp2350sim_cpu_common::CpuRunState::Halted).unwrap_or(true)
    }

    fn halt(&mut self, core: CoreId) {
        if let Some(c) = self.state.core_mut(core) {
            c.run_state = rp2350sim_cpu_common::CpuRunState::Halted;
        }
    }

    fn resume(&mut self, core: CoreId) {
        if let Some(c) = self.state.core_mut(core) {
            c.run_state = rp2350sim_cpu_common::CpuRunState::Running;
        }
    }

    fn core_count(&self) -> usize {
        self.state.cores.len()
    }
}