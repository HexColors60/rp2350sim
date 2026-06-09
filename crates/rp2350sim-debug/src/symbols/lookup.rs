//! Symbol lookup.

use super::elf_symbols::SymbolTable;

/// Symbol lookup helper.
pub struct SymbolLookup {
    /// Symbol table.
    table: SymbolTable,
}

impl Default for SymbolLookup {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolLookup {
    /// Create a new symbol lookup.
    pub fn new() -> Self {
        Self {
            table: SymbolTable::new(),
        }
    }

    /// Get the symbol table.
    pub fn table(&self) -> &SymbolTable {
        &self.table
    }

    /// Get a mutable reference to the symbol table.
    pub fn table_mut(&mut self) -> &mut SymbolTable {
        &mut self.table
    }

    /// Find the symbol containing an address.
    pub fn find_containing(&self, addr: u32) -> Option<(String, u32)> {
        // Find the symbol that contains this address
        for (sym_addr, symbol) in self.table.symbols.iter() {
            if addr >= *sym_addr && addr < sym_addr + symbol.size {
                return Some((symbol.name.clone(), *sym_addr));
            }
        }
        None
    }
}