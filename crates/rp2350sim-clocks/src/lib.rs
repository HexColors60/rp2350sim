//! RP2350 Clock System

pub mod clocks;
pub mod divider;
pub mod domains;
pub mod freq;
pub mod gating;
pub mod pll;
pub mod reset_tree;

pub use clocks::Clocks;
pub use domains::ClockDomains;