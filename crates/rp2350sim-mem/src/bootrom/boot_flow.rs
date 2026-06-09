//! Boot flow implementation.

use rp2350sim_core::consts::{SRAM_BASE, XIP_BASE};

/// Boot mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootMode {
    /// Boot from flash/XIP
    Flash,
    /// Boot from SRAM
    Sram,
    /// Boot from USB
    Usb,
}

impl Default for BootMode {
    fn default() -> Self {
        Self::Flash
    }
}

/// Boot flow controller.
#[derive(Debug)]
pub struct BootFlow {
    mode: BootMode,
    boot_complete: bool,
}

impl Default for BootFlow {
    fn default() -> Self {
        Self::new()
    }
}

impl BootFlow {
    pub fn new() -> Self {
        Self {
            mode: BootMode::Flash,
            boot_complete: false,
        }
    }

    /// Set the boot mode.
    pub fn set_mode(&mut self, mode: BootMode) {
        self.mode = mode;
    }

    /// Get the boot mode.
    pub fn mode(&self) -> BootMode {
        self.mode
    }

    /// Get the initial PC for the given boot mode.
    pub fn initial_pc(&self) -> u32 {
        match self.mode {
            BootMode::Flash => {
                // Read the reset vector from the vector table at XIP_BASE
                // In a real implementation, this would read from flash
                XIP_BASE
            }
            BootMode::Sram => SRAM_BASE,
            BootMode::Usb => {
                // USB boot would load to SRAM and start there
                SRAM_BASE
            }
        }
    }

    /// Get the initial SP for the given boot mode.
    pub fn initial_sp(&self) -> u32 {
        match self.mode {
            BootMode::Flash => {
                // Read the initial SP from the vector table
                // In a real implementation, this would read from flash
                SRAM_BASE + 0x80000 // Top of SRAM
            }
            BootMode::Sram | BootMode::Usb => SRAM_BASE + 0x80000,
        }
    }

    /// Mark boot as complete.
    pub fn complete(&mut self) {
        self.boot_complete = true;
    }

    /// Check if boot is complete.
    pub fn is_complete(&self) -> bool {
        self.boot_complete
    }

    /// Reset the boot flow.
    pub fn reset(&mut self) {
        self.boot_complete = false;
    }
}