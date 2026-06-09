//! ELF symbol loading.

use std::collections::HashMap;

/// ELF symbol.
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol name.
    pub name: String,
    /// Symbol address.
    pub address: u32,
    /// Symbol size.
    pub size: u32,
    /// Symbol type.
    pub kind: SymbolKind,
}

/// Symbol type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// Function.
    Function,
    /// Variable.
    Variable,
    /// Section.
    Section,
    /// File (source file symbol).
    File,
    /// Other.
    Other,
}

/// Symbol table loaded from ELF.
#[derive(Debug, Default)]
pub struct SymbolTable {
    /// Symbols by address.
    pub symbols: HashMap<u32, Symbol>,
    /// Symbols by name.
    by_name: HashMap<String, u32>,
    /// Sorted addresses for range lookups.
    sorted_addresses: Vec<u32>,
    /// Functions only (sorted by address).
    functions: Vec<Symbol>,
}

impl SymbolTable {
    /// Create a new symbol table.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a symbol.
    pub fn add(&mut self, symbol: Symbol) {
        self.by_name.insert(symbol.name.clone(), symbol.address);
        self.symbols.insert(symbol.address, symbol);
        self.sorted_addresses.clear(); // Will be rebuilt on next lookup
        self.functions.clear();
    }

    /// Load symbols from a list of symbol info.
    pub fn load_from_info(&mut self, symbols: &[rp2350sim_mem::loader::SymbolInfo]) {
        for sym in symbols {
            let kind = match sym.kind {
                rp2350sim_mem::loader::SymbolKind::Function => SymbolKind::Function,
                rp2350sim_mem::loader::SymbolKind::Variable => SymbolKind::Variable,
                rp2350sim_mem::loader::SymbolKind::Section => SymbolKind::Section,
                rp2350sim_mem::loader::SymbolKind::File => SymbolKind::File,
                rp2350sim_mem::loader::SymbolKind::Other => SymbolKind::Other,
            };
            
            self.add(Symbol {
                name: sym.name.clone(),
                address: sym.address,
                size: sym.size,
                kind,
            });
        }
        
        // Rebuild sorted structures
        self.rebuild_sorted();
    }

    /// Rebuild sorted structures after adding symbols.
    fn rebuild_sorted(&mut self) {
        // Sort addresses
        self.sorted_addresses = self.symbols.keys().copied().collect();
        self.sorted_addresses.sort();

        // Build sorted function list
        self.functions = self.symbols
            .values()
            .filter(|s| s.kind == SymbolKind::Function)
            .cloned()
            .collect();
        self.functions.sort_by_key(|s| s.address);
    }

    /// Look up a symbol by address.
    pub fn by_address(&self, addr: u32) -> Option<&Symbol> {
        self.symbols.get(&addr)
    }

    /// Look up a symbol by name.
    pub fn by_name(&self, name: &str) -> Option<&Symbol> {
        self.by_name.get(name).and_then(|addr| self.symbols.get(addr))
    }

    /// Find the function containing an address.
    /// Returns the symbol and the offset within it.
    pub fn find_function(&self, addr: u32) -> Option<(&Symbol, u32)> {
        // Binary search in sorted functions
        let idx = self.functions.partition_point(|s| s.address <= addr);
        
        if idx == 0 {
            return None;
        }
        
        let func = &self.functions[idx - 1];
        let offset = addr - func.address;
        
        // Check if address is within function bounds
        if func.size > 0 && offset >= func.size {
            return None;
        }
        
        Some((func, offset))
    }

    /// Find the nearest symbol before or at an address.
    pub fn find_nearest(&self, addr: u32) -> Option<&Symbol> {
        let idx = self.sorted_addresses.partition_point(|&a| a <= addr);
        
        if idx == 0 {
            return None;
        }
        
        let nearest_addr = self.sorted_addresses[idx - 1];
        self.symbols.get(&nearest_addr)
    }

    /// Get all functions.
    pub fn functions(&self) -> &[Symbol] {
        &self.functions
    }

    /// Get all symbols.
    pub fn all_symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.values()
    }

    /// Get symbol count.
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Clear all symbols.
    pub fn clear(&mut self) {
        self.symbols.clear();
        self.by_name.clear();
        self.sorted_addresses.clear();
        self.functions.clear();
    }
}