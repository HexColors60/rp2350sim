//! IRQ injection.

/// IRQ injector.
#[derive(Debug, Default)]
pub struct IrqInjector {
    pending: Vec<u8>,
}

impl IrqInjector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn inject(&mut self, irq: u8) {
        if !self.pending.contains(&irq) {
            self.pending.push(irq);
        }
    }

    pub fn take(&mut self) -> Option<u8> {
        self.pending.pop()
    }

    pub fn clear(&mut self) {
        self.pending.clear();
    }
}