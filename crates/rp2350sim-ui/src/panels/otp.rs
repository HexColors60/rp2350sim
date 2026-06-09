//! OTP (One-Time Programmable) memory panel for RP2350 simulator.

use egui::{Color32, RichText, Ui, Vec2};

/// OTP memory panel for RP2350.
pub struct OtpPanel {
    selected_row: usize,
    edit_value: String,
}

impl Default for OtpPanel {
    fn default() -> Self {
        Self {
            selected_row: 0,
            edit_value: String::new(),
        }
    }
}

impl OtpPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(&self) -> &'static str {
        "OTP"
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut OtpState) {
        ui.vertical(|ui| {
            ui.label(RichText::new("OTP Memory Panel").strong());
            ui.separator();

            // Warning banner
            self.draw_warning(ui);

            ui.add_space(8.0);

            // Status display
            self.draw_status(ui, state);

            ui.add_space(8.0);

            // Control section
            self.draw_control(ui, state);

            ui.add_space(8.0);

            // Data view
            self.draw_data_view(ui, state);

            ui.add_space(8.0);

            // Row editor
            self.draw_row_editor(ui, state);

            ui.add_space(8.0);

            // Info
            self.draw_info(ui);
        });
    }

    fn draw_warning(&self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(
                RichText::new("⚠ ONE-TIME PROGRAMMABLE MEMORY")
                    .color(Color32::from_rgb(255, 100, 100))
                    .strong(),
            );
            ui.label(
                RichText::new("Once programmed, OTP bits cannot be erased or reprogrammed!")
                    .color(Color32::from_rgb(255, 150, 150)),
            );
        });
    }

    fn draw_status(&self, ui: &mut Ui, state: &OtpState) {
        ui.group(|ui| {
            ui.label(RichText::new("Status").strong());
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Write Enable:");
                let (we_color, we_text) = if state.write_enabled {
                    (Color32::GREEN, "ENABLED")
                } else {
                    (Color32::GRAY, "DISABLED")
                };
                let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 5.0, we_color);
                ui.label(RichText::new(we_text).color(we_color));
            });

            ui.horizontal(|ui| {
                ui.label("Locked:");
                let (lock_color, lock_text) = if state.locked {
                    (Color32::RED, "LOCKED")
                } else {
                    (Color32::GREEN, "UNLOCKED")
                };
                let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 5.0, lock_color);
                ui.label(RichText::new(lock_text).color(lock_color));
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Control Register:");
                ui.monospace(RichText::new(format!("0x{:08X}", state.ctrl))
                    .color(Color32::from_rgb(100, 200, 255)));
            });

            ui.horizontal(|ui| {
                ui.label("Status Register:");
                ui.monospace(RichText::new(format!("0x{:08X}", state.status))
                    .color(Color32::from_rgb(100, 200, 255)));
            });
        });
    }

    fn draw_control(&self, ui: &mut Ui, state: &mut OtpState) {
        ui.group(|ui| {
            ui.label(RichText::new("Control").strong());
            ui.separator();

            ui.horizontal(|ui| {
                if state.write_enabled {
                    if ui.button("Disable Write").clicked() {
                        state.write_enabled = false;
                    }
                } else {
                    if ui.button("Enable Write").clicked() {
                        state.write_enabled = true;
                    }
                }
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                if state.locked {
                    ui.label(RichText::new("🔒 OTP is permanently locked")
                        .color(Color32::RED));
                } else {
                    if ui.button("Lock OTP (IRREVERSIBLE)").clicked() {
                        state.locked = true;
                        state.write_enabled = false;
                    }
                }
            });
        });
    }

    fn draw_data_view(&mut self, ui: &mut Ui, state: &mut OtpState) {
        ui.group(|ui| {
            ui.label(RichText::new("OTP Memory Map").strong());
            ui.separator();

            // Legend
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 2.0, Color32::from_rgb(100, 150, 255));
                ui.label("Factory Data (Read-Only)");
                ui.add_space(16.0);
                let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 2.0, Color32::from_rgb(100, 200, 100));
                ui.label("User Data (Programmable)");
            });

            ui.add_space(4.0);
            ui.separator();

            // Hex dump - show 4 rows at a time
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for row_group in 0..16 {
                        let start_row = row_group * 4;
                        ui.horizontal(|ui| {
                            // Row label
                            ui.monospace(
                                RichText::new(format!("{:02X}:", start_row))
                                    .color(Color32::GRAY)
                            );

                            // Show 4 rows
                            for row in start_row..(start_row + 4) {
                                if row < 64 {
                                    let value = state.rows[row];
                                    let is_factory = row < 16;
                                    let is_selected = row == self.selected_row;

                                    let bg_color = if is_factory {
                                        Color32::from_rgb(40, 60, 80)
                                    } else {
                                        Color32::from_rgb(40, 80, 60)
                                    };

                                    let text_color = if is_factory {
                                        Color32::from_rgb(100, 150, 255)
                                    } else {
                                        Color32::from_rgb(100, 200, 100)
                                    };

                                    let (rect, response) = ui.allocate_exact_size(
                                        Vec2::new(80.0, 20.0),
                                        egui::Sense::click()
                                    );

                                    // Draw background
                                    let actual_bg = if is_selected {
                                        Color32::from_rgb(80, 80, 120)
                                    } else {
                                        bg_color
                                    };
                                    ui.painter().rect_filled(rect, 2.0, actual_bg);

                                    // Draw text
                                    ui.painter().text(
                                        rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        format!("{:08X}", value),
                                        egui::FontId::monospace(12.0),
                                        text_color
                                    );

                                    // Handle click
                                    if response.clicked() {
                                        self.selected_row = row;
                                        self.edit_value = format!("{:08X}", value);
                                    }
                                }
                            }
                        });
                    }
                });
        });
    }

    fn draw_row_editor(&mut self, ui: &mut Ui, state: &mut OtpState) {
        ui.group(|ui| {
            ui.label(RichText::new("Row Editor").strong());
            ui.separator();

            let is_factory = self.selected_row < 16;
            let row_type = if is_factory { "Factory" } else { "User" };
            let type_color = if is_factory {
                Color32::from_rgb(100, 150, 255)
            } else {
                Color32::from_rgb(100, 200, 100)
            };

            ui.horizontal(|ui| {
                ui.label("Row:");
                ui.add(egui::DragValue::new(&mut self.selected_row).clamp_range(0..=63));
                ui.label(RichText::new(format!("({})", row_type)).color(type_color));
            });

            ui.add_space(4.0);

            // Row info
            let current_value = state.rows[self.selected_row];
            ui.horizontal(|ui| {
                ui.label("Current Value:");
                ui.monospace(RichText::new(format!("0x{:08X}", current_value))
                    .color(Color32::YELLOW));
            });

            // Check if already programmed (non-zero)
            let is_programmed = current_value != 0;

            ui.add_space(4.0);

            if is_factory {
                ui.label(
                    RichText::new("⛔ Factory rows are read-only")
                        .color(Color32::from_rgb(255, 150, 150))
                );
            } else if is_programmed {
                ui.label(
                    RichText::new("⚠ This row is already programmed (non-zero)")
                        .color(Color32::YELLOW)
                );
            }

            ui.add_space(4.0);

            // Edit field
            ui.horizontal(|ui| {
                ui.label("New Value:");
                ui.add(egui::TextEdit::singleline(&mut self.edit_value)
                    .desired_width(100.0)
                    .hint_text("Hex value"));

                // Parse button
                if let Ok(val) = u32::from_str_radix(self.edit_value.trim().trim_start_matches("0x"), 16) {
                    ui.monospace(RichText::new(format!("= {}", val))
                        .color(Color32::GRAY));
                } else {
                    ui.label(RichText::new("Invalid hex").color(Color32::RED));
                }
            });

            ui.add_space(4.0);

            // Program button
            ui.horizontal(|ui| {
                let can_program = !is_factory
                    && !state.locked
                    && state.write_enabled
                    && u32::from_str_radix(self.edit_value.trim().trim_start_matches("0x"), 16).is_ok();

                let button = if is_factory {
                    ui.add_enabled(false, egui::Button::new("Program (Read-Only)"))
                } else if state.locked {
                    ui.add_enabled(false, egui::Button::new("Program (Locked)"))
                } else if !state.write_enabled {
                    ui.add_enabled(false, egui::Button::new("Program (Write Disabled)"))
                } else {
                    ui.add_enabled(can_program, egui::Button::new("Program Row"))
                };

                if button.clicked() {
                    if let Ok(val) = u32::from_str_radix(self.edit_value.trim().trim_start_matches("0x"), 16) {
                        // OTP programming: can only change 0 bits to 1
                        state.rows[self.selected_row] |= val;
                    }
                }
            });
        });
    }

    fn draw_info(&self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("Info").strong());
            ui.separator();

            ui.label("OTP (One-Time Programmable) Memory:");
            ui.label("• Base Address: 0x4001_8000");
            ui.label("• 64 rows × 32-bit words (256 bytes total)");
            ui.label("• Rows 0-15: Factory data (read-only)");
            ui.label("• Rows 16-63: User data (programmable once)");

            ui.separator();

            ui.label("Programming Notes:");
            ui.label("• OTP bits can only be programmed from 0 to 1");
            ui.label("• Once a bit is set to 1, it cannot be cleared");
            ui.label("• Locking makes OTP permanently read-only");
            ui.label("• Write enable must be set before programming");

            ui.separator();

            ui.label("Typical Factory Data:");
            ui.label("• Row 0-3: Device ID / Serial Number");
            ui.label("• Row 4-7: Flash Calibration");
            ui.label("• Row 8-11: Boot Configuration");
            ui.label("• Row 12-15: Reserved");
        });
    }
}

