//! Scripting API.

mod simulator;
mod memory;
mod gpio;

pub use simulator::{SimulatorApi, SimulatorControl};
pub use memory::{MemoryApi, MemoryControl};
pub use gpio::{GpioApi, GpioControl};