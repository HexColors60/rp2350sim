//! DWARF debug information parsing.
//!
//! This module provides DWARF debug info parsing using the gimli and addr2line crates.

use std::path::Path;
use std::sync::Arc;

use addr2line::gimli::Reader;
use addr2line::ObjectContext;
use object::{Object, ObjectSymbol};

/// DWARF debug info loader.
pub struct DwarfDebugInfo {
    /// addr2line context for lookups (owned via Arc)
    context: Option<Arc<ObjectContext>>,
    /// Loaded flag
    loaded: bool,
    /// Function cache
    functions: Vec<FunctionInfo>,
    /// Raw data storage
    data: Option<Arc<Vec<u8>>>,
}

impl DwarfDebugInfo {
    /// Create a new empty DWARF debug info.
    pub fn new() -> Self {
        Self {
            context: None,
            loaded: false,
            functions: Vec::new(),
            data: None,
        }
    }

    /// Load debug info from an ELF file.
    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, DwarfError> {
        let data = std::fs::read(path.as_ref())
            .map_err(|e| DwarfError::IoError(e.to_string()))?;
        self.load_from_bytes(&data)
    }

    /// Load debug info from ELF bytes.
    pub fn load_from_bytes(&mut self, data: &[u8]) -> Result<usize, DwarfError> {
        // Store data
        let data = Arc::new(data.to_vec());
        
        // Parse object file
        let object = object::File::parse(data.as_slice())
            .map_err(|e| DwarfError::ParseError(format!("Failed to parse object: {}", e)))?;

        // Create addr2line context
        let context = ObjectContext::new(&object)
            .map_err(|e| DwarfError::ParseError(format!("Failed to create context: {}", e)))?;

        // Extract functions from symbol table (simpler approach)
        let functions = extract_functions(&object)?;

        self.context = Some(Arc::new(context));
        self.functions = functions;
        self.data = Some(data);
        self.loaded = true;

        Ok(self.functions.len())
    }

    /// Find source location for an address.
    pub fn find_location(&self, addr: u64) -> Option<SourceLocation> {
        let context = self.context.as_ref()?;

        let mut frames = context.find_frames(addr).skip_all_loads().ok()?;
        let frame = frames.next().ok()??;
        let location = frame.location?;
        
        Some(SourceLocation {
            file: location.file?.to_string(),
            line: location.line,
            column: location.column,
        })
    }

    /// Find function containing an address.
    pub fn find_function(&self, addr: u64) -> Option<FunctionInfo> {
        // First try addr2line
        if let Some(context) = &self.context {
            if let Ok(mut frames) = context.find_frames(addr).skip_all_loads() {
                if let Ok(Some(frame)) = frames.next() {
                    if let Some(name) = frame.function {
                        return Some(FunctionInfo {
                            name: name.name.to_string().unwrap_or_default().to_string(),
                            address: addr,
                            size: None,
                        });
                    }
                }
            }
        }

        // Fall back to binary search in cached functions
        let idx = self.functions.partition_point(|f| f.address <= addr);
        if idx == 0 {
            return None;
        }

        let func = &self.functions[idx - 1];
        if let Some(size) = func.size {
            if addr >= func.address + size {
                return None;
            }
        }

        Some(func.clone())
    }

    /// Find source location and function for an address.
    pub fn find_location_and_function(&self, addr: u64) -> Option<(SourceLocation, FunctionInfo)> {
        let loc = self.find_location(addr)?;
        let func = self.find_function(addr)?;
        Some((loc, func))
    }

    /// Check if debug info is loaded.
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Get all functions.
    pub fn functions(&self) -> &[FunctionInfo] {
        &self.functions
    }

    /// Get function count.
    pub fn function_count(&self) -> usize {
        self.functions.len()
    }
}

impl Default for DwarfDebugInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract function information from symbol table.
fn extract_functions<'a>(object: &object::File<'a, &'a [u8]>) -> Result<Vec<FunctionInfo>, DwarfError> {
    let mut functions = Vec::new();

    // Use symbol table for function info (more reliable)
    for symbol in object.symbols() {
        let name = symbol.name().ok();
        let address = symbol.address();
        let size = symbol.size();
        
        // Filter for function symbols
        let kind = symbol.kind();
        if kind == object::SymbolKind::Text {
            if let Some(name_str) = name {
                if !name_str.is_empty() && address > 0 {
                    functions.push(FunctionInfo {
                        name: name_str.to_string(),
                        address,
                        size: if size > 0 { Some(size) } else { None },
                    });
                }
            }
        }
    }

    // Sort by address
    functions.sort_by_key(|f| f.address);

    Ok(functions)
}

/// Source location information.
#[derive(Debug, Clone)]
pub struct SourceLocation {
    /// Source file path
    pub file: String,
    /// Line number (1-based)
    pub line: Option<u32>,
    /// Column number (1-based)
    pub column: Option<u32>,
}

impl SourceLocation {
    /// Format as a string.
    pub fn format(&self) -> String {
        if let (Some(line), Some(col)) = (self.line, self.column) {
            format!("{}:{}:{}", self.file, line, col)
        } else if let Some(line) = self.line {
            format!("{}:{}", self.file, line)
        } else {
            self.file.clone()
        }
    }
}

/// Function information.
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    /// Function name
    pub name: String,
    /// Function start address
    pub address: u64,
    /// Function size (if known)
    pub size: Option<u64>,
}

/// DWARF error type.
#[derive(Debug, Clone)]
pub enum DwarfError {
    /// IO error
    IoError(String),
    /// Parse error
    ParseError(String),
    /// No debug info
    NoDebugInfo,
}

impl std::fmt::Display for DwarfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(msg) => write!(f, "IO error: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::NoDebugInfo => write!(f, "No debug info"),
        }
    }
}

impl std::error::Error for DwarfError {}