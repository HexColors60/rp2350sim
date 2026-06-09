#![allow(dead_code)]

//! Save state serialization.

use rp2350sim_core::Result;
use rp2350sim_soc::Soc;
use serde::{Deserialize, Serialize};

/// Save state format version.
pub const SAVE_VERSION: u32 = 1;

/// Magic number for save files.
pub const SAVE_MAGIC: &[u8; 4] = b"RPSV";

/// Complete save state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveState {
    /// Format version.
    pub version: u32,
    /// Timestamp when saved.
    pub timestamp: u64,
    /// CPU state.
    pub cpu: CpuSaveState,
    /// Memory state.
    pub memory: MemorySaveState,
    /// Device states.
    pub devices: DeviceSaveState,
    /// Clock state.
    pub clocks: ClockSaveState,
}

/// CPU save state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuSaveState {
    /// Active CPU architecture.
    pub arch: String,
    /// Core states.
    pub cores: Vec<CoreSaveState>,
}

/// Single core save state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreSaveState {
    /// Core ID.
    pub id: u8,
    /// Program counter.
    pub pc: u32,
    /// General purpose registers.
    pub regs: [u32; 16],
    /// Stack pointer (MSP).
    pub msp: u32,
    /// Stack pointer (PSP).
    pub psp: u32,
    /// Link register.
    pub lr: u32,
    /// Program status register.
    pub xpsr: u32,
    /// Cycle count.
    pub cycles: u64,
    /// Instruction count.
    pub instructions: u64,
}

/// Memory save state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySaveState {
    /// SRAM contents.
    pub sram: Vec<u8>,
    /// Flash contents (optional).
    pub flash: Option<Vec<u8>>,
    /// Memory region checksums.
    pub checksums: Vec<MemoryChecksum>,
}

/// Memory region checksum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryChecksum {
    /// Region start address.
    pub start: u32,
    /// Region size.
    pub size: u32,
    /// CRC32 checksum.
    pub checksum: u32,
}

/// Device save state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSaveState {
    /// GPIO state.
    pub gpio: GpioSaveState,
    /// UART states.
    pub uarts: Vec<UartSaveState>,
    /// Timer state.
    pub timer: TimerSaveState,
    /// PIO state.
    pub pio: PioSaveState,
}

/// GPIO save state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpioSaveState {
    /// Pin states.
    pub pins: Vec<PinSaveState>,
}

/// Pin save state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinSaveState {
    /// Pin number.
    pub pin: u8,
    /// Output value.
    pub out: bool,
    /// Input value.
    pub input: bool,
    /// Direction (true = output).
    pub output_enable: bool,
    /// Function select.
    pub function: u8,
    /// Pull-up enable.
    pub pull_up: bool,
    /// Pull-down enable.
    pub pull_down: bool,
}

/// UART save state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UartSaveState {
    /// UART index.
    pub index: u8,
    /// Baud rate.
    pub baud: u32,
    /// TX FIFO.
    pub tx_fifo: Vec<u8>,
    /// RX FIFO.
    pub rx_fifo: Vec<u8>,
    /// Control register.
    pub cr: u32,
    /// Interrupt mask.
    pub imsc: u32,
}

/// Timer save state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerSaveState {
    /// Current time value.
    pub time: u64,
    /// Alarm values.
    pub alarms: [u32; 4],
    /// Armed alarms.
    pub armed: u8,
}

/// PIO save state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PioSaveState {
    /// PIO index.
    pub index: u8,
    /// Instruction memory.
    pub instr_mem: Vec<u16>,
    /// State machine states.
    pub state_machines: Vec<SmSaveState>,
}

/// State machine save state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmSaveState {
    /// State machine index.
    pub index: u8,
    /// Program counter.
    pub pc: u8,
    /// TX FIFO.
    pub tx_fifo: Vec<u32>,
    /// RX FIFO.
    pub rx_fifo: Vec<u32>,
}

/// Clock save state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockSaveState {
    /// System clock frequency.
    pub sys_clk: u32,
    /// Peripheral clock frequencies.
    pub peri_clks: Vec<u32>,
}

/// Save the current state.
pub fn save_state(soc: &Soc) -> Result<Vec<u8>> {
    let state = SaveState::from_soc(soc);
    save_to_bytes(&state)
}

/// Save state to bytes.
pub fn save_to_bytes(state: &SaveState) -> Result<Vec<u8>> {
    // Use bincode for efficient binary serialization
    let mut output = Vec::new();
    
    // Write magic number
    output.extend_from_slice(SAVE_MAGIC);
    
    // Write version
    output.extend_from_slice(&state.version.to_le_bytes());
    
    // Serialize the state
    let serialized = bincode::serialize(state)
        .map_err(|e| rp2350sim_core::Error::SerializationError(e.to_string()))?;
    
    output.extend_from_slice(&serialized);
    
    Ok(output)
}

