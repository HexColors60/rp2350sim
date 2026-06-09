//! CPU backend trait.

use rp2350sim_core::{CoreId, CpuStepResult, Result};

/// CPU backend trait.
pub trait CpuBackend: Send + Sync {
    /// Reset the CPU.
    fn reset(&mut self);

    /// Execute a single step.
    fn step(&mut self) -> Result<CpuStepResult>;

    /// Run for a number of cycles.
    fn run_for_cycles(&mut self, cycles: u64) -> Result<CpuStepResult>;

    /// Set an interrupt line.
    fn set_irq(&mut self, line: usize, level: bool);

    /// Read a register value.
    fn read_reg(&self, core: CoreId, reg: usize) -> u64;

    /// Write a register value.
    fn write_reg(&mut self, core: CoreId, reg: usize, value: u64);

    /// Get the program counter.
    fn pc(&self, core: CoreId) -> u64;

    /// Set the program counter.
    fn set_pc(&mut self, core: CoreId, value: u64);

    /// Check if the CPU is halted.
    fn is_halted(&self, core: CoreId) -> bool;

    /// Halt the CPU.
    fn halt(&mut self, core: CoreId);

    /// Resume the CPU.
    fn resume(&mut self, core: CoreId);

    /// Get the number of cores.
    fn core_count(&self) -> usize;
}