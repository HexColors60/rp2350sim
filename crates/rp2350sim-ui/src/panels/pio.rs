//! PIO panel for RP2350 simulator.

use super::{PeripheralEvent, PeripheralState};
use egui::{Color32, RichText, Ui, Vec2};

/// PIO panel with state machine monitoring and program control.
pub struct PioPanel {
    selected_pio: usize,
    selected_sm: usize,
    program_input: String,
    show_pin_signals: bool,
    show_fifo_detail: bool,
}

impl Default for PioPanel {
    fn default() -> Self {
        Self {
            selected_pio: 0,
            selected_sm: 0,
            program_input: String::new(),
            show_pin_signals: true,
            show_fifo_detail: true,
        }
    }
}

impl PioPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(&self) -> &'static str {
        "PIO"
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        ui.vertical(|ui| {
            // PIO selector
            ui.horizontal(|ui| {
                ui.label(RichText::new("PIO Panel").strong());
                ui.separator();
                ui.selectable_value(&mut self.selected_pio, 0, "PIO0");
                ui.selectable_value(&mut self.selected_pio, 1, "PIO1");
                ui.separator();
                ui.checkbox(&mut self.show_pin_signals, "Pins");
                ui.checkbox(&mut self.show_fifo_detail, "FIFOs");
            });
            ui.separator();

            // Status bar
            self.draw_status_bar(ui, state);

            ui.add_space(4.0);

            // State machine tabs
            let selected_sm = self.selected_sm;
            let selected_pio = self.selected_pio;
            ui.horizontal(|ui| {
                for sm in 0..4 {
                    let sm_state = &state.pio[selected_pio].state_machines[sm];
                    let label = format!("SM{} {}", sm, if sm_state.enabled { "●" } else { "○" });
                    let color = if sm_state.enabled { Color32::GREEN } else { Color32::GRAY };
                    if ui.selectable_label(selected_sm == sm, RichText::new(label).color(color)).clicked() {
                        self.selected_sm = sm;
                    }
                }
            });
            ui.separator();

            // State machine details
            self.draw_state_machine(ui, state);

            ui.add_space(4.0);

            // Program memory
            self.draw_program_memory(ui, state);
        });
    }

    fn draw_status_bar(&self, ui: &mut Ui, state: &PeripheralState) {
        let pio = &state.pio[self.selected_pio];
        ui.horizontal(|ui| {
            // Enable status
            let enable_color = if pio.enabled { Color32::GREEN } else { Color32::RED };
            let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
            ui.painter().circle_filled(rect.center(), 5.0, enable_color);
            ui.label(if pio.enabled { "Enabled" } else { "Disabled" });

            ui.separator();

            // Active SMs
            let active_sms = pio.state_machines.iter().filter(|sm| sm.enabled).count();
            ui.label(format!("Active SMs: {}/4", active_sms));

            ui.separator();

            // Program status
            ui.label(if pio.program_loaded { "Program Loaded" } else { "No Program" });

            ui.separator();

            // IRQ status
            if pio.irq_flags != 0 {
                ui.label(RichText::new(format!("IRQ: {:08X}", pio.irq_flags)).color(Color32::YELLOW));
            }
        });
    }

    fn draw_state_machine(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        let pio_idx = self.selected_pio;
        let sm_idx = self.selected_sm;
        let sm = &state.pio[pio_idx].state_machines[sm_idx];
        let enabled = sm.enabled;
        let pc = sm.pc;
        let clock_div = sm.clock_div;
        let executing = sm.executing;
        let stalled = sm.stalled;
        let tx_fifo = sm.tx_fifo.clone();
        let rx_fifo = sm.rx_fifo.clone();
        let pins_out = sm.pins_out;
        let pins_in = sm.pins_in;
        let out_pins = sm.out_pins;
        let in_pins = sm.in_pins;
        let side_set_pins = sm.side_set_pins;

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("State Machine {}", sm_idx)).strong());

                // Enable/disable button
                let enable_text = if enabled { "Disable" } else { "Enable" };
                if ui.button(enable_text).clicked() {
                    if enabled {
                        state.events.push(PeripheralEvent::PioStopSm(pio_idx, sm_idx as u8));
                    } else {
                        state.events.push(PeripheralEvent::PioStartSm(pio_idx, sm_idx as u8));
                    }
                    state.pio[pio_idx].state_machines[sm_idx].enabled = !enabled;
                }

                // Reset button
                if ui.small_button("Reset").clicked() {
                    state.pio[pio_idx].state_machines[sm_idx].pc = 0;
                }
            });
            ui.separator();

            // Status indicators
            ui.horizontal(|ui| {
                let exec_color = if executing { Color32::GREEN } else { Color32::GRAY };
                let (rect, _) = ui.allocate_exact_size(Vec2::new(10.0, 10.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 4.0, exec_color);
                ui.label(if executing { "Executing" } else { "Idle" });

                let stall_color = if stalled { Color32::RED } else { Color32::GRAY };
                let (rect, _) = ui.allocate_exact_size(Vec2::new(10.0, 10.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 4.0, stall_color);
                ui.label(if stalled { "Stalled" } else { "Running" });
            });

            ui.add_space(4.0);

            // PC and clock divider
            egui::Grid::new("sm_config").spacing([10.0, 4.0]).show(ui, |ui| {
                ui.label("PC:");
                ui.horizontal(|ui| {
                    ui.monospace(RichText::new(format!("0x{:02X}", pc)).color(Color32::YELLOW));
                    ui.label(RichText::new(format!("({})", pc)).color(Color32::GRAY).size(10.0));
                });
                ui.end_row();

                ui.label("Clock Div:");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut state.pio[pio_idx].state_machines[sm_idx].clock_div));
                    let freq = if clock_div > 0 { 125_000_000.0 / clock_div as f64 } else { 0.0 };
                    ui.label(RichText::new(format!("({:.1} MHz)", freq / 1_000_000.0)).color(Color32::GRAY).size(10.0));
                });
                ui.end_row();

                ui.label("Pin Config:");
                ui.horizontal(|ui| {
                    ui.label(format!("OUT: {}-{}", out_pins, out_pins + 7));
                    ui.label(format!("IN: {}-{}", in_pins, in_pins + 7));
                    if side_set_pins > 0 {
                        ui.label(format!("SIDE: {}", side_set_pins));
                    }
                });
                ui.end_row();
            });

            ui.add_space(4.0);

            // FIFOs with visual bars
            if self.show_fifo_detail {
                self.draw_fifo_panel(ui, &tx_fifo, &rx_fifo);
            } else {
                ui.horizontal(|ui| {
                    ui.label("TX FIFO:");
                    let tx_color = if tx_fifo.is_empty() { Color32::GRAY } else { Color32::GREEN };
                    ui.label(RichText::new(format!("{}/8", tx_fifo.len())).color(tx_color));
                    ui.label("RX FIFO:");
                    let rx_color = if rx_fifo.is_empty() { Color32::GRAY } else { Color32::from_rgb(100, 200, 255) };
                    ui.label(RichText::new(format!("{}/8", rx_fifo.len())).color(rx_color));
                });
            }

            ui.add_space(4.0);

            // Pin state visualization
            if self.show_pin_signals {
                self.draw_pin_panel(ui, pins_out, pins_in);
            }
        });
    }

    fn draw_fifo_panel(&self, ui: &mut Ui, tx_fifo: &[u32], rx_fifo: &[u32]) {
        ui.label(RichText::new("FIFOs").strong());

        egui::Grid::new("fifo_grid").spacing([8.0, 4.0]).show(ui, |ui| {
            // TX FIFO
            ui.label("TX:");
            let bar_width = 120.0;
            let bar_height = 16.0;
            let (rect, _) = ui.allocate_exact_size(Vec2::new(bar_width, bar_height), egui::Sense::hover());

            // Background
            ui.painter().rect_filled(rect, 2.0, Color32::from_rgb(30, 30, 40));

            // Fill segments
            let segment_width = bar_width / 8.0;
            for i in 0..8 {
                let filled = i < tx_fifo.len();
                let seg_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.left() + i as f32 * segment_width + 1.0, rect.top() + 1.0),
                    Vec2::new(segment_width - 2.0, bar_height - 2.0),
                );
                let color = if filled { Color32::from_rgb(255, 200, 100) } else { Color32::from_rgb(50, 50, 60) };
                ui.painter().rect_filled(seg_rect, 1.0, color);
            }

            ui.label(RichText::new(format!("{}/8", tx_fifo.len())).color(Color32::YELLOW));
            ui.end_row();

            // RX FIFO
            ui.label("RX:");
            let (rect, _) = ui.allocate_exact_size(Vec2::new(bar_width, bar_height), egui::Sense::hover());

            ui.painter().rect_filled(rect, 2.0, Color32::from_rgb(30, 30, 40));

            for i in 0..8 {
                let filled = i < rx_fifo.len();
                let seg_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.left() + i as f32 * segment_width + 1.0, rect.top() + 1.0),
                    Vec2::new(segment_width - 2.0, bar_height - 2.0),
                );
                let color = if filled { Color32::from_rgb(100, 200, 255) } else { Color32::from_rgb(50, 50, 60) };
                ui.painter().rect_filled(seg_rect, 1.0, color);
            }

            ui.label(RichText::new(format!("{}/8", rx_fifo.len())).color(Color32::from_rgb(100, 200, 255)));
            ui.end_row();
        });

        // FIFO contents
        if !tx_fifo.is_empty() {
            ui.collapsing("TX FIFO Contents", |ui| {
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    let hex: String = tx_fifo.iter()
                        .map(|v| format!("{:08X} ", v))
                        .collect();
                    ui.monospace(RichText::new(hex).color(Color32::YELLOW).size(10.0));
                });
            });
        }

        if !rx_fifo.is_empty() {
            ui.collapsing("RX FIFO Contents", |ui| {
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    let hex: String = rx_fifo.iter()
                        .map(|v| format!("{:08X} ", v))
                        .collect();
                    ui.monospace(RichText::new(hex).color(Color32::from_rgb(100, 200, 255)).size(10.0));
                });
            });
        }
    }

    fn draw_pin_panel(&self, ui: &mut Ui, pins_out: u32, pins_in: u32) {
        ui.label(RichText::new("Pin State").strong());

        egui::Grid::new("pin_grid").spacing([4.0, 4.0]).show(ui, |ui| {
            ui.label("OUT:");
            self.draw_pin_bits(ui, pins_out, Color32::GREEN);
            ui.end_row();

            ui.label("IN:");
            self.draw_pin_bits(ui, pins_in, Color32::from_rgb(100, 200, 255));
            ui.end_row();
        });
    }

    fn draw_pin_bits(&self, ui: &mut Ui, value: u32, active_color: Color32) {
        for i in 0..8 {
            let bit = (value >> i) & 1 != 0;
            let color = if bit { active_color } else { Color32::from_rgb(40, 40, 50) };
            let (rect, _) = ui.allocate_exact_size(Vec2::new(16.0, 16.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 2.0, color);
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("{}", i),
                egui::FontId::proportional(9.0),
                if bit { Color32::BLACK } else { Color32::GRAY },
            );
        }
    }

    fn draw_program_memory(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        let pio_idx = self.selected_pio;
        let sm_idx = self.selected_sm;
        let pc = state.pio[pio_idx].state_machines[sm_idx].pc;
        let instruction_memory = state.pio[pio_idx].instruction_memory;
        let _program_loaded = state.pio[pio_idx].program_loaded;

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Program Memory").strong());
                ui.label(RichText::new("(32 instructions)").color(Color32::GRAY).size(10.0));
            });
            ui.separator();

            // Instruction memory display
            egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                egui::Grid::new("pio_instructions").spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label(RichText::new("Addr").strong().size(10.0));
                    ui.label(RichText::new("Hex").strong().size(10.0));
                    ui.label(RichText::new("Disassembly").strong().size(10.0));
                    ui.end_row();

                    for (addr, instr) in instruction_memory.iter().enumerate() {
                        // Skip empty instructions
                        if *instr == 0 && addr > 0 && instruction_memory[addr - 1] == 0 {
                            continue;
                        }

                        let is_pc = pc as usize == addr;
                        let bg_color = if is_pc { Color32::from_rgb(40, 60, 40) } else { Color32::TRANSPARENT };

                        ui.scope(|ui| {
                            ui.set_clip_rect(ui.max_rect());
                            let rect = ui.max_rect();
                            ui.painter().rect_filled(rect, 0.0, bg_color);

                            ui.label(RichText::new(format!("{:02X}", addr)).size(10.0));
                            ui.monospace(RichText::new(format!("{:04X}", instr)).color(Color32::YELLOW).size(10.0));
                            ui.monospace(RichText::new(self.disassemble_pio(*instr)).color(Color32::WHITE).size(10.0));
                        });
                        ui.end_row();
                    }
                });
            });

            ui.add_space(8.0);

            // Program loader
            ui.label(RichText::new("Load Program").strong());
            ui.add(
                egui::TextEdit::multiline(&mut self.program_input)
                    .desired_width(ui.available_width())
                    .desired_rows(3)
                    .hint_text("Enter PIO program (hex, one instruction per line)")
            );

            ui.horizontal(|ui| {
                if ui.button("Load").clicked() {
                    if let Ok(program) = self.parse_pio_program() {
                        state.events.push(PeripheralEvent::PioLoadProgram(pio_idx, program));
                        state.pio[pio_idx].program_loaded = true;
                    }
                }
                if ui.button("Clear").clicked() {
                    self.program_input.clear();
                }
            });

            // Example programs
            ui.label("Examples:");
            ui.horizontal(|ui| {
                if ui.small_button("Blink").clicked() {
                    // Simple blink program
                    self.program_input = "; PIO Blink example\nE081  ; SET PINDIRS, 1\n6021  ; MOV PIN, 1\n0040  ; JMP 0\n".to_string();
                }
                if ui.small_button("Square Wave").clicked() {
                    self.program_input = "; Square wave\nE081  ; SET PINDIRS, 1\n6021  ; MOV PIN, 1\n6020  ; MOV PIN, 0\n0040  ; JMP 0\n".to_string();
                }
                if ui.small_button("Shift Out").clicked() {
                    self.program_input = "; Shift data out\n8020  ; PULL\n6021  ; OUT PIN, 1\n0000  ; JMP 0\n".to_string();
                }
            });
        });
    }

    fn disassemble_pio(&self, instr: u16) -> String {
        // PIO instruction decoding
        let major = (instr >> 13) & 0x7;

        match major {
            0b000 => {
                // JMP
                let condition = (instr >> 5) & 0x7;
                let addr = instr & 0x1F;
                let cond_str = match condition {
                    0 => "",
                    1 => "!X ",
                    2 => "X-- ",
                    3 => "!Y ",
                    4 => "Y-- ",
                    5 => "X!=Y ",
                    6 => "PIN ",
                    7 => "!OSRE ",
                    _ => "",
                };
                format!("JMP {}0x{:02X}", cond_str, addr)
            }
            0b001 => {
                // WAIT
                let polarity = (instr >> 7) & 0x1;
                let source = (instr >> 5) & 0x3;
                let index = instr & 0x1F;
                let source_str = match source {
                    0 => "GPIO",
                    1 => "PIN",
                    2 => "IRQ",
                    _ => "???",
                };
                let pol_str = if polarity == 1 { "1 " } else { "0 " };
                format!("WAIT {}{} {}", pol_str, source_str, index)
            }
            0b010 => {
                // IN
                let source = (instr >> 5) & 0x7;
                let bits = instr & 0x1F;
                let source_str = match source {
                    0 => "PINS",
                    1 => "X",
                    2 => "Y",
                    3 => "NULL",
                    6 => "ISR",
                    7 => "OSR",
                    _ => "???",
                };
                format!("IN {}, {}", source_str, bits)
            }
            0b011 => {
                // OUT
                let dest = (instr >> 5) & 0x7;
                let bits = instr & 0x1F;
                let dest_str = match dest {
                    0 => "PINS",
                    1 => "X",
                    2 => "Y",
                    3 => "NULL",
                    4 => "PINDIRS",
                    5 => "PC",
                    6 => "ISR",
                    7 => "EXEC",
                    _ => "???",
                };
                format!("OUT {}, {}", dest_str, bits)
            }
            0b100 => {
                // PUSH
                let iffull = (instr >> 6) & 0x1;
                let block = (instr >> 5) & 0x1;
                let mut flags = String::new();
                if iffull != 0 { flags.push_str("IFFULL "); }
                if block != 0 { flags.push_str("BLOCK "); }
                format!("PUSH {}", flags)
            }
            0b101 => {
                // PULL
                let ifempty = (instr >> 6) & 0x1;
                let block = (instr >> 5) & 0x1;
                let mut flags = String::new();
                if ifempty != 0 { flags.push_str("IFEMPTY "); }
                if block != 0 { flags.push_str("BLOCK "); }
                format!("PULL {}", flags)
            }
            0b110 => {
                // MOV
                let dest = (instr >> 5) & 0x7;
                let op = (instr >> 3) & 0x3;
                let source = instr & 0x7;
                let dest_str = match dest {
                    0 => "PINS",
                    1 => "X",
                    2 => "Y",
                    3 => "EXEC",
                    4 => "PC",
                    5 => "ISR",
                    6 => "OSR",
                    _ => "???",
                };
                let op_str = match op {
                    0 => "",
                    1 => "~",
                    2 => "::",
                    _ => "",
                };
                let source_str = match source {
                    0 => "PINS",
                    1 => "X",
                    2 => "Y",
                    3 => "NULL",
                    4 => "STATUS",
                    5 => "ISR",
                    6 => "OSR",
                    _ => "???",
                };
                format!("MOV {}, {}{}", dest_str, op_str, source_str)
            }
            0b111 => {
                // IRQ or SET
                let _delay_side = (instr >> 8) & 0x1F;
                if (instr >> 5) & 0x1 == 0 {
                    // SET
                    let dest = (instr >> 5) & 0x7;
                    let data = instr & 0x1F;
                    let dest_str = match dest {
                        0 => "PINS",
                        1 => "X",
                        2 => "Y",
                        4 => "PINDIRS",
                        _ => "???",
                    };
                    format!("SET {}, {}", dest_str, data)
                } else {
                    // IRQ
                    let clear = (instr >> 6) & 0x1;
                    let wait = (instr >> 5) & 0x1;
                    let irq_num = instr & 0x1F;
                    let mut flags = String::new();
                    if clear != 0 { flags.push_str("CLEAR "); }
                    if wait != 0 { flags.push_str("WAIT "); }
                    format!("IRQ {}{}", flags, irq_num)
                }
            }
            _ => format!("UNKNOWN {:04X}", instr),
        }
    }

    fn parse_pio_program(&self) -> Result<Vec<u16>, ()> {
        let mut program = Vec::new();
        for line in self.program_input.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') || line.starts_with("//") {
                continue;
            }
            // Remove inline comments
            let line = if let Some(pos) = line.find(';') {
                &line[..pos]
            } else if let Some(pos) = line.find("//") {
                &line[..pos]
            } else {
                line
            };
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(instr) = u16::from_str_radix(line, 16) {
                program.push(instr);
            }
        }
        Ok(program)
    }
}