//! UI system for RP2350 simulator.
//!
//! Provides a graphical user interface using egui.

use egui::{Context, Color32, RichText, Ui as EguiUi, Sense};

/// UI system.
#[derive(Debug)]
pub struct Ui {
    /// CPU state view.
    cpu_view: CpuView,
    /// Memory view.
    memory_view: MemoryView,
    /// Console view.
    console_view: ConsoleView,
    /// Disassembly view.
    disasm_view: DisasmView,
    /// GPIO view.
    gpio_view: GpioView,
    /// Waveform view.
    waveform_view: WaveformView,
    /// Board view.
    board_view: BoardView,
    /// UART terminal view.
    uart_view: UartTerminalView,
}

impl Default for Ui {
    fn default() -> Self {
        Self::new()
    }
}

impl Ui {
    /// Create a new UI system.
    pub fn new() -> Self {
        Self {
            cpu_view: CpuView::new(),
            memory_view: MemoryView::new(),
            console_view: ConsoleView::new(),
            disasm_view: DisasmView::new(),
            gpio_view: GpioView::new(),
            waveform_view: WaveformView::new(),
            board_view: BoardView::new(),
            uart_view: UartTerminalView::new(),
        }
    }

    /// Draw the main UI.
    pub fn draw(&mut self, ctx: &Context, state: &mut UiState) {
        // Handle keyboard shortcuts
        ctx.input(|i| {
            // F5 - Start/Run
            if i.key_pressed(egui::Key::F5) {
                state.events.push(UiEvent::Start);
            }
            // F6 - Pause
            if i.key_pressed(egui::Key::F6) {
                state.events.push(UiEvent::Stop);
            }
            // F7 - Step
            if i.key_pressed(egui::Key::F7) {
                state.events.push(UiEvent::Step);
            }
            // F8 - Reset
            if i.key_pressed(egui::Key::F8) {
                state.events.push(UiEvent::Reset);
            }
            // Ctrl+O - Load Binary
            if i.modifiers.ctrl && i.key_pressed(egui::Key::O) {
                state.events.push(UiEvent::LoadBinary);
            }
        });

        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Load Binary...").clicked() {
                        state.events.push(UiEvent::LoadBinary);
                        ui.close_menu();
                    }
                    if ui.button("Exit").clicked() {
                        state.events.push(UiEvent::Exit);
                        ui.close_menu();
                    }
                });
                ui.menu_button("Run", |ui| {
                    if ui.button("Start").clicked() {
                        state.events.push(UiEvent::Start);
                        ui.close_menu();
                    }
                    if ui.button("Pause").clicked() {
                        state.events.push(UiEvent::Stop);
                        ui.close_menu();
                    }
                    if ui.button("Step").clicked() {
                        state.events.push(UiEvent::Step);
                        ui.close_menu();
                    }
                    if ui.button("Reset").clicked() {
                        state.events.push(UiEvent::Reset);
                        ui.close_menu();
                    }
                });
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut state.show_cpu_view, "CPU Registers");
                    ui.checkbox(&mut state.show_memory_view, "Memory");
                    ui.checkbox(&mut state.show_disasm_view, "Disassembly");
                    ui.checkbox(&mut state.show_gpio_view, "GPIO");
                    ui.checkbox(&mut state.show_waveform_view, "Waveform");
                    ui.checkbox(&mut state.show_console_view, "Console");
                    ui.checkbox(&mut state.show_board_view, "Virtual Board");
                    ui.checkbox(&mut state.show_uart_view, "UART Terminal");
                    ui.separator();
                    ui.checkbox(&mut state.show_peripherals, "Peripheral Panels");
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        state.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });

        // Toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button(">").clicked() {
                    state.events.push(UiEvent::Start);
                }
                if ui.button("||").clicked() {
                    state.events.push(UiEvent::Stop);
                }
                if ui.button("Step").clicked() {
                    state.events.push(UiEvent::Step);
                }
                if ui.button("Reset").clicked() {
                    state.events.push(UiEvent::Reset);
                }
                ui.separator();
                ui.label(format!("Speed: {}", state.sim_speed));
                ui.add(egui::Slider::new(&mut state.sim_speed, 1..=100000));
            });
        });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Core: {}", state.core_name));
                ui.separator();
                ui.label(format!("PC: 0x{:08X}", state.pc));
                ui.separator();
                ui.label(format!("Cycles: {}", state.cycles));
                ui.separator();
                if state.running {
                    ui.label(RichText::new("Running").color(Color32::GREEN));
                } else {
                    ui.label(RichText::new("Stopped").color(Color32::RED));
                }
            });
        });

        // Main panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Left panel
                ui.vertical(|ui| {
                    if state.show_board_view {
                        self.board_view.draw(ui, state);
                    }
                    if state.show_uart_view {
                        self.uart_view.draw(ui, state);
                    }
                    if state.show_memory_view {
                        self.memory_view.draw(ui, state);
                    }
                });
                
                // Right panel
                ui.vertical(|ui| {
                    if state.show_cpu_view {
                        self.cpu_view.draw(ui, state);
                    }
                    if state.show_gpio_view {
                        self.gpio_view.draw(ui, state);
                    }
                    if state.show_disasm_view {
                        self.disasm_view.draw(ui, state);
                    }
                    if state.show_waveform_view {
                        self.waveform_view.draw(ui, state);
                    }
                    if state.show_console_view {
                        self.console_view.draw(ui, state);
                    }
                });
            });
        });

        // About dialog
        if state.show_about {
            egui::Window::new("About RP2350 Simulator")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading("RP2350 Simulator");
                    ui.label("Version 0.1.0");
                    ui.separator();
                    ui.label("A full-system simulator for Raspberry Pi Pico 2 W / RP2350.");
                    ui.label("");
                    ui.label("Features:");
                    ui.label("- ARM Cortex-M33 and Hazard3 RISC-V emulation");
                    ui.label("- GPIO, UART, SPI, I2C peripherals");
                    ui.label("- Virtual board with buttons and LEDs");
                    ui.label("- UART terminal for serial communication");
                    ui.separator();
                    if ui.button("Close").clicked() {
                        state.show_about = false;
                    }
                });
        }
    }
}

