//! HSTX (High-Speed Transmitter) panel for RP2350 simulator.

use egui::{Color32, RichText, Ui};

/// HSTX panel for RP2350 high-speed transmitter.
pub struct HstxPanel {
    test_data: u32,
}

impl Default for HstxPanel {
    fn default() -> Self {
        Self {
            test_data: 0,
        }
    }
}

impl HstxPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(&self) -> &'static str {
        "HSTX"
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut HstxState) {
        ui.vertical(|ui| {
            ui.label(RichText::new("HSTX - High-Speed Transmitter").strong());
            ui.separator();

            // Status section
            self.draw_status(ui, state);

            ui.add_space(8.0);

            // FIFO section
            self.draw_fifo(ui, state);

            ui.add_space(8.0);

            // Control section
            self.draw_control(ui, state);

            ui.add_space(8.0);

            // Test section
            self.draw_test(ui, state);

            ui.add_space(8.0);

            // Interrupt section
            self.draw_interrupt(ui, state);

            ui.add_space(8.0);

            // Config section
            self.draw_config(ui, state);

            ui.add_space(8.0);

            // Actions section
            self.draw_actions(ui, state);

            ui.add_space(8.0);

            // Info section
            self.draw_info(ui);
        });
    }

    fn draw_status(&self, ui: &mut Ui, state: &HstxState) {
        ui.group(|ui| {
            ui.label(RichText::new("Status").strong());
            ui.separator();

            // Enabled indicator
            let color = if state.enabled {
                Color32::GREEN
            } else {
                Color32::DARK_GRAY
            };
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 5.0, color);
                ui.label(format!("Enabled: {}", state.enabled));
            });

            ui.add_space(4.0);

            // FIFO count
            ui.horizontal(|ui| {
                ui.label("FIFO Count:");
                let fifo_color = if state.fifo_count >= 12 {
                    Color32::RED
                } else if state.fifo_count >= 8 {
                    Color32::YELLOW
                } else {
                    Color32::from_rgb(100, 200, 100)
                };
                ui.label(
                    RichText::new(format!("{}/16", state.fifo_count))
                        .color(fifo_color)
                        .strong(),
                );
            });

            // FIFO level bar
            let fifo_ratio = state.fifo_count as f32 / 16.0;
            let bar_color = if state.fifo_count >= 12 {
                Color32::RED
            } else if state.fifo_count >= 8 {
                Color32::YELLOW
            } else {
                Color32::from_rgb(100, 200, 100)
            };
            ui.add(
                egui::ProgressBar::new(fifo_ratio)
                    .fill(bar_color)
                    .text(format!("FIFO {}%", (fifo_ratio * 100.0) as u32)),
            );
        });
    }

    fn draw_fifo(&self, ui: &mut Ui, state: &HstxState) {
        ui.group(|ui| {
            ui.label(RichText::new("FIFO Contents (Last 16 entries)").strong());
            ui.separator();

            if state.fifo_data.is_empty() {
                ui.label(RichText::new("FIFO is empty").color(Color32::GRAY));
            } else {
                // Display FIFO entries in a grid (4 columns)
                egui::Grid::new("hstx_fifo_grid")
                    .num_columns(4)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        for (i, &data) in state.fifo_data.iter().rev().take(16).enumerate() {
                            ui.label(format!("[{}]", i));
                            ui.monospace(
                                RichText::new(format!("0x{:08X}", data))
                                    .color(Color32::from_rgb(100, 200, 255)),
                            );
                            if (i + 1) % 2 == 0 {
                                ui.end_row();
                            }
                        }
                    });
            }
        });
    }

    fn draw_control(&self, ui: &mut Ui, state: &mut HstxState) {
        ui.group(|ui| {
            ui.label(RichText::new("Control").strong());
            ui.separator();

            if ui.checkbox(&mut state.enabled, "Enable HSTX").changed() {
                // State changed
            }

            ui.add_space(4.0);
            ui.label(format!("Current state: {}", if state.enabled { "Running" } else { "Stopped" }));
        });
    }

    fn draw_test(&mut self, ui: &mut Ui, state: &mut HstxState) {
        ui.group(|ui| {
            ui.label(RichText::new("Test").strong());
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Test Data:");
                let mut test_data = self.test_data as i64;
                ui.add(
                    egui::DragValue::new(&mut test_data)
                        .clamp_range(0i64..=0xFFFFFFFFi64),
                );
                self.test_data = test_data as u32;
            });


            ui.add_space(4.0);

            if ui.button("Push to FIFO").clicked() {
                if state.fifo_count < 16 {
                    state.fifo_data.push(self.test_data);
                    state.fifo_count += 1;
                }
            }
        });
    }

    fn draw_interrupt(&self, ui: &mut Ui, state: &mut HstxState) {
        ui.group(|ui| {
            ui.label(RichText::new("Interrupt").strong());
            ui.separator();

            // IRQ enabled
            let enabled_color = if state.irq_enabled {
                Color32::GREEN
            } else {
                Color32::DARK_GRAY
            };
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 5.0, enabled_color);
                ui.label(format!("IRQ Enabled: {}", state.irq_enabled));
            });

            ui.add_space(4.0);

            // IRQ pending
            let pending_color = if state.irq_pending {
                Color32::RED
            } else {
                Color32::DARK_GRAY
            };
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 5.0, pending_color);
                ui.label(format!("IRQ Pending: {}", state.irq_pending));
            });

            ui.add_space(4.0);

            ui.checkbox(&mut state.irq_enabled, "Enable Interrupts");
        });
    }

    fn draw_config(&self, ui: &mut Ui, state: &mut HstxState) {
        ui.group(|ui| {
            ui.label(RichText::new("Config Register").strong());
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Config:");
                ui.monospace(
                    RichText::new(format!("0x{:08X}", state.config))
                        .color(Color32::from_rgb(100, 200, 255)),
                );
            });

            ui.add_space(4.0);

            // Decode config fields
            let mode = state.config & 0x3;
            let shift_32 = (state.config >> 2) & 0x1;
            let clock_div = (state.config >> 4) & 0xFF;

            ui.label("Config Fields:");
            ui.horizontal(|ui| {
                ui.label("  Mode:");
                ui.monospace(format!("{}", mode));
            });
            ui.horizontal(|ui| {
                ui.label("  Shift32:");
                ui.monospace(format!("{}", shift_32));
            });
            ui.horizontal(|ui| {
                ui.label("  Clock Div:");
                ui.monospace(format!("{}", clock_div));
            });
        });
    }

    fn draw_actions(&mut self, ui: &mut Ui, state: &mut HstxState) {
        ui.group(|ui| {
            ui.label(RichText::new("Actions").strong());
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Clear FIFO").clicked() {
                    state.fifo_data.clear();
                    state.fifo_count = 0;
                }

                if ui.button("Clear IRQ Pending").clicked() {
                    state.irq_pending = false;
                }

                if ui.button("Reset").clicked() {
                    *state = HstxState::default();
                }
            });
        });
    }

    fn draw_info(&self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("Info").strong());
            ui.separator();
            ui.label("HSTX (High-Speed Transmitter) for RP2350:");
            ui.label("- Base address: 0x400C_0000");
            ui.label("- FIFO for TX data (16 entries)");
            ui.label("- Configurable output format");
            ui.label("- Interrupt support for FIFO events");
            ui.separator();
            ui.label("Used for:");
            ui.label("- DSI (Display Serial Interface)");
            ui.label("- High-speed data streaming");
        });
    }
}

/// HSTX state for the panel.
#[derive(Debug, Clone)]
pub struct HstxState {
    /// HSTX enabled
    pub enabled: bool,
    /// Number of entries in FIFO
    pub fifo_count: usize,
    /// FIFO data entries
    pub fifo_data: Vec<u32>,
    /// Interrupt enabled
    pub irq_enabled: bool,
    /// Interrupt pending
    pub irq_pending: bool,
    /// Configuration register
    pub config: u32,
}

impl Default for HstxState {
    fn default() -> Self {
        Self {
            enabled: false,
            fifo_count: 0,
            fifo_data: Vec::with_capacity(16),
            irq_enabled: false,
            irq_pending: false,
            config: 0,
        }
    }
}