//! Symbol map.

use std::collections::HashMap;

/// Symbol map.
#[derive(Debug, Default)]
pub struct SymbolMap {
    symbols: HashMap<u32, String>,
}

impl SymbolMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, addr: u32, name: String) {
        self.symbols.insert(addr, name);
    }

    pub fn find(&self, addr: u32) -> Option<&String> {
        self.symbols.get(&addr)
    }
}