//! IRQ routing.

use rp2350sim_core::CoreId;

/// IRQ routing configuration.
#[derive(Debug, Clone, Default)]
pub struct IrqRouting {
    routes: Vec<CoreId>,
}

impl IrqRouting {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn route(&mut self, irq: u8, core: CoreId) {
        if irq as usize >= self.routes.len() {
            self.routes.resize(irq as usize + 1, CoreId::CORE0);
        }
        self.routes[irq as usize] = core;
    }

    pub fn get_route(&self, irq: u8) -> CoreId {
        self.routes.get(irq as usize).copied().unwrap_or(CoreId::CORE0)
    }
}