//! IRQ handling.


/// IRQ state.
#[derive(Debug, Clone, Default)]
pub struct IrqState {
    /// Pending IRQs
    pending: u32,
    /// Enabled IRQs
    enabled: u32,
}

impl IrqState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_pending(&mut self, irq: u8) {
        self.pending |= 1 << irq;
    }

    pub fn clear_pending(&mut self, irq: u8) {
        self.pending &= !(1 << irq);
    }

    pub fn is_pending(&self, irq: u8) -> bool {
        (self.pending & (1 << irq)) != 0
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

    pub fn highest_pending(&self) -> Option<u8> {
        let pending_enabled = self.pending & self.enabled;
        if pending_enabled == 0 {
            None
        } else {
            Some(pending_enabled.trailing_zeros() as u8)
        }
    }

    pub fn reset(&mut self) {
        self.pending = 0;
        self.enabled = 0;
    }
}