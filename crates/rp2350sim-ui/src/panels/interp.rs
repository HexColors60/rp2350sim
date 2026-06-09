//! Interp (Interpolator) panel for RP2350 simulator.

use egui::{Color32, RichText, Ui};

/// Interpolator state for the panel.
#[derive(Debug, Clone)]
pub struct InterpState {
    // Lane 0
    pub accum0: u32,
    pub base0: u32,
    pub base1: u32,
    pub ctrl0: u32,
    // Lane 1
    pub accum1: u32,
    pub ctrl1: u32,
    // Result
    pub peek0: u32,
    pub peek1: u32,
    pub signed_mode: bool,
}

impl Default for InterpState {
    fn default() -> Self {
        Self {
            accum0: 0,
            base0: 0,
            base1: 0,
            ctrl0: 0,
            accum1: 0,
            ctrl1: 0,
            peek0: 0,
            peek1: 0,
            signed_mode: false,
        }
    }
}

/// Interpolator panel.
pub struct InterpPanel {
    selected_lane: usize,
}

impl Default for InterpPanel {
    fn default() -> Self {
        Self {
            selected_lane: 0,
        }
    }
}

impl InterpPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(&self) -> &'static str {
        "Interpolator"
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut InterpState) {
        ui.vertical(|ui| {
            ui.label(RichText::new("Interpolator Panel").strong());
            ui.separator();

            // Lane selection
            self.draw_lane_selector(ui);

            ui.add_space(8.0);

            // Lane 0
            self.draw_lane0(ui, state);

            ui.add_space(8.0);

            // Lane 1
            self.draw_lane1(ui, state);

            ui.add_space(8.0);

            // Control
            self.draw_control(ui, state);

            ui.add_space(8.0);

            // Results
            self.draw_results(ui, state);

            ui.add_space(8.0);

            // Operations
            self.draw_operations(ui, state);

            ui.add_space(8.0);

            // Info
            self.draw_info(ui);
        });
    }

    fn draw_lane_selector(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("Lane Selection").strong());
            ui.separator();

            ui.horizontal(|ui| {
                ui.radio_value(&mut self.selected_lane, 0, "Lane 0");
                ui.radio_value(&mut self.selected_lane, 1, "Lane 1");
            });
        });
    }

    fn draw_lane0(&self, ui: &mut Ui, state: &mut InterpState) {
        ui.group(|ui| {
            ui.label(RichText::new("Lane 0").strong());
            ui.separator();

            // Accumulator
            ui.horizontal(|ui| {
                ui.label("ACCUM0:");
                let mut accum = state.accum0 as i64;
                ui.add(egui::DragValue::new(&mut accum).clamp_range(0i64..=0xFFFFFFFFi64));
                ui.monospace(format!("(0x{:08X})", accum as u32));
                state.accum0 = accum as u32;
            });

            ui.add_space(4.0);

            // BASE0
            ui.horizontal(|ui| {
                ui.label("BASE0:");
                let mut base = state.base0 as i64;
                ui.add(egui::DragValue::new(&mut base).clamp_range(0i64..=0xFFFFFFFFi64));
                ui.monospace(format!("(0x{:08X})", base as u32));
                state.base0 = base as u32;
            });

            ui.add_space(4.0);

            // BASE1
            ui.horizontal(|ui| {
                ui.label("BASE1:");
                let mut base = state.base1 as i64;
                ui.add(egui::DragValue::new(&mut base).clamp_range(0i64..=0xFFFFFFFFi64));
                ui.monospace(format!("(0x{:08X})", base as u32));
                state.base1 = base as u32;
            });

            ui.add_space(4.0);

            // CTRL0 display
            ui.horizontal(|ui| {
                ui.label("CTRL0:");
                ui.monospace(
                    RichText::new(format!("0x{:08X}", state.ctrl0))
                        .color(Color32::from_rgb(100, 200, 255)),
                );
            });

            // CTRL0 bit fields
            self.draw_ctrl_fields(ui, state.ctrl0, 0);
        });
    }

    fn draw_lane1(&self, ui: &mut Ui, state: &mut InterpState) {
        ui.group(|ui| {
            ui.label(RichText::new("Lane 1").strong());
            ui.separator();

            // Accumulator
            ui.horizontal(|ui| {
                ui.label("ACCUM1:");
                let mut accum = state.accum1 as i64;
                ui.add(egui::DragValue::new(&mut accum).clamp_range(0i64..=0xFFFFFFFFi64));
                ui.monospace(format!("(0x{:08X})", accum as u32));
                state.accum1 = accum as u32;
            });

            ui.add_space(4.0);

            // CTRL1 display
            ui.horizontal(|ui| {
                ui.label("CTRL1:");
                ui.monospace(
                    RichText::new(format!("0x{:08X}", state.ctrl1))
                        .color(Color32::from_rgb(100, 200, 255)),
                );
            });

            // CTRL1 bit fields
            self.draw_ctrl_fields(ui, state.ctrl1, 1);
        });
    }

    fn draw_ctrl_fields(&self, ui: &mut Ui, ctrl: u32, lane: usize) {
        ui.add_space(4.0);

        egui::Grid::new(format!("ctrl{}_grid", lane))
            .num_columns(2)
            .spacing([10.0, 2.0])
            .show(ui, |ui| {
                // SHIFT field (bits 0-4)
                let shift = ctrl & 0x1F;
                ui.label("SHIFT:");
                ui.monospace(format!("{}", shift));
                ui.end_row();

                // MASK_LSB field (bits 5-9)
                let mask_lsb = (ctrl >> 5) & 0x1F;
                ui.label("MASK_LSB:");
                ui.monospace(format!("{}", mask_lsb));
                ui.end_row();

                // MASK_MSB field (bits 10-14)
                let mask_msb = (ctrl >> 10) & 0x1F;
                ui.label("MASK_MSB:");
                ui.monospace(format!("{}", mask_msb));
                ui.end_row();

                // SIGNED bit (bit 30)
                let signed = (ctrl >> 30) & 1;
                ui.label("SIGNED:");
                ui.monospace(
                    RichText::new(if signed != 0 { "Yes" } else { "No" })
                        .color(if signed != 0 { Color32::YELLOW } else { Color32::GRAY }),
                );
                ui.end_row();

                // CROSS_INPUT bit (bit 29)
                let cross_input = (ctrl >> 29) & 1;
                ui.label("CROSS_INPUT:");
                ui.label(if cross_input != 0 { "Enabled" } else { "Disabled" });
                ui.end_row();

                // ADD_RAW bit (bit 28)
                let add_raw = (ctrl >> 28) & 1;
                ui.label("ADD_RAW:");
                ui.label(if add_raw != 0 { "Yes" } else { "No" });
                ui.end_row();

                // CLAMP bit (bit 22, lane 0 only)
                if lane == 0 {
                    let clamp = (ctrl >> 22) & 1;
                    ui.label("CLAMP:");
                    ui.label(if clamp != 0 { "Enabled" } else { "Disabled" });
                    ui.end_row();
                }

                // OVERF bit (bit 23, lane 0 only)
                if lane == 0 {
                    let overf = (ctrl >> 23) & 1;
                    ui.label("OVERF:");
                    ui.label(if overf != 0 { "Yes" } else { "No" });
                    ui.end_row();
                }
            });
    }

    fn draw_control(&self, ui: &mut Ui, state: &mut InterpState) {
        ui.group(|ui| {
            ui.label(RichText::new("Control").strong());
            ui.separator();

            // Signed mode toggle
            ui.horizontal(|ui| {
                ui.label("Signed Mode:");
                ui.checkbox(&mut state.signed_mode, "Enable signed arithmetic");
            });

            ui.add_space(4.0);

            // Update CTRL registers based on signed mode
            if state.signed_mode {
                state.ctrl0 |= 1 << 30;
                state.ctrl1 |= 1 << 30;
            } else {
                state.ctrl0 &= !(1 << 30);
                state.ctrl1 &= !(1 << 30);
            }

            // Mode description
            ui.label(if state.signed_mode {
                "Signed mode: Values are interpreted as signed 32-bit integers"
            } else {
                "Unsigned mode: Values are interpreted as unsigned 32-bit integers"
            });
        });
    }

    fn draw_results(&self, ui: &mut Ui, state: &mut InterpState) {
        ui.group(|ui| {
            ui.label(RichText::new("Results (PEEK)").strong());
            ui.separator();

            // Calculate simulated peek values (simplified: just show ACCUM + BASE)
            let peek0 = state.accum0.wrapping_add(state.base0);
            let peek1 = state.accum1.wrapping_add(state.base1);

            // PEEK0
            ui.horizontal(|ui| {
                ui.label("PEEK0:");
                ui.monospace(
                    RichText::new(format!("0x{:08X}", peek0))
                        .color(Color32::from_rgb(0, 255, 150)),
                );

                // Show as signed if enabled
                if state.signed_mode {
                    let signed_val = peek0 as i32;
                    ui.label(RichText::new(format!("({})", signed_val)).color(Color32::GRAY));
                }
            });

            ui.add_space(4.0);

            // PEEK1
            ui.horizontal(|ui| {
                ui.label("PEEK1:");
                ui.monospace(
                    RichText::new(format!("0x{:08X}", peek1))
                        .color(Color32::from_rgb(0, 255, 150)),
                );

                // Show as signed if enabled
                if state.signed_mode {
                    let signed_val = peek1 as i32;
                    ui.label(RichText::new(format!("({})", signed_val)).color(Color32::GRAY));
                }
            });

            ui.add_space(8.0);

            // Result visualization bar
            ui.label("Lane 0 Result Visualization:");
            self.draw_result_bar(ui, peek0, state.signed_mode);

            ui.add_space(4.0);

            ui.label("Lane 1 Result Visualization:");
            self.draw_result_bar(ui, peek1, state.signed_mode);
        });
    }

    fn draw_result_bar(&self, ui: &mut Ui, value: u32, signed: bool) {
        let (rect, _) =
            ui.allocate_exact_size(egui::Vec2::new(ui.available_width(), 20.0), egui::Sense::hover());

        // Draw background
        ui.painter()
            .rect_filled(rect, 4.0, Color32::from_rgb(40, 40, 50));

        // Calculate fill ratio
        let fill_ratio = if signed {
            let signed_val = value as i32;
            (signed_val as f64 / i32::MAX as f64).clamp(0.0, 1.0)
        } else {
            value as f64 / u32::MAX as f64
        };

        let filled_width = rect.width() * fill_ratio as f32;
        let filled_rect =
            egui::Rect::from_min_size(rect.min, egui::vec2(filled_width, rect.height()));

        // Color based on value
        let color = if signed {
            let signed_val = value as i32;
            if signed_val < 0 {
                Color32::from_rgb(255, 100, 100)
            } else if signed_val < 0x40000000 {
                Color32::GREEN
            } else {
                Color32::from_rgb(100, 200, 255)
            }
        } else {
            if value < 0x40000000 {
                Color32::GREEN
            } else if value < 0x80000000 {
                Color32::YELLOW
            } else {
                Color32::from_rgb(100, 200, 255)
            }
        };

        ui.painter().rect_filled(filled_rect, 4.0, color);
    }

    fn draw_operations(&self, ui: &mut Ui, state: &mut InterpState) {
        ui.group(|ui| {
            ui.label(RichText::new("Operations").strong());
            ui.separator();

            // Add to accumulator buttons
            ui.label("Add to Accumulator:");

            ui.horizontal(|ui| {
                if ui.button("Add 1 to ACCUM0").clicked() {
                    state.accum0 = state.accum0.wrapping_add(1);
                }
                if ui.button("Add 10 to ACCUM0").clicked() {
                    state.accum0 = state.accum0.wrapping_add(10);
                }
                if ui.button("Add 0x100 to ACCUM0").clicked() {
                    state.accum0 = state.accum0.wrapping_add(0x100);
                }
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                if ui.button("Add 1 to ACCUM1").clicked() {
                    state.accum1 = state.accum1.wrapping_add(1);
                }
                if ui.button("Add 10 to ACCUM1").clicked() {
                    state.accum1 = state.accum1.wrapping_add(10);
                }
                if ui.button("Add 0x100 to ACCUM1").clicked() {
                    state.accum1 = state.accum1.wrapping_add(0x100);
                }
            });

            ui.add_space(8.0);

            // Reset buttons
            ui.label("Reset:");

            ui.horizontal(|ui| {
                if ui.button("Reset Lane 0").clicked() {
                    state.accum0 = 0;
                    state.base0 = 0;
                    state.base1 = 0;
                    state.ctrl0 = 0;
                }
                if ui.button("Reset Lane 1").clicked() {
                    state.accum1 = 0;
                    state.ctrl1 = 0;
                }
                if ui.button("Reset All").clicked() {
                    *state = InterpState::default();
                }
            });

            ui.add_space(8.0);

            // Preset values
            ui.label("Presets:");

            ui.horizontal(|ui| {
                if ui.small_button("Graphics Mode").clicked() {
                    // Typical setup for pixel blending
                    state.ctrl0 = 0b00_001_0000_0000_00000_00000_00000;
                    state.ctrl1 = 0b00_001_0000_0000_00000_00000_00000;
                    state.base0 = 0;
                    state.base1 = 0xFF;
                }
                if ui.small_button("Audio Mode").clicked() {
                    // Typical setup for audio sample interpolation
                    state.ctrl0 = 0b00_000_0000_0000_00000_00000_10000; // SHIFT=16
                    state.ctrl1 = 0b00_000_0000_0000_00000_00000_10000;
                }
            });
        });
    }

    fn draw_info(&self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("Info").strong());
            ui.separator();

            ui.label("The Interpolator is a hardware accelerator for:");
            ui.label("- Graphics: texture mapping, color blending");
            ui.label("- Audio: sample interpolation, mixing");
            ui.label("- Fixed-point arithmetic operations");
            ui.separator();

            egui::Grid::new("interp_info_grid")
                .num_columns(2)
                .spacing([10.0, 2.0])
                .show(ui, |ui| {
                    ui.label("Base address:");
                    ui.monospace(RichText::new("0x400D_0000").color(Color32::from_rgb(100, 200, 255)));
                    ui.end_row();

                    ui.label("Lanes:");
                    ui.label("2 per interpolator");
                    ui.end_row();

                    ui.label("Registers:");
                    ui.label("ACCUM, BASE0, BASE1, CTRL, PEEK");
                    ui.end_row();

                    ui.label("Modes:");
                    ui.label("Signed/Unsigned");
                    ui.end_row();
                });

            ui.separator();

            ui.label("CTRL register fields:");
            ui.label("  SHIFT[4:0]    - Shift amount for accumulator");
            ui.label("  MASK_LSB[9:5] - Mask LSB position");
            ui.label("  MASK_MSB[14:10] - Mask MSB position");
            ui.label("  SIGNED[30]    - Signed mode enable");
            ui.label("  CROSS_INPUT[29] - Use lane 1 result as input");
            ui.label("  ADD_RAW[28]   - Add raw accumulator (no shift)");
        });
    }
}