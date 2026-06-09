//! Read dispatch.

use rp2350sim_core::AccessWidth;

/// Read dispatcher.
pub struct ReadDispatcher {
    hooks: Vec<Box<dyn super::ReadDispatchHook>>,
}

impl Default for ReadDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadDispatcher {
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    pub fn add_hook<H: super::ReadDispatchHook + 'static>(&mut self, hook: H) {
        self.hooks.push(Box::new(hook));
    }

    pub fn dispatch(&self, addr: u32, width: AccessWidth) -> Option<u64> {
        for hook in &self.hooks {
            if let Some(value) = hook.dispatch(addr, width) {
                return Some(value);
            }
        }
        None
    }
}