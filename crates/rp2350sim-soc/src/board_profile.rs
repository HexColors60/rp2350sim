//! Board profile.

use rp2350sim_core::SimulatorConfig;

/// Board profile.
#[derive(Debug, Clone)]
pub struct BoardProfile {
    pub name: String,
    pub config: SimulatorConfig,
}

impl BoardProfile {
    pub fn pico2w() -> Self {
        Self {
            name: "Pico 2 W".to_string(),
            config: SimulatorConfig::default(),
        }
    }
}