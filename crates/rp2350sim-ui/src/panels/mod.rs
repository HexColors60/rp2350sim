//! Peripheral panels for RP2350 simulator.
//!
//! Provides detailed GUI panels for each peripheral.
#![allow(dead_code)]
#![allow(unused_imports)]

mod mpu;
mod reset;
mod nvic;
mod systick;

mod bootram;

mod sysinfo;

mod gpio;
mod uart;
mod spi;
mod i2c;
mod i2s;
mod pio;
mod pll;
mod adc_pwm;
mod rtc;
mod sha256;
mod timer;
mod trng;
mod usb;
mod dma;
mod xip;
mod symbols;
mod powman;
mod plic;
mod busctrl;

mod hstx;

mod watchdog;
mod otp;

mod interp;

mod coresight;
pub use gpio::GpioPanel;
pub use uart::UartPanel;
pub use spi::SpiPanel;
pub use i2c::I2cPanel;
pub use i2s::{I2sPanel, I2sState};
pub use pio::PioPanel;
pub use pll::{PllPanel, PllState};
pub use adc_pwm::AdcPwmPanel;
pub use rtc::{RtcPanel, RtcState};
pub use sha256::{Sha256Panel, Sha256State};
pub use timer::TimerPanel;
pub use trng::{TrngPanel, TrngState};
pub use usb::UsbPanel;
pub use dma::{DmaPanel, DmaState, DmaChannelState};
pub use xip::{XipPanel, XipState};
pub use symbols::{SymbolsPanel, SymbolsState, SymbolEvent, SymbolEntry, SymbolKind};
pub use watchdog::{WatchdogPanel, WatchdogState};
pub use otp::{OtpPanel, OtpState};
pub use powman::{PowmanPanel, PowmanState, PowerState};
pub use plic::{PlicPanel, PlicState};
pub use busctrl::{BusCtrlPanel, BusCtrlState};
pub use sysinfo::{SysinfoPanel, SysinfoState};
pub use bootram::{BootramPanel, BootramState};
pub use hstx::{HstxPanel, HstxState};
pub use interp::{InterpPanel, InterpState};
pub use mpu::{MpuPanel, MpuState, MpuRegion};
pub use coresight::{CoreSightPanel, CoreSightState};
pub use reset::ResetPanel;
pub use nvic::NvicPanel;
pub use systick::SysTickPanel;



use egui::{Color32, RichText, Ui};

/// Peripheral panel tab identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeripheralTab {
    Gpio,
    Uart,
    Spi,
    I2c,
    I2s,
    Pio,
    Pll,
    AdcPwm,
    Rtc,
    Sha256,
    Timer,
    Trng,
    Usb,
    Dma,
    Xip,
    Symbols,
    Watchdog,
    Sysinfo,
    Powman,
    Plic,
    Otp,
    Hstx,

    Interp,
    Bootram,
    BusCtrl,
    Coresight,
    Mpu,
    Reset,
    Nvic,
    SysTick,
}

impl PeripheralTab {
    /// Get the tab name.
    pub fn name(&self) -> &'static str {
        match self {
            PeripheralTab::Gpio => "GPIO",
            PeripheralTab::Uart => "UART",
            PeripheralTab::Spi => "SPI",
            PeripheralTab::I2c => "I2C",
            PeripheralTab::I2s => "I2S",
            PeripheralTab::Pio => "PIO",
            PeripheralTab::Pll => "PLL",
            PeripheralTab::AdcPwm => "ADC/PWM",
            PeripheralTab::Rtc => "RTC",
            PeripheralTab::Sha256 => "SHA256",
            PeripheralTab::Timer => "Timer",
            PeripheralTab::Trng => "TRNG",
            PeripheralTab::Usb => "USB",
            PeripheralTab::Dma => "DMA",
            PeripheralTab::Xip => "XIP",
            PeripheralTab::Symbols => "Symbols",
            PeripheralTab::Watchdog => "Watchdog",
            PeripheralTab::Sysinfo => "SysInfo",
            PeripheralTab::Powman => "Powman",
            PeripheralTab::Plic => "PLIC",
            PeripheralTab::Otp => "OTP",
            PeripheralTab::Hstx => "HSTX",

            PeripheralTab::Interp => "Interp",
            PeripheralTab::Bootram => "Boot RAM",
            PeripheralTab::BusCtrl => "BusCtrl",
            PeripheralTab::Coresight => "CoreSight",
            PeripheralTab::Mpu => "MPU",
            PeripheralTab::Reset => "Reset",
            PeripheralTab::Nvic => "NVIC",
            PeripheralTab::SysTick => "SysTick",
        }
    }

    /// Get all tabs.
    pub fn all() -> &'static [PeripheralTab] {
        &[
            PeripheralTab::Gpio,
            PeripheralTab::Uart,
            PeripheralTab::Spi,
            PeripheralTab::I2c,
            PeripheralTab::I2s,
            PeripheralTab::Pio,
            PeripheralTab::Pll,
            PeripheralTab::AdcPwm,
            PeripheralTab::Rtc,
            PeripheralTab::Sha256,
            PeripheralTab::Timer,
            PeripheralTab::Trng,
            PeripheralTab::Usb,
            PeripheralTab::Dma,
            PeripheralTab::Xip,
            PeripheralTab::Symbols,
            PeripheralTab::Watchdog,
            PeripheralTab::Sysinfo,
            PeripheralTab::Powman,
            PeripheralTab::Plic,
            PeripheralTab::Otp,
            PeripheralTab::Hstx,

            PeripheralTab::Interp,
            PeripheralTab::Bootram,
            PeripheralTab::BusCtrl,
            PeripheralTab::Coresight,
            PeripheralTab::Mpu,
            PeripheralTab::Reset,
            PeripheralTab::Nvic,
            PeripheralTab::SysTick,
        ]
    }
}