/// OTP state for the panel.
#[derive(Debug, Clone)]
pub struct OtpState {
    /// Control register value
    pub ctrl: u32,
    /// Status register value
    pub status: u32,
    /// Write enable flag
    pub write_enabled: bool,
    /// Lock status (permanent)
    pub locked: bool,
    /// All 64 rows of OTP data
    pub rows: [u32; 64],
}

impl Default for OtpState {
    fn default() -> Self {
        Self {
            ctrl: 0,
            status: 0,
            write_enabled: false,
            locked: false,
            rows: [0; 64],
        }
    }
}

impl OtpState {
    /// Create OTP state with factory data pre-populated
    pub fn with_factory_data() -> Self {
        let mut state = Self::default();

        // Simulated factory data (typical values)
        state.rows[0] = 0x2350_0001;  // Device ID (RP2350 variant)
        state.rows[1] = 0x0000_1234;  // Serial number low
        state.rows[2] = 0x0000_5678;  // Serial number high
        state.rows[3] = 0x0000_0000;  // Reserved
        state.rows[4] = 0x0030_0000;  // Flash calibration
        state.rows[5] = 0x0000_0000;  // Reserved
        state.rows[6] = 0x0000_0000;  // Reserved
        state.rows[7] = 0x0000_0000;  // Reserved
        state.rows[8] = 0x0000_0001;  // Boot configuration
        state.rows[9] = 0x0000_0000;  // Reserved
        state.rows[10] = 0x0000_0000; // Reserved
        state.rows[11] = 0x0000_0000; // Reserved
        state.rows[12] = 0x0000_0000; // Reserved
        state.rows[13] = 0x0000_0000; // Reserved
        state.rows[14] = 0x0000_0000; // Reserved
        state.rows[15] = 0x0000_0000; // Reserved

        // Rows 16-63 are user data, start as 0 (unprogrammed)
        // rows[16..64] already initialized to 0

        state
    }

    /// Get a reference to factory data (rows 0-15)
    pub fn factory_data(&self) -> &[u32; 16] {
        self.rows[0..16].try_into().unwrap()
    }

    /// Get a reference to user data (rows 16-63)
    pub fn user_data(&self) -> &[u32; 48] {
        self.rows[16..64].try_into().unwrap()
    }

    /// Check if a specific row is factory data
    pub fn is_factory_row(&self, row: usize) -> bool {
        row < 16
    }

    /// Check if a specific row is programmed (non-zero)
    pub fn is_programmed(&self, row: usize) -> bool {
        self.rows.get(row).map_or(false, |&v| v != 0)
    }

    /// Program a row (OR operation - can only set bits to 1)
    /// Returns true if successful, false if row is factory or locked
    pub fn program_row(&mut self, row: usize, value: u32) -> bool {
        if self.locked || row < 16 || !self.write_enabled {
            return false;
        }

        // OTP can only program 0 bits to 1
        self.rows[row] |= value;
        true
    }
}