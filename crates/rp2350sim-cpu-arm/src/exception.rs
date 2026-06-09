//! ARM exception handling.

use rp2350sim_cpu_common::ExceptionKind;

/// ARM exception state.
#[derive(Debug, Clone, Default)]
pub struct ArmExceptionState {
    /// Active exceptions
    active: Vec<ExceptionKind>,
    /// Pending exceptions
    pending: Vec<ExceptionKind>,
}

impl ArmExceptionState {
    pub fn new() -> Self {
        Self::default()
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

    pub fn reset(&mut self) {
        self.active.clear();
        self.pending.clear();
    }
}