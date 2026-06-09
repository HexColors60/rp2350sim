//! Hazard3 core implementation.

use crate::state::Hazard3CoreState;
use rp2350sim_core::CoreId;

/// Hazard3 RISC-V core.
pub struct Hazard3Core {
    state: Hazard3CoreState,
}

impl Hazard3Core {
    pub fn new(id: CoreId) -> Self {
        Self {
            state: Hazard3CoreState::new(id),
        }
    }

    pub fn state(&self) -> &Hazard3CoreState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut Hazard3CoreState {
        &mut self.state
    }

    pub fn reset(&mut self) {
        self.state.reset();
    }
}