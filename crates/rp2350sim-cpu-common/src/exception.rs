//! Exception handling.

use serde::{Deserialize, Serialize};

/// Exception types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExceptionKind {
    Reset,
    Nmi,
    HardFault,
    MemManage,
    BusFault,
    UsageFault,
    SVCall,
    DebugMonitor,
    PendSV,
    SysTick,
    Interrupt(u8),
}

impl ExceptionKind {
    /// Get the exception number.
    pub fn exception_number(&self) -> u8 {
        match self {
            Self::Reset => 1,
            Self::Nmi => 2,
            Self::HardFault => 3,
            Self::MemManage => 4,
            Self::BusFault => 5,
            Self::UsageFault => 6,
            Self::SVCall => 11,
            Self::DebugMonitor => 12,
            Self::PendSV => 14,
            Self::SysTick => 15,
            Self::Interrupt(n) => 16 + n,
        }
    }

    /// Get the priority (lower = higher priority).
    pub fn priority(&self) -> i8 {
        match self {
            Self::Reset => -3,
            Self::Nmi => -2,
            Self::HardFault => -1,
            _ => 0,
        }
    }
}

/// Exception information.
#[derive(Debug, Clone)]
pub struct ExceptionInfo {
    pub kind: ExceptionKind,
    pub pc: u32,
    pub sp: u32,
    pub lr: u32,
    pub xpsr: u32,
}

/// Exception state.
#[derive(Debug, Clone, Default)]
pub struct ExceptionState {
    /// Active exceptions
    active: Vec<ExceptionKind>,
    /// Pending exceptions
    pending: Vec<ExceptionKind>,
}

impl ExceptionState {
    pub fn new() -> Self {
        Self {
            active: Vec::new(),
            pending: Vec::new(),
        }
    }

    pub fn pend(&mut self, kind: ExceptionKind) {
        if !self.pending.contains(&kind) {
            self.pending.push(kind);
        }
    }

    pub fn activate(&mut self, kind: ExceptionKind) {
        self.pending.retain(|k| k != &kind);
        if !self.active.contains(&kind) {
            self.active.push(kind);
        }
    }

    pub fn deactivate(&mut self, kind: ExceptionKind) {
        self.active.retain(|k| k != &kind);
    }

    pub fn highest_pending(&self) -> Option<ExceptionKind> {
        self.pending.iter().min_by_key(|k| k.priority()).copied()
    }

    pub fn is_active(&self, kind: ExceptionKind) -> bool {
        self.active.contains(&kind)
    }

    pub fn is_pending(&self, kind: ExceptionKind) -> bool {
        self.pending.contains(&kind)
    }

    pub fn reset(&mut self) {
        self.active.clear();
        self.pending.clear();
    }
}