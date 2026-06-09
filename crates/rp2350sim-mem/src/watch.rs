//! Memory watchpoints.

use serde::{Deserialize, Serialize};

/// Watchpoint kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WatchKind {
    /// Watch for reads
    Read,
    /// Watch for writes
    Write,
    /// Watch for both reads and writes
    Access,
}

/// Memory watchpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watchpoint {
    /// Watchpoint address.
    pub addr: u64,
    /// Size in bytes.
    pub size: usize,
    /// Watch kind.
    pub kind: WatchKind,
    /// Whether the watchpoint is enabled.
    pub enabled: bool,
    /// Optional condition expression.
    pub condition: Option<String>,
}

impl Watchpoint {
    pub fn new(addr: u64, size: usize, kind: WatchKind) -> Self {
        Self {
            addr,
            size,
            kind,
            enabled: true,
            condition: None,
        }
    }

    pub fn read(addr: u64, size: usize) -> Self {
        Self::new(addr, size, WatchKind::Read)
    }

    pub fn write(addr: u64, size: usize) -> Self {
        Self::new(addr, size, WatchKind::Write)
    }

    pub fn access(addr: u64, size: usize) -> Self {
        Self::new(addr, size, WatchKind::Access)
    }

    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        self.condition = Some(condition.into());
        self
    }

    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub fn matches(&self, addr: u64, is_write: bool) -> bool {
        if !self.enabled {
            return false;
        }
        if addr < self.addr || addr >= self.addr + self.size as u64 {
            return false;
        }
        match self.kind {
            WatchKind::Read => !is_write,
            WatchKind::Write => is_write,
            WatchKind::Access => true,
        }
    }
}