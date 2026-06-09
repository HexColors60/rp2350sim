//! Clock divider.

/// Clock divider.
#[derive(Debug, Clone, Default)]
pub struct ClockDivider {
    div: u32,
    frac: u8,
}

impl ClockDivider {
    pub fn new() -> Self {
        Self { div: 1, frac: 0 }
    }

    pub fn set_div(&mut self, div: u32) {
        self.div = div.max(1);
    }

    pub fn set_frac(&mut self, frac: u8) {
        self.frac = frac;
    }

    pub fn divide(&self, freq: u64) -> u64 {
        if self.div == 0 {
            return 0;
        }
        freq / self.div as u64
    }
}