/// Manager for all peripheral panels.
pub struct PeripheralPanelManager {
    /// Currently selected tab.
    selected_tab: PeripheralTab,
    /// GPIO panel.
    gpio_panel: GpioPanel,
    /// UART panel.
    uart_panel: UartPanel,
    /// SPI panel.
    spi_panel: SpiPanel,
    /// I2C panel.
    i2c_panel: I2cPanel,
    /// I2S panel.
    i2s_panel: I2sPanel,
    /// PIO panel.
    pio_panel: PioPanel,
    /// PLL panel.
    pll_panel: PllPanel,
    /// ADC/PWM panel.
    adc_pwm_panel: AdcPwmPanel,
    /// RTC panel.
    rtc_panel: RtcPanel,
    /// SHA-256 panel.
    sha256_panel: Sha256Panel,
    /// Timer panel.
    timer_panel: TimerPanel,
    /// TRNG panel.
    trng_panel: TrngPanel,
    /// USB panel.
    usb_panel: UsbPanel,
    /// DMA panel.
    dma_panel: DmaPanel,
    /// XIP panel.
    xip_panel: XipPanel,
    /// Symbols panel.
    symbols_panel: SymbolsPanel,
    /// Watchdog panel.
    watchdog_panel: WatchdogPanel,
    /// Sysinfo panel.
    sysinfo_panel: SysinfoPanel,
    /// OTP panel.
    otp_panel: OtpPanel,
    /// PLIC panel.
    plic_panel: PlicPanel,
    /// Power Manager panel.
    powman_panel: PowmanPanel,
    /// HSTX panel.
    hstx_panel: HstxPanel,
    /// Interpolator panel.
    interp_panel: InterpPanel,
    /// MPU panel.
    mpu_panel: MpuPanel,
    /// Boot RAM panel.
    bootram_panel: BootramPanel,
    /// Bus Controller panel.
    busctrl_panel: BusCtrlPanel,
    /// CoreSight panel.
    coresight_panel: CoreSightPanel,
    reset_panel: ResetPanel,
    nvic_panel: NvicPanel,
    systick_panel: SysTickPanel,
}
impl Default for PeripheralPanelManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PeripheralPanelManager {
    /// Create a new peripheral panel manager.
    pub fn new() -> Self {
        Self {
            selected_tab: PeripheralTab::Gpio,
            gpio_panel: GpioPanel::new(),
            uart_panel: UartPanel::new(),
            spi_panel: SpiPanel::new(),
            i2c_panel: I2cPanel::new(),
            i2s_panel: I2sPanel::new(),
            pio_panel: PioPanel::new(),
            pll_panel: PllPanel::new(),
            adc_pwm_panel: AdcPwmPanel::new(),
            rtc_panel: RtcPanel::new(),
            sha256_panel: Sha256Panel::new(),
            timer_panel: TimerPanel::new(),
            trng_panel: TrngPanel::new(),
            usb_panel: UsbPanel::new(),
            dma_panel: DmaPanel::new(),
            xip_panel: XipPanel::new(),
            symbols_panel: SymbolsPanel::new(),
            watchdog_panel: WatchdogPanel::new(),
            sysinfo_panel: SysinfoPanel::new(),
            otp_panel: OtpPanel::new(),
            plic_panel: PlicPanel::new(),
            powman_panel: PowmanPanel::new(),
            hstx_panel: HstxPanel::new(),
            interp_panel: InterpPanel::new(),
            bootram_panel: BootramPanel::new(),
            busctrl_panel: BusCtrlPanel::new(),
            coresight_panel: CoreSightPanel::new(),
            mpu_panel: MpuPanel::new(),
            reset_panel: ResetPanel::new(),
            nvic_panel: NvicPanel::new(),
            systick_panel: SysTickPanel::new(),
        }
    }

