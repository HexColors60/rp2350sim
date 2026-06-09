//! Button device.


/// Virtual button.
#[derive(Debug, Clone, Default)]
pub struct Button {
    pub pressed: bool,
}

impl Button {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn press(&mut self) {
        self.pressed = true;
    }

    pub fn release(&mut self) {
        self.pressed = false;
    }
}