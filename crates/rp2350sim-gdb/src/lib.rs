//! GDB Remote Serial Protocol stub for RP2350 simulator.
//!
//! This module implements the GDB RSP (Remote Serial Protocol) allowing
//! GDB to connect to the simulator for debugging.

pub mod protocol;
pub mod server;
pub mod stub;
pub mod target;

pub use protocol::{GdbCommand, GdbResponse, GdbError};
pub use stub::GdbStub;
pub use target::GdbTarget;
pub use server::{GdbServer, GdbServerBuilder, GdbServerConfig};