/// CPU state view.
#[derive(Debug, Default)]
pub struct CpuView;

impl CpuView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw(&mut self, ui: &mut EguiUi, state: &mut UiState) {
        ui.group(|ui| {
            ui.label(RichText::new("CPU Registers").strong());
            ui.separator();
            
            egui::Grid::new("registers").show(ui, |ui| {
                for i in 0..16 {
                    let reg_name = match i {
                        13 => "SP".to_string(),
                        14 => "LR".to_string(),
                        15 => "PC".to_string(),
                        _ => format!("R{}", i),
                    };
                    ui.label(reg_name);
                    ui.monospace(format!("0x{:08X}", state.registers[i]));
                    if i % 2 == 1 {
                        ui.end_row();
                    }
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Flags:");
                let n = (state.flags >> 31) & 1 != 0;
                let z = (state.flags >> 30) & 1 != 0;
                let c = (state.flags >> 29) & 1 != 0;
                let v = (state.flags >> 28) & 1 != 0;
                ui.label(RichText::new("N").color(if n { Color32::GREEN } else { Color32::DARK_GRAY }));
                ui.label(RichText::new("Z").color(if z { Color32::GREEN } else { Color32::DARK_GRAY }));
                ui.label(RichText::new("C").color(if c { Color32::GREEN } else { Color32::DARK_GRAY }));
                ui.label(RichText::new("V").color(if v { Color32::GREEN } else { Color32::DARK_GRAY }));
            });
        });
    }
}

/// Memory view.
#[derive(Debug)]
pub struct MemoryView {
    base_address: u32,
}

impl Default for MemoryView {
    fn default() -> Self {
        Self { base_address: 0x20000000 }
    }
}

impl MemoryView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw(&mut self, ui: &mut EguiUi, state: &mut UiState) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Memory View").strong());
                ui.separator();
                ui.label(format!("Address: 0x{:08X}", self.base_address));
                ui.label(RichText::new("(SRAM)").color(Color32::DARK_GRAY));
            });
            ui.separator();

            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                let rows = 16;
                let cols = 16;
                egui::Grid::new("memory_grid").show(ui, |ui| {
                    // Header row
                    ui.label("");
                    for col in 0..cols {
                        ui.label(RichText::new(format!("{:02X}", col)).color(Color32::DARK_GRAY));
                    }
                    ui.end_row();

                    // Data rows
                    for row in 0..rows {
                        let addr = self.base_address + (row * cols) as u32;
                        ui.label(RichText::new(format!("{:08X}", addr)).color(Color32::DARK_GRAY));

                        for col in 0..cols {
                            let offset = (row * cols + col) as usize;
                            let byte = state.memory.get(offset).copied().unwrap_or(0xFF);
                            ui.monospace(RichText::new(format!("{:02X}", byte)).color(Color32::LIGHT_GRAY));
                        }
                        ui.end_row();
                    }
                });
            });
        });
    }
}

