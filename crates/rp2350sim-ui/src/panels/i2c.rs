//! I2C panel for RP2350 simulator.

use super::{PeripheralEvent, PeripheralState, I2cTransaction, I2cDevice};
use egui::{Color32, RichText, Ui, Vec2};

/// I2C panel with bus monitoring and device management.
pub struct I2cPanel {
    selected_i2c: usize,
    scan_running: bool,
    scan_progress: u8,
    test_address: String,
    test_data: String,
    show_signals: bool,
}

impl Default for I2cPanel {
    fn default() -> Self {
        Self {
            selected_i2c: 0,
            scan_running: false,
            scan_progress: 0,
            test_address: String::from("50"),
            test_data: String::new(),
            show_signals: true,
        }
    }
}

impl I2cPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(&self) -> &'static str {
        "I2C"
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        ui.vertical(|ui| {
            // I2C selector
            ui.horizontal(|ui| {
                ui.label(RichText::new("I2C Panel").strong());
                ui.separator();
                ui.selectable_value(&mut self.selected_i2c, 0, "I2C0");
                ui.selectable_value(&mut self.selected_i2c, 1, "I2C1");
            });
            ui.separator();

            // Status bar
            self.draw_status_bar(ui, state);

            ui.add_space(4.0);

            // Signal visualization toggle
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_signals, "Show Bus Signals");
            });

            if self.show_signals {
                self.draw_signal_display(ui, state);
            }

            ui.add_space(4.0);

            // Transaction log
            self.draw_transaction_log(ui, state);

            ui.add_space(4.0);

            // Configuration
            self.draw_config(ui, state);

            ui.add_space(4.0);

            // Device list
            self.draw_device_list(ui, state);

            ui.add_space(4.0);

            // Test panel
            self.draw_test_panel(ui, state);
        });
    }

    fn draw_status_bar(&self, ui: &mut Ui, state: &PeripheralState) {
        let i2c = &state.i2c[self.selected_i2c];
        ui.horizontal(|ui| {
            // Enable status
            let enable_color = if i2c.enabled { Color32::GREEN } else { Color32::RED };
            let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
            ui.painter().circle_filled(rect.center(), 5.0, enable_color);
            ui.label(if i2c.enabled { "Enabled" } else { "Disabled" });

            ui.separator();

            // Mode
            ui.label(if i2c.master_mode { "Master" } else { "Slave" });

            ui.separator();

            // Clock
            let clock_khz = i2c.clock_rate as f64 / 1000.0;
            ui.label(format!("Clock: {:.0} kHz", clock_khz));

            ui.separator();

            // Bus state
            let state_color = match i2c.bus_state.as_str() {
                "Idle" => Color32::GRAY,
                "Busy" => Color32::YELLOW,
                "Selected" => Color32::GREEN,
                _ => Color32::GRAY,
            };
            ui.label(RichText::new(&i2c.bus_state).color(state_color));

            ui.separator();

            // Device count
            ui.label(format!("Devices: {}", i2c.attached_devices.len()));
        });

        // Statistics bar
        ui.horizontal(|ui| {
            ui.label(RichText::new("Stats:").strong());
            ui.label(format!("TX: {}", i2c.total_tx_bytes));
            ui.label(format!("RX: {}", i2c.total_rx_bytes));
            ui.label(format!("Txns: {}", i2c.total_transactions));
        });
    }

    fn draw_signal_display(&self, ui: &mut Ui, state: &PeripheralState) {
        let i2c = &state.i2c[self.selected_i2c];
        ui.group(|ui| {
            ui.label(RichText::new("Bus Signals").strong());
            ui.separator();

            let signal_height = 24.0;
            let signal_width = ui.available_width() - 20.0;

            egui::Grid::new("i2c_signals").spacing([4.0, 2.0]).show(ui, |ui| {
                // SCL (Clock)
                ui.label("SCL:");
                let (rect, _) = ui.allocate_exact_size(Vec2::new(signal_width, signal_height), egui::Sense::hover());
                self.draw_signal_line(ui, rect, i2c.scl_high, Color32::from_rgb(0, 200, 255));
                ui.end_row();

                // SDA (Data)
                ui.label("SDA:");
                let (rect, _) = ui.allocate_exact_size(Vec2::new(signal_width, signal_height), egui::Sense::hover());
                self.draw_signal_line(ui, rect, i2c.sda_high, Color32::YELLOW);
                ui.end_row();
            });

            // Bus state description
            ui.horizontal(|ui| {
                let state_desc = match i2c.bus_state.as_str() {
                    "Idle" => "Bus idle (both lines high)",
                    "Busy" => "Bus busy (transaction in progress)",
                    "Selected" => "Device selected (address matched)",
                    _ => "Unknown state",
                };
                ui.label(RichText::new(state_desc).size(10.0).color(Color32::GRAY));
            });
        });
    }

    fn draw_signal_line(&self, ui: &mut Ui, rect: egui::Rect, is_high: bool, color: Color32) {
        let painter = ui.painter();
        let h = rect.height();
        let y_low = rect.top() + h * 0.8;
        let y_high = rect.top() + h * 0.2;

        // Draw the signal line
        let y = if is_high { y_high } else { y_low };
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            egui::Stroke::new(2.0, color),
        );

        // Draw pull-up indicator (dashed line at top)
        let dash_color = Color32::from_rgb(60, 60, 60);
        for x in (rect.left() as i32..rect.right() as i32).step_by(8) {
            painter.line_segment(
                [egui::pos2(x as f32, y_high), egui::pos2((x + 4) as f32, y_high)],
                egui::Stroke::new(1.0, dash_color),
            );
        }

        // Label
        let label = if is_high { "HIGH" } else { "LOW" };
        painter.text(
            egui::pos2(rect.right() - 30.0, rect.top() + 4.0),
            egui::Align2::RIGHT_TOP,
            label,
            egui::FontId::monospace(10.0),
            color,
        );
    }

    fn draw_transaction_log(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        let i2c_idx = self.selected_i2c;
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Bus Monitor").strong());
                if ui.button("Clear").clicked() {
                    state.i2c[i2c_idx].transactions.clear();
                    state.i2c[i2c_idx].total_tx_bytes = 0;
                    state.i2c[i2c_idx].total_rx_bytes = 0;
                    state.i2c[i2c_idx].total_transactions = 0;
                }
            });
            ui.separator();

            let i2c = &state.i2c[i2c_idx];
            egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                if i2c.transactions.is_empty() {
                    ui.label(RichText::new("No transactions yet").color(Color32::GRAY));
                } else {
                    egui::Grid::new("i2c_transactions").spacing([8.0, 4.0]).show(ui, |ui| {
                        ui.label(RichText::new("Time").strong());
                        ui.label(RichText::new("Addr").strong());
                        ui.label(RichText::new("Dir").strong());
                        ui.label(RichText::new("Data").strong());
                        ui.label(RichText::new("ACK").strong());
                        ui.end_row();

                        for txn in i2c.transactions.iter().rev().take(30) {
                            ui.label(format!("{}", txn.timestamp));

                            // Address with R/W bit indicator
                            let addr_display = format!("0x{:02X}", txn.address >> 1);
                            ui.monospace(RichText::new(addr_display).color(Color32::from_rgb(100, 200, 255)));

                            // Direction
                            let (dir_text, dir_color) = if txn.read {
                                ("READ", Color32::GREEN)
                            } else {
                                ("WRITE", Color32::YELLOW)
                            };
                            ui.label(RichText::new(dir_text).color(dir_color).size(10.0));

                            // Data
                            let data_hex: String = txn.data.iter()
                                .map(|b| format!("{:02X}", b))
                                .collect::<Vec<_>>()
                                .join(" ");
                            ui.monospace(RichText::new(data_hex).color(Color32::WHITE));

                            // ACK status
                            let ack_count = txn.ack.iter().filter(|&&a| a).count();
                            let nack_count = txn.ack.len() - ack_count;
                            if nack_count == 0 {
                                ui.label(RichText::new("✓ ACK").color(Color32::GREEN));
                            } else if ack_count == 0 {
                                ui.label(RichText::new("✗ NACK").color(Color32::RED));
                            } else {
                                ui.label(RichText::new(format!("✓{} ✗{}", ack_count, nack_count))
                                    .color(Color32::YELLOW));
                            }

                            ui.end_row();
                        }
                    });
                }
            });
        });
    }

    fn draw_config(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        let i2c_idx = self.selected_i2c;
        let i2c = &mut state.i2c[i2c_idx];

        ui.group(|ui| {
            ui.label(RichText::new("Configuration").strong());
            ui.separator();

            egui::Grid::new("i2c_config").spacing([10.0, 4.0]).show(ui, |ui| {
                ui.label("Enable:");
                ui.checkbox(&mut i2c.enabled, "");
                ui.end_row();

                ui.label("Mode:");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut i2c.master_mode, true, "Master");
                    ui.selectable_value(&mut i2c.master_mode, false, "Slave");
                });
                ui.end_row();

                ui.label("Clock Rate:");
                egui::ComboBox::from_id_source(format!("i2c_clock_{}", i2c_idx))
                    .selected_text(format!("{} Hz", i2c.clock_rate))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut i2c.clock_rate, 100_000, "100 kHz (Standard)");
                        ui.selectable_value(&mut i2c.clock_rate, 400_000, "400 kHz (Fast)");
                        ui.selectable_value(&mut i2c.clock_rate, 1_000_000, "1 MHz (Fast+)");
                    });
                ui.end_row();
            });
        });
    }

    fn draw_device_list(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        let i2c_idx = self.selected_i2c;
        let device_count = state.i2c[i2c_idx].attached_devices.len();
        let devices: Vec<I2cDevice> = state.i2c[i2c_idx].attached_devices.clone();

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Attached Devices").strong());
                if ui.button("Scan Bus").clicked() {
                    self.scan_running = true;
                    self.scan_progress = 0;
                }
            });
            ui.separator();

            if device_count == 0 {
                ui.label(RichText::new("No devices attached").color(Color32::GRAY));
                ui.label(RichText::new("Click 'Scan Bus' or '+ Add Device'").size(10.0).color(Color32::GRAY));
            } else {
                egui::Grid::new("i2c_devices").spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label(RichText::new("Addr").strong());
                    ui.label(RichText::new("Name").strong());
                    ui.label(RichText::new("Type").strong());
                    ui.label(RichText::new("Action").strong());
                    ui.end_row();

                    for (idx, device) in devices.iter().enumerate() {
                        // Address with icon
                        ui.horizontal(|ui| {
                            let icon = self.get_device_icon(&device.device_type);
                            ui.label(RichText::new(icon).size(12.0));
                            ui.monospace(format!("0x{:02X}", device.address));
                        });

                        ui.label(&device.name);
                        ui.label(RichText::new(&device.device_type).color(Color32::GRAY));

                        // Remove button
                        if ui.small_button("✕").clicked() {
                            state.i2c[i2c_idx].attached_devices.remove(idx);
                        }
                        ui.end_row();
                    }
                });
            }

            ui.add_space(4.0);

            // Add device
            ui.horizontal(|ui| {
                if ui.button("+ Add Device").clicked() {
                    state.i2c[i2c_idx].attached_devices.push(I2cDevice {
                        address: 0x50,
                        name: "EEPROM".to_string(),
                        device_type: "24C02".to_string(),
                    });
                }
            });
        });
    }

    fn get_device_icon(&self, device_type: &str) -> &'static str {
        match device_type.to_lowercase().as_str() {
            "24c02" | "24c04" | "24c08" | "24c16" | "24c32" | "24c64" | "24c256" | "24c512" => "💾",
            "ds3231" | "ds1307" | "pcf8523" => "🕐",
            "bmp180" | "bmp280" | "bme280" => "🌡",
            "mpu6050" | "mpu9250" => " gyro",
            "ssd1306" | "ssd1309" => "📺",
            "pcf8574" | "pcf8575" => "🔌",
            _ => "📦",
        }
    }

    fn draw_test_panel(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        let i2c_idx = self.selected_i2c;

        ui.group(|ui| {
            ui.label(RichText::new("Test Transaction").strong());
            ui.separator();

            // Address input
            ui.horizontal(|ui| {
                ui.label("Address (7-bit hex):");
                ui.add(
                    egui::TextEdit::singleline(&mut self.test_address)
                        .desired_width(60.0)
                        .hint_text("50")
                );
            });

            ui.add_space(4.0);

            // Data input
            ui.horizontal(|ui| {
                ui.label("Data (hex):");
                ui.add(
                    egui::TextEdit::singleline(&mut self.test_data)
                        .desired_width(ui.available_width())
                        .hint_text("00 01 02...")
                );
            });

            ui.add_space(4.0);

            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("Write").clicked() {
                    if let (Ok(addr), Ok(data)) = (self.parse_address(), self.parse_hex_data()) {
                        state.events.push(PeripheralEvent::I2cTransaction(
                            i2c_idx,
                            I2cTransaction {
                                timestamp: 0,
                                address: addr << 1,  // Convert to 8-bit address
                                read: false,
                                data,
                                ack: vec![true],
                            }
                        ));
                    }
                }
                if ui.button("Read 4 bytes").clicked() {
                    if let Ok(addr) = self.parse_address() {
                        state.events.push(PeripheralEvent::I2cTransaction(
                            i2c_idx,
                            I2cTransaction {
                                timestamp: 0,
                                address: addr << 1 | 1,  // Read bit
                                read: true,
                                data: vec![0; 4],
                                ack: vec![true],
                            }
                        ));
                    }
                }
            });

            // Presets
            ui.label("Quick Addresses:");
            ui.horizontal(|ui| {
                if ui.small_button("EEPROM (0x50)").clicked() {
                    self.test_address = "50".to_string();
                }
                if ui.small_button("Temp (0x48)").clicked() {
                    self.test_address = "48".to_string();
                }
                if ui.small_button("RTC (0x68)").clicked() {
                    self.test_address = "68".to_string();
                }
                if ui.small_button("OLED (0x3C)").clicked() {
                    self.test_address = "3C".to_string();
                }
            });
        });
    }

    fn parse_address(&self) -> Result<u8, ()> {
        u8::from_str_radix(&self.test_address, 16).map_err(|_| ())
    }

    fn parse_hex_data(&self) -> Result<Vec<u8>, ()> {
        let mut data = Vec::new();
        for part in self.test_data.split_whitespace() {
            if let Ok(byte) = u8::from_str_radix(part, 16) {
                data.push(byte);
            }
        }
        Ok(data)
    }
}