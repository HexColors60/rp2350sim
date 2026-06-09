//! ELF file loader.

use goblin::elf::Elf;
use rp2350sim_core::Result;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// ELF loader.
pub struct ElfLoader;

impl ElfLoader {
    /// Load an ELF file from path and return information.
    /// This is a convenience method for the `info` command.
    pub fn load_from_path(path: &Path) -> Result<ElfInfo> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        
        Self::parse(&data)
    }
    
    /// Parse ELF data and return information (without loading into memory).
    pub fn parse(data: &[u8]) -> Result<ElfInfo> {
        let elf = Elf::parse(data).map_err(|e| {
            rp2350sim_core::Error::Serialization(format!("Failed to parse ELF: {}", e))
        })?;

        let mut info = ElfInfo::default();
        info.entry_point = elf.header.e_entry as u32;
        info.architecture = Self::detect_architecture(&elf);
        info.is_little_endian = elf.little_endian;

        // Extract section information
        for shdr in &elf.section_headers {
            let name = elf.shdr_strtab.get_at(shdr.sh_name).unwrap_or("?");
            let section_type = Self::section_type_name(shdr.sh_type);
            
            if shdr.sh_size > 0 && shdr.sh_addr > 0 {
                info.sections.push(SectionInfo {
                    name: name.to_string(),
                    address: shdr.sh_addr as u32,
                    size: shdr.sh_size as usize,
                    section_type: section_type.to_string(),
                    flags: shdr.sh_flags,
                });
            }
        }

        // Extract symbols
        Self::extract_symbols(&elf, data, &mut info.symbols);
        info.symbols.sort_by_key(|s| s.address);
        
        Ok(info)
    }

    /// Load an ELF file into memory buffers and return information.
    /// This is the primary method used by the SoC.
    pub fn load_into_memory<R: Read>(
        reader: &mut R,
        flash: &mut [u8],
        sram: &mut [u8],
    ) -> Result<ElfInfo> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        let elf = Elf::parse(&data).map_err(|e| {
            rp2350sim_core::Error::Serialization(format!("Failed to parse ELF: {}", e))
        })?;

        let mut info = ElfInfo::default();
        info.entry_point = elf.header.e_entry as u32;
        info.architecture = Self::detect_architecture(&elf);
        info.is_little_endian = elf.little_endian;

        // Load program segments
        for phdr in &elf.program_headers {
            if phdr.p_type != goblin::elf::program_header::PT_LOAD {
                continue;
            }

            let vaddr = phdr.p_vaddr as u32;
            let memsz = phdr.p_memsz as usize;
            let filesz = phdr.p_filesz as usize;
            let offset = phdr.p_offset as usize;

            // Determine target memory
            let loaded = if vaddr >= 0x1000_0000 && vaddr < 0x2000_0000 {
                // Flash/XIP region
                let flash_offset = (vaddr - 0x1000_0000) as usize;
                if flash_offset + memsz <= flash.len() {
                    if offset + filesz <= data.len() {
                        flash[flash_offset..flash_offset + filesz]
                            .copy_from_slice(&data[offset..offset + filesz]);
                    }
                    if memsz > filesz {
                        flash[flash_offset + filesz..flash_offset + memsz].fill(0);
                    }
                    true
                } else {
                    false
                }
            } else if vaddr >= 0x2000_0000 && vaddr < 0x3000_0000 {
                // SRAM region
                let sram_offset = (vaddr - 0x2000_0000) as usize;
                if sram_offset + memsz <= sram.len() {
                    if offset + filesz <= data.len() {
                        sram[sram_offset..sram_offset + filesz]
                            .copy_from_slice(&data[offset..offset + filesz]);
                    }
                    if memsz > filesz {
                        sram[sram_offset + filesz..sram_offset + memsz].fill(0);
                    }
                    true
                } else {
                    false
                }
            } else {
                false
            };

            if loaded {
                info.sections.push(SectionInfo {
                    name: format!("LOAD@0x{:08X}", vaddr),
                    address: vaddr,
                    size: memsz,
                    section_type: "LOAD".to_string(),
                    flags: phdr.p_flags as u64,
                });
            }
        }

        // Extract symbols
        Self::extract_symbols(&elf, &data, &mut info.symbols);
        info.symbols.sort_by_key(|s| s.address);
        
        Ok(info)
    }

    /// Load only symbols from an ELF file (without loading code).
    pub fn load_symbols<R: Read>(reader: &mut R) -> Result<Vec<SymbolInfo>> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        let elf = Elf::parse(&data).map_err(|e| {
            rp2350sim_core::Error::Serialization(format!("Failed to parse ELF: {}", e))
        })?;

        let mut symbols = Vec::new();
        Self::extract_symbols(&elf, &data, &mut symbols);
        symbols.sort_by_key(|s| s.address);
        
        Ok(symbols)
    }

    // Private helper methods

    fn detect_architecture(elf: &Elf) -> Architecture {
        if elf.header.e_machine == goblin::elf::header::EM_ARM {
            if elf.is_64 { Architecture::Arm64 } else { Architecture::Arm32 }
        } else if elf.header.e_machine == goblin::elf::header::EM_RISCV {
            if elf.is_64 { Architecture::RiscV64 } else { Architecture::RiscV32 }
        } else {
            Architecture::Unknown
        }
    }

    fn section_type_name(sh_type: u32) -> &'static str {
        match sh_type {
            goblin::elf::section_header::SHT_NULL => "NULL",
            goblin::elf::section_header::SHT_PROGBITS => "PROGBITS",
            goblin::elf::section_header::SHT_SYMTAB => "SYMTAB",
            goblin::elf::section_header::SHT_STRTAB => "STRTAB",
            goblin::elf::section_header::SHT_RELA => "RELA",
            goblin::elf::section_header::SHT_HASH => "HASH",
            goblin::elf::section_header::SHT_DYNAMIC => "DYNAMIC",
            goblin::elf::section_header::SHT_NOTE => "NOTE",
            goblin::elf::section_header::SHT_NOBITS => "NOBITS",
            goblin::elf::section_header::SHT_REL => "REL",
            goblin::elf::section_header::SHT_DYNSYM => "DYNSYM",
            0x70000003 => "ARM_ATTRIBUTES",
            _ => "OTHER",
        }
    }

    fn extract_symbols(elf: &Elf, data: &[u8], symbols: &mut Vec<SymbolInfo>) {
        for shdr in &elf.section_headers {
            if shdr.sh_type == goblin::elf::section_header::SHT_SYMTAB 
                || shdr.sh_type == goblin::elf::section_header::SHT_DYNSYM 
            {
                let strtab = &elf.shdr_strtab;
                let sym_offset = shdr.sh_offset as usize;
                let sym_size = shdr.sh_size as usize;
                
                if sym_offset + sym_size > data.len() {
                    continue;
                }
                
                let sym_data = &data[sym_offset..sym_offset + sym_size];
                let sym_entry_size = if elf.is_64 { 24 } else { 16 };
                let num_syms = sym_size / sym_entry_size;
                
                for i in 0..num_syms {
                    let entry_offset = i * sym_entry_size;
                    if entry_offset + sym_entry_size > sym_data.len() {
                        break;
                    }
                    
                    let (st_name, st_value, st_size, st_info) = if elf.is_64 {
                        let st_name = u32::from_le_bytes([
                            sym_data[entry_offset], sym_data[entry_offset+1],
                            sym_data[entry_offset+2], sym_data[entry_offset+3]
                        ]);
                        let st_info = sym_data[entry_offset + 4];
                        let st_value = u64::from_le_bytes([
                            sym_data[entry_offset+8], sym_data[entry_offset+9],
                            sym_data[entry_offset+10], sym_data[entry_offset+11],
                            sym_data[entry_offset+12], sym_data[entry_offset+13],
                            sym_data[entry_offset+14], sym_data[entry_offset+15]
                        ]);
                        let st_size = u64::from_le_bytes([
                            sym_data[entry_offset+16], sym_data[entry_offset+17],
                            sym_data[entry_offset+18], sym_data[entry_offset+19],
                            sym_data[entry_offset+20], sym_data[entry_offset+21],
                            sym_data[entry_offset+22], sym_data[entry_offset+23]
                        ]);
                        (st_name, st_value, st_size, st_info)
                    } else {
                        let st_name = u32::from_le_bytes([
                            sym_data[entry_offset], sym_data[entry_offset+1],
                            sym_data[entry_offset+2], sym_data[entry_offset+3]
                        ]);
                        let st_value = u32::from_le_bytes([
                            sym_data[entry_offset+4], sym_data[entry_offset+5],
                            sym_data[entry_offset+6], sym_data[entry_offset+7]
                        ]);
                        let st_size = u32::from_le_bytes([
                            sym_data[entry_offset+8], sym_data[entry_offset+9],
                            sym_data[entry_offset+10], sym_data[entry_offset+11]
                        ]);
                        let st_info = sym_data[entry_offset + 12];
                        (st_name, st_value as u64, st_size as u64, st_info)
                    };
                    
                    if st_value == 0 {
                        continue;
                    }
                    
                    let name = strtab.get_at(st_name as usize).unwrap_or("?");
                    if name.is_empty() {
                        continue;
                    }
                    
                    let kind = match st_info & 0xf {
                        goblin::elf::sym::STT_FUNC => SymbolKind::Function,
                        goblin::elf::sym::STT_OBJECT => SymbolKind::Variable,
                        goblin::elf::sym::STT_SECTION => SymbolKind::Section,
                        goblin::elf::sym::STT_FILE => SymbolKind::File,
                        _ => SymbolKind::Other,
                    };
                    
                    let binding = match st_info >> 4 {
                        goblin::elf::sym::STB_LOCAL => SymbolBinding::Local,
                        goblin::elf::sym::STB_GLOBAL => SymbolBinding::Global,
                        goblin::elf::sym::STB_WEAK => SymbolBinding::Weak,
                        _ => SymbolBinding::Other,
                    };
                    
                    symbols.push(SymbolInfo {
                        name: name.to_string(),
                        address: st_value as u32,
                        size: st_size as u32,
                        kind,
                        binding,
                    });
                }
                break; // Only process first symbol table
            }
        }
    }
}