/// Console view.
#[derive(Debug)]
pub struct ConsoleView {
    input: String,
}

impl Default for ConsoleView {
    fn default() -> Self {
        Self { input: String::new() }
    }
}

impl ConsoleView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw(&mut self, ui: &mut EguiUi, state: &mut UiState) {
        ui.group(|ui| {
            ui.label(RichText::new("Console").strong());
            ui.separator();
            
            egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                for line in &state.console_output {
                    ui.monospace(RichText::new(line).color(Color32::LIGHT_GRAY));
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label(">");
                let response = ui.text_edit_singleline(&mut self.input);
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if !self.input.is_empty() {
                        state.events.push(UiEvent::ConsoleCommand(self.input.clone()));
                        self.input.clear();
                    }
                }
            });
        });
    }
}

/// Disassembly view.
#[derive(Debug, Default)]
pub struct DisasmView;

impl DisasmView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw(&mut self, ui: &mut EguiUi, state: &mut UiState) {
        ui.group(|ui| {
            ui.label(RichText::new("Disassembly").strong());
            ui.separator();

            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                if state.disassembly.is_empty() {
                    ui.label(RichText::new("No disassembly available").color(Color32::DARK_GRAY));
                    ui.label("Load firmware to see disassembly");
                } else {
                    egui::Grid::new("disasm_grid").show(ui, |ui| {
                        for (addr, instr) in &state.disassembly {
                            ui.label(RichText::new(format!("{:08X}", addr)).color(Color32::DARK_GRAY));
                            ui.monospace(RichText::new(instr).color(Color32::LIGHT_GRAY));
                            ui.end_row();
                        }
                    });
                }
            });
        });
    }
}

/// GPIO view.
#[derive(Debug, Default)]
pub struct GpioView;

impl GpioView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw(&mut self, ui: &mut EguiUi, state: &mut UiState) {
        ui.group(|ui| {
            ui.label(RichText::new("GPIO").strong());
            ui.separator();
            
            egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                egui::Grid::new("gpio_pins").show(ui, |ui| {
                    for pin in 0..30 {
                        let value = state.gpio_values.get(pin).copied().unwrap_or(false);
                        let color = if value { Color32::GREEN } else { Color32::DARK_GRAY };
                        
                        ui.horizontal(|ui| {
                            ui.label(format!("GPIO{}", pin));
                            ui.label(RichText::new(if value { "1" } else { "0" }).color(color));
                        });
                        
                        if (pin + 1) % 5 == 0 {
                            ui.end_row();
                        }
                    }
                });
            });
        });
    }
}

/// Waveform view.
#[derive(Debug, Default)]
pub struct WaveformView;

impl WaveformView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw(&mut self, ui: &mut EguiUi, state: &mut UiState) {
        ui.group(|ui| {
            ui.label(RichText::new("Waveform View").strong());
            ui.separator();

            if state.waveform_signals.is_empty() {
                ui.label(RichText::new("No signals to display").color(Color32::DARK_GRAY));
                ui.label("Run simulation to capture signals");
            } else {
                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for signal in &state.waveform_signals {
                        ui.horizontal(|ui| {
                            ui.label(&signal.name);
                            // Draw simple signal visualization
                            let (rect, _) = ui.allocate_exact_size(
                                egui::vec2(300.0, 20.0),
                                Sense::hover()
                            );
                            let step_width = rect.width() / signal.values.len().max(1) as f32;
                            for (i, (_, value)) in signal.values.iter().enumerate() {
                                let x = rect.min.x + i as f32 * step_width;
                                let y = if *value { rect.min.y } else { rect.max.y - 2.0 };
                                let height = if *value { 10.0 } else { 2.0 };
                                ui.painter().rect_filled(
                                    egui::Rect::from_min_size(
                                        egui::pos2(x, y),
                                        egui::vec2(step_width, height)
                                    ),
                                    0.0,
                                    if *value { Color32::GREEN } else { Color32::DARK_GRAY }
                                );
                            }
                        });
                    }
                });
            }
        });
    }
}

