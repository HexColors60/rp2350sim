//! ARM CPU state.

use crate::{exception::ArmExceptionState, nvic::Nvic, psr::Psr, systick::SysTick};
use rp2350sim_core::CoreId;
use rp2350sim_cpu_common::{CpuRunState, RunMode};

/// ARM CPU core state.
#[derive(Debug, Clone)]
pub struct ArmCoreState {
    /// Core ID
    pub id: CoreId,
    /// Run state
    pub run_state: CpuRunState,
    /// General purpose registers R0-R12
    pub r: [u32; 13],
    /// Stack pointer (MSP)
    pub msp: u32,
    /// Stack pointer (PSP)
    pub psp: u32,
    /// Link register
    pub lr: u32,
    /// Program counter
    pub pc: u32,
    /// Program status register
    pub xpsr: Psr,
    /// Control register
    pub control: u32,
    /// Primask
    pub primask: u32,
    /// Basepri
    pub basepri: u32,
    /// Faultmask
    pub faultmask: u32,
    /// Current mode
    pub mode: RunMode,
    /// Cycle count
    pub cycles: u64,
    /// Instruction count
    pub instructions: u64,
    /// IT block state (If-Then block)
    /// Bits 7:4 = firstcond, Bits 3:0 = mask
    /// When mask == 0, no IT block is active
    pub it_state: u8,
}

impl Default for ArmCoreState {
    fn default() -> Self {
        Self::new(CoreId::CORE0)
    }
}

impl ArmCoreState {
    pub fn new(id: CoreId) -> Self {
        Self {
            id,
            run_state: CpuRunState::Halted,
            r: [0; 13],
            msp: 0,
            psp: 0,
            lr: 0,
            pc: 0,
            xpsr: Psr::default(),
            control: 0,
            primask: 0,
            basepri: 0,
            faultmask: 0,
            mode: RunMode::Thread,
            cycles: 0,
            instructions: 0,
            it_state: 0,
        }
    }

    /// Get the current stack pointer.
    pub fn sp(&self) -> u32 {
        if self.control & 2 != 0 {
            self.psp
        } else {
            self.msp
        }
    }

    /// Set the current stack pointer.
    pub fn set_sp(&mut self, value: u32) {
        if self.control & 2 != 0 {
            self.psp = value;
        } else {
            self.msp = value;
        }
    }

    /// Check if IT block is active.
    pub fn is_it_block_active(&self) -> bool {
        self.it_state != 0
    }

    /// Get the condition for the current IT block instruction.
    /// Returns (condition, should_execute) where condition is the condition code
    /// and should_execute indicates if the instruction should execute.
    pub fn get_it_condition(&self) -> (u8, bool) {
        let firstcond = (self.it_state >> 4) & 0xF;
        let mask = self.it_state & 0xF;

        // Determine the condition for this instruction
        // The mask bits determine if we use firstcond or its inverse
        // Bit 0 of mask is for the first instruction after IT
        // Bit 1 is for the second, etc.
        let cond = if mask & 1 != 0 {
            firstcond ^ 1 // Invert condition
        } else {
            firstcond
        };

        (cond, true)
    }

    /// Advance IT block state (called after each instruction in the block).
    pub fn advance_it_state(&mut self) {
        let mask = self.it_state & 0xF;
        if mask != 0 {
            // Shift right by 1 to advance to next instruction
            let new_mask = mask >> 1;
            if new_mask == 0 {
                // IT block complete
                self.it_state = 0;
            } else {
                self.it_state = (self.it_state & 0xF0) | new_mask;
            }
        } else {
            self.it_state = 0;
        }
    }

    /// Set IT block state.
    pub fn set_it_state(&mut self, firstcond: u8, mask: u8) {
        self.it_state = (firstcond << 4) | (mask & 0xF);
    }

    /// Clear IT block state.
    pub fn clear_it_state(&mut self) {
        self.it_state = 0;
    }

    /// Get a register value.
    /// Note: Reading PC (R15) returns PC + 4 as per ARM architecture.
    pub fn get_reg(&self, reg: usize) -> u32 {
        match reg {
            0..=12 => self.r[reg],
            13 => self.sp(),
            14 => self.lr,
            15 => self.pc.wrapping_add(4),
            _ => 0,
        }
    }

    /// Set a register value.
    pub fn set_reg(&mut self, reg: usize, value: u32) {
        match reg {
            0..=12 => self.r[reg] = value,
            13 => self.set_sp(value),
            14 => self.lr = value,
            15 => self.pc = value,
            _ => {}
        }
    }

    /// Reset the core.
    pub fn reset(&mut self) {
        self.run_state = CpuRunState::Halted;
        self.r.fill(0);
        self.msp = 0;
        self.psp = 0;
        self.lr = 0;
        self.pc = 0;
        self.xpsr = Psr::default();
        self.control = 0;
        self.primask = 0;
        self.basepri = 0;
        self.faultmask = 0;
        self.mode = RunMode::Thread;
        self.cycles = 0;
        self.instructions = 0;
    }
}

/// Full ARM CPU state.
#[derive(Debug, Clone)]
pub struct ArmState {
    /// Core states
    pub cores: Vec<ArmCoreState>,
    /// NVIC
    pub nvic: Nvic,
    /// SysTick
    pub systick: SysTick,
    /// Exception state
    pub exception: ArmExceptionState,
    /// Current tick
    pub tick: u64,
}

impl Default for ArmState {
    fn default() -> Self {
        Self::new(2)
    }
}

impl ArmState {
    pub fn new(core_count: usize) -> Self {
        let cores = (0..core_count)
            .map(|i| ArmCoreState::new(CoreId(i as u8)))
            .collect();
        Self {
            cores,
            nvic: Nvic::new(),
            systick: SysTick::new(),
            exception: ArmExceptionState::new(),
            tick: 0,
        }
    }

    pub fn core(&self, id: CoreId) -> Option<&ArmCoreState> {
        self.cores.get(id.index())
    }

    pub fn core_mut(&mut self, id: CoreId) -> Option<&mut ArmCoreState> {
        self.cores.get_mut(id.index())
    }

    pub fn reset(&mut self) {
        for core in &mut self.cores {
            core.reset();
        }
        self.nvic.reset();
        self.systick.reset();
        self.exception.reset();
        self.tick = 0;
    }
}
