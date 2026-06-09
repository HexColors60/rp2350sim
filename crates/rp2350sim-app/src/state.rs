#![allow(dead_code)]

//! Application state.

use rp2350sim_core::{RunState, SimulatorConfig};

/// Application state.
#[derive(Debug, Clone)]
pub struct AppState {
    pub run_state: RunState,
    pub tick: u64,
    pub config: SimulatorConfig,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            run_state: RunState::Halted,
            tick: 0,
            config: SimulatorConfig::default(),
        }
    }
}