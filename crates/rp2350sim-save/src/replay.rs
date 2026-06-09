#![allow(dead_code)]

//! Replay system.


/// Replay controller.
#[derive(Debug, Default)]
pub struct Replay {
    events: Vec<u8>,
    position: usize,
}

impl Replay {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(&mut self, data: Vec<u8>) {
        self.events = data;
        self.position = 0;
    }
}