/// Virtual board view with interactive buttons and LEDs.
#[derive(Debug)]
pub struct BoardView {
    button_states: [bool; 4],
}

impl Default for BoardView {
    fn default() -> Self {
        Self { button_states: [false; 4] }
    }
}

impl BoardView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw(&mut self, ui: &mut EguiUi, state: &mut UiState) {
        ui.group(|ui| {
            ui.label(RichText::new("Virtual Board - Pico 2 W").strong());
            ui.separator();
            
            // LED indicator
            ui.horizontal(|ui| {
                ui.label("LED (GPIO25):");
                let led_on = state.gpio_values.get(25).copied().unwrap_or(false);
                let led_color = if led_on { Color32::GREEN } else { Color32::DARK_GRAY };
                let (rect, _) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), Sense::hover());
                ui.painter().circle_filled(rect.center(), 8.0, led_color);
            });
            
            ui.add_space(5.0);
            
            // Virtual buttons
            ui.label("Virtual Buttons:");
            ui.horizontal(|ui| {
                for i in 0..4 {
                    let pressed = self.button_states[i];
                    let bg_color = if pressed { Color32::from_rgb(0, 120, 200) } else { Color32::from_rgb(60, 60, 70) };
                    
                    let response = ui.add_sized(egui::vec2(50.0, 25.0), egui::Button::new(
                        RichText::new(format!("BTN{}", i)).color(if pressed { Color32::WHITE } else { Color32::GRAY })
                    ).fill(bg_color));
                    
                    if response.is_pointer_button_down_on() {
                        if !self.button_states[i] {
                            self.button_states[i] = true;
                            state.events.push(UiEvent::ButtonPress(i, true));
                        }
                    } else if self.button_states[i] {
                        self.button_states[i] = false;
                        state.events.push(UiEvent::ButtonPress(i, false));
                    }
                }
            });
            
            ui.add_space(5.0);
            
            // GPIO indicators
            ui.label("GPIO Status:");
            ui.horizontal(|ui| {
                for pin in 0..16 {
                    let value = state.gpio_values.get(pin).copied().unwrap_or(false);
                    let color = if value { Color32::from_rgb(0, 255, 100) } else { Color32::from_rgb(40, 40, 50) };
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), Sense::hover());
                    ui.painter().rect_filled(rect, 2.0, color);
                }
            });
            
            ui.add_space(5.0);
            
            // UART status
            ui.horizontal(|ui| {
                ui.label("UART0:");
                let tx_color = if state.uart_tx_active { Color32::YELLOW } else { Color32::DARK_GRAY };
                let rx_color = if state.uart_rx_active { Color32::from_rgb(0, 200, 255) } else { Color32::DARK_GRAY };
                
                let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), Sense::hover());
                ui.painter().rect_filled(rect, 2.0, tx_color);
                ui.label("TX");
                
                let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), Sense::hover());
                ui.painter().rect_filled(rect, 2.0, rx_color);
                ui.label("RX");
                
                ui.label(format!("({}/{})", state.uart_tx_count, state.uart_rx_count));
            });
        });
    }
}

/// UART Terminal view.
#[derive(Debug)]
pub struct UartTerminalView {
    input: String,
    echo_mode: bool,
}

impl Default for UartTerminalView {
    fn default() -> Self {
        Self { input: String::new(), echo_mode: true }
    }
}

impl UartTerminalView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw(&mut self, ui: &mut EguiUi, state: &mut UiState) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("UART Terminal").strong());
                ui.checkbox(&mut self.echo_mode, "Echo");
                if ui.button("Clear").clicked() {
                    state.uart_output.clear();
                }
            });
            ui.separator();
            
            // Terminal output
            let output_height = 150.0;
            egui::ScrollArea::vertical()
                .max_height(output_height)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    let output_text: String = state.uart_output.iter().take(1000).cloned().collect();
                    ui.monospace(RichText::new(&output_text).color(Color32::from_rgb(0, 255, 150)));
                });
            
            ui.separator();
            
            // Input
            ui.horizontal(|ui| {
                ui.label(">");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.input)
                        .desired_width(ui.available_width() - 100.0)
                        .hint_text("Type command...")
                );
                
                if (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) || ui.button("Send").clicked() {
                    if !self.input.is_empty() {
                        if self.echo_mode {
                            state.uart_output.push(format!("> {}\n", self.input));
                        }
                        state.events.push(UiEvent::UartSend(format!("{}\n", self.input)));
                        self.input.clear();
                        response.request_focus();
                    }
                }
            });
            
            // Quick buttons
            ui.horizontal(|ui| {
                if ui.small_button("help").clicked() {
                    state.events.push(UiEvent::UartSend("help\n".to_string()));
                    if self.echo_mode { state.uart_output.push("> help\n".to_string()); }
                }
                if ui.small_button("led on").clicked() {
                    state.events.push(UiEvent::UartSend("led on\n".to_string()));
                    if self.echo_mode { state.uart_output.push("> led on\n".to_string()); }
                }
                if ui.small_button("led off").clicked() {
                    state.events.push(UiEvent::UartSend("led off\n".to_string()));
                    if self.echo_mode { state.uart_output.push("> led off\n".to_string()); }
                }
                if ui.small_button("status").clicked() {
                    state.events.push(UiEvent::UartSend("status\n".to_string()));
                    if self.echo_mode { state.uart_output.push("> status\n".to_string()); }
                }
            });
        });
    }
}

