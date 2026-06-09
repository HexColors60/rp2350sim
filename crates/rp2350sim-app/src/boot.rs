#![allow(dead_code)]

//! Boot sequence.

use rp2350sim_core::{Result, CpuArch};
use rp2350sim_soc::Soc;
use rp2350sim_mem::loader::ElfLoader;
use std::path::Path;

/// Boot configuration.
#[derive(Debug, Clone)]
pub struct BootConfig {
    /// CPU architecture.
    pub cpu_arch: CpuArch,
    /// Firmware path (optional).
    pub firmware: Option<String>,
    /// Boot from flash.
    pub boot_from_flash: bool,
    /// Initial PC (optional).
    pub initial_pc: Option<u32>,
    /// Initial SP (optional).
    pub initial_sp: Option<u32>,
}

impl Default for BootConfig {
    fn default() -> Self {
        Self {
            cpu_arch: CpuArch::Arm,
            firmware: None,
            boot_from_flash: true,
            initial_pc: None,
            initial_sp: None,
        }
    }
}

/// Boot the simulator.
pub fn boot(config: &BootConfig) -> Result<Soc> {
    tracing::info!("Booting RP2350 simulator with {:?} architecture", config.cpu_arch);

    // Create SoC
    let mut soc = Soc::new(config.cpu_arch);

    // Initialize memory
    initialize_memory(&mut soc)?;

    // Load firmware if provided
    if let Some(ref firmware_path) = config.firmware {
        load_firmware(&mut soc, firmware_path)?;
    }

    // Set initial PC and SP
    if let Some(pc) = config.initial_pc {
        soc.write_reg(15, pc); // PC is register 15
    }

    if let Some(sp) = config.initial_sp {
        soc.write_reg(13, sp); // SP is register 13
    }

    // Initialize peripherals
    initialize_peripherals(&mut soc)?;

    // Reset the SoC
    soc.reset();

    tracing::info!("Boot complete");
    Ok(soc)
}

/// Initialize memory regions.
fn initialize_memory(_soc: &mut Soc) -> Result<()> {
    // Initialize SRAM
    // SRAM is already initialized in Soc::new()

    // Initialize Boot ROM
    // Boot ROM would be loaded from a file or embedded data
    // For now, we'll use a minimal boot ROM

    tracing::debug!("Memory initialized");
    Ok(())
}

/// Load firmware from file.
fn load_firmware(soc: &mut Soc, path: &str) -> Result<()> {
    let path = Path::new(path);

    if !path.exists() {
        return Err(rp2350sim_core::Error::FileNotFound(path.display().to_string()));
    }

    // Determine file type by extension
    let extension = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "elf" => load_elf(soc, path),
        "bin" => load_bin(soc, path),
        "uf2" => load_uf2(soc, path),
        "hex" => load_hex(soc, path),
        _ => {
            tracing::warn!("Unknown firmware format: {}, trying ELF", extension);
            load_elf(soc, path)
        }
    }
}

/// Load ELF firmware.
fn load_elf(soc: &mut Soc, path: &Path) -> Result<()> {
    tracing::info!("Loading ELF firmware from {}", path.display());

    let data = std::fs::read(path)
        .map_err(|e| rp2350sim_core::Error::IoError(e.to_string()))?;

    // Use ElfLoader to parse and load
    let mut flash = vec![0u8; 16 * 1024 * 1024];
    let mut sram = vec![0u8; 520 * 1024];

    let mut cursor = std::io::Cursor::new(&data);
    let info = ElfLoader::load(&mut cursor, &mut flash, &mut sram)?;

    // Load flash into SoC
    soc.load_firmware(&flash)?;

    // Load SRAM sections
    for section in &info.sections {
        if section.address >= 0x20000000 {
            let offset = (section.address - 0x20000000) as usize;
            if offset + section.size <= sram.len() {
                soc.write_memory(section.address, &sram[offset..offset + section.size]);
            }
        }
    }

    // Load symbols into the symbol table
    if !info.symbols.is_empty() {
        soc.symbols.load_from_info(&info.symbols);
        tracing::info!("Loaded {} symbols", info.symbols.len());
    }

    // Set entry point
    soc.write_reg(15, info.entry_point);
    tracing::info!("Entry point: 0x{:08X}", info.entry_point);

    Ok(())
}

/// Load binary firmware.
fn load_bin(soc: &mut Soc, path: &Path) -> Result<()> {
    tracing::info!("Loading binary firmware from {}", path.display());

    let data = std::fs::read(path)
        .map_err(|e| rp2350sim_core::Error::IoError(e.to_string()))?;

    // Load at flash base address
    let flash_base = 0x10000000u32;
    soc.load_firmware_at(flash_base, &data)?;

    // Set PC to flash base
    soc.write_reg(15, flash_base);

    tracing::info!("Loaded {} bytes at 0x{:08X}", data.len(), flash_base);
    Ok(())
}

