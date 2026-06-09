#![allow(dead_code)]

//! Load state deserialization.

use rp2350sim_core::Result;
use rp2350sim_soc::Soc;
use super::save::{SaveState, SAVE_MAGIC, SAVE_VERSION};

/// Load state from bytes.
pub fn load_from_bytes(data: &[u8]) -> Result<SaveState> {
    if data.len() < 8 {
        return Err(rp2350sim_core::Error::InvalidSaveFormat("Data too short".to_string()));
    }
    
    // Check magic number
    if &data[0..4] != SAVE_MAGIC {
        return Err(rp2350sim_core::Error::InvalidSaveFormat("Invalid magic number".to_string()));
    }
    
    // Check version
    let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    if version > SAVE_VERSION {
        return Err(rp2350sim_core::Error::InvalidSaveFormat(format!(
            "Unsupported save version: {} (max: {})",
            version, SAVE_VERSION
        )));
    }
    
    // Deserialize the state
    let state: SaveState = bincode::deserialize(&data[8..])
        .map_err(|e| rp2350sim_core::Error::SerializationError(e.to_string()))?;
    
    Ok(state)
}

/// Load state from file.
pub fn load_from_file(path: &std::path::Path) -> Result<SaveState> {
    let data = std::fs::read(path)
        .map_err(|e| rp2350sim_core::Error::IoError(e.to_string()))?;
    load_from_bytes(&data)
}

/// Load state into SoC.
pub fn load_state(soc: &mut Soc, data: &[u8]) -> Result<()> {
    let state = load_from_bytes(data)?;
    state.apply_to_soc(soc);
    Ok(())
}

impl SaveState {
    /// Apply save state to SoC.
    pub fn apply_to_soc(&self, soc: &mut Soc) {
        // Apply CPU state
        self.cpu.apply_to_soc(soc);
        
        // Apply memory state
        self.memory.apply_to_soc(soc);
        
        // Apply device states
        self.devices.apply_to_soc(soc);
        
        // Apply clock state
        self.clocks.apply_to_soc(soc);
    }
}

impl super::save::CpuSaveState {
    fn apply_to_soc(&self, soc: &mut Soc) {
        // Apply CPU core state
        if let Some(core) = self.cores.first() {
            // Set PC
            soc.set_pc(core.pc);
            // Set SP (MSP)
            soc.set_sp(core.msp);
            // Set general purpose registers
            for i in 0..16 {
                if i != 13 && i != 15 { // Skip SP and PC (handled separately)
                    soc.write_reg(i, core.regs[i]);
                }
            }
        }
    }
}

impl super::save::MemorySaveState {
    fn apply_to_soc(&self, soc: &mut Soc) {
        // Apply SRAM contents
        if !self.sram.is_empty() {
            let sram_data = &self.sram;
            let sram_len = sram_data.len().min(soc.sram.total_size());
            if sram_len > 0 {
                soc.sram.write(0, &sram_data[..sram_len]);
            }
        }
        
        // Apply Flash contents if present
        if let Some(ref flash_data) = self.flash {
            let flash_len = flash_data.len().min(soc.flash.size());
            if flash_len > 0 {
                let flash = soc.flash.data_mut();
                flash[..flash_len].copy_from_slice(&flash_data[..flash_len]);
            }
        }
    }
}

impl super::save::DeviceSaveState {
    fn apply_to_soc(&self, soc: &mut Soc) {
        // Apply GPIO state
        self.gpio.apply_to_soc(soc);
        
        // Apply UART states
        for uart_state in &self.uarts {
            uart_state.apply_to_soc(soc);
        }
        
        // Apply Timer state
        self.timer.apply_to_soc(soc);
        
        // Apply PIO state
        self.pio.apply_to_soc(soc);
    }
}

impl super::save::GpioSaveState {
    fn apply_to_soc(&self, soc: &mut Soc) {
        for pin_state in &self.pins {
            let pin = pin_state.pin as usize;
            if pin < 48 {
                // Set input value (for external input simulation)
                soc.set_gpio_input(pin, pin_state.input);
            }
        }
    }
}

impl super::save::UartSaveState {
    fn apply_to_soc(&self, soc: &mut Soc) {
        // UART state restoration would require additional API
        // For now, we just note which UARTs were in use
        let _uart = match self.index {
            0 => &soc.uart0,
            1 => &soc.uart1,
            _ => return,
        };
        // UART restore would need write_register() API
    }
}

impl super::save::TimerSaveState {
    fn apply_to_soc(&self, soc: &mut Soc) {
        // Set the timer value via memory-mapped writes
        // Timer TIMEHR register at 0x400B0028
        let _ = soc.timer;
        // Timer restoration would require additional API
    }
}

impl super::save::PioSaveState {
    fn apply_to_soc(&self, soc: &mut Soc) {
        // PIO state restoration would require additional API
        let _pio = match self.index {
            0 => &soc.pio0,
            1 => &soc.pio1,
            _ => return,
        };
    }
}

impl super::save::ClockSaveState {
    fn apply_to_soc(&self, _soc: &mut Soc) {
        // Clock state restoration would require additional API
        // The clocks struct is public but may not have setter methods
    }
}