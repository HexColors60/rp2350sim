//! Hazard3 CPU state.

use rp2350sim_core::CoreId;
use rp2350sim_cpu_common::CpuRunState;
use crate::csr::CsrState;

/// Hazard3 CPU core state.
#[derive(Debug, Clone)]
pub struct Hazard3CoreState {
    /// Core ID
    pub id: CoreId,
    /// Run state
    pub run_state: CpuRunState,
    /// General purpose registers x0-x31
    pub x: [u32; 32],
    /// Program counter
    pub pc: u32,
    /// CSR state
    pub csr: CsrState,
    /// Cycle count
    pub cycles: u64,
    /// Instruction count
    pub instructions: u64,
    /// Pending trap flag
    pub pending_trap: bool,
    /// Trap cause
    pub trap_cause: u32,
}

impl Default for Hazard3CoreState {
    fn default() -> Self {
        Self::new(CoreId::CORE0)
    }
}

impl Hazard3CoreState {
    pub fn new(id: CoreId) -> Self {
        Self {
            id,
            run_state: CpuRunState::Halted,
            x: [0; 32],
            pc: 0,
            csr: CsrState::default(),
            cycles: 0,
            instructions: 0,
            pending_trap: false,
            trap_cause: 0,
        }
    }

    /// Get a register value (x0 is always 0).
    pub fn get_reg(&self, reg: usize) -> u32 {
        if reg == 0 {
            0
        } else {
            self.x.get(reg).copied().unwrap_or(0)
        }
    }

    /// Set a register value (x0 is read-only).
    pub fn set_reg(&mut self, reg: usize, value: u32) {
        if reg > 0 && reg < 32 {
            self.x[reg] = value;
        }
    }

    /// Reset the core.
    pub fn reset(&mut self) {
        self.run_state = CpuRunState::Halted;
        self.x.fill(0);
        self.pc = 0;
        self.cycles = 0;
        self.instructions = 0;
        self.pending_trap = false;
        self.trap_cause = 0;
    }
}

/// Full Hazard3 CPU state.
#[derive(Debug, Clone, Default)]
pub struct Hazard3State {
    /// Core states
    pub cores: Vec<Hazard3CoreState>,
    /// Current tick
    pub tick: u64,
}

impl Hazard3State {
    pub fn new(core_count: usize) -> Self {
        let cores = (0..core_count)
            .map(|i| Hazard3CoreState::new(CoreId(i as u8)))
            .collect();
        Self {
            cores,
            tick: 0,
        }
    }

    pub fn core(&self, id: CoreId) -> Option<&Hazard3CoreState> {
        self.cores.get(id.index())
    }

    pub fn core_mut(&mut self, id: CoreId) -> Option<&mut Hazard3CoreState> {
        self.cores.get_mut(id.index())
    }

    pub fn reset(&mut self) {
        for core in &mut self.cores {
            core.reset();
        }
        self.tick = 0;
    }
}