    /// Draw the peripheral panels with tab selector.
    pub fn draw(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                for tab in PeripheralTab::all() {
                    let selected = self.selected_tab == *tab;
                    if ui.selectable_label(selected, tab.name()).clicked() {
                        self.selected_tab = *tab;
                    }
                }
            });
            ui.separator();

            // Draw selected panel
            match self.selected_tab {
                PeripheralTab::Gpio => self.gpio_panel.draw(ui, state),
                PeripheralTab::Uart => self.uart_panel.draw(ui, state),
                PeripheralTab::Spi => self.spi_panel.draw(ui, state),
                PeripheralTab::I2c => self.i2c_panel.draw(ui, state),
                PeripheralTab::I2s => self.i2s_panel.draw(ui, &mut state.i2s),
                PeripheralTab::Pio => self.pio_panel.draw(ui, state),
                PeripheralTab::Pll => self.pll_panel.draw(ui, &mut state.pll),
                PeripheralTab::AdcPwm => self.adc_pwm_panel.draw(ui, state),
                PeripheralTab::Rtc => self.rtc_panel.draw(ui, &mut state.rtc),
                PeripheralTab::Sha256 => self.sha256_panel.draw(ui, &mut state.sha256),
                PeripheralTab::Timer => self.timer_panel.draw(ui, state),
                PeripheralTab::Trng => self.trng_panel.draw(ui, &mut state.trng),
                PeripheralTab::Usb => self.usb_panel.draw(ui, state),
                PeripheralTab::Dma => self.dma_panel.draw(ui, &mut state.dma),
                PeripheralTab::Xip => self.xip_panel.draw(ui, &mut state.xip),
                PeripheralTab::Symbols => {
                    if let Some(event) = self.symbols_panel.show(ui, None) {
                        match event {
                            SymbolEvent::Selected(addr) => {
                                tracing::info!("Symbol selected: 0x{:08X}", addr);
                            }
                            SymbolEvent::GotoAddress(addr) => {
                                state.events.push(PeripheralEvent::MemoryViewGoto(addr));
                            }
                        }
                    }
                }
                PeripheralTab::Watchdog => self.watchdog_panel.draw(ui, &mut state.watchdog),
                PeripheralTab::Sysinfo => self.sysinfo_panel.draw(ui, &mut state.sysinfo),
                PeripheralTab::Plic => self.plic_panel.draw(ui, &mut state.plic),
                PeripheralTab::Powman => self.powman_panel.draw(ui, &mut state.powman),
                PeripheralTab::Otp => self.otp_panel.draw(ui, &mut state.otp),
                PeripheralTab::Hstx => self.hstx_panel.draw(ui, &mut state.hstx),

                PeripheralTab::Interp => self.interp_panel.draw(ui, &mut state.interp),
                PeripheralTab::Bootram => self.bootram_panel.draw(ui, &mut state.bootram),
                PeripheralTab::BusCtrl => self.busctrl_panel.draw(ui, &mut state.busctrl),
                PeripheralTab::Mpu => self.mpu_panel.draw(ui, &mut state.mpu),
                PeripheralTab::Coresight => self.coresight_panel.draw(ui, &mut state.coresight),
                PeripheralTab::Reset => self.reset_panel.draw(ui, state),
                PeripheralTab::Nvic => self.nvic_panel.draw(ui, state),
                PeripheralTab::SysTick => self.systick_panel.draw(ui, state),
            }
        });
    }

    /// Get the currently selected tab.
    pub fn selected_tab(&self) -> PeripheralTab {
        self.selected_tab
    }

    /// Set the selected tab.
    pub fn set_selected_tab(&mut self, tab: PeripheralTab) {
        self.selected_tab = tab;
    }
}

