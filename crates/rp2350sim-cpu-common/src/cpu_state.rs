//! CPU state.

use rp2350sim_core::CoreId;

/// CPU run state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CpuRunState {
    #[default]
    Running,
    Halted,
    Sleeping,
    DebugHalted,
}

/// CPU core state.
#[derive(Debug, Clone, Default)]
pub struct CpuCoreState {
    /// Core ID
    pub id: CoreId,
    /// Run state
    pub run_state: CpuRunState,
    /// Program counter
    pub pc: u64,
    /// Stack pointer
    pub sp: u64,
    /// General purpose registers
    pub regs: [u64; 32],
    /// Current cycle count
    pub cycles: u64,
    /// Instruction count
    pub instructions: u64,
}

impl CpuCoreState {
    pub fn new(id: CoreId) -> Self {
        Self {
            id,
            run_state: CpuRunState::Halted,
            pc: 0,
            sp: 0,
            regs: [0; 32],
            cycles: 0,
            instructions: 0,
        }
    }

    pub fn reset(&mut self) {
        self.run_state = CpuRunState::Halted;
        self.pc = 0;
        self.sp = 0;
        self.regs.fill(0);
        self.cycles = 0;
        self.instructions = 0;
    }
}

/// Full CPU state.
#[derive(Debug, Clone, Default)]
pub struct CpuState {
    /// Core states
    pub cores: Vec<CpuCoreState>,
    /// Current tick
    pub tick: u64,
}

impl CpuState {
    pub fn new(core_count: usize) -> Self {
        let cores = (0..core_count)
            .map(|i| CpuCoreState::new(CoreId(i as u8)))
            .collect();
        Self {
            cores,
            tick: 0,
        }
    }

    pub fn reset(&mut self) {
        for core in &mut self.cores {
            core.reset();
        }
        self.tick = 0;
    }

    pub fn core(&self, id: CoreId) -> Option<&CpuCoreState> {
        self.cores.get(id.index())
    }

    pub fn core_mut(&mut self, id: CoreId) -> Option<&mut CpuCoreState> {
        self.cores.get_mut(id.index())
    }
}