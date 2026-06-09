//! CPU profiles.

mod arm_dual;
mod hazard3_dual;
mod test_minimal;

pub use arm_dual::ArmDualProfile;
pub use hazard3_dual::Hazard3DualProfile;
pub use test_minimal::TestMinimalProfile;