/// Peripheral state shared across panels.
#[derive(Debug)]
pub struct PeripheralState {
    // GPIO state
    pub gpio_values: [bool; 48],
    pub gpio_directions: [bool; 48],  // true = output
    pub gpio_pullups: [bool; 48],
    pub gpio_pulldowns: [bool; 48],
    pub gpio_functions: [u8; 48],  // Alternate function select
    pub gpio_interrupts: [bool; 48],  // Interrupt pending
    pub gpio_drive_strength: [u8; 48],  // Drive strength (2mA, 4mA, 8mA, 12mA)
    pub gpio_slew_fast: [bool; 48],  // Fast slew rate

    // UART state
    pub uart: [UartState; 2],

    // SPI state
    pub spi: [SpiState; 2],

    // I2C state
    pub i2c: [I2cState; 2],

    // I2S state
    pub i2s: I2sState,

    // PIO state
    pub pio: [PioState; 2],

    // PLL state
    pub pll: PllState,

    // ADC state
    pub adc_values: [u16; 4],
    pub adc_enabled: [bool; 4],

    // PWM state (8 slices, 2 channels each = 16 channels)
    pub pwm_duty: [u16; 16],
    pub pwm_enabled: [bool; 16],
    pub pwm_phase_correct: [bool; 8],
    pub pwm_invert_a: [bool; 8],
    pub pwm_invert_b: [bool; 8],
    pub pwm_divider: [u16; 8],  // 8.4 fixed point, integer part
    pub pwm_top: [u16; 8],

    // RTC state
    pub rtc: RtcState,

    // SHA-256 state
    pub sha256: Sha256State,

    // Timer state
    pub timer_value: u64,
    pub timer_running: bool,

    // TRNG state
    pub trng: TrngState,

    // USB state
    pub usb_connected: bool,
    pub usb_device_mode: bool,

    // DMA state
    pub dma: DmaState,

    // XIP state
    pub xip: XipState,

    // Symbols state
    pub symbols: SymbolsState,

    // Watchdog state
    pub watchdog: WatchdogState,

    // OTP state
    pub otp: OtpState,

    // PLIC state
    pub plic: PlicState,

    // Sysinfo state
    pub sysinfo: SysinfoState,

    // Power Manager state
    pub powman: PowmanState,

    // Boot RAM state
    pub bootram: BootramState,

    // HSTX state
    pub hstx: HstxState,

    // Interpolator state
    pub interp: InterpState,

    // Memory view state
    pub memory_base: u32,

    // Events
    pub events: Vec<PeripheralEvent>,

    // Bus Controller state
    pub busctrl: BusCtrlState,
    // CoreSight state
    pub coresight: CoreSightState,
    // MPU state
    pub mpu: MpuState,
    
    // Reset state
    pub reset: ResetState,
    
    // NVIC state
    pub nvic: NvicState,
    
    // SysTick state
    pub systick: SysTickState,
}

impl Default for PeripheralState {
    fn default() -> Self {
        Self {
            gpio_values: [false; 48],
            gpio_directions: [false; 48],
            gpio_pullups: [false; 48],
            gpio_pulldowns: [false; 48],
            gpio_functions: [0; 48],
            gpio_interrupts: [false; 48],
            gpio_drive_strength: [0; 48],  // 0 = 2mA, 1 = 4mA, 2 = 8mA, 3 = 12mA
            gpio_slew_fast: [false; 48],
            uart: [UartState::default(), UartState::default()],
            spi: [SpiState::default(), SpiState::default()],
            i2c: [I2cState::default(), I2cState::default()],
            i2s: I2sState::default(),
            pio: [PioState::default(), PioState::default()],
            pll: PllState::default(),
            adc_values: [0; 4],
            adc_enabled: [false; 4],
            pwm_duty: [0; 16],
            pwm_enabled: [false; 16],
            pwm_phase_correct: [false; 8],
            pwm_invert_a: [false; 8],
            pwm_invert_b: [false; 8],
            pwm_divider: [1; 8],
            pwm_top: [0xFFFF; 8],
            rtc: RtcState::default(),
            sha256: Sha256State::default(),
            timer_value: 0,
            timer_running: false,
            trng: TrngState::default(),
            usb_connected: false,
            usb_device_mode: true,
            dma: DmaState::default(),
            xip: XipState::default(),
            symbols: SymbolsState::default(),
            watchdog: WatchdogState::default(),
            otp: OtpState::default(),
            plic: PlicState::default(),
            sysinfo: SysinfoState::default(),
            powman: PowmanState::default(),
            bootram: BootramState::default(),
            hstx: HstxState::default(),
            interp: InterpState::default(),
            memory_base: 0x20000000,  // SRAM base
            mpu: MpuState::default(),
            events: Vec::new(),
            busctrl: BusCtrlState::default(),
            coresight: CoreSightState::default(),
            reset: ResetState::default(),
            nvic: NvicState::default(),
            systick: SysTickState::default(),
        }

    }
}

