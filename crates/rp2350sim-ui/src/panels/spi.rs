//! SPI panel for RP2350 simulator.

use super::{PeripheralEvent, PeripheralState};
use egui::{Color32, RichText, Ui, Vec2};

/// SPI panel with transaction monitoring and configuration.
pub struct SpiPanel {
    selected_spi: usize,
    selected_cs: u8,
    test_data: String,
    show_signals: bool,
    show_fifo: bool,
}

impl Default for SpiPanel {
    fn default() -> Self {
        Self {
            selected_spi: 0,
            selected_cs: 0,
            test_data: String::from("00 01 02 03"),
            show_signals: true,
            show_fifo: true,
        }
    }
}

impl SpiPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(&self) -> &'static str {
        "SPI"
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        ui.vertical(|ui| {
            // SPI selector
            ui.horizontal(|ui| {
                ui.label(RichText::new("SPI Panel").strong());
                ui.separator();
                ui.selectable_value(&mut self.selected_spi, 0, "SPI0");
                ui.selectable_value(&mut self.selected_spi, 1, "SPI1");
            });
            ui.separator();

            // Status bar
            self.draw_status_bar(ui, state);

            ui.add_space(4.0);

            // Signal visualization
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_signals, "Show Signals");
                ui.checkbox(&mut self.show_fifo, "Show FIFO");
            });

            if self.show_signals {
                self.draw_signal_display(ui, state);
            }

            if self.show_fifo {
                self.draw_fifo_display(ui, state);
            }

            ui.add_space(4.0);

            // Transaction log
            self.draw_transaction_log(ui, state);

            ui.add_space(4.0);

            // Configuration
            self.draw_config(ui, state);

            ui.add_space(4.0);

            // Test panel
            self.draw_test_panel(ui, state);
        });
    }

    fn draw_status_bar(&self, ui: &mut Ui, state: &PeripheralState) {
        let spi = &state.spi[self.selected_spi];
        ui.horizontal(|ui| {
            // Enable status
            let enable_color = if spi.enabled { Color32::GREEN } else { Color32::RED };
            let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
            ui.painter().circle_filled(rect.center(), 5.0, enable_color);
            ui.label(if spi.enabled { "Enabled" } else { "Disabled" });

            ui.separator();

            // Mode
            ui.label(if spi.master_mode { "Master" } else { "Slave" });

            ui.separator();

            // Clock
            let clock_mhz = spi.clock_rate as f64 / 1_000_000.0;
            if clock_mhz >= 1.0 {
                ui.label(format!("Clock: {:.2} MHz", clock_mhz));
            } else {
                let clock_khz = spi.clock_rate as f64 / 1_000.0;
                ui.label(format!("Clock: {:.1} kHz", clock_khz));
            }

            ui.separator();

            // Mode
            let mode = (spi.cpol as u8) << 1 | spi.cpha as u8;
            ui.label(RichText::new(format!("Mode {}", mode)).color(Color32::from_rgb(100, 200, 255)));

            ui.separator();

            // Data bits
            ui.label(format!("{}-bit", spi.data_bits));

            ui.separator();

            // Bit order
            ui.label(if spi.lsb_first { "LSB" } else { "MSB" });
        });

        // Statistics bar
        ui.horizontal(|ui| {
            ui.label(RichText::new("Stats:").strong());
            ui.label(format!("TX: {}", spi.total_bytes_tx));
            ui.label(format!("RX: {}", spi.total_bytes_rx));
            ui.label(format!("Txns: {}", spi.transactions.len()));
        });
    }

    fn draw_signal_display(&self, ui: &mut Ui, state: &PeripheralState) {
        let spi = &state.spi[self.selected_spi];
        ui.group(|ui| {
            ui.label(RichText::new("Signal Lines").strong());
            ui.separator();

            let signal_height = 24.0;
            let signal_width = ui.available_width() - 20.0;

            // Draw signal lines
            egui::Grid::new("spi_signals").spacing([4.0, 2.0]).show(ui, |ui| {
                // SCK (Clock)
                ui.label("SCK:");
                let (rect, _) = ui.allocate_exact_size(Vec2::new(signal_width, signal_height), egui::Sense::hover());
                let idle_high = spi.cpol;
                let color = Color32::from_rgb(0, 200, 255);
                self.draw_clock_signal(ui, rect, idle_high, color);
                ui.end_row();

                // MOSI (Master Out)
                ui.label("MOSI:");
                let (rect, _) = ui.allocate_exact_size(Vec2::new(signal_width, signal_height), egui::Sense::hover());
                self.draw_data_signal(ui, rect, spi.tx_fifo.first().copied(), Color32::YELLOW);
                ui.end_row();

                // MISO (Master In)
                ui.label("MISO:");
                let (rect, _) = ui.allocate_exact_size(Vec2::new(signal_width, signal_height), egui::Sense::hover());
                self.draw_data_signal(ui, rect, spi.rx_fifo.first().copied(), Color32::from_rgb(100, 255, 150));
                ui.end_row();

                // CS (Chip Select)
                ui.label("CS:");
                let (rect, _) = ui.allocate_exact_size(Vec2::new(signal_width, signal_height), egui::Sense::hover());
                let cs_active = spi.cs_active;
                self.draw_cs_signal(ui, rect, cs_active);
                ui.end_row();
            });

            // Mode description
            let mode = (spi.cpol as u8) << 1 | spi.cpha as u8;
            let mode_desc = match mode {
                0 => "Mode 0: Idle Low, Sample Rising Edge",
                1 => "Mode 1: Idle Low, Sample Falling Edge",
                2 => "Mode 2: Idle High, Sample Falling Edge",
                3 => "Mode 3: Idle High, Sample Rising Edge",
                _ => "Unknown",
            };
            ui.label(RichText::new(mode_desc).size(10.0).color(Color32::GRAY));
        });
    }

    fn draw_clock_signal(&self, ui: &mut Ui, rect: egui::Rect, idle_high: bool, color: Color32) {
        let painter = ui.painter();
        let h = rect.height();
        let w = rect.width();
        let y_low = rect.top() + h * 0.8;
        let y_high = rect.top() + h * 0.2;

        // Draw baseline
        let base_y = if idle_high { y_high } else { y_low };
        painter.line_segment(
            [egui::pos2(rect.left(), base_y), egui::pos2(rect.right(), base_y)],
            egui::Stroke::new(1.5, color),
        );

        // Draw clock pulses
        let pulse_width = w / 8.0;
        for i in 0..8 {
            let x_start = rect.left() + i as f32 * pulse_width;
            let x_mid = x_start + pulse_width / 2.0;
            let x_end = x_start + pulse_width;

            if idle_high {
                // Start high, go low, back high
                painter.line_segment(
                    [egui::pos2(x_start, y_high), egui::pos2(x_start, y_low)],
                    egui::Stroke::new(1.5, color),
                );
                painter.line_segment(
                    [egui::pos2(x_start, y_low), egui::pos2(x_mid, y_low)],
                    egui::Stroke::new(1.5, color),
                );
                painter.line_segment(
                    [egui::pos2(x_mid, y_low), egui::pos2(x_mid, y_high)],
                    egui::Stroke::new(1.5, color),
                );
                painter.line_segment(
                    [egui::pos2(x_mid, y_high), egui::pos2(x_end, y_high)],
                    egui::Stroke::new(1.5, color),
                );
            } else {
                // Start low, go high, back low
                painter.line_segment(
                    [egui::pos2(x_start, y_low), egui::pos2(x_start, y_high)],
                    egui::Stroke::new(1.5, color),
                );
                painter.line_segment(
                    [egui::pos2(x_start, y_high), egui::pos2(x_mid, y_high)],
                    egui::Stroke::new(1.5, color),
                );
                painter.line_segment(
                    [egui::pos2(x_mid, y_high), egui::pos2(x_mid, y_low)],
                    egui::Stroke::new(1.5, color),
                );
                painter.line_segment(
                    [egui::pos2(x_mid, y_low), egui::pos2(x_end, y_low)],
                    egui::Stroke::new(1.5, color),
                );
            }
        }
    }

    fn draw_data_signal(&self, ui: &mut Ui, rect: egui::Rect, data: Option<u8>, color: Color32) {
        let painter = ui.painter();
        let h = rect.height();
        let y_low = rect.top() + h * 0.8;
        let y_high = rect.top() + h * 0.2;

        // Draw baseline
        painter.line_segment(
            [egui::pos2(rect.left(), y_low), egui::pos2(rect.right(), y_low)],
            egui::Stroke::new(1.0, Color32::from_rgb(60, 60, 60)),
        );

        if let Some(byte) = data {
            // Draw data bits
            let bit_width = rect.width() / 8.0;
            for i in 0..8 {
                let bit = (byte >> (7 - i)) & 1;
                let x_start = rect.left() + i as f32 * bit_width;
                let x_end = x_start + bit_width;
                let y = if bit != 0 { y_high } else { y_low };

                painter.line_segment(
                    [egui::pos2(x_start, y), egui::pos2(x_end, y)],
                    egui::Stroke::new(1.5, color),
                );

                // Vertical transitions
                if i > 0 {
                    let prev_bit = (byte >> (7 - (i - 1))) & 1;
                    if prev_bit != bit {
                        painter.line_segment(
                            [egui::pos2(x_start, y_low), egui::pos2(x_start, y_high)],
                            egui::Stroke::new(1.5, color),
                        );
                    }
                }
            }

            // Show hex value
            painter.text(
                egui::pos2(rect.right() - 30.0, rect.top() + 4.0),
                egui::Align2::RIGHT_TOP,
                format!("{:02X}", byte),
                egui::FontId::monospace(10.0),
                color,
            );
        }
    }

    fn draw_cs_signal(&self, ui: &mut Ui, rect: egui::Rect, active: bool) {
        let painter = ui.painter();
        let h = rect.height();
        let y_inactive = rect.top() + h * 0.2;
        let y_active = rect.top() + h * 0.8;

        let color = if active { Color32::GREEN } else { Color32::from_rgb(100, 100, 100) };

        // CS is active low
        if active {
            painter.line_segment(
                [egui::pos2(rect.left(), y_active), egui::pos2(rect.right(), y_active)],
                egui::Stroke::new(2.0, color),
            );
        } else {
            painter.line_segment(
                [egui::pos2(rect.left(), y_inactive), egui::pos2(rect.right(), y_inactive)],
                egui::Stroke::new(2.0, color),
            );
        }

        // Label
        let label = if active { "Active" } else { "Inactive" };
        painter.text(
            egui::pos2(rect.right() - 30.0, rect.top() + 4.0),
            egui::Align2::RIGHT_TOP,
            label,
            egui::FontId::proportional(10.0),
            color,
        );
    }

    fn draw_fifo_display(&self, ui: &mut Ui, state: &PeripheralState) {
        let spi = &state.spi[self.selected_spi];
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("FIFO Status").strong());
            });
            ui.separator();

            ui.horizontal(|ui| {
                // TX FIFO
                ui.vertical(|ui| {
                    ui.label(RichText::new("TX FIFO").color(Color32::YELLOW));
                    self.draw_fifo_bar(ui, spi.tx_fifo.len(), 8, Color32::YELLOW);
                    ui.monospace(format!("{}/8", spi.tx_fifo.len()));
                });

                ui.separator();

                // RX FIFO
                ui.vertical(|ui| {
                    ui.label(RichText::new("RX FIFO").color(Color32::from_rgb(100, 255, 150)));
                    self.draw_fifo_bar(ui, spi.rx_fifo.len(), 8, Color32::from_rgb(100, 255, 150));
                    ui.monospace(format!("{}/8", spi.rx_fifo.len()));
                });

                ui.separator();

                // FIFO contents
                ui.vertical(|ui| {
                    ui.label("TX Data:");
                    if spi.tx_fifo.is_empty() {
                        ui.label(RichText::new("(empty)").color(Color32::GRAY));
                    } else {
                        let hex: String = spi.tx_fifo.iter()
                            .map(|b| format!("{:02X}", b))
                            .collect::<Vec<_>>()
                            .join(" ");
                        ui.monospace(RichText::new(hex).color(Color32::YELLOW));
                    }

                    ui.label("RX Data:");
                    if spi.rx_fifo.is_empty() {
                        ui.label(RichText::new("(empty)").color(Color32::GRAY));
                    } else {
                        let hex: String = spi.rx_fifo.iter()
                            .map(|b| format!("{:02X}", b))
                            .collect::<Vec<_>>()
                            .join(" ");
                        ui.monospace(RichText::new(hex).color(Color32::from_rgb(100, 255, 150)));
                    }
                });
            });
        });
    }

    fn draw_fifo_bar(&self, ui: &mut Ui, level: usize, max: usize, color: Color32) {
        let bar_width = 120.0;
        let bar_height = 16.0;
        let (rect, _) = ui.allocate_exact_size(Vec2::new(bar_width, bar_height), egui::Sense::hover());

        // Background
        ui.painter().rect_filled(rect, 2.0, Color32::from_rgb(40, 40, 50));

        // Filled portion
        let fill_width = (level as f32 / max as f32) * bar_width;
        let fill_rect = egui::Rect::from_min_size(rect.min, Vec2::new(fill_width, bar_height));
        ui.painter().rect_filled(fill_rect, 2.0, color);
    }

    fn draw_transaction_log(&mut self, ui: &mut Ui, state: &PeripheralState) {
        let spi = &state.spi[self.selected_spi];
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Transaction Log").strong());
            });
            ui.separator();

            egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                if spi.transactions.is_empty() {
                    ui.label(RichText::new("No transactions yet").color(Color32::GRAY));
                } else {
                    egui::Grid::new("spi_transactions").spacing([8.0, 4.0]).show(ui, |ui| {
                        ui.label(RichText::new("Time").strong());
                        ui.label(RichText::new("CS").strong());
                        ui.label(RichText::new("TX").strong());
                        ui.label(RichText::new("RX").strong());
                        ui.end_row();

                        for txn in spi.transactions.iter().rev().take(30) {
                            ui.label(format!("{}", txn.timestamp));

                            let cs_color = if txn.cs == 0 { Color32::GREEN } else { Color32::GRAY };
                            ui.label(RichText::new(format!("CS{}", txn.cs)).color(cs_color));

                            // TX data
                            let tx_hex: String = txn.data_out.iter()
                                .map(|b| format!("{:02X}", b))
                                .collect::<Vec<_>>()
                                .join(" ");
                            ui.monospace(RichText::new(tx_hex).color(Color32::YELLOW));

                            // RX data
                            let rx_hex: String = txn.data_in.iter()
                                .map(|b| format!("{:02X}", b))
                                .collect::<Vec<_>>()
                                .join(" ");
                            ui.monospace(RichText::new(rx_hex).color(Color32::from_rgb(100, 255, 200)));

                            ui.end_row();
                        }
                    });
                }
            });
        });
    }

    fn draw_config(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        let spi_idx = self.selected_spi;
        let spi = &mut state.spi[spi_idx];

        ui.group(|ui| {
            ui.label(RichText::new("Configuration").strong());
            ui.separator();

            egui::Grid::new("spi_config").spacing([10.0, 4.0]).show(ui, |ui| {
                ui.label("Enable:");
                ui.checkbox(&mut spi.enabled, "");
                ui.end_row();

                ui.label("Mode:");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut spi.master_mode, true, "Master");
                    ui.selectable_value(&mut spi.master_mode, false, "Slave");
                });
                ui.end_row();

                ui.label("Clock Rate:");
                ui.add(egui::DragValue::new(&mut spi.clock_rate).suffix(" Hz").clamp_range(1..=100_000_000));
                ui.end_row();

                ui.label("Data Bits:");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut spi.data_bits, 8, "8-bit");
                    ui.selectable_value(&mut spi.data_bits, 16, "16-bit");
                    ui.selectable_value(&mut spi.data_bits, 32, "32-bit");
                });
                ui.end_row();

                ui.label("Bit Order:");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut spi.lsb_first, false, "MSB first");
                    ui.selectable_value(&mut spi.lsb_first, true, "LSB first");
                });
                ui.end_row();

                ui.label("CPOL:");
                ui.checkbox(&mut spi.cpol, "Clock Polarity");
                ui.end_row();

                ui.label("CPHA:");
                ui.checkbox(&mut spi.cpha, "Clock Phase");
                ui.end_row();
            });

            ui.add_space(4.0);

            // Clear statistics button
            ui.horizontal(|ui| {
                if ui.button("Clear Statistics").clicked() {
                    spi.total_bytes_tx = 0;
                    spi.total_bytes_rx = 0;
                    spi.transactions.clear();
                }
                if ui.button("Clear FIFOs").clicked() {
                    spi.tx_fifo.clear();
                    spi.rx_fifo.clear();
                }
            });
        });
    }

    fn draw_test_panel(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        let spi_idx = self.selected_spi;

        ui.group(|ui| {
            ui.label(RichText::new("Test Transfer").strong());
            ui.separator();

            // CS select
            ui.horizontal(|ui| {
                ui.label("CS:");
                for cs in 0..4 {
                    let selected = self.selected_cs == cs;
                    if ui.selectable_label(selected, format!("CS{}", cs)).clicked() {
                        self.selected_cs = cs;
                    }
                }
            });

            ui.add_space(4.0);

            // Data input
            ui.label("Data (hex, space-separated):");
            ui.add(
                egui::TextEdit::singleline(&mut self.test_data)
                    .desired_width(ui.available_width())
                    .hint_text("00 01 02 03...")
            );

            ui.add_space(4.0);

            // Quick buttons
            ui.horizontal(|ui| {
                if ui.button("Send").clicked() {
                    if let Ok(data) = self.parse_hex_data(&self.test_data) {
                        state.events.push(PeripheralEvent::SpiTransfer(spi_idx, data));
                    }
                }
                if ui.button("Send 0x00").clicked() {
                    state.events.push(PeripheralEvent::SpiTransfer(spi_idx, vec![0x00]));
                }
                if ui.button("Send 0xFF").clicked() {
                    state.events.push(PeripheralEvent::SpiTransfer(spi_idx, vec![0xFF]));
                }
            });

            // Presets
            ui.label("Presets:");
            ui.horizontal(|ui| {
                if ui.small_button("Read ID").clicked() {
                    self.test_data = "9F 00 00 00".to_string();
                }
                if ui.small_button("Read Status").clicked() {
                    self.test_data = "05".to_string();
                }
                if ui.small_button("Write Enable").clicked() {
                    self.test_data = "06".to_string();
                }
                if ui.small_button("Chip Erase").clicked() {
                    self.test_data = "C7".to_string();
                }
            });
        });
    }

    fn parse_hex_data(&self, input: &str) -> Result<Vec<u8>, ()> {
        let mut data = Vec::new();
        for part in input.split_whitespace() {
            if let Ok(byte) = u8::from_str_radix(part, 16) {
                data.push(byte);
            } else {
                return Err(());
            }
        }
        Ok(data)
    }
}