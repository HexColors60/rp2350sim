//! Main application with macroquad/egui GUI.
#![allow(dead_code)]

use crate::config::Config;
use rp2350sim_core::{Result, CpuArch};
use rp2350sim_soc::Soc;
use rp2350sim_ui::{Ui, UiState, UiEvent, FirmwareLoaderDialog, SaveStateDialog, PeripheralState, PeripheralEvent, PeripheralPanelManager};
use rp2350sim_debug::DebugController;
use rp2350sim_trace::Trace;
use std::time::Instant;

#[cfg(feature = "gui-bevy")]
use bevy_egui::egui;
#[cfg(not(feature = "gui-bevy"))]
use egui;

/// Main application.
pub struct App {
    config: Config,
    running: bool,
    /// SoC simulation.
    soc: Option<Soc>,
    /// UI system.
    ui: Ui,
    /// UI state.
    ui_state: UiState,
    /// Peripheral state.
    peripheral_state: PeripheralState,
    /// Peripheral panel manager.
    peripheral_panels: PeripheralPanelManager,
    /// Debugger.
    debugger: DebugController,
    /// Tracer.
    tracer: Trace,
    /// Last frame time.
    last_frame: Instant,
    /// Simulation speed (instructions per frame).
    sim_speed: u64,
    /// Paused state.
    paused: bool,
    /// Single step mode.
    single_step: bool,
    /// Firmware loader dialog.
    firmware_dialog: FirmwareLoaderDialog,
    /// Save state dialog.
    save_dialog: SaveStateDialog,
    /// Pending firmware path to load.
    pending_firmware: Option<String>,
    /// Demo mode counter.
    demo_counter: u64,
    /// Demo mode input buffer.
    demo_input: String,
}

impl App {
    /// Create a new application.
    pub fn new(config: Config) -> Self {
        let mut ui_state = UiState::new();
        ui_state.core_name = match config.sim.cpu_arch {
            CpuArch::Arm => "ARM Cortex-M33".to_string(),
            CpuArch::Hazard3 => "Hazard3 RISC-V".to_string(),
        };

        Self {
            config,
            running: false,
            soc: None,
            ui: Ui::new(),
            ui_state,
            peripheral_state: PeripheralState::default(),
            peripheral_panels: PeripheralPanelManager::new(),
            debugger: DebugController::new(),
            tracer: Trace::new(),
            last_frame: Instant::now(),
            sim_speed: 1000,
            paused: true,
            single_step: false,
            firmware_dialog: FirmwareLoaderDialog::new(),
            save_dialog: SaveStateDialog::new(),
            pending_firmware: None,
            demo_counter: 0,
            demo_input: String::new(),
        }
    }

    /// Run the application.
    pub fn run(&mut self) -> Result<()> {
        self.running = true;

        if self.config.headless {
            self.run_headless(u64::MAX)
        } else {
            self.run_gui()
        }
    }


    /// Run with GUI.
    fn run_gui(&mut self) -> Result<()> {
        tracing::info!("Starting GUI mode");

        // Initialize SoC
        self.soc = Some(Soc::new(self.config.sim.cpu_arch));

        // The actual GUI loop is handled by macroquad
        // This function returns immediately, and the GUI
        // is driven by the event loop in main.rs

        Ok(())
    }

    /// Stop the application.
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Check if running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Load firmware from file.
    pub fn load_firmware(&mut self, path: &str) -> Result<()> {
        use std::path::Path;
        
        let firmware_path = Path::new(path);
        if !firmware_path.exists() {
            return Err(rp2350sim_core::Error::FileNotFound(path.to_string()));
        }
        
        // Initialize SoC if needed
        if self.soc.is_none() {
            self.soc = Some(Soc::new(self.config.sim.cpu_arch));
        }
        
        // Get firmware data
        let data = std::fs::read(firmware_path)?;
        
        // Load into SoC memory at default address (0x10000000 for XIP)
        if let Some(ref mut soc) = self.soc {
            let base_addr = 0x10000000u32; // XIP base address
            soc.write_memory(base_addr, &data);
            
            // Set entry point
            soc.set_pc(base_addr);
            
            tracing::info!("Loaded {} bytes of firmware at 0x{:08X}", data.len(), base_addr);
        }
        
        Ok(())
    }