// Keep the old API for backward compatibility
impl ElfLoader {
    /// Load an ELF file into memory (old API for compatibility).
    /// 
    /// Deprecated: Use `load_into_memory` instead.
    pub fn load<R: Read>(reader: &mut R, flash: &mut [u8], sram: &mut [u8]) -> Result<ElfInfo> {
        Self::load_into_memory(reader, flash, sram)
    }
}

/// Architecture type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Architecture {
    #[default]
    Unknown,
    Arm32,
    Arm64,
    RiscV32,
    RiscV64,
}

impl std::fmt::Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Architecture::Arm32 => write!(f, "ARM (32-bit)"),
            Architecture::Arm64 => write!(f, "ARM (64-bit)"),
            Architecture::RiscV32 => write!(f, "RISC-V (32-bit)"),
            Architecture::RiscV64 => write!(f, "RISC-V (64-bit)"),
            Architecture::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Section information.
#[derive(Debug, Clone)]
pub struct SectionInfo {
    pub name: String,
    pub address: u32,
    pub size: usize,
    pub section_type: String,
    pub flags: u64,
}

/// Symbol kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Variable,
    Section,
    File,
    Other,
}

impl std::fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolKind::Function => write!(f, "FUNC"),
            SymbolKind::Variable => write!(f, "OBJ"),
            SymbolKind::Section => write!(f, "SECT"),
            SymbolKind::File => write!(f, "FILE"),
            SymbolKind::Other => write!(f, "OTHER"),
        }
    }
}

