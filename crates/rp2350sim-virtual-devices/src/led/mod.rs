//! LED device.


/// Virtual LED.
#[derive(Debug, Clone, Default)]
pub struct Led {
    pub state: bool,
    pub color: (u8, u8, u8),
}

impl Led {
    pub fn new() -> Self {
        Self {
            state: false,
            color: (0, 255, 0),
        }
    }

    pub fn set(&mut self, state: bool) {
        self.state = state;
    }

    pub fn toggle(&mut self) {
        self.state = !self.state;
    }
}