    /// Run headless simulation for a specified number of cycles.
    pub fn run_headless(&mut self, max_cycles: u64) -> Result<()> {
        tracing::info!("Running headless simulation for {} cycles", max_cycles);
        
        self.running = true;
        self.paused = false;
        
        // Initialize SoC if needed
        if self.soc.is_none() {
            self.soc = Some(Soc::new(self.config.sim.cpu_arch));
        }
        
        let mut cycles = 0u64;
        while self.running && cycles < max_cycles {
            if let Some(ref mut soc) = self.soc {
                soc.step()?;
                cycles += 1;
            }
        }
        
        tracing::info!("Completed {} cycles", cycles);
        Ok(())
    }
    /// Run headless simulation with instruction tracing.
    pub fn run_headless_with_trace(&mut self, max_cycles: u64) -> Result<()> {
        tracing::info!("Running headless simulation with trace for {} cycles", max_cycles);
        
        self.running = true;
        self.paused = false;
        
        // Initialize SoC if needed
        if self.soc.is_none() {
            self.soc = Some(Soc::new(self.config.sim.cpu_arch));
        }
        
        let mut cycles = 0u64;
        while self.running && cycles < max_cycles {
            if let Some(ref mut soc) = self.soc {
                // Print instruction before execution
                let pc = soc.read_reg(15);
                println!("Cycle {}: PC=0x{:08X}", cycles, pc);
                
                soc.step()?;
                cycles += 1;
            }
        }
        
        tracing::info!("Completed {} cycles", cycles);
        Ok(())
    }

    /// Dump CPU registers to stdout.
    pub fn dump_registers(&self) {
        if let Some(ref soc) = self.soc {
            println!("CPU Registers:");
            for i in 0..16 {
                let value = soc.read_reg(i);
                let name = match i {
                    0 => "R0",
                    1 => "R1",
                    2 => "R2",
                    3 => "R3",
                    4 => "R4",
                    5 => "R5",
                    6 => "R6",
                    7 => "R7",
                    8 => "R8",
                    9 => "R9",
                    10 => "R10/SL",
                    11 => "R11/FP",
                    12 => "R12/IP",
                    13 => "SP",
                    14 => "LR",
                    15 => "PC",
                    _ => "?",
                };
                println!("  {:8}: 0x{:08X}", name, value);
            }
            
            // Print flags
            let flags = soc.flags();
            println!("  Flags:    N={} Z={} C={} V={}", 
                (flags >> 31) & 1,  // N
                (flags >> 30) & 1,  // Z
                (flags >> 29) & 1,  // C
                (flags >> 28) & 1   // V
            );
        } else {
            println!("No SoC initialized");
        }
    }