/// Load UF2 firmware.
fn load_uf2(soc: &mut Soc, path: &Path) -> Result<()> {
    tracing::info!("Loading UF2 firmware from {}", path.display());

    let data = std::fs::read(path)
        .map_err(|e| rp2350sim_core::Error::IoError(e.to_string()))?;

    // Parse UF2 format
    // UF2 has 512-byte blocks with a specific header
    const UF2_MAGIC_START0: u32 = 0x0A324655; // "UF2\n"
    const UF2_MAGIC_START1: u32 = 0x9E5D5157;
    const UF2_BLOCK_SIZE: usize = 512;
    const UF2_PAYLOAD_SIZE: usize = 256;

    let mut total_loaded = 0;

    for chunk in data.chunks(UF2_BLOCK_SIZE) {
        if chunk.len() < 32 {
            continue;
        }

        // Check magic numbers
        let magic0 = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let magic1 = u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);

        if magic0 != UF2_MAGIC_START0 || magic1 != UF2_MAGIC_START1 {
            continue;
        }

        // Get target address
        let addr = u32::from_le_bytes([chunk[12], chunk[13], chunk[14], chunk[15]]);
        let size = u32::from_le_bytes([chunk[16], chunk[17], chunk[18], chunk[19]]) as usize;

        // Load payload
        let payload = &chunk[32..32 + size.min(UF2_PAYLOAD_SIZE)];
        let _ = soc.load_firmware_at(addr, payload);

        total_loaded += payload.len();
    }

    // Set PC to flash base
    soc.write_reg(15, 0x10000000);

    tracing::info!("Loaded {} bytes from UF2", total_loaded);
    Ok(())
}

/// Load Intel HEX firmware.
fn load_hex(soc: &mut Soc, path: &Path) -> Result<()> {
    tracing::info!("Loading HEX firmware from {}", path.display());

    let data = std::fs::read_to_string(path)
        .map_err(|e| rp2350sim_core::Error::IoError(e.to_string()))?;

    let mut base_addr = 0u32;
    let mut total_loaded = 0;

    for line in data.lines() {
        if !line.starts_with(':') {
            continue;
        }

        let line = &line[1..]; // Skip ':'
        if line.len() < 10 {
            continue;
        }

        // Parse record
        let byte_count = u8::from_str_radix(&line[0..2], 16).unwrap_or(0) as usize;
        let addr = u16::from_str_radix(&line[2..6], 16).unwrap_or(0) as u32;
        let record_type = u8::from_str_radix(&line[6..8], 16).unwrap_or(0);

        match record_type {
            0x00 => {
                // Data record
                let full_addr = base_addr + addr;
                let mut payload = Vec::with_capacity(byte_count);
                for i in 0..byte_count {
                    let offset = 8 + i * 2;
                    if offset + 2 > line.len() {
                        break;
                    }
                    let byte = u8::from_str_radix(&line[offset..offset + 2], 16).unwrap_or(0);
                    payload.push(byte);
                }
                let _ = soc.load_firmware_at(full_addr, &payload);
                total_loaded += payload.len();
            }
            0x01 => {
                // End of file
                break;
            }
            0x02 => {
                // Extended segment address
                if byte_count == 2 && line.len() >= 12 {
                    base_addr = u16::from_str_radix(&line[8..12], 16).unwrap_or(0) as u32 * 16;
                }
            }
            0x04 => {
                // Extended linear address
                if byte_count == 2 && line.len() >= 12 {
                    base_addr = (u16::from_str_radix(&line[8..12], 16).unwrap_or(0) as u32) << 16;
                }
            }
            _ => {}
        }
    }

    // Set PC to flash base
    soc.write_reg(15, 0x10000000);

    tracing::info!("Loaded {} bytes from HEX", total_loaded);
    Ok(())
}

/// Initialize peripherals to default state.
fn initialize_peripherals(_soc: &mut Soc) -> Result<()> {
    // Initialize GPIO
    // GPIO pins start in input mode with pull-ups disabled

    // Initialize UARTs
    // UARTs start disabled

    // Initialize SPI
    // SPI starts disabled

    // Initialize I2C
    // I2C starts disabled

    // Initialize Timer
    // Timer starts at 0

    // Initialize PIO
    // PIO state machines start disabled

    tracing::debug!("Peripherals initialized");
    Ok(())
}