//! IRQ enable state.

/// IRQ enable state.
#[derive(Debug, Clone, Default)]
pub struct EnableState {
    enabled: u64,
}

impl EnableState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enable(&mut self, irq: u8) {
        self.enabled |= 1 << irq;
    }

    pub fn disable(&mut self, irq: u8) {
        self.enabled &= !(1 << irq);
    }

    pub fn is_enabled(&self, irq: u8) -> bool {
        (self.enabled & (1 << irq)) != 0
    }
}