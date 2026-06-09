//! MPU (Memory Protection Unit) panel for RP2350 simulator.

use egui::{Color32, RichText, Ui, Vec2};

/// MPU region state.
#[derive(Debug, Clone)]
pub struct MpuRegion {
    pub enabled: bool,
    pub base: u32,
    pub limit: u32,
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
    pub attr_index: u8,
}

impl Default for MpuRegion {
    fn default() -> Self {
        Self {
            enabled: false,
            base: 0,
            limit: 0,
            readable: true,
            writable: true,
            executable: true,
            attr_index: 0,
        }
    }
}

/// MPU state for the panel.
#[derive(Debug, Clone)]
pub struct MpuState {
    pub enabled: bool,
    pub regions: [MpuRegion; 8],
    /// Address to check against regions
    pub check_address: u32,
}

impl Default for MpuState {
    fn default() -> Self {
        Self {
            enabled: false,
            regions: Default::default(),
            check_address: 0x20000000,
        }
    }
}

/// MPU panel.
pub struct MpuPanel {
    selected_region: usize,
}

impl Default for MpuPanel {
    fn default() -> Self {
        Self {
            selected_region: 0,
        }
    }
}

impl MpuPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(&self) -> &'static str {
        "MPU"
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut MpuState) {
        ui.vertical(|ui| {
            ui.label(RichText::new("MPU - Memory Protection Unit").strong());
            ui.separator();

            // Global controls
            self.draw_global(ui, state);

            ui.add_space(8.0);

            // Region table
            self.draw_regions_table(ui, state);

            ui.add_space(8.0);

            // Region editor
            self.draw_region_editor(ui, state);

            ui.add_space(8.0);

            // Permission matrix
            self.draw_permission_matrix(ui, state);

            ui.add_space(8.0);

            // Address check
            self.draw_address_check(ui, state);

            ui.add_space(8.0);

            // Info
            self.draw_info(ui);
        });
    }

    fn draw_global(&self, ui: &mut Ui, state: &mut MpuState) {
        ui.group(|ui| {
            ui.label(RichText::new("Global Control").strong());
            ui.separator();

            ui.horizontal(|ui| {
                let color = if state.enabled {
                    Color32::GREEN
                } else {
                    Color32::GRAY
                };
                let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 5.0, color);
                ui.checkbox(&mut state.enabled, "MPU Enabled");
            });

            ui.add_space(4.0);

            let enabled_count = state.regions.iter().filter(|r| r.enabled).count();
            ui.horizontal(|ui| {
                ui.label("Active regions:");
                ui.label(
                    RichText::new(format!("{}/8", enabled_count))
                        .color(Color32::from_rgb(100, 200, 255)),
                );
            });

            // Quick actions
            ui.horizontal(|ui| {
                if ui.button("Enable All Regions").clicked() {
                    for region in &mut state.regions {
                        region.enabled = true;
                    }
                }
                if ui.button("Disable All Regions").clicked() {
                    for region in &mut state.regions {
                        region.enabled = false;
                    }
                }
            });
        });
    }

    fn draw_regions_table(&mut self, ui: &mut Ui, state: &mut MpuState) {
        ui.group(|ui| {
            ui.label(RichText::new("Memory Regions").strong());
            ui.separator();

            egui::Grid::new("mpu_regions_grid")
                .num_columns(7)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("#").strong());
                    ui.label(RichText::new("En").strong());
                    ui.label(RichText::new("Base").strong());
                    ui.label(RichText::new("Limit").strong());
                    ui.label(RichText::new("Perms").strong());
                    ui.label(RichText::new("Attr").strong());
                    ui.label(RichText::new("").strong());
                    ui.end_row();

                    for (i, region) in state.regions.iter().enumerate() {
                        let selected = self.selected_region == i;
                        let border_color = if region.enabled {
                            Color32::GREEN
                        } else {
                            Color32::GRAY
                        };

                        // Region number with selection indicator
                        let region_label = format!("{}", i);
                        if selected {
                            ui.label(RichText::new(region_label).color(Color32::YELLOW));
                        } else {
                            ui.label(region_label);
                        }

                        // Enabled checkbox (color indicator)
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
                        ui.painter()
                            .circle_filled(rect.center(), 5.0, border_color);

                        // Base address
                        ui.monospace(
                            RichText::new(format!("0x{:08X}", region.base))
                                .color(Color32::from_rgb(100, 200, 255)),
                        );

                        // Limit address
                        ui.monospace(
                            RichText::new(format!("0x{:08X}", region.limit))
                                .color(Color32::from_rgb(100, 200, 255)),
                        );

                        // Permissions (R/W/X)
                        let perms = format!(
                            "{}{}{}",
                            if region.readable { "R" } else { "-" },
                            if region.writable { "W" } else { "-" },
                            if region.executable { "X" } else { "-" }
                        );
                        ui.monospace(RichText::new(perms).color(Color32::from_rgb(255, 200, 100)));

                        // Attr index
                        ui.label(format!("{}", region.attr_index));

                        // Select button
                        if ui.small_button("Edit").clicked() {
                            self.selected_region = i;
                        }

                        ui.end_row();
                    }
                });
        });
    }

    fn draw_region_editor(&mut self, ui: &mut Ui, state: &mut MpuState) {
        ui.group(|ui| {
            ui.label(
                RichText::new(format!("Region {} Editor", self.selected_region)).strong(),
            );
            ui.separator();

            let region = &mut state.regions[self.selected_region];

            // Region selector
            ui.horizontal(|ui| {
                ui.label("Select Region:");
                for i in 0..8 {
                    let label = format!("{}", i);
                    if ui
                        .add(egui::SelectableLabel::new(
                            self.selected_region == i,
                            label,
                        ))
                        .clicked()
                    {
                        self.selected_region = i;
                    }
                }
            });

            ui.add_space(8.0);

            // Enable toggle
            ui.horizontal(|ui| {
                let color = if region.enabled {
                    Color32::GREEN
                } else {
                    Color32::GRAY
                };
                let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 5.0, color);
                ui.checkbox(&mut region.enabled, "Region Enabled");
            });

            ui.add_space(8.0);

            // Address range
            ui.horizontal(|ui| {
                ui.label("Base Address:");
                let mut base = region.base as i64;
                ui.add(egui::DragValue::new(&mut base).clamp_range(0..=i64::MAX));
                region.base = base as u32;
            });

            ui.horizontal(|ui| {
                ui.label("Limit Address:");
                let mut limit = region.limit as i64;
                ui.add(egui::DragValue::new(&mut limit).clamp_range(0..=i64::MAX));
                region.limit = limit as u32;
            });

            // Size display
            if region.limit >= region.base {
                let size = region.limit - region.base + 1;
                ui.horizontal(|ui| {
                    ui.label("Region Size:");
                    ui.monospace(
                        RichText::new(format!("{} bytes (0x{:X})", size, size))
                            .color(Color32::from_rgb(100, 200, 255)),
                    );
                });
            }

            ui.add_space(8.0);

            // Permissions
            ui.label(RichText::new("Permissions:").strong());
            ui.horizontal(|ui| {
                let r_color = if region.readable {
                    Color32::GREEN
                } else {
                    Color32::DARK_GRAY
                };
                let w_color = if region.writable {
                    Color32::GREEN
                } else {
                    Color32::DARK_GRAY
                };
                let x_color = if region.executable {
                    Color32::GREEN
                } else {
                    Color32::DARK_GRAY
                };

                let (rect_r, _) =
                    ui.allocate_exact_size(Vec2::new(24.0, 24.0), egui::Sense::click());
                let painter = ui.painter();
                painter.rect_filled(rect_r, 4.0, r_color);
                painter.text(
                    rect_r.center(),
                    egui::Align2::CENTER_CENTER,
                    "R",
                    egui::FontId::proportional(14.0),
                    Color32::WHITE,
                );
                if ui.interact(rect_r, ui.id().with("r_perm"), egui::Sense::click()).clicked() {
                    region.readable = !region.readable;
                }

                ui.add_space(4.0);

                let (rect_w, _) =
                    ui.allocate_exact_size(Vec2::new(24.0, 24.0), egui::Sense::click());
                let painter = ui.painter();
                painter.rect_filled(rect_w, 4.0, w_color);
                painter.text(
                    rect_w.center(),
                    egui::Align2::CENTER_CENTER,
                    "W",
                    egui::FontId::proportional(14.0),
                    Color32::WHITE,
                );
                if ui.interact(rect_w, ui.id().with("w_perm"), egui::Sense::click()).clicked() {
                    region.writable = !region.writable;
                }

                ui.add_space(4.0);

                let (rect_x, _) =
                    ui.allocate_exact_size(Vec2::new(24.0, 24.0), egui::Sense::click());
                let painter = ui.painter();
                painter.rect_filled(rect_x, 4.0, x_color);
                painter.text(
                    rect_x.center(),
                    egui::Align2::CENTER_CENTER,
                    "X",
                    egui::FontId::proportional(14.0),
                    Color32::WHITE,
                );
                if ui.interact(rect_x, ui.id().with("x_perm"), egui::Sense::click()).clicked() {
                    region.executable = !region.executable;
                }
            });

            ui.add_space(8.0);

            // Attribute index
            ui.horizontal(|ui| {
                ui.label("Memory Attribute Index:");
                let mut attr = region.attr_index as i32;
                ui.add(egui::DragValue::new(&mut attr).clamp_range(0..=7));
                region.attr_index = attr as u8;
            });

            ui.add_space(8.0);

            // Preset regions
            ui.label(RichText::new("Presets:").strong());
            ui.horizontal(|ui| {
                if ui.small_button("Flash (XIP)").clicked() {
                    region.base = 0x10000000;
                    region.limit = 0x10FFFFFF;
                    region.readable = true;
                    region.writable = false;
                    region.executable = true;
                }
                if ui.small_button("SRAM").clicked() {
                    region.base = 0x20000000;
                    region.limit = 0x2007FFFF;
                    region.readable = true;
                    region.writable = true;
                    region.executable = true;
                }
                if ui.small_button("Peripherals").clicked() {
                    region.base = 0x40000000;
                    region.limit = 0x4FFFFFFF;
                    region.readable = true;
                    region.writable = true;
                    region.executable = false;
                }
            });
        });
    }

    fn draw_permission_matrix(&self, ui: &mut Ui, state: &MpuState) {
        ui.group(|ui| {
            ui.label(RichText::new("Permission Matrix").strong());
            ui.separator();

            egui::Grid::new("permission_matrix")
                .num_columns(9)
                .spacing([6.0, 4.0])
                .show(ui, |ui| {
                    // Header
                    ui.label("");
                    for i in 0..8 {
                        ui.label(RichText::new(format!("{}", i)).strong());
                    }
                    ui.end_row();

                    // R row
                    ui.label(RichText::new("R").color(Color32::GREEN));
                    for region in &state.regions {
                        let color = if region.enabled && region.readable {
                            Color32::GREEN
                        } else {
                            Color32::DARK_GRAY
                        };
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::new(20.0, 20.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, color);
                    }
                    ui.end_row();

                    // W row
                    ui.label(RichText::new("W").color(Color32::YELLOW));
                    for region in &state.regions {
                        let color = if region.enabled && region.writable {
                            Color32::YELLOW
                        } else {
                            Color32::DARK_GRAY
                        };
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::new(20.0, 20.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, color);
                    }
                    ui.end_row();

                    // X row
                    ui.label(RichText::new("X").color(Color32::from_rgb(255, 100, 100)));
                    for region in &state.regions {
                        let color = if region.enabled && region.executable {
                            Color32::from_rgb(255, 100, 100)
                        } else {
                            Color32::DARK_GRAY
                        };
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::new(20.0, 20.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, color);
                    }
                    ui.end_row();
                });

            ui.add_space(4.0);
            ui.label("Green = Read | Yellow = Write | Red = Execute | Gray = Disabled");
        });
    }

    fn draw_address_check(&mut self, ui: &mut Ui, state: &mut MpuState) {
        ui.group(|ui| {
            ui.label(RichText::new("Address Check").strong());
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Address:");
                let mut addr = state.check_address as i64;
                ui.add(egui::DragValue::new(&mut addr).clamp_range(0..=i64::MAX));
                state.check_address = addr as u32;
            });

            ui.add_space(8.0);

            // Find matching regions
            let matching_regions: Vec<(usize, &MpuRegion)> = state
                .regions
                .iter()
                .enumerate()
                .filter(|(_, r)| {
                    r.enabled
                        && state.check_address >= r.base
                        && state.check_address <= r.limit
                })
                .collect();

            if matching_regions.is_empty() {
                ui.horizontal(|ui| {
                    let (rect, _) =
                        ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
                    ui.painter()
                        .circle_filled(rect.center(), 5.0, Color32::GRAY);
                    ui.label(RichText::new("No matching region").color(Color32::GRAY));
                });
            } else {
                ui.label(RichText::new("Matching regions:").strong());
                for (idx, region) in &matching_regions {
                    ui.horizontal(|ui| {
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
                        ui.painter()
                            .circle_filled(rect.center(), 5.0, Color32::GREEN);
                        ui.label(format!("Region {}:", idx));
                        ui.monospace(
                            RichText::new(format!("0x{:08X}-0x{:08X}", region.base, region.limit))
                                .color(Color32::from_rgb(100, 200, 255)),
                        );

                        let perms = format!(
                            "{}{}{}",
                            if region.readable { "R" } else { "-" },
                            if region.writable { "W" } else { "-" },
                            if region.executable { "X" } else { "-" }
                        );
                        ui.label(
                            RichText::new(perms).color(Color32::from_rgb(255, 200, 100)),
                        );
                    });
                }
            }

            ui.add_space(8.0);

            // Quick address buttons
            ui.horizontal(|ui| {
                if ui.small_button("Flash Base").clicked() {
                    state.check_address = 0x10000000;
                }
                if ui.small_button("SRAM Base").clicked() {
                    state.check_address = 0x20000000;
                }
                if ui.small_button("Periph Base").clicked() {
                    state.check_address = 0x40000000;
                }
                if ui.small_button("ROM Base").clicked() {
                    state.check_address = 0x00000000;
                }
            });
        });
    }

    fn draw_info(&self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("Info").strong());
            ui.separator();
            ui.label("RP2350 MPU (PMSA - Protected Memory System Architecture):");
            ui.label("- 8 configurable memory regions");
            ui.label("- Each region has base/limit address range");
            ui.label("- R/W/X permissions per region");
            ui.label("- Memory attribute index (cache policy, etc.)");
            ui.separator();
            ui.label("Region priorities: Lower region number = higher priority");
            ui.label("When regions overlap, the lower-numbered region takes precedence.");
        });
    }
}