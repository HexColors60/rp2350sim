//! ARM core implementation.

use crate::state::ArmCoreState;
use rp2350sim_core::CoreId;

/// ARM Cortex-M33 core.
pub struct ArmCore {
    state: ArmCoreState,
}

impl ArmCore {
    pub fn new(id: CoreId) -> Self {
        Self {
            state: ArmCoreState::new(id),
        }
    }

    pub fn state(&self) -> &ArmCoreState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut ArmCoreState {
        &mut self.state
    }

    pub fn reset(&mut self) {
        self.state.reset();
    }
}