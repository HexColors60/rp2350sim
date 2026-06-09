//! Wiring for external connections.

use rp2350sim_core::PinId;

/// Wire connection.
#[derive(Debug, Clone)]
pub struct Wire {
    pub from: PinId,
    pub to: PinId,
}

/// Wiring manager.
#[derive(Debug, Default)]
pub struct Wiring {
    wires: Vec<Wire>,
}

impl Wiring {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn connect(&mut self, from: PinId, to: PinId) {
        self.wires.push(Wire { from, to });
    }
}