/// Reset controller state.
#[derive(Debug, Clone, Default)]
pub struct ResetState {
    /// RESET register value (bits indicate which peripherals are in reset)
    pub reset_reg: u32,
    /// WDSEL register value (watchdog reset select)
    pub wdsel: u32,
    /// RESET_DONE register value (1 = peripheral reset complete)
    pub reset_done: u32,
}

/// NVIC state.
#[derive(Debug, Clone)]
pub struct NvicState {
    /// Enabled interrupt bits
    pub enabled: [u32; 4],
    /// Pending interrupt bits
    pub pending: [u32; 4],
    /// Active interrupt bits
    pub active: [u32; 4],
    /// Priority registers (128 interrupts, 8 bits each)
    pub priority: Vec<u8>,
    /// Vector table offset
    pub vtor: u32,
}

impl Default for NvicState {
    fn default() -> Self {
        Self {
            enabled: [0; 4],
            pending: [0; 4],
            active: [0; 4],
            priority: vec![0; 128],
            vtor: 0,
        }
    }
}

/// SysTick state.
#[derive(Debug, Clone, Default)]
pub struct SysTickState {
    /// Control and Status Register
    pub csr: u32,
    /// Reload Value Register
    pub rvr: u32,
    /// Current Value Register
    pub cvr: u32,
    /// Calibration Value Register
    pub calib: u32,
}

/// UART state.
#[derive(Debug, Clone)]
pub struct UartState {
    pub enabled: bool,
    pub baud_rate: u32,
    pub tx_fifo: Vec<u8>,
    pub rx_fifo: Vec<u8>,
    pub tx_count: u64,
    pub rx_count: u64,
    pub parity: Parity,
    pub stop_bits: u8,
    pub data_bits: u8,
    /// Flow control enabled
    pub flow_control: bool,
    /// TX overrun error
    pub tx_overrun: bool,
    /// RX overrun error
    pub rx_overrun: bool,
    /// Framing error
    pub framing_error: bool,
    /// Parity error
    pub parity_error: bool,
    /// TX interrupt enabled
    pub tx_int_enabled: bool,
    /// RX interrupt enabled
    pub rx_int_enabled: bool,
    /// TX line state (true = idle/high, false = active/low)
    pub tx_line_high: bool,
    /// RX line state
    pub rx_line_high: bool,
    /// CTS line state (clear to send)
    pub cts_high: bool,
    /// RTS line state (ready to receive)
    pub rts_high: bool,
}

