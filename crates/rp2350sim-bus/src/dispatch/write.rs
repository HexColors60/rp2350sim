//! Write dispatch.

use rp2350sim_core::AccessWidth;

/// Write dispatcher.
pub struct WriteDispatcher {
    hooks: Vec<Box<dyn super::WriteDispatchHook>>,
}

impl Default for WriteDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl WriteDispatcher {
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    pub fn add_hook<H: super::WriteDispatchHook + 'static>(&mut self, hook: H) {
        self.hooks.push(Box::new(hook));
    }

    pub fn dispatch(&self, addr: u32, width: AccessWidth, value: u64) -> bool {
        for hook in &self.hooks {
            if hook.dispatch(addr, width, value) {
                return true;
            }
        }
        false
    }
}