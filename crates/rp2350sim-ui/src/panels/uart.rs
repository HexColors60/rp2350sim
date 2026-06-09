//! UART panel for RP2350 simulator.
#![allow(dead_code)]

use super::{PeripheralEvent, PeripheralState, Parity};
use egui::{Color32, RichText, Ui, Vec2};

/// UART panel with terminal and configuration.
pub struct UartPanel {
    selected_uart: usize,
    input_buffer: String,
    auto_scroll: bool,
    show_hex: bool,
    show_signals: bool,
    show_timestamps: bool,
    terminal_output: Vec<TerminalLine>,
}

#[derive(Clone)]
struct TerminalLine {
    timestamp: u64,
    direction: Direction,
    data: Vec<u8>,
}

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Tx,
    Rx,
}

impl Default for UartPanel {
    fn default() -> Self {
        Self {
            selected_uart: 0,
            input_buffer: String::new(),
            auto_scroll: true,
            show_hex: false,
            show_signals: true,
            show_timestamps: true,
            terminal_output: Vec::new(),
        }
    }
}

impl UartPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(&self) -> &'static str {
        "UART"
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        ui.vertical(|ui| {
            // UART selector
            ui.horizontal(|ui| {
                ui.label(RichText::new("UART Panel").strong());
                ui.separator();
                ui.selectable_value(&mut self.selected_uart, 0, "UART0");
                ui.selectable_value(&mut self.selected_uart, 1, "UART1");
            });
            ui.separator();

            // Status bar
            self.draw_status_bar(ui, state);

            ui.add_space(4.0);

            // Signal visualization
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_signals, "Show Signals");
                ui.checkbox(&mut self.show_timestamps, "Timestamps");
            });

            if self.show_signals {
                self.draw_signal_display(ui, state);
            }

            ui.add_space(4.0);

            // Main content - Terminal
            self.draw_terminal(ui, state);

            ui.add_space(4.0);

            // Configuration
            self.draw_config(ui, state);
        });
    }

    fn draw_status_bar(&self, ui: &mut Ui, state: &PeripheralState) {
        let uart = &state.uart[self.selected_uart];
        ui.horizontal(|ui| {
            // Enable status
            let enable_color = if uart.enabled { Color32::GREEN } else { Color32::RED };
            let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
            ui.painter().circle_filled(rect.center(), 5.0, enable_color);
            ui.label(if uart.enabled { "Enabled" } else { "Disabled" });

            ui.separator();

            // Baud rate with calculated bit time
            let bit_time_us = if uart.baud_rate > 0 {
                1_000_000.0 / uart.baud_rate as f64
            } else {
                0.0
            };
            ui.label(format!("{} baud ({:.2}μs/bit)", uart.baud_rate, bit_time_us));

            ui.separator();

            // TX/RX counts
            ui.label(RichText::new(format!("TX: {}", uart.tx_count)).color(Color32::YELLOW));
            ui.label(RichText::new(format!("RX: {}", uart.rx_count)).color(Color32::from_rgb(100, 255, 200)));

            ui.separator();

            // FIFO status with visual bar
            self.draw_fifo_indicator(ui, "TX", uart.tx_fifo.len(), 32, Color32::YELLOW);
            self.draw_fifo_indicator(ui, "RX", uart.rx_fifo.len(), 32, Color32::from_rgb(100, 255, 200));

            ui.separator();

            // Flow control
            if uart.flow_control {
                ui.label(RichText::new("CTS/RTS").color(Color32::from_rgb(0, 255, 255)));
            }

            // Interrupt status
            if uart.tx_int_enabled || uart.rx_int_enabled {
                ui.label(RichText::new("IRQ").color(Color32::YELLOW));
            }

            // Error indicators
            if uart.tx_overrun || uart.rx_overrun || uart.framing_error || uart.parity_error {
                ui.label(RichText::new("ERR").color(Color32::RED));
            }
        });
    }

    fn draw_fifo_indicator(&self, ui: &mut Ui, label: &str, level: usize, max: usize, color: Color32) {
        ui.horizontal(|ui| {
            ui.label(format!("{}:", label));
            let bar_width = 60.0;
            let bar_height = 12.0;
            let (rect, _) = ui.allocate_exact_size(Vec2::new(bar_width, bar_height), egui::Sense::hover());

            // Background
            ui.painter().rect_filled(rect, 2.0, Color32::from_rgb(40, 40, 50));

            // Fill
            let fill_width = (level as f32 / max as f32) * bar_width;
            let fill_rect = egui::Rect::from_min_size(rect.min, Vec2::new(fill_width, bar_height));
            ui.painter().rect_filled(fill_rect, 2.0, color);

            // Text
            ui.label(format!("{}/{}", level, max));
        });
    }

    fn draw_signal_display(&self, ui: &mut Ui, state: &PeripheralState) {
        let uart = &state.uart[self.selected_uart];
        ui.group(|ui| {
            ui.label(RichText::new("Signal Lines").strong());
            ui.separator();

            let signal_height = 20.0;
            let signal_width = ui.available_width() - 20.0;

            egui::Grid::new("uart_signals").spacing([4.0, 2.0]).show(ui, |ui| {
                // TX line
                ui.label("TX:");
                let (rect, _) = ui.allocate_exact_size(Vec2::new(signal_width, signal_height), egui::Sense::hover());
                self.draw_signal_line(ui, rect, uart.tx_line_high, Color32::YELLOW, "TX");

                // Status indicators
                ui.horizontal(|ui| {
                    if uart.tx_line_high {
                        ui.label(RichText::new("IDLE").color(Color32::GRAY));
                    } else {
                        ui.label(RichText::new("ACTIVE").color(Color32::YELLOW));
                    }
                });
                ui.end_row();

                // RX line
                ui.label("RX:");
                let (rect, _) = ui.allocate_exact_size(Vec2::new(signal_width, signal_height), egui::Sense::hover());
                self.draw_signal_line(ui, rect, uart.rx_line_high, Color32::from_rgb(100, 255, 200), "RX");

                ui.horizontal(|ui| {
                    if uart.rx_line_high {
                        ui.label(RichText::new("IDLE").color(Color32::GRAY));
                    } else {
                        ui.label(RichText::new("ACTIVE").color(Color32::from_rgb(100, 255, 200)));
                    }
                });
                ui.end_row();

                // CTS (if flow control enabled)
                if uart.flow_control {
                    ui.label("CTS:");
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(signal_width, signal_height), egui::Sense::hover());
                    let cts_color = if uart.cts_high { Color32::GREEN } else { Color32::RED };
                    self.draw_signal_line(ui, rect, uart.cts_high, cts_color, "CTS");
                    ui.label(if uart.cts_high { "Can Send" } else { "Blocked" });
                    ui.end_row();

                    // RTS
                    ui.label("RTS:");
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(signal_width, signal_height), egui::Sense::hover());
                    let rts_color = if uart.rts_high { Color32::GREEN } else { Color32::RED };
                    self.draw_signal_line(ui, rect, uart.rts_high, rts_color, "RTS");
                    ui.label(if uart.rts_high { "Ready" } else { "Busy" });
                    ui.end_row();
                }
            });

            // Frame format display
            ui.add_space(4.0);
            let frame_info = format!(
                "Frame: {}{}{}{}  ({} start + {} data + {} parity + {} stop = {} bits)",
                uart.data_bits,
                match uart.parity {
                    Parity::None => "N",
                    Parity::Even => "E",
                    Parity::Odd => "O",
                },
                uart.stop_bits,
                if uart.flow_control { " FC" } else { "" },
                1, // start bit
                uart.data_bits,
                if uart.parity == Parity::None { 0 } else { 1 },
                uart.stop_bits,
                1 + uart.data_bits + if uart.parity == Parity::None { 0 } else { 1 } + uart.stop_bits
            );
            ui.label(RichText::new(frame_info).size(10.0).color(Color32::GRAY));
        });
    }

    fn draw_signal_line(&self, ui: &mut Ui, rect: egui::Rect, is_high: bool, color: Color32, _label: &str) {
        let painter = ui.painter();
        let h = rect.height();
        let y_low = rect.top() + h * 0.75;
        let y_high = rect.top() + h * 0.25;

        // Draw idle level (dashed line)
        let idle_color = Color32::from_rgb(60, 60, 60);
        for x in (rect.left() as i32..rect.right() as i32).step_by(6) {
            painter.line_segment(
                [egui::pos2(x as f32, y_high), egui::pos2((x + 3) as f32, y_high)],
                egui::Stroke::new(1.0, idle_color),
            );
        }

        // Draw current signal level
        let y = if is_high { y_high } else { y_low };
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            egui::Stroke::new(2.0, color),
        );

        // Draw transition if not idle
        if !is_high {
            painter.line_segment(
                [egui::pos2(rect.left(), y_high), egui::pos2(rect.left(), y_low)],
                egui::Stroke::new(2.0, color),
            );
        }
    }

    fn draw_terminal(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        let uart_idx = self.selected_uart;
        let uart = &state.uart[uart_idx];
        let tx_fifo = uart.tx_fifo.clone();
        let rx_fifo = uart.rx_fifo.clone();

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Terminal").strong());
                ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
                ui.checkbox(&mut self.show_hex, "Hex");
                if ui.button("Clear").clicked() {
                    state.uart[uart_idx].tx_fifo.clear();
                    state.uart[uart_idx].rx_fifo.clear();
                    self.terminal_output.clear();
                }
            });
            ui.separator();

            // Output area
            let output_height = 180.0;
            egui::ScrollArea::vertical()
                .max_height(output_height)
                .stick_to_bottom(self.auto_scroll)
                .show(ui, |ui| {
                    // Display TX data
                    if !tx_fifo.is_empty() {
                        ui.horizontal(|ui| {
                            if self.show_timestamps {
                                ui.label(RichText::new(format!("[{:08X}]", 0)).color(Color32::GRAY).size(10.0));
                            }
                            ui.label(RichText::new("TX>").color(Color32::YELLOW));
                            let text = if self.show_hex {
                                tx_fifo.iter().map(|b| format!("{:02X} ", b)).collect::<String>()
                            } else {
                                format_data_display(&tx_fifo)
                            };
                            ui.monospace(RichText::new(text).color(Color32::from_rgb(255, 200, 100)));
                        });
                    }

                    // Display RX data
                    if !rx_fifo.is_empty() {
                        ui.horizontal(|ui| {
                            if self.show_timestamps {
                                ui.label(RichText::new(format!("[{:08X}]", 0)).color(Color32::GRAY).size(10.0));
                            }
                            ui.label(RichText::new("RX>").color(Color32::from_rgb(100, 255, 200)));
                            let text = if self.show_hex {
                                rx_fifo.iter().map(|b| format!("{:02X} ", b)).collect::<String>()
                            } else {
                                format_data_display(&rx_fifo)
                            };
                            ui.monospace(RichText::new(text).color(Color32::from_rgb(150, 255, 220)));
                        });
                    }

                    if tx_fifo.is_empty() && rx_fifo.is_empty() {
                        ui.label(RichText::new("(No data)").color(Color32::GRAY));
                    }
                });

            ui.separator();

            // Input area
            ui.horizontal(|ui| {
                ui.label(RichText::new(">").color(Color32::YELLOW));
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.input_buffer)
                        .desired_width(ui.available_width() - 100.0)
                        .hint_text("Enter data to send...")
                );

                let send_clicked = ui.button("Send").clicked();
                let enter_pressed = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                if send_clicked || enter_pressed {
                    if !self.input_buffer.is_empty() {
                        let data = self.input_buffer.as_bytes().to_vec();
                        state.events.push(PeripheralEvent::UartSend(uart_idx, data));
                        self.input_buffer.clear();
                        response.request_focus();
                    }
                }
            });

            // Quick send buttons
            ui.horizontal(|ui| {
                if ui.small_button("AT").clicked() {
                    state.events.push(PeripheralEvent::UartSend(uart_idx, b"AT\r\n".to_vec()));
                }
                if ui.small_button("AT+RST").clicked() {
                    state.events.push(PeripheralEvent::UartSend(uart_idx, b"AT+RST\r\n".to_vec()));
                }
                if ui.small_button("AT+GMR").clicked() {
                    state.events.push(PeripheralEvent::UartSend(uart_idx, b"AT+GMR\r\n".to_vec()));
                }
                if ui.small_button("\\r\\n").clicked() {
                    self.input_buffer.push_str("\r\n");
                }
                if ui.small_button("0x00").clicked() {
                    state.events.push(PeripheralEvent::UartSend(uart_idx, vec![0x00]));
                }
                if ui.small_button("0xFF").clicked() {
                    state.events.push(PeripheralEvent::UartSend(uart_idx, vec![0xFF]));
                }
            });
        });
    }

    fn draw_config(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        let uart_idx = self.selected_uart;
        let uart = &mut state.uart[uart_idx];

        ui.group(|ui| {
            ui.label(RichText::new("Configuration").strong());
            ui.separator();

            egui::Grid::new("uart_config").spacing([10.0, 4.0]).show(ui, |ui| {
                ui.label("Enable:");
                ui.checkbox(&mut uart.enabled, "");
                ui.end_row();

                ui.label("Baud Rate:");
                egui::ComboBox::from_id_source(format!("baud_rate_{}", uart_idx))
                    .selected_text(format!("{}", uart.baud_rate))
                    .width(100.0)
                    .show_ui(ui, |ui| {
                        let bauds = [1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600];
                        for &baud in &bauds {
                            ui.selectable_value(&mut uart.baud_rate, baud, format!("{}", baud));
                        }
                    });
                ui.end_row();

                ui.label("Data Bits:");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut uart.data_bits, 7, "7");
                    ui.selectable_value(&mut uart.data_bits, 8, "8");
                });
                ui.end_row();

                ui.label("Parity:");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut uart.parity, Parity::None, "None");
                    ui.selectable_value(&mut uart.parity, Parity::Even, "Even");
                    ui.selectable_value(&mut uart.parity, Parity::Odd, "Odd");
                });
                ui.end_row();

                ui.label("Stop Bits:");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut uart.stop_bits, 1, "1");
                    ui.selectable_value(&mut uart.stop_bits, 2, "2");
                });
                ui.end_row();

                ui.label("Flow Control:");
                ui.checkbox(&mut uart.flow_control, "CTS/RTS");
                ui.end_row();

                ui.label("Interrupts:");
                ui.horizontal(|ui| {
                    ui.checkbox(&mut uart.tx_int_enabled, "TX Empty");
                    ui.checkbox(&mut uart.rx_int_enabled, "RX Full");
                });
                ui.end_row();
            });

            ui.add_space(4.0);

            // Error status
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(RichText::new("Errors:").strong());
                let tx_color = if uart.tx_overrun { Color32::RED } else { Color32::DARK_GRAY };
                let rx_color = if uart.rx_overrun { Color32::RED } else { Color32::DARK_GRAY };
                let frame_color = if uart.framing_error { Color32::RED } else { Color32::DARK_GRAY };
                let parity_color = if uart.parity_error { Color32::RED } else { Color32::DARK_GRAY };

                ui.label(RichText::new("TX-OVR").color(tx_color));
                ui.label(RichText::new("RX-OVR").color(rx_color));
                ui.label(RichText::new("FRM").color(frame_color));
                ui.label(RichText::new("PAR").color(parity_color));

                if ui.small_button("Clear").clicked() {
                    uart.tx_overrun = false;
                    uart.rx_overrun = false;
                    uart.framing_error = false;
                    uart.parity_error = false;
                }
            });

            ui.add_space(4.0);

            // Statistics
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(RichText::new("Stats:").strong());
                ui.label(format!("TX: {}", uart.tx_count));
                ui.label(format!("RX: {}", uart.rx_count));
                if ui.small_button("Reset").clicked() {
                    uart.tx_count = 0;
                    uart.rx_count = 0;
                }
            });
        });
    }
}

/// Format data for display, showing printable ASCII and hex for non-printable.
fn format_data_display(data: &[u8]) -> String {
    let mut result = String::new();
    for &b in data {
        if b >= 0x20 && b < 0x7F {
            result.push(b as char);
        } else if b == 0x0D {
            result.push_str("⏎");
        } else if b == 0x0A {
            result.push_str("↓");
        } else if b == 0x09 {
            result.push_str("→");
        } else {
            result.push_str(&format!("[{:02X}]", b));
        }
    }
    result
}