//! CPU run mode.

use serde::{Deserialize, Serialize};

/// CPU run mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum RunMode {
    /// Thread mode (unprivileged)
    #[default]
    Thread,
    /// Handler mode (exception handler)
    Handler,
}

/// CPU privilege level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum PrivilegeLevel {
    #[default]
    Privileged,
    Unprivileged,
}

/// CPU mode state.
#[derive(Debug, Clone, Copy, Default)]
pub struct CpuMode {
    pub run_mode: RunMode,
    pub privilege: PrivilegeLevel,
}

impl CpuMode {
    pub fn new() -> Self {
        Self {
            run_mode: RunMode::Thread,
            privilege: PrivilegeLevel::Privileged,
        }
    }

    pub fn enter_handler(&mut self) {
        self.run_mode = RunMode::Handler;
        self.privilege = PrivilegeLevel::Privileged;
    }

    pub fn exit_handler(&mut self) {
        self.run_mode = RunMode::Thread;
    }

    pub fn set_privilege(&mut self, level: PrivilegeLevel) {
        self.privilege = level;
    }
}