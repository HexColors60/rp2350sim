//! Boot RAM panel for RP2350 simulator.

use egui::{Color32, RichText, Ui, Vec2};

/// Boot RAM state for the panel.
#[derive(Debug, Clone)]
pub struct BootramState {
    pub enabled: bool,
    pub locked: bool,
    pub write_protected: bool,
    pub data: [u32; 2048], // 8KB / 4 = 2048 words
}

impl Default for BootramState {
    fn default() -> Self {
        Self {
            enabled: true,
            locked: false,
            write_protected: false,
            data: [0; 2048],
        }
    }
}

/// Boot RAM panel.
pub struct BootramPanel {
    selected_offset: usize,
    edit_value: u32,
    scroll_offset: usize,
    load_pattern: u8,
}

impl Default for BootramPanel {
    fn default() -> Self {
        Self {
            selected_offset: 0,
            edit_value: 0,
            scroll_offset: 0,
            load_pattern: 0,
        }
    }
}

impl BootramPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(&self) -> &'static str {
        "Boot RAM"
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut BootramState) {
        ui.vertical(|ui| {
            ui.label(RichText::new("Boot RAM - 8KB Boot Memory").strong());
            ui.separator();

            // Status
            self.draw_status(ui, state);

            ui.add_space(8.0);

            // Control
            self.draw_control(ui, state);

            ui.add_space(8.0);

            // Data View
            self.draw_data_view(ui, state);

            ui.add_space(8.0);

            // Editor
            self.draw_editor(ui, state);

            ui.add_space(8.0);

            // Load Code
            self.draw_load_code(ui, state);

            ui.add_space(8.0);

            // Info
            self.draw_info(ui);
        });
    }

    fn draw_status(&self, ui: &mut Ui, state: &BootramState) {
        ui.group(|ui| {
            ui.label(RichText::new("Status").strong());
            ui.separator();

            ui.horizontal(|ui| {
                // Enabled indicator
                let (color, status) = if state.enabled {
                    (Color32::GREEN, "Enabled")
                } else {
                    (Color32::GRAY, "Disabled")
                };
                let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 5.0, color);
                ui.label(RichText::new(format!("Enabled: {}", status)).color(color));
            });

            ui.horizontal(|ui| {
                // Locked indicator
                let (color, status) = if state.locked {
                    (Color32::RED, "LOCKED")
                } else {
                    (Color32::from_rgb(100, 200, 100), "Unlocked")
                };
                let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 5.0, color);
                ui.label(RichText::new(format!("Lock: {}", status)).color(color));
            });

            ui.horizontal(|ui| {
                // Write protect indicator
                let (color, status) = if state.write_protected {
                    (Color32::YELLOW, "Protected")
                } else {
                    (Color32::GRAY, "Writable")
                };
                let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 5.0, color);
                ui.label(RichText::new(format!("Write: {}", status)).color(color));
            });
        });
    }

    fn draw_control(&mut self, ui: &mut Ui, state: &mut BootramState) {
        ui.group(|ui| {
            ui.label(RichText::new("Control").strong());
            ui.separator();

            ui.horizontal(|ui| {
                // Enable/Disable
                if ui.button("Enable").clicked() && !state.locked {
                    state.enabled = true;
                }
                if ui.button("Disable").clicked() && !state.locked {
                    state.enabled = false;
                }
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                // Lock button (requires key pattern)
                if ui.button("Lock (Key: 0xDEADBEEF)").clicked() && !state.locked {
                    state.locked = true;
                    state.write_protected = true;
                }
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                // Clear button
                if ui
                    .add_enabled(!state.write_protected, egui::Button::new("Clear All"))
                    .clicked()
                {
                    state.data.fill(0);
                }

                // Write protect toggle
                if ui
                    .add_enabled(!state.locked, egui::Button::new("Toggle Protect"))
                    .clicked()
                {
                    state.write_protected = !state.write_protected;
                }
            });

            if state.locked {
                ui.label(
                    RichText::new("Boot RAM is locked. Reset required to unlock.")
                        .color(Color32::YELLOW),
                );
            }
        });
    }

    fn draw_data_view(&mut self, ui: &mut Ui, state: &BootramState) {
        ui.group(|ui| {
            ui.label(RichText::new("Data View (Hex Dump)").strong());
            ui.separator();

            // Show a window of 16 words (64 bytes) at a time
            let words_per_page = 16;
            let total_words = state.data.len();

            // Navigation
            ui.horizontal(|ui| {
                ui.label("Offset:");
                ui.add(
                    egui::DragValue::new(&mut self.scroll_offset)
                        .clamp_range(0..=(total_words - words_per_page)),
                );
                ui.label(format!("of {} words", total_words));

                if ui.small_button("<<").clicked() {
                    self.scroll_offset = self.scroll_offset.saturating_sub(words_per_page);
                }
                if ui.small_button("<").clicked() {
                    self.scroll_offset = self.scroll_offset.saturating_sub(4);
                }
                if ui.small_button(">").clicked() {
                    self.scroll_offset = (self.scroll_offset + 4).min(total_words - words_per_page);
                }
                if ui.small_button(">>").clicked() {
                    self.scroll_offset =
                        (self.scroll_offset + words_per_page).min(total_words - words_per_page);
                }
            });

            ui.add_space(4.0);

            // Column headers
            ui.horizontal(|ui| {
                ui.monospace(RichText::new("Addr     ").strong());
                for i in 0..4 {
                    ui.monospace(RichText::new(format!("+{:X}     ", i)).strong());
                }
            });
            ui.separator();

            // Data rows
            let start = self.scroll_offset;
            let end = (start + words_per_page).min(total_words);

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for row_start in (start..end).step_by(4) {
                        let row_end = (row_start + 4).min(end);
                        ui.horizontal(|ui| {
                            // Address
                            ui.monospace(
                                RichText::new(format!("0x{:04X}", row_start * 4))
                                    .color(Color32::from_rgb(100, 150, 200)),
                            );

                            // Data words
                            for i in row_start..row_end {
                                let word = state.data[i];
                                let color = if word == 0 {
                                    Color32::GRAY
                                } else {
                                    Color32::from_rgb(100, 200, 255)
                                };
                                ui.monospace(RichText::new(format!("{:08X} ", word)).color(color));
                            }
                            // Fill remaining columns
                            for _ in row_end..(row_start + 4) {
                                ui.monospace("         ");
                            }
                        });
                    }
                });
        });
    }

    fn draw_editor(&mut self, ui: &mut Ui, state: &mut BootramState) {
        ui.group(|ui| {
            ui.label(RichText::new("Editor").strong());
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Offset (word):");
                ui.add(
                    egui::DragValue::new(&mut self.selected_offset)
                        .clamp_range(0..=2047),
                );
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Value:");
                let mut hex_str = format!("{:08X}", self.edit_value);
                ui.add(egui::TextEdit::singleline(&mut hex_str).desired_width(80.0));
                if let Ok(val) = u32::from_str_radix(&hex_str, 16) {
                    self.edit_value = val;
                }
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                // Read button
                if ui.button("Read").clicked() {
                    self.edit_value = state.data[self.selected_offset];
                }

                // Write button
                let can_write = !state.write_protected && !state.locked;
                if ui
                    .add_enabled(can_write, egui::Button::new("Write"))
                    .clicked()
                {
                    state.data[self.selected_offset] = self.edit_value;
                }
            });

            // Show current value at offset
            ui.add_space(4.0);
            let current_value = state.data[self.selected_offset];
            ui.horizontal(|ui| {
                ui.label("Current value:");
                ui.monospace(RichText::new(format!("0x{:08X}", current_value)).color(
                    if current_value == 0 {
                        Color32::GRAY
                    } else {
                        Color32::from_rgb(100, 200, 255)
                    },
                ));
            });
        });
    }

    fn draw_load_code(&mut self, ui: &mut Ui, state: &mut BootramState) {
        ui.group(|ui| {
            ui.label(RichText::new("Load Boot Code").strong());
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Pattern:");
                egui::ComboBox::new("load_pattern", "")
                    .selected_text(match self.load_pattern {
                        0 => "Simple Loop",
                        1 => "LED Blink",
                        2 => "Memory Test",
                        3 => "Boot Stub",
                        _ => "Unknown",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.load_pattern, 0, "Simple Loop");
                        ui.selectable_value(&mut self.load_pattern, 1, "LED Blink");
                        ui.selectable_value(&mut self.load_pattern, 2, "Memory Test");
                        ui.selectable_value(&mut self.load_pattern, 3, "Boot Stub");
                    });
            });

            ui.add_space(4.0);

            if ui
                .add_enabled(!state.write_protected, egui::Button::new("Load Pattern"))
                .clicked()
            {
                self.load_pattern(state);
            }

            ui.add_space(4.0);
            ui.label("Simulates loading boot code into Boot RAM.");
        });
    }

    fn load_pattern(&self, state: &mut BootramState) {
        match self.load_pattern {
            0 => {
                // Simple loop pattern
                state.data[0] = 0x0000_006F; // j 0 (infinite loop)
                state.data[1] = 0x0000_0013; // nop
            }
            1 => {
                // LED blink pattern (simulated)
                state.data[0] = 0x0000_0137; // lui x2, 0x0
                state.data[1] = 0x3030_0093; // addi x1, x0, 0x3030 (GPIO base)
                state.data[2] = 0x0010_00A3; // sb x0, 1(x1) - toggle
                state.data[3] = 0x0000_006F; // j 0
            }
            2 => {
                // Memory test pattern
                state.data[0] = 0x2000_00B7; // lui x1, 0x20000 (SRAM base)
                state.data[1] = 0x1234_5607; // addi x0, x0, 0x1234_5678 (test value)
                state.data[2] = 0x0010_9023; // sb x0, 1(x1)
                state.data[3] = 0x0010_C003; // lb x0, 1(x1)
                state.data[4] = 0x0000_006F; // j 0
            }
            3 => {
                // Boot stub pattern
                state.data[0] = 0x2000_00B7; // lui x1, 0x20000
                state.data[1] = 0x0000_2097; // auipc x1, 0
                state.data[2] = 0x0040_00EF; // jal x0, +4
                state.data[3] = 0x0000_006F; // j 0
            }
            _ => {}
        }
    }

    fn draw_info(&self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("Info").strong());
            ui.separator();

            ui.label("Boot RAM is used during the boot process:");
            ui.label("- Base address: 0x4000_0000");
            ui.label("- Size: 8 KB (2048 x 32-bit words)");
            ui.label("- Accessible during boot sequence");
            ui.label("- Can be locked with key 0xDEADBEEF");
            ui.separator();
            ui.label("Locking prevents any modifications until reset.");
            ui.label("Write protection allows reads but blocks writes.");
        });
    }
}