//! Firmware management.

use std::path::PathBuf;

/// Firmware configuration.
#[derive(Debug, Clone)]
pub struct Firmware {
    /// Firmware name.
    pub name: String,
    /// Path to firmware file.
    pub path: PathBuf,
    /// Firmware format.
    pub format: FirmwareFormat,
}

/// Firmware format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirmwareFormat {
    /// ELF format.
    Elf,
    /// Intel HEX format.
    Hex,
    /// Raw binary.
    Bin,
}

impl Firmware {
    /// Create a new firmware configuration.
    pub fn new(name: &str, path: PathBuf) -> Self {
        let format = Self::detect_format(&path);
        Self {
            name: name.to_string(),
            path,
            format,
        }
    }

    /// Detect firmware format from path.
    fn detect_format(path: &PathBuf) -> FirmwareFormat {
        match path.extension().and_then(|e| e.to_str()) {
            Some("elf") => FirmwareFormat::Elf,
            Some("hex") => FirmwareFormat::Hex,
            Some("bin") => FirmwareFormat::Bin,
            _ => FirmwareFormat::Bin,
        }
    }
}