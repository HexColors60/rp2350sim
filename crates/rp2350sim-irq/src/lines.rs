//! IRQ lines.

use std::collections::HashMap;

/// IRQ line state.
#[derive(Debug, Clone, Default)]
pub struct IrqLines {
    lines: HashMap<u8, bool>,
}

impl IrqLines {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, line: u8, level: bool) {
        self.lines.insert(line, level);
    }

    pub fn get(&self, line: u8) -> bool {
        self.lines.get(&line).copied().unwrap_or(false)
    }

    pub fn active_lines(&self) -> Vec<u8> {
        self.lines.iter().filter(|(_, &v)| v).map(|(&k, _)| k).collect()
    }
}