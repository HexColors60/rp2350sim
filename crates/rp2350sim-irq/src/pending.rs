//! IRQ pending state.


/// IRQ pending state.
#[derive(Debug, Clone, Default)]
pub struct PendingIrqs {
    pending: u64,
}

impl PendingIrqs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, irq: u8) {
        self.pending |= 1 << irq;
    }

    pub fn clear(&mut self, irq: u8) {
        self.pending &= !(1 << irq);
    }

    pub fn is_set(&self, irq: u8) -> bool {
        (self.pending & (1 << irq)) != 0
    }

    pub fn highest(&self) -> Option<u8> {
        if self.pending == 0 {
            None
        } else {
            Some(self.pending.trailing_zeros() as u8)
        }
    }
}