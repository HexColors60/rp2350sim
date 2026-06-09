//! Symbol management.

mod dwarf;
mod elf_symbols;
mod lookup;
mod map;

pub use dwarf::{DwarfDebugInfo, DwarfError, SourceLocation, FunctionInfo};
pub use elf_symbols::{Symbol, SymbolKind, SymbolTable};
pub use lookup::SymbolLookup;
pub use map::SymbolMap;