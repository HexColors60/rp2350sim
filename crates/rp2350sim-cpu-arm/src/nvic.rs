//! NVIC (Nested Vectored Interrupt Controller).


/// NVIC state.
#[derive(Debug, Clone)]
pub struct Nvic {
    /// Interrupt set-enable registers
    iser: [u32; 8],
    /// Interrupt clear-enable registers
    icer: [u32; 8],
    /// Interrupt set-pending registers
    ispr: [u32; 8],
    /// Interrupt clear-pending registers
    icpr: [u32; 8],
    /// Interrupt active bit registers
    iabr: [u32; 8],
    /// Interrupt priority registers
    ipr: [u8; 240],
    /// IRQ lines
    irq_lines: [bool; 240],
}

impl Default for Nvic {
    fn default() -> Self {
        Self {
            iser: [0; 8],
            icer: [0; 8],
            ispr: [0; 8],
            icpr: [0; 8],
            iabr: [0; 8],
            ipr: [0u8; 240],
            irq_lines: [false; 240],
        }
    }
}

impl Nvic {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set an IRQ line.
    pub fn set_irq(&mut self, line: usize, level: bool) {
        if line < 240 {
            self.irq_lines[line] = level;
            if level {
                self.set_pending(line);
            }
        }
    }

    /// Set an interrupt pending.
    pub fn set_pending(&mut self, irq: usize) {
        if irq >= 240 {
            return;
        }
        let reg = irq / 32;
        let bit = irq % 32;
        self.ispr[reg] |= 1 << bit;
    }

    /// Clear a pending interrupt.
    pub fn clear_pending(&mut self, irq: usize) {
        if irq >= 240 {
            return;
        }
        let reg = irq / 32;
        let bit = irq % 32;
        self.icpr[reg] |= 1 << bit;
        self.ispr[reg] &= !(1 << bit);
    }

    /// Check if an interrupt is pending.
    pub fn is_pending(&self, irq: usize) -> bool {
        if irq >= 240 {
            return false;
        }
        let reg = irq / 32;
        let bit = irq % 32;
        (self.ispr[reg] & (1 << bit)) != 0
    }

    /// Enable an interrupt.
    pub fn enable(&mut self, irq: usize) {
        if irq >= 240 {
            return;
        }
        let reg = irq / 32;
        let bit = irq % 32;
        self.iser[reg] |= 1 << bit;
    }

    /// Disable an interrupt.
    pub fn disable(&mut self, irq: usize) {
        if irq >= 240 {
            return;
        }
        let reg = irq / 32;
        let bit = irq % 32;
        self.icer[reg] |= 1 << bit;
        self.iser[reg] &= !(1 << bit);
    }

    /// Check if an interrupt is enabled.
    pub fn is_enabled(&self, irq: usize) -> bool {
        if irq >= 240 {
            return false;
        }
        let reg = irq / 32;
        let bit = irq % 32;
        (self.iser[reg] & (1 << bit)) != 0
    }

    /// Get interrupt priority.
    pub fn get_priority(&self, irq: usize) -> u8 {
        if irq < 240 {
            self.ipr[irq]
        } else {
            0
        }
    }

    /// Set interrupt priority.
    pub fn set_priority(&mut self, irq: usize, priority: u8) {
        if irq < 240 {
            self.ipr[irq] = priority;
        }
    }

    /// Get the highest priority pending and enabled interrupt.
    pub fn highest_pending(&self) -> Option<usize> {
        let mut best: Option<(usize, u8)> = None;
        for reg in 0..8 {
            let pending = self.ispr[reg] & self.iser[reg];
            if pending == 0 {
                continue;
            }
            for bit in 0..32 {
                if (pending & (1 << bit)) != 0 {
                    let irq = reg * 32 + bit;
                    let priority = self.get_priority(irq);
                    if best.map_or(true, |(_, p)| priority < p) {
                        best = Some((irq, priority));
                    }
                }
            }
        }
        best.map(|(irq, _)| irq)
    }

    /// Reset the NVIC.
    pub fn reset(&mut self) {
        self.iser.fill(0);
        self.icer.fill(0);
        self.ispr.fill(0);
        self.icpr.fill(0);
        self.iabr.fill(0);
        self.ipr.fill(0);
        self.irq_lines.fill(false);
    }
}
