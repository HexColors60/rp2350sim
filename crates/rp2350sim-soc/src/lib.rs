//! RP2350 SoC

pub mod board;
pub mod board_profile;
pub mod boot_mode;
pub mod fabric;
pub mod firmware_test;
pub mod gdb;
pub mod init;
pub mod memory_layout;
pub mod profiles;
pub mod reset;
pub mod soc;
pub mod system;
pub mod wiring;

pub use soc::Soc;
pub use gdb::SocGdbTarget;
pub use firmware_test::{FirmwareTestHarness, FirmwareBuilder, RunResult};