/// UI state.
#[derive(Debug)]
pub struct UiState {
    pub core_name: String,
    pub pc: u32,
    pub sp: u32,
    pub lr: u32,
    pub registers: [u32; 16],
    pub flags: u32,
    pub cycles: u64,
    pub instructions: u64,
    pub running: bool,
    pub sim_speed: u64,
    pub memory: Vec<u8>,
    pub gpio_values: Vec<bool>,
    pub gpio_directions: Vec<bool>,
    pub console_output: Vec<String>,
    pub disassembly: Vec<(u32, String)>,
    pub waveform_signals: Vec<WaveformSignal>,
    pub events: Vec<UiEvent>,
    pub show_cpu_view: bool,
    pub show_memory_view: bool,
    pub show_disasm_view: bool,
    pub show_gpio_view: bool,
    pub show_console_view: bool,
    pub show_waveform_view: bool,
    pub show_board_view: bool,
    pub show_uart_view: bool,
    pub show_about: bool,
    pub show_peripherals: bool,
    pub uart_output: Vec<String>,
    pub uart_tx_count: u64,
    pub uart_rx_count: u64,
    pub uart_tx_active: bool,
    pub uart_rx_active: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            core_name: "ARM Cortex-M33".to_string(),
            show_cpu_view: true,
            show_memory_view: false,
            show_disasm_view: false,
            show_gpio_view: true,
            show_console_view: true,
            show_waveform_view: false,
            show_board_view: true,
            show_uart_view: true,
            memory: Vec::new(), // Don't pre-allocate large memory buffer
            gpio_values: vec![false; 48],
            gpio_directions: vec![false; 48],
            sim_speed: 1000,
            uart_output: Vec::with_capacity(100), // Pre-allocate small capacity
            uart_tx_count: 0,
            uart_rx_count: 0,
            uart_tx_active: false,
            uart_rx_active: false,
            pc: 0,
            sp: 0,
            lr: 0,
            registers: [0; 16],
            flags: 0,
            cycles: 0,
            instructions: 0,
            running: false,
            console_output: Vec::new(),
            disassembly: Vec::new(),
            waveform_signals: Vec::new(),
            events: Vec::new(),
            show_about: false,
            show_peripherals: true,
        }
    }
}

impl UiState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn uart_receive(&mut self, data: &str) {
        // Limit output buffer size
        if self.uart_output.len() > 1000 {
            self.uart_output.drain(0..100);
        }
        self.uart_output.push(data.to_string());
        self.uart_rx_count += data.len() as u64;
        self.uart_rx_active = true;
    }
    
    pub fn clear_uart_activity(&mut self) {
        self.uart_tx_active = false;
        self.uart_rx_active = false;
    }
}

/// Waveform signal.
#[derive(Debug, Clone)]
pub struct WaveformSignal {
    pub name: String,
    pub values: Vec<(u64, bool)>,
}

impl WaveformSignal {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), values: Vec::new() }
    }
}

/// UI events.
#[derive(Debug, Clone)]
pub enum UiEvent {
    Start,
    Stop,
    Step,
    Reset,
    LoadBinary,
    LoadElf,
    SaveState,
    LoadState,
    Exit,
    ConsoleCommand(String),
    BreakpointAdd(u32),
    BreakpointRemove(u32),
    GpioToggle(usize),
    ButtonPress(usize, bool),
    UartSend(String),
}