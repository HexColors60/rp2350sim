//! Hazard3 dual-core profile.

use rp2350sim_core::SimulatorConfig;

/// Hazard3 dual-core profile.
pub struct Hazard3DualProfile;

impl Hazard3DualProfile {
    pub fn config() -> SimulatorConfig {
        SimulatorConfig {
            cpu_arch: rp2350sim_core::CpuArch::Hazard3,
            core_count: 2,
            ..Default::default()
        }
    }
}