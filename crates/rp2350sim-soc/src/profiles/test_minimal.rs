//! Minimal test profile.

use rp2350sim_core::SimulatorConfig;

/// Minimal test profile.
pub struct TestMinimalProfile;

impl TestMinimalProfile {
    pub fn config() -> SimulatorConfig {
        SimulatorConfig {
            cpu_arch: rp2350sim_core::CpuArch::Arm,
            core_count: 1,
            trace_enable: true,
            ..Default::default()
        }
    }
}