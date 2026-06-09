//! Symbol management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Symbol information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub address: u32,
    pub size: u32,
    pub kind: SymbolKind,
}

/// Symbol kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Object,
    Section,
    File,
    Unknown,
}

/// Symbol table.
#[derive(Debug, Default)]
pub struct SymbolTable {
    symbols: Vec<Symbol>,
    name_index: HashMap<String, usize>,
    addr_index: HashMap<u32, usize>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, symbol: Symbol) {
        let idx = self.symbols.len();
        self.name_index.insert(symbol.name.clone(), idx);
        self.addr_index.insert(symbol.address, idx);
        self.symbols.push(symbol);
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Symbol> {
        self.name_index.get(name).map(|&i| &self.symbols[i])
    }

    pub fn find_by_address(&self, addr: u32) -> Option<&Symbol> {
        // Find the symbol containing this address
        self.symbols.iter().find(|s| {
            addr >= s.address && addr < s.address + s.size
        })
    }

    pub fn find_exact(&self, addr: u32) -> Option<&Symbol> {
        self.addr_index.get(&addr).map(|&i| &self.symbols[i])
    }

    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    pub fn clear(&mut self) {
        self.symbols.clear();
        self.name_index.clear();
        self.addr_index.clear();
    }
}