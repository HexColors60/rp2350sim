//! Dispatch hooks.

use rp2350sim_core::AccessWidth;

/// Dispatch hook for read operations.
pub trait ReadDispatchHook: Send + Sync {
    fn dispatch(&self, addr: u32, width: AccessWidth) -> Option<u64>;
}

/// Dispatch hook for write operations.
pub trait WriteDispatchHook: Send + Sync {
    fn dispatch(&self, addr: u32, width: AccessWidth, value: u64) -> bool;
}

/// Combined dispatch hook.
pub trait DispatchHook: ReadDispatchHook + WriteDispatchHook {}

impl<T: ReadDispatchHook + WriteDispatchHook> DispatchHook for T {}