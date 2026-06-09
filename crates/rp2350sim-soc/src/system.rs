//! System controller.

/// System controller.
#[derive(Debug, Default)]
pub struct System {
    pub tick: u64,
}

impl System {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&mut self) {
        self.tick += 1;
    }
}