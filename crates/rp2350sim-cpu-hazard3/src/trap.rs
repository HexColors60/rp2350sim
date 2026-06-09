//! Trap handling.


/// Trap cause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapCause {
    InstructionMisaligned,
    InstructionAccessFault,
    IllegalInstruction,
    Breakpoint,
    LoadMisaligned,
    LoadAccessFault,
    StoreMisaligned,
    StoreAccessFault,
    UserEcall,
    SupervisorEcall,
    MachineEcall,
    InstructionPageFault,
    LoadPageFault,
    StorePageFault,
    Interrupt(u8),
}

impl TrapCause {
    pub fn code(&self) -> u32 {
        match self {
            Self::InstructionMisaligned => 0,
            Self::InstructionAccessFault => 1,
            Self::IllegalInstruction => 2,
            Self::Breakpoint => 3,
            Self::LoadMisaligned => 4,
            Self::LoadAccessFault => 5,
            Self::StoreMisaligned => 6,
            Self::StoreAccessFault => 7,
            Self::UserEcall => 8,
            Self::SupervisorEcall => 9,
            Self::MachineEcall => 11,
            Self::InstructionPageFault => 12,
            Self::LoadPageFault => 13,
            Self::StorePageFault => 15,
            Self::Interrupt(n) => 0x80000000 | (*n as u32),
        }
    }
}

/// Trap state.
#[derive(Debug, Clone, Default)]
pub struct TrapState {
    pub cause: Option<TrapCause>,
    pub epc: u32,
    pub tval: u32,
}

impl TrapState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn trap(&mut self, cause: TrapCause, epc: u32, tval: u32) {
        self.cause = Some(cause);
        self.epc = epc;
        self.tval = tval;
    }

    pub fn clear(&mut self) {
        self.cause = None;
        self.epc = 0;
        self.tval = 0;
    }
}