//! Reset controller.

/// Reset controller.
#[derive(Debug, Default)]
pub struct Reset {
    reset_pending: bool,
}

impl Reset {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn request_reset(&mut self) {
        self.reset_pending = true;
    }

    pub fn is_reset_pending(&self) -> bool {
        self.reset_pending
    }

    pub fn clear_reset(&mut self) {
        self.reset_pending = false;
    }
}