    /// Dump simulation statistics.
    pub fn dump_stats(&self) {
        if let Some(ref soc) = self.soc {
            println!("
Simulation Statistics:");
            println!("  Cycles:       {}", soc.cycles());
            println!("  Instructions: {}", soc.instructions());
        }
    }

    /// Start GUI mode.
    pub fn start_gui(&mut self) {
        self.running = true;
        // Initialize SoC for GUI mode
        if self.soc.is_none() {
            self.soc = Some(Soc::new(self.config.sim.cpu_arch));
        }
    }

    /// Get mutable UI state.
    pub fn ui_state(&mut self) -> &mut UiState {
        &mut self.ui_state
    }

    /// Get UI reference.
    pub fn ui(&mut self) -> &mut Ui {
        &mut self.ui
    }

    /// Handle UI events.
    pub fn handle_events(&mut self) {
        let events: Vec<UiEvent> = self.ui_state.events.drain(..).collect();

        for event in events {
            match event {
                UiEvent::Start => {
                    self.paused = false;
                    self.ui_state.running = true;
                    tracing::info!("Simulation started");
                }
                UiEvent::Stop => {
                    self.paused = true;
                    self.ui_state.running = false;
                    tracing::info!("Simulation stopped");
                }
                UiEvent::Step => {
                    self.single_step = true;
                    self.paused = false;
                }
                UiEvent::Reset => {
                    self.soc = Some(Soc::new(self.config.sim.cpu_arch));
                    self.paused = true;
                    self.ui_state.running = false;
                    self.ui_state.cycles = 0;
                    self.ui_state.instructions = 0;
                    tracing::info!("Simulation reset");
                }
                UiEvent::LoadBinary => {
                    tracing::info!("Load binary requested");
                    self.firmware_dialog.open();
                }
                UiEvent::LoadElf => {
                    tracing::info!("Load ELF requested");
                    self.firmware_dialog.open();
                }
                UiEvent::SaveState => {
                    tracing::info!("Save state requested");
                    self.save_dialog.open();
                }
                UiEvent::LoadState => {
                    tracing::info!("Load state requested");
                    // For now, use firmware dialog for loading state files too
                    self.firmware_dialog.open();
                }
                UiEvent::Exit => {
                    self.running = false;
                }
                UiEvent::ConsoleCommand(cmd) => {
                    self.handle_console_command(&cmd);
                }
                UiEvent::BreakpointAdd(addr) => {
                    self.debugger.add_breakpoint(addr);
                    tracing::info!("Breakpoint added at 0x{:08X}", addr);
                }
                UiEvent::BreakpointRemove(addr) => {
                    self.debugger.remove_breakpoint(addr);
                    tracing::info!("Breakpoint removed at 0x{:08X}", addr);
                }
                UiEvent::GpioToggle(pin) => {
                    if let Some(ref mut soc) = self.soc {
                        soc.toggle_gpio(pin);
                    }
                }
                UiEvent::ButtonPress(button, pressed) => {
                    // Virtual buttons are connected to GPIO 16-19
                    let gpio_pin = 16 + button;
                    if let Some(ref mut soc) = self.soc {
                        soc.set_gpio_input(gpio_pin, pressed);
                    }
                    self.ui_state.console_output.push(
                        format!("Button {} {}", button, if pressed { "pressed" } else { "released" })
                    );
                }
                UiEvent::UartSend(data) => {
                    // Send data to UART0
                    if let Some(ref mut soc) = self.soc {
                        for byte in data.bytes() {
                            soc.uart_push_rx(0, byte);
                        }
                        self.ui_state.uart_tx_count += data.len() as u64;
                        self.ui_state.uart_tx_active = true;
                    }
                    
                    // Process demo commands
                    self.process_demo_uart(&data);
                    
                    tracing::debug!("UART send: {} bytes", data.len());
                }
            }
        }
    }
    
    /// Process demo UART commands.
    fn process_demo_uart(&mut self, data: &str) {
        // Accumulate input
        self.demo_input.push_str(data);
        
        // Process complete lines
        while let Some(newline_pos) = self.demo_input.find('\n') {
            let line = self.demo_input[..newline_pos].trim().to_string();
            self.demo_input = self.demo_input[newline_pos + 1..].to_string();
            
            // Process command
            let response = self.process_demo_command(&line);
            self.ui_state.uart_output.push(response);
            self.ui_state.uart_output.push("\n> ".to_string());
            self.ui_state.uart_rx_count += 1;
            self.ui_state.uart_rx_active = true;
        }
    }
    
    /// Process a demo command and return response.
    fn process_demo_command(&mut self, cmd: &str) -> String {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        
        if parts.is_empty() {
            return String::new();
        }
        
        match parts[0].to_lowercase().as_str() {
            "help" => {
                "Available commands:\n\
                 help        - Show this help\n\
                 led on      - Turn on LED (GPIO 25)\n\
                 led off     - Turn off LED\n\
                 led toggle  - Toggle LED\n\
                 gpio <n>    - Show GPIO value\n\
                 gpio <n> on - Set GPIO high\n\
                 gpio <n> off- Set GPIO low\n\
                 status      - Show system status\n\
                 echo <msg>  - Echo message\n\
                 clear       - Clear terminal".to_string()
            }
            "led" => {
                if parts.len() < 2 {
                    return "Usage: led on|off|toggle".to_string();
                }
                match parts[1].to_lowercase().as_str() {
                    "on" => {
                        self.ui_state.gpio_values[25] = true;
                        "LED ON".to_string()
                    }
                    "off" => {
                        self.ui_state.gpio_values[25] = false;
                        "LED OFF".to_string()
                    }
                    "toggle" => {
                        self.ui_state.gpio_values[25] = !self.ui_state.gpio_values[25];
                        if self.ui_state.gpio_values[25] { "LED ON".to_string() } else { "LED OFF".to_string() }
                    }
                    _ => "Usage: led on|off|toggle".to_string()
                }
            }
            "gpio" => {
                if parts.len() < 2 {
                    return "Usage: gpio <pin> [on|off]".to_string();
                }
                if let Ok(pin) = parts[1].parse::<usize>() {
                    if pin >= 48 {
                        return format!("Invalid pin: {} (0-47)", pin);
                    }
                    if parts.len() >= 3 {
                        match parts[2].to_lowercase().as_str() {
                            "on" | "1" | "high" => {
                                self.ui_state.gpio_values[pin] = true;
                                format!("GPIO {} = HIGH", pin)
                            }
                            "off" | "0" | "low" => {
                                self.ui_state.gpio_values[pin] = false;
                                format!("GPIO {} = LOW", pin)
                            }
                            _ => format!("GPIO {} = {}", pin, if self.ui_state.gpio_values[pin] { "HIGH" } else { "LOW" })
                        }
                    } else {
                        format!("GPIO {} = {}", pin, if self.ui_state.gpio_values[pin] { "HIGH" } else { "LOW" })
                    }
                } else {
                    format!("Invalid pin number: {}", parts[1])
                }
            }
            "status" => {
                format!(
                    "RP2350 Status:\n\
                     Core: {}\n\
                     PC: 0x{:08X}\n\
                     Cycles: {}\n\
                     LED (GPIO25): {}\n\
                     UART TX: {} bytes\n\
                     UART RX: {} bytes",
                    self.ui_state.core_name,
                    self.ui_state.pc,
                    self.ui_state.cycles,
                    if self.ui_state.gpio_values[25] { "ON" } else { "OFF" },
                    self.ui_state.uart_tx_count,
                    self.ui_state.uart_rx_count
                )
            }
            "echo" => {
                if parts.len() > 1 {
                    parts[1..].join(" ")
                } else {
                    String::new()
                }
            }
            "clear" => {
                self.ui_state.uart_output.clear();
                "Terminal cleared".to_string()
            }
            _ => {
                format!("Unknown command: {}. Type 'help' for available commands.", parts[0])
            }
        }
    }

    /// Handle console command.
    fn handle_console_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.split_whitespace().collect();

        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "help" => {
                self.ui_state.console_output.push("Available commands:".to_string());
                self.ui_state.console_output.push("  help          - Show this help".to_string());
                self.ui_state.console_output.push("  run           - Start simulation".to_string());
                self.ui_state.console_output.push("  stop          - Stop simulation".to_string());
                self.ui_state.console_output.push("  step [n]      - Step n instructions (default 1)".to_string());
                self.ui_state.console_output.push("  reset         - Reset simulation".to_string());
                self.ui_state.console_output.push("  reg [n]       - Show register n".to_string());
                self.ui_state.console_output.push("  pc            - Show PC".to_string());
                self.ui_state.console_output.push("  bp <addr>     - Add breakpoint".to_string());
                self.ui_state.console_output.push("  gpio <pin>    - Toggle GPIO pin".to_string());
                self.ui_state.console_output.push("  speed <n>     - Set simulation speed".to_string());
            }
            "run" => {
                self.paused = false;
                self.ui_state.running = true;
                self.ui_state.console_output.push("Simulation started".to_string());
            }
            "stop" => {
                self.paused = true;
                self.ui_state.running = false;
                self.ui_state.console_output.push("Simulation stopped".to_string());
            }
            "step" => {
                let steps: u64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
                if let Some(ref mut soc) = self.soc {
                    let mut result = Ok(());
                    for _ in 0..steps {
                        if let Err(e) = soc.step() {
                            result = Err(e);
                            break;
                        }
                    }
                    if let Err(e) = result {
                        self.ui_state.console_output.push(format!("Error: {}", e));
                    } else {
                        self.ui_state.console_output.push(format!("Stepped {} instructions", steps));
                    }
                    self.update_ui_state();
                }
            }
            "reset" => {
                self.soc = Some(Soc::new(self.config.sim.cpu_arch));
                self.paused = true;
                self.ui_state.running = false;
                self.ui_state.console_output.push("Simulation reset".to_string());
            }
            "reg" => {
                if let Some(reg_str) = parts.get(1) {
                    if let Ok(reg) = usize::from_str_radix(reg_str, 10) {
                        if reg < 16 {
                            self.ui_state.console_output.push(format!("R{} = 0x{:08X}", reg, self.ui_state.registers[reg]));
                        } else {
                            self.ui_state.console_output.push("Invalid register number (0-15)".to_string());
                        }
                    } else {
                        self.ui_state.console_output.push("Invalid register number".to_string());
                    }
                } else {
                    let mut s = String::new();
                    for i in 0..16 {
                        s.push_str(&format!("R{}=0x{:08X} ", i, self.ui_state.registers[i]));
                        if i % 4 == 3 {
                            s.push('\n');
                        }
                    }
                    self.ui_state.console_output.push(s);
                }
            }
            "pc" => {
                self.ui_state.console_output.push(format!("PC = 0x{:08X}", self.ui_state.pc));
            }
            "bp" => {
                if let Some(addr_str) = parts.get(1) {
                    if let Ok(addr) = u32::from_str_radix(addr_str, 16) {
                        self.debugger.add_breakpoint(addr);
                        self.ui_state.console_output.push(format!("Breakpoint added at 0x{:08X}", addr));
                    } else {
                        self.ui_state.console_output.push("Invalid address".to_string());
                    }
                } else {
                    self.ui_state.console_output.push("Usage: bp <address>".to_string());
                }
            }
            "gpio" => {
                if let Some(pin_str) = parts.get(1) {
                    if let Ok(pin) = usize::from_str_radix(pin_str, 10) {
                        if let Some(ref mut soc) = self.soc {
                            soc.toggle_gpio(pin);
                            self.ui_state.console_output.push(format!("GPIO {} toggled", pin));
                        } else {
                            self.ui_state.console_output.push("SoC not initialized".to_string());
                        }
                    } else {
                        self.ui_state.console_output.push("Invalid pin number".to_string());
                    }
                } else {
                    self.ui_state.console_output.push("Usage: gpio <pin>".to_string());
                }
            }
            "speed" => {
                if let Some(speed_str) = parts.get(1) {
                    if let Ok(speed) = speed_str.parse() {
                        self.sim_speed = speed;
                        self.ui_state.console_output.push(format!("Simulation speed set to {}", speed));
                    } else {
                        self.ui_state.console_output.push("Invalid speed value".to_string());
                    }
                } else {
                    self.ui_state.console_output.push(format!("Current speed: {}", self.sim_speed));
                }
            }
            _ => {
                self.ui_state.console_output.push(format!("Unknown command: {}. Type 'help' for available commands.", parts[0]));
            }
        }
    }

    /// Update UI state from simulation.
    pub fn update_ui_state(&mut self) {
        if let Some(ref soc) = self.soc {
            self.ui_state.pc = soc.pc();
            self.ui_state.sp = soc.sp();
            self.ui_state.lr = soc.lr();
            self.ui_state.cycles = soc.cycles();
            self.ui_state.instructions = soc.instructions();

            // Update registers
            for i in 0..16 {
                self.ui_state.registers[i] = soc.read_reg(i);
            }

            // Update flags
            self.ui_state.flags = soc.flags();

            // Update GPIO
            for pin in 0..30 {
                self.ui_state.gpio_values[pin] = soc.gpio_value(pin);
                self.ui_state.gpio_directions[pin] = soc.gpio_direction(pin);
            }

            // Update memory view (256 bytes window at SRAM base)
            self.ui_state.memory = soc.read_memory(0x20000000, 256);

            // Update peripheral state
            self.update_peripheral_state();
        }
    }

    /// Update peripheral state from simulation.
    pub fn update_peripheral_state(&mut self) {
        if let Some(ref soc) = self.soc {
            // Update GPIO state
            for pin in 0..48 {
                self.peripheral_state.gpio_values[pin] = soc.gpio_value(pin);
                self.peripheral_state.gpio_directions[pin] = soc.gpio_direction(pin);
            }

            // Update timer
            self.peripheral_state.timer_value = soc.timer_value();
            self.peripheral_state.timer_running = soc.timer_running();

            // Update ADC values
            for ch in 0..4 {
                self.peripheral_state.adc_values[ch] = soc.adc_value(ch);
            }

            // Update PWM duty cycles
            for ch in 0..24 {
                self.peripheral_state.pwm_duty[ch] = soc.pwm_duty(ch);
            }

            // Update UART state
            for uart_idx in 0..2 {
                self.peripheral_state.uart[uart_idx].enabled = soc.uart_enabled(uart_idx);
                self.peripheral_state.uart[uart_idx].baud_rate = soc.uart_baud_rate(uart_idx);
                self.peripheral_state.uart[uart_idx].tx_count = soc.uart_tx_len(uart_idx) as u64;
                self.peripheral_state.uart[uart_idx].rx_count = soc.uart_rx_len(uart_idx) as u64;
            }

            // Update SPI state
            for spi_idx in 0..2 {
                self.peripheral_state.spi[spi_idx].enabled = soc.spi_enabled(spi_idx);
                self.peripheral_state.spi[spi_idx].clock_rate = soc.spi_clock_rate(spi_idx);
                self.peripheral_state.spi[spi_idx].cpol = soc.spi_cpol(spi_idx);
                self.peripheral_state.spi[spi_idx].cpha = soc.spi_cpha(spi_idx);
            }

            // Update I2C state
            for i2c_idx in 0..2 {
                self.peripheral_state.i2c[i2c_idx].enabled = soc.i2c_enabled(i2c_idx);
                self.peripheral_state.i2c[i2c_idx].clock_rate = soc.i2c_clock_rate(i2c_idx);
            }

            // Update PIO state
            for pio_idx in 0..2 {
                self.peripheral_state.pio[pio_idx].enabled = soc.pio_enabled(pio_idx);
                for sm_idx in 0..4 {
                    let sm = &mut self.peripheral_state.pio[pio_idx].state_machines[sm_idx];
                    sm.enabled = soc.pio_sm_enabled(pio_idx, sm_idx);
                    sm.pc = soc.pio_sm_pc(pio_idx, sm_idx);
                    sm.executing = sm.enabled;
                }
            }

            // Update USB state
            self.peripheral_state.usb_connected = soc.usb_connected();
            self.peripheral_state.usb_device_mode = soc.usb_device_mode();
        }
    }

    /// Handle peripheral events.
    pub fn handle_peripheral_events(&mut self) {
        let events: Vec<PeripheralEvent> = self.peripheral_state.events.drain(..).collect();

        for event in events {
            match event {
                PeripheralEvent::GpioToggle(pin, value) => {
                    if let Some(ref mut soc) = self.soc {
                        soc.set_gpio_input(pin, value);
                        tracing::debug!("GPIO {} set to {}", pin, value);
                    }
                }
                PeripheralEvent::GpioSetDirection(_pin, _output) => {
                    // Not implemented in SoC yet
                }
                PeripheralEvent::GpioSetFunction(_pin, _func) => {
                    // Not implemented in SoC yet
                }
                PeripheralEvent::UartSend(uart_idx, data) => {
                    if let Some(ref mut soc) = self.soc {
                        for byte in &data {
                            soc.uart_push_rx(uart_idx, *byte);
                        }
                        tracing::debug!("UART{} sent {} bytes", uart_idx, data.len());
                    }
                }
                PeripheralEvent::UartReceive(_uart_idx, _data) => {
                    // Handled by simulation
                }
                PeripheralEvent::UartSetBaud(_uart_idx, _baud) => {
                    // Not implemented in SoC yet
                }
                PeripheralEvent::SpiTransfer(spi_idx, data) => {
                    if let Some(ref mut soc) = self.soc {
                        for byte in &data {
                            soc.spi_transfer(spi_idx, *byte);
                        }
                    }
                }
                PeripheralEvent::I2cTransaction(i2c_idx, txn) => {
                    if let Some(ref mut soc) = self.soc {
                        if txn.read {
                            soc.i2c_read(i2c_idx, txn.address, txn.data.len() as u8);
                        } else {
                            soc.i2c_write(i2c_idx, txn.address, &txn.data);
                        }
                    }
                }
                PeripheralEvent::PioLoadProgram(pio_idx, program) => {
                    if let Some(ref mut soc) = self.soc {
                        soc.pio_load_program(pio_idx, &program);
                        tracing::info!("PIO{} loaded {} instructions", pio_idx, program.len());
                    }
                }
                PeripheralEvent::PioStartSm(pio_idx, sm_idx) => {
                    if let Some(ref mut soc) = self.soc {
                        soc.pio_start_sm(pio_idx, sm_idx as usize);
                    }
                }
                PeripheralEvent::PioStopSm(pio_idx, sm_idx) => {
                    if let Some(ref mut soc) = self.soc {
                        soc.pio_stop_sm(pio_idx, sm_idx as usize);
                    }
                }
                PeripheralEvent::AdcSetValue(ch, value) => {
                    if let Some(ref mut soc) = self.soc {
                        soc.set_adc_value(ch as usize, value);
                    }
                }
                PeripheralEvent::PwmSetDuty(ch, duty) => {
                    if let Some(ref mut soc) = self.soc {
                        soc.set_pwm_duty(ch as usize, duty);
                    }
                }
                PeripheralEvent::MemoryViewGoto(addr) => {
                    // Set memory view base address
                    self.peripheral_state.memory_base = addr;
                    tracing::debug!("Memory view goto: 0x{:08X}", addr);
                }
            }
        }
    }

    /// Step simulation.
    pub fn step_simulation(&mut self) -> Result<()> {
        // Handle peripheral events first
        self.handle_peripheral_events();

        if let Some(ref mut soc) = self.soc {
            if !self.paused {
                for _ in 0..self.sim_speed {
                    // Check breakpoints
                    if self.debugger.has_breakpoint(soc.pc()) {
                        self.paused = true;
                        self.ui_state.running = false;
                        self.ui_state.console_output.push(
                            format!("Breakpoint hit at 0x{:08X}", soc.pc())
                        );
                        break;
                    }

                    soc.step()?;

                    if self.single_step {
                        self.single_step = false;
                        self.paused = true;
                        break;
                    }
                }

                self.update_ui_state();
                
                // Check for UART output
                self.check_uart_output();
            }
            
            // Run demo mode if no firmware is loaded
            self.run_demo_mode();
        }
        Ok(())
    }
    
    /// Check for UART output from simulation.
    fn check_uart_output(&mut self) {
        if let Some(ref mut soc) = self.soc {
            // Read any data from UART0 TX FIFO
            while soc.uart_tx_has_data(0) {
                if let Some(byte) = soc.uart_pop_tx(0) {
                    // Convert byte to character
                    if byte >= 0x20 && byte < 0x7F {
                        self.ui_state.uart_output.push((byte as char).to_string());
                    } else if byte == b'\n' {
                        self.ui_state.uart_output.push("\n".to_string());
                    } else if byte == b'\r' {
                        // Ignore CR
                    } else {
                        self.ui_state.uart_output.push(format!("[0x{:02X}]", byte));
                    }
                    self.ui_state.uart_rx_count += 1;
                    self.ui_state.uart_rx_active = true;
                }
            }
        }
    }
    
    /// Run demo mode when no firmware is loaded.
    fn run_demo_mode(&mut self) {
        // Demo mode: simulate UART echo and LED control
        // This runs when the simulation is running but no real firmware is executing
        if self.demo_counter == 0 {
            // Initial welcome message
            self.ui_state.uart_output.push("RP2350 Simulator Demo Mode\n".to_string());
            self.ui_state.uart_output.push("Type 'help' for commands\n".to_string());
            self.ui_state.uart_output.push("> ".to_string());
        }
        
        self.demo_counter += 1;
        
        // Clear UART activity indicators after a short time
        if self.demo_counter % 60 == 0 {
            self.ui_state.clear_uart_activity();
        }
    }

    /// Draw UI.
    pub fn draw_ui(&mut self, ctx: &egui::Context) {
        // Process file dialogs
        if let Some(path) = self.firmware_dialog.show(ctx) {
            self.pending_firmware = Some(path);
        }
        if let Some(path) = self.save_dialog.show(ctx) {
            // Save state to file
            if let Some(ref soc) = self.soc {
                match rp2350sim_save::save_state(soc) {
                    Ok(data) => {
                        if let Err(e) = std::fs::write(&path, data) {
                            self.ui_state.console_output.push(format!("Error saving state: {}", e));
                        } else {
                            self.ui_state.console_output.push(format!("State saved to: {}", path));
                        }
                    }
                    Err(e) => {
                        self.ui_state.console_output.push(format!("Error saving state: {}", e));
                    }
                }
            }
        }
        
        // Load pending firmware
        if let Some(path) = self.pending_firmware.take() {
            let _ = self.load_firmware(&path);
        }
        
        self.ui.draw(ctx, &mut self.ui_state);

        // Draw peripheral panels window
        if self.ui_state.show_peripherals {
            egui::Window::new("Peripherals")
                .default_size([600.0, 400.0])
                .show(ctx, |ui| {
                    self.peripheral_panels.draw(ui, &mut self.peripheral_state);
                });
        }
    }

    /// Get peripheral state.
    pub fn peripheral_state(&mut self) -> &mut PeripheralState {
        &mut self.peripheral_state
    }

    /// Get peripheral panels manager.
    pub fn peripheral_panels(&mut self) -> &mut PeripheralPanelManager {
        &mut self.peripheral_panels
    }

    /// Toggle peripheral panels visibility.
    pub fn toggle_peripherals(&mut self) {
        self.ui_state.show_peripherals = !self.ui_state.show_peripherals;
    }

    /// Check if peripheral panels are visible.
    pub fn is_peripherals_visible(&self) -> bool {
        self.ui_state.show_peripherals
    }

    /// Get simulation speed.
    pub fn sim_speed(&self) -> u64 {
        self.sim_speed
    }

    /// Set simulation speed.
    pub fn set_sim_speed(&mut self, speed: u64) {
        self.sim_speed = speed;
    }

    /// Check if paused.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Set paused state.
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
        self.ui_state.running = !paused;
    }

    /// Get the configuration.
    pub fn get_config(&self) -> &Config {
        &self.config
    }

    /// Clone the configuration.
    pub fn clone_config(&self) -> Config {
        self.config.clone()
    }
}