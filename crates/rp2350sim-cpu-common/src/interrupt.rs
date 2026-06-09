//! Interrupt handling.


/// Interrupt state.
#[derive(Debug, Clone, Default)]
pub struct InterruptState {
    /// Pending interrupts
    pending: u32,
    /// Enabled interrupts
    enabled: u32,
    /// Active interrupts
    active: u32,
}

impl InterruptState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set an interrupt pending.
    pub fn set_pending(&mut self, irq: u8) {
        self.pending |= 1 << irq;
    }

    /// Clear a pending interrupt.
    pub fn clear_pending(&mut self, irq: u8) {
        self.pending &= !(1 << irq);
    }

    /// Check if an interrupt is pending.
    pub fn is_pending(&self, irq: u8) -> bool {
        (self.pending & (1 << irq)) != 0
    }

    /// Enable an interrupt.
    pub fn enable(&mut self, irq: u8) {
        self.enabled |= 1 << irq;
    }

    /// Disable an interrupt.
    pub fn disable(&mut self, irq: u8) {
        self.enabled &= !(1 << irq);
    }

    /// Check if an interrupt is enabled.
    pub fn is_enabled(&self, irq: u8) -> bool {
        (self.enabled & (1 << irq)) != 0
    }

    /// Set an interrupt active.
    pub fn set_active(&mut self, irq: u8) {
        self.active |= 1 << irq;
    }

    /// Clear an active interrupt.
    pub fn clear_active(&mut self, irq: u8) {
        self.active &= !(1 << irq);
    }

    /// Check if an interrupt is active.
    pub fn is_active(&self, irq: u8) -> bool {
        (self.active & (1 << irq)) != 0
    }

    /// Get the highest priority pending and enabled interrupt.
    pub fn highest_pending(&self) -> Option<u8> {
        let pending_enabled = self.pending & self.enabled;
        if pending_enabled == 0 {
            None
        } else {
            Some(pending_enabled.trailing_zeros() as u8)
        }
    }

    /// Reset the interrupt state.
    pub fn reset(&mut self) {
        self.pending = 0;
        self.enabled = 0;
        self.active = 0;
    }
}