/// Symbol binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolBinding {
    Local,
    Global,
    Weak,
    Other,
}

/// Symbol information.
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub address: u32,
    pub size: u32,
    pub kind: SymbolKind,
    pub binding: SymbolBinding,
}

/// ELF file information.
#[derive(Debug, Clone, Default)]
pub struct ElfInfo {
    pub entry_point: u32,
    pub architecture: Architecture,
    pub is_little_endian: bool,
    pub sections: Vec<SectionInfo>,
    pub symbols: Vec<SymbolInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Create a minimal valid ARM ELF32 file header
    fn create_minimal_elf() -> Vec<u8> {
        let mut elf = vec![0u8; 256]; // 256 bytes to accommodate code at offset 128+
        
        // ELF magic
        elf[0..4].copy_from_slice(b"\x7fELF");
        
        // ELF32, little-endian, ARM
        elf[4] = 1;  // 32-bit
        elf[5] = 1;  // Little-endian
        elf[6] = 1;  // ELF version 1
        elf[7] = 0;  // OS/ABI
        
        // e_type = ET_EXEC (2)
        elf[16..18].copy_from_slice(&2u16.to_le_bytes());
        
        // e_machine = EM_ARM (40)
        elf[18..20].copy_from_slice(&40u16.to_le_bytes());
        
        // e_version = 1
        elf[20..24].copy_from_slice(&1u32.to_le_bytes());
        
        // e_entry = 0x10000100 (entry point)
        elf[24..28].copy_from_slice(&0x10000100u32.to_le_bytes());
        
        // e_phoff = 0x34 (program header offset, right after ELF header)
        elf[28..32].copy_from_slice(&0x34u32.to_le_bytes());
        
        // e_shoff = 0 (no section headers for minimal)
        elf[32..36].copy_from_slice(&0u32.to_le_bytes());
        
        // e_flags = 0x5000000 (ARM EABI)
        elf[36..40].copy_from_slice(&0x05000000u32.to_le_bytes());
        
        // e_ehsize = 52 (ELF header size)
        elf[40..42].copy_from_slice(&52u16.to_le_bytes());
        
        // e_phentsize = 32 (program header entry size)
        elf[42..44].copy_from_slice(&32u16.to_le_bytes());
        
        // e_phnum = 1 (one program header)
        elf[44..46].copy_from_slice(&1u16.to_le_bytes());
        
        // e_shentsize = 0
        elf[46..48].copy_from_slice(&0u16.to_le_bytes());
        
        // e_shnum = 0
        elf[48..50].copy_from_slice(&0u16.to_le_bytes());
        
        // e_shstrndx = 0
        elf[50..52].copy_from_slice(&0u16.to_le_bytes());
        
        // Program header at offset 0x34 (52)
        let ph_offset = 52;
        
        // p_type = PT_LOAD (1)
        elf[ph_offset..ph_offset+4].copy_from_slice(&1u32.to_le_bytes());
        
        // p_offset = 0x80 (data starts at offset 128)
        elf[ph_offset+4..ph_offset+8].copy_from_slice(&0x80u32.to_le_bytes());
        
        // p_vaddr = 0x10000100
        elf[ph_offset+8..ph_offset+12].copy_from_slice(&0x10000100u32.to_le_bytes());
        
        // p_paddr = 0x10000100
        elf[ph_offset+12..ph_offset+16].copy_from_slice(&0x10000100u32.to_le_bytes());
        
        // p_filesz = 4 (4 bytes of code)
        elf[ph_offset+16..ph_offset+20].copy_from_slice(&4u32.to_le_bytes());
        
        // p_memsz = 4
        elf[ph_offset+20..ph_offset+24].copy_from_slice(&4u32.to_le_bytes());
        
        // p_flags = 5 (R-X)
        elf[ph_offset+24..ph_offset+28].copy_from_slice(&5u32.to_le_bytes());
        
        // p_align = 4
        elf[ph_offset+28..ph_offset+32].copy_from_slice(&4u32.to_le_bytes());
        
        // Code at offset 0x80 (128): NOP (MOV R0, R0)
        elf[128..132].copy_from_slice(&[0x00, 0x00, 0x00, 0xE1]);
        
        elf
    }