/// Save state to file.
pub fn save_to_file(state: &SaveState, path: &std::path::Path) -> Result<()> {
    let data = save_to_bytes(state)?;
    std::fs::write(path, data)
        .map_err(|e| rp2350sim_core::Error::IoError(e.to_string()))?;
    Ok(())
}

impl SaveState {
    /// Create save state from SoC.
    pub fn from_soc(soc: &Soc) -> Self {
        Self {
            version: SAVE_VERSION,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            cpu: CpuSaveState::from_soc(soc),
            memory: MemorySaveState::from_soc(soc),
            devices: DeviceSaveState::from_soc(soc),
            clocks: ClockSaveState::from_soc(soc),
        }
    }
}

impl CpuSaveState {
    fn from_soc(soc: &Soc) -> Self {
        // Extract CPU state from SoC
        let core0 = CoreSaveState {
            id: 0,
            pc: soc.pc(),
            regs: {
                let mut regs = [0u32; 16];
                for i in 0..16 {
                    regs[i] = soc.read_reg(i);
                }
                regs
            },
            msp: soc.sp(),
            psp: 0, // Would need PSP API
            lr: soc.lr(),
            xpsr: soc.flags(),
            cycles: soc.cycles(),
            instructions: soc.instructions(),
        };
        
        // Core 1 would need separate API
        let core1 = CoreSaveState {
            id: 1,
            pc: 0,
            regs: [0; 16],
            msp: 0,
            psp: 0,
            lr: 0,
            xpsr: 0,
            cycles: 0,
            instructions: 0,
        };
        
        Self {
            arch: if soc.cpu_arm.is_some() { "arm" } else { "hazard3" }.to_string(),
            cores: vec![core0, core1],
        }
    }
}

impl MemorySaveState {
    fn from_soc(soc: &Soc) -> Self {
        // Extract SRAM contents
        let sram_size = soc.sram.total_size();
        let sram = soc.sram.read(0, sram_size);
        
        // Extract Flash contents
        let flash_size = soc.flash.size();
        let flash_data = soc.flash.data();
        let flash = if !flash_data.is_empty() {
            Some(flash_data[..flash_size.min(flash_data.len())].to_vec())
        } else {
            None
        };
        
        Self {
            sram,
            flash,
            checksums: Vec::new(),
        }
    }
}

impl DeviceSaveState {
    fn from_soc(soc: &Soc) -> Self {
        Self {
            gpio: GpioSaveState::from_soc(soc),
            uarts: vec![
                UartSaveState::from_soc_uart(0, &soc.uart0),
                UartSaveState::from_soc_uart(1, &soc.uart1),
            ],
            timer: TimerSaveState::from_soc(soc),
            pio: PioSaveState::from_soc(0, &soc.pio0),
        }
    }
}

impl GpioSaveState {
    fn from_soc(soc: &Soc) -> Self {
        Self {
            pins: (0..48u8).map(|pin| PinSaveState {
                pin,
                out: soc.gpio_value(pin as usize),
                input: soc.gpio_value(pin as usize),
                output_enable: soc.gpio_direction(pin as usize),
                function: 0,
                pull_up: false,
                pull_down: false,
            }).collect(),
        }
    }
}

impl UartSaveState {
    fn from_soc_uart(index: u8, _uart: &rp2350sim_devices::uart::Uart) -> Self {
        Self {
            index,
            baud: 115200,
            tx_fifo: Vec::new(),
            rx_fifo: Vec::new(),
            cr: 0,
            imsc: 0,
        }
    }
}

impl TimerSaveState {
    fn from_soc(_soc: &Soc) -> Self {
        Self {
            time: 0,
            alarms: [0; 4],
            armed: 0,
        }
    }
}

impl PioSaveState {
    fn from_soc(index: u8, _pio: &rp2350sim_devices::pio::Pio) -> Self {
        Self {
            index,
            instr_mem: vec![0; 32],
            state_machines: (0..4).map(|i| SmSaveState {
                index: i,
                pc: 0,
                tx_fifo: Vec::new(),
                rx_fifo: Vec::new(),
            }).collect(),
        }
    }
}

impl ClockSaveState {
    fn from_soc(_soc: &Soc) -> Self {
        Self {
            sys_clk: 150_000_000, // Default 150 MHz
            peri_clks: vec![150_000_000; 32],
        }
    }
}