impl Default for UartState {
    fn default() -> Self {
        Self {
            enabled: false,
            baud_rate: 115200,
            tx_fifo: Vec::with_capacity(32),
            rx_fifo: Vec::with_capacity(32),
            tx_count: 0,
            rx_count: 0,
            parity: Parity::None,
            stop_bits: 1,
            data_bits: 8,
            flow_control: false,
            tx_overrun: false,
            rx_overrun: false,
            framing_error: false,
            parity_error: false,
            tx_int_enabled: false,
            rx_int_enabled: false,
            tx_line_high: true,  // Idle state is high
            rx_line_high: true,
            cts_high: true,      // Can send by default
            rts_high: true,      // Ready to receive by default
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Parity {
    None,
    Even,
    Odd,
}

/// SPI state.
#[derive(Debug, Clone)]
pub struct SpiState {
    pub enabled: bool,
    pub clock_rate: u32,
    pub cpol: bool,
    pub cpha: bool,
    pub master_mode: bool,
    pub lsb_first: bool,
    pub data_bits: u8,
    pub tx_fifo: Vec<u8>,
    pub rx_fifo: Vec<u8>,
    pub transactions: Vec<SpiTransaction>,
    pub total_bytes_tx: u64,
    pub total_bytes_rx: u64,
    pub cs_active: bool,
}

impl Default for SpiState {
    fn default() -> Self {
        Self {
            enabled: false,
            clock_rate: 1_000_000,
            cpol: false,
            cpha: false,
            master_mode: true,
            lsb_first: false,
            data_bits: 8,
            tx_fifo: Vec::with_capacity(8),
            rx_fifo: Vec::with_capacity(8),
            transactions: Vec::with_capacity(100),
            total_bytes_tx: 0,
            total_bytes_rx: 0,
            cs_active: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpiTransaction {
    pub timestamp: u64,
    pub cs: u8,
    pub data_out: Vec<u8>,
    pub data_in: Vec<u8>,
}

/// I2C state.
#[derive(Debug, Clone)]
pub struct I2cState {
    pub enabled: bool,
    pub clock_rate: u32,
    pub master_mode: bool,
    pub transactions: Vec<I2cTransaction>,
    pub attached_devices: Vec<I2cDevice>,
    pub total_tx_bytes: u64,
    pub total_rx_bytes: u64,
    pub total_transactions: u64,
    pub scl_high: bool,
    pub sda_high: bool,
    pub bus_state: String,
}

impl Default for I2cState {
    fn default() -> Self {
        Self {
            enabled: false,
            clock_rate: 100_000,
            master_mode: true,
            transactions: Vec::with_capacity(100),
            attached_devices: Vec::new(),
            total_tx_bytes: 0,
            total_rx_bytes: 0,
            total_transactions: 0,
            scl_high: true,
            sda_high: true,
            bus_state: "Idle".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct I2cTransaction {
    pub timestamp: u64,
    pub address: u8,
    pub read: bool,
    pub data: Vec<u8>,
    pub ack: Vec<bool>,
}

#[derive(Debug, Clone)]
pub struct I2cDevice {
    pub address: u8,
    pub name: String,
    pub device_type: String,
}

/// PIO state.
#[derive(Debug, Clone)]
pub struct PioState {
    pub enabled: bool,
    pub state_machines: [StateMachineState; 4],
    pub instruction_memory: [u16; 32],
    pub program_loaded: bool,
    pub irq_flags: u32,
}

impl Default for PioState {
    fn default() -> Self {
        Self {
            enabled: false,
            state_machines: Default::default(),
            instruction_memory: [0; 32],
            program_loaded: false,
            irq_flags: 0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StateMachineState {
    pub enabled: bool,
    pub pc: u8,
    pub clock_div: u16,
    pub tx_fifo: Vec<u32>,
    pub rx_fifo: Vec<u32>,
    pub pins_out: u32,
    pub pins_in: u32,
    pub executing: bool,
    pub stalled: bool,
    /// Starting pin number for OUT operations
    pub out_pins: u8,
    /// Starting pin number for IN operations
    pub in_pins: u8,
    /// Number of side-set pins
    pub side_set_pins: u8,
}

/// Peripheral events.
#[derive(Debug, Clone)]
pub enum PeripheralEvent {
    // GPIO events
    GpioToggle(usize, bool),
    GpioSetDirection(usize, bool),
    GpioSetFunction(usize, u8),

    // UART events
    UartSend(usize, Vec<u8>),
    UartReceive(usize, Vec<u8>),
    UartSetBaud(usize, u32),

    // SPI events
    SpiTransfer(usize, Vec<u8>),

    // I2C events
    I2cTransaction(usize, I2cTransaction),

    // PIO events
    PioLoadProgram(usize, Vec<u16>),
    PioStartSm(usize, u8),
    PioStopSm(usize, u8),

    // ADC events
    AdcSetValue(u8, u16),

    // PWM events
    PwmSetDuty(u8, u16),

    // Memory view events
    MemoryViewGoto(u32),
}

/// Helper function to draw a status indicator.
fn status_indicator(ui: &mut Ui, label: &str, active: bool) {
    let color = if active { Color32::GREEN } else { Color32::DARK_GRAY };
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
        ui.painter().circle_filled(rect.center(), 5.0, color);
        ui.label(label);
    });
}

/// Helper function to draw a hex display.
fn hex_display(ui: &mut Ui, label: &str, value: u32) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.monospace(RichText::new(format!("0x{:08X}", value)).color(Color32::from_rgb(0, 200, 255)));
    });
}

/// Helper function to draw a register field.
fn register_field(ui: &mut Ui, name: &str, value: u32, bits: &[(u8, u8, &str)]) {
    ui.group(|ui| {
        ui.label(RichText::new(name).strong());
        ui.monospace(format!("0x{:08X}", value));
        ui.separator();
        for (start, end, field_name) in bits {
            let mask = if start == end {
                1 << start
            } else {
                ((1 << (end - start + 1)) - 1) << start
            };
            let field_value = (value & mask) >> start;
            ui.horizontal(|ui| {
                ui.label(format!("{}[{}:{}]:", field_name, end, start));
                ui.monospace(format!("{}", field_value));
            });
        }
    });
}