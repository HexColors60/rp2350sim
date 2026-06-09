//! ARM dual-core profile.

use rp2350sim_core::SimulatorConfig;

/// ARM dual-core profile.
pub struct ArmDualProfile;

impl ArmDualProfile {
    pub fn config() -> SimulatorConfig {
        SimulatorConfig {
            cpu_arch: rp2350sim_core::CpuArch::Arm,
            core_count: 2,
            ..Default::default()
        }
    }
}