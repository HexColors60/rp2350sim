//! Board representation.

use crate::soc::Soc;

/// Board representation.
pub struct Board {
    pub name: String,
    pub soc: Soc,
}

impl Board {
    pub fn new_pico2w() -> Self {
        Self {
            name: "Pico 2 W".to_string(),
            soc: Soc::new(rp2350sim_core::CpuArch::Arm),
        }
    }
}