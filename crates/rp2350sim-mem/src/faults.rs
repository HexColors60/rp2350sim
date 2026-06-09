//! Memory faults.

use rp2350sim_core::AccessType;

/// Memory fault information.
#[derive(Debug, Clone)]
pub struct MemoryFault {
    pub addr: u64,
    pub access_type: AccessType,
    pub reason: FaultReason,
}

/// Reason for a memory fault.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultReason {
    /// Address not mapped
    Unmapped,
    /// Permission denied
    PermissionDenied,
    /// Misaligned access
    Misaligned,
    /// Bus error
    BusError,
    /// Watchpoint hit
    Watchpoint,
    /// Debugger halt
    DebuggerHalt,
}