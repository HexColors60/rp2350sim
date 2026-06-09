//! Clock gating.

/// Clock gate state.
#[derive(Debug, Clone, Default)]
pub struct ClockGate {
    enabled: bool,
}

impl ClockGate {
    pub fn new() -> Self {
        Self { enabled: true }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}