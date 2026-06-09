//! Trace system.


/// Trace system.
#[derive(Debug, Default)]
pub struct Trace {
    enabled: bool,
}

impl Trace {
    pub fn new() -> Self {
        Self::default()
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