    #[test]
    fn test_elf_parse_minimal() {
        let elf_data = create_minimal_elf();
        let result = ElfLoader::parse(&elf_data);
        
        assert!(result.is_ok());
        let info = result.unwrap();
        
        assert_eq!(info.entry_point, 0x10000100);
        assert_eq!(info.architecture, Architecture::Arm32);
        assert!(info.is_little_endian);
    }

    #[test]
    fn test_elf_parse_invalid_magic() {
        let mut elf_data = create_minimal_elf();
        elf_data[0..4].copy_from_slice(b"XELF"); // Invalid magic
        
        let result = ElfLoader::parse(&elf_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_elf_parse_empty() {
        let elf_data: Vec<u8> = Vec::new();
        let result = ElfLoader::parse(&elf_data);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_elf_parse_too_short() {
        let elf_data = vec![0x7f, b'E', b'L', b'F']; // Only magic
        let result = ElfLoader::parse(&elf_data);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_elf_load_into_memory_flash() {
        let elf_data = create_minimal_elf();
        let mut cursor = Cursor::new(&elf_data);
        
        let mut flash = vec![0u8; 16 * 1024 * 1024]; // 16MB flash
        let mut sram = vec![0u8; 512 * 1024]; // 512KB SRAM
        
        let result = ElfLoader::load_into_memory(&mut cursor, &mut flash, &mut sram);
        
        assert!(result.is_ok());
        let info = result.unwrap();
        
        assert_eq!(info.entry_point, 0x10000100);
        
        // Check that code was loaded at flash offset 0x100
        // Entry point 0x10000100 maps to flash offset 0x100
        assert_eq!(flash[0x100], 0x00);
        assert_eq!(flash[0x101], 0x00);
        assert_eq!(flash[0x102], 0x00);
        assert_eq!(flash[0x103], 0xE1);
    }

    #[test]
    fn test_elf_load_into_memory_sram() {
        // Create ELF that loads to SRAM (0x20000000)
        let mut elf = create_minimal_elf();
        
        // Change p_vaddr to SRAM address
        let ph_offset = 52;
        elf[ph_offset+8..ph_offset+12].copy_from_slice(&0x20000000u32.to_le_bytes());
        elf[ph_offset+12..ph_offset+16].copy_from_slice(&0x20000000u32.to_le_bytes());
        
        // Update entry point
        elf[24..28].copy_from_slice(&0x20000000u32.to_le_bytes());
        
        let mut cursor = Cursor::new(&elf);
        
        let mut flash = vec![0u8; 16 * 1024 * 1024];
        let mut sram = vec![0u8; 512 * 1024];
        
        let result = ElfLoader::load_into_memory(&mut cursor, &mut flash, &mut sram);
        
        assert!(result.is_ok());
        let info = result.unwrap();
        
        assert_eq!(info.entry_point, 0x20000000);
        
        // Check that code was loaded at SRAM offset 0
        assert_eq!(sram[0], 0x00);
        assert_eq!(sram[1], 0x00);
        assert_eq!(sram[2], 0x00);
        assert_eq!(sram[3], 0xE1);
    }

    #[test]
    fn test_elf_load_symbols() {
        let elf_data = create_minimal_elf();
        let mut cursor = Cursor::new(&elf_data);
        
        let symbols = ElfLoader::load_symbols(&mut cursor);
        
        // Minimal ELF without symbol table should return empty vec
        assert!(symbols.is_ok());
    }

    #[test]
    fn test_architecture_display() {
        assert_eq!(format!("{}", Architecture::Arm32), "ARM (32-bit)");
        assert_eq!(format!("{}", Architecture::Arm64), "ARM (64-bit)");
        assert_eq!(format!("{}", Architecture::RiscV32), "RISC-V (32-bit)");
        assert_eq!(format!("{}", Architecture::RiscV64), "RISC-V (64-bit)");
        assert_eq!(format!("{}", Architecture::Unknown), "Unknown");
    }

    #[test]
    fn test_symbol_kind_display() {
        assert_eq!(format!("{}", SymbolKind::Function), "FUNC");
        assert_eq!(format!("{}", SymbolKind::Variable), "OBJ");
        assert_eq!(format!("{}", SymbolKind::Section), "SECT");
        assert_eq!(format!("{}", SymbolKind::File), "FILE");
        assert_eq!(format!("{}", SymbolKind::Other), "OTHER");
    }

    #[test]
    fn test_elf_detect_arm32() {
        let mut elf_data = create_minimal_elf();
        // EM_ARM = 40
        elf_data[18..20].copy_from_slice(&40u16.to_le_bytes());
        elf_data[4] = 1; // 32-bit
        
        let info = ElfLoader::parse(&elf_data).unwrap();
        assert_eq!(info.architecture, Architecture::Arm32);
    }

    #[test]
    fn test_elf_detect_riscv32() {
        let mut elf_data = create_minimal_elf();
        // EM_RISCV = 243
        elf_data[18..20].copy_from_slice(&243u16.to_le_bytes());
        elf_data[4] = 1; // 32-bit
        
        let info = ElfLoader::parse(&elf_data).unwrap();
        assert_eq!(info.architecture, Architecture::RiscV32);
    }

    #[test]
    fn test_elf_section_info() {
        let elf_data = create_minimal_elf();
        let info = ElfLoader::parse(&elf_data).unwrap();
        
        // Minimal ELF may or may not have sections
        // Just verify entry point and architecture
        assert_eq!(info.entry_point, 0x10000100);
        assert_eq!(info.architecture, Architecture::Arm32);
    }
}
