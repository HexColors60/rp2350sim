//! CoreSight Debug Components panel for RP2350 simulator.

use egui::{Color32, RichText, Ui};

/// CoreSight state for the panel.
#[derive(Debug, Clone)]
pub struct CoreSightState {
    // DWT - Data Watchpoint and Trace
    /// Cycle counter enabled
    pub cyccnt_enabled: bool,
    /// Cycle count value
    pub cycle_count: u32,
    /// DWT comparator values
    pub comparator_values: [u32; 4],
    /// DWT comparator enabled flags
    pub comparator_enabled: [bool; 4],
    // FPB - Flash Patch and Breakpoint
    /// FPB enabled
    pub fpb_enabled: bool,
    /// Hardware breakpoint addresses
    pub breakpoints: [u32; 8],
    /// Hardware breakpoint enabled flags
    pub breakpoints_enabled: [bool; 8],
    // ITM - Instrumentation Trace Macrocell
    /// ITM enabled
    pub itm_enabled: bool,
    /// ITM stimulus ports enabled mask
    pub itm_stim_ports: u32,
}

impl Default for CoreSightState {
    fn default() -> Self {
        Self {
            cyccnt_enabled: false,
            cycle_count: 0,
            comparator_values: [0; 4],
            comparator_enabled: [false; 4],
            fpb_enabled: false,
            breakpoints: [0; 8],
            breakpoints_enabled: [false; 8],
            itm_enabled: false,
            itm_stim_ports: 0,
        }
    }
}

/// CoreSight Debug Components panel.
pub struct CoreSightPanel;

impl Default for CoreSightPanel {
    fn default() -> Self {
        Self
    }
}

impl CoreSightPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(&self) -> &'static str {
        "CoreSight"
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut CoreSightState) {
        ui.vertical(|ui| {
            ui.label(RichText::new("CoreSight Debug Components").strong());
            ui.separator();

            // DWT - Cycle Counter
            self.draw_cycle_counter(ui, state);

            ui.add_space(8.0);

            // DWT - Comparators
            self.draw_comparators(ui, state);

            ui.add_space(8.0);

            // FPB - Breakpoints
            self.draw_breakpoints(ui, state);

            ui.add_space(8.0);

            // ITM - Trace
            self.draw_itm(ui, state);

            ui.add_space(8.0);

            // Controls
            self.draw_controls(ui, state);

            ui.add_space(8.0);

            // Info
            self.draw_info(ui);
        });
    }

    fn draw_cycle_counter(&self, ui: &mut Ui, state: &mut CoreSightState) {
        ui.group(|ui| {
            ui.label(RichText::new("DWT - Cycle Counter").strong());
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Enable:");
                ui.checkbox(&mut state.cyccnt_enabled, "");
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Cycle Count:");
                let color = if state.cyccnt_enabled {
                    Color32::from_rgb(100, 200, 255)
                } else {
                    Color32::GRAY
                };
                ui.monospace(RichText::new(format!("0x{:08X} ({})", state.cycle_count, state.cycle_count)).color(color));
            });

            ui.add_space(4.0);

            // Simulate cycle increment when enabled
            if state.cyccnt_enabled {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(8.0, 8.0), egui::Sense::hover());
                    ui.painter().circle_filled(rect.center(), 4.0, Color32::GREEN);
                    ui.label(RichText::new("Running").color(Color32::GREEN));
                });
            } else {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(8.0, 8.0), egui::Sense::hover());
                    ui.painter().circle_filled(rect.center(), 4.0, Color32::GRAY);
                    ui.label(RichText::new("Stopped").color(Color32::GRAY));
                });
            }
        });
    }

    fn draw_comparators(&self, ui: &mut Ui, state: &mut CoreSightState) {
        ui.group(|ui| {
            ui.label(RichText::new("DWT - Comparators").strong());
            ui.separator();

            egui::Grid::new("dwt_comparators_grid")
                .num_columns(4)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Comp");
                    ui.label("Address");
                    ui.label("Enabled");
                    ui.label("Status");
                    ui.end_row();

                    for i in 0..4 {
                        ui.label(format!("C{}", i));

                        let mut value = state.comparator_values[i];
                        ui.add(egui::DragValue::new(&mut value).clamp_range(0..=u32::MAX));

                        let mut enabled = state.comparator_enabled[i];
                        ui.checkbox(&mut enabled, "");
                        state.comparator_enabled[i] = enabled;

                        if enabled {
                            ui.label(RichText::new("Active").color(Color32::GREEN));
                        } else {
                            ui.label(RichText::new("Disabled").color(Color32::GRAY));
                        }

                        ui.end_row();
                    }
                });

            ui.add_space(4.0);
            ui.label("Comparators can trigger on data addresses, PC values, or cycle count matches.");
        });
    }

    fn draw_breakpoints(&self, ui: &mut Ui, state: &mut CoreSightState) {
        ui.group(|ui| {
            ui.label(RichText::new("FPB - Hardware Breakpoints").strong());
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("FPB Enable:");
                ui.checkbox(&mut state.fpb_enabled, "");
            });

            ui.add_space(4.0);

            // Breakpoint count
            let bp_count = state.breakpoints_enabled.iter().filter(|&&e| e).count();
            ui.horizontal(|ui| {
                ui.label("Active Breakpoints:");
                ui.monospace(RichText::new(format!("{}/8", bp_count)).color(
                    if bp_count > 0 { Color32::YELLOW } else { Color32::GRAY }
                ));
            });

            ui.add_space(4.0);

            egui::Grid::new("fpb_breakpoints_grid")
                .num_columns(4)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    ui.label("BP");
                    ui.label("Address");
                    ui.label("Enabled");
                    ui.label("Type");
                    ui.end_row();

                    for i in 0..8 {
                        ui.label(format!("BP{}", i));

                        let mut addr = state.breakpoints[i];
                        ui.add(egui::DragValue::new(&mut addr).clamp_range(0..=u32::MAX));

                        let mut enabled = state.breakpoints_enabled[i];
                        ui.checkbox(&mut enabled, "");
                        state.breakpoints_enabled[i] = enabled;

                        if enabled {
                            // Determine if this is flash or RAM address
                            let bp_type = if state.breakpoints[i] >= 0x10000000 && state.breakpoints[i] < 0x15000000 {
                                "Flash"
                            } else if state.breakpoints[i] >= 0x20000000 && state.breakpoints[i] < 0x20082000 {
                                "RAM"
                            } else {
                                "Other"
                            };
                            ui.label(RichText::new(bp_type).color(Color32::from_rgb(255, 165, 0)));
                        } else {
                            ui.label(RichText::new("-").color(Color32::GRAY));
                        }

                        ui.end_row();
                    }
                });
        });
    }

    fn draw_itm(&self, ui: &mut Ui, state: &mut CoreSightState) {
        ui.group(|ui| {
            ui.label(RichText::new("ITM - Instrumentation Trace Macrocell").strong());
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("ITM Enable:");
                ui.checkbox(&mut state.itm_enabled, "");
            });

            ui.add_space(4.0);

            ui.label("Stimulus Ports (0-31):");
            self.draw_port_bits(ui, state.itm_stim_ports);

            ui.add_space(4.0);

            let active_ports = (0..32).filter(|i| (state.itm_stim_ports & (1 << i)) != 0).count();
            ui.horizontal(|ui| {
                ui.label("Active Ports:");
                ui.monospace(RichText::new(format!("{}", active_ports)).color(
                    if active_ports > 0 { Color32::from_rgb(100, 200, 255) } else { Color32::GRAY }
                ));
            });
        });
    }

    fn draw_port_bits(&self, ui: &mut Ui, ports: u32) {
        ui.horizontal(|ui| {
            for i in 0..32 {
                if i > 0 && i % 16 == 0 {
                    ui.end_row();
                }
                let is_set = (ports & (1 << i)) != 0;
                let color = if is_set { Color32::GREEN } else { Color32::from_rgb(60, 60, 60) };
                let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(12.0, 12.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 2.0, color);
                if i % 4 == 0 {
                    ui.label(format!("{}", i));
                }
            }
        });
    }

    fn draw_controls(&self, ui: &mut Ui, state: &mut CoreSightState) {
        ui.group(|ui| {
            ui.label(RichText::new("Controls").strong());
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Reset Cycle Counter").clicked() {
                    state.cycle_count = 0;
                }
                if ui.button("Enable All Comparators").clicked() {
                    state.comparator_enabled = [true; 4];
                }
                if ui.button("Disable All Comparators").clicked() {
                    state.comparator_enabled = [false; 4];
                }
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                if ui.button("Clear All Breakpoints").clicked() {
                    state.breakpoints = [0; 8];
                    state.breakpoints_enabled = [false; 8];
                }
                if ui.button("Reset All").clicked() {
                    *state = CoreSightState::default();
                }
            });

            ui.add_space(4.0);

            // Manual cycle increment for testing
            ui.horizontal(|ui| {
                if ui.button("+1000 Cycles").clicked() {
                    state.cycle_count = state.cycle_count.wrapping_add(1000);
                }
                if ui.button("+1M Cycles").clicked() {
                    state.cycle_count = state.cycle_count.wrapping_add(1_000_000);
                }
            });
        });
    }

    fn draw_info(&self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("Info").strong());
            ui.separator();

            ui.label("CoreSight Debug Components for ARM Cortex-M33:");
            ui.label("- Base address: 0xE000_0000 (PPB)");
            ui.separator();

            ui.label("DWT (Data Watchpoint and Trace):");
            ui.label("- 32-bit cycle counter (CYCCNT)");
            ui.label("- 4 comparators for address/data matching");
            ui.label("- Supports watchpoints and event tracing");
            ui.separator();

            ui.label("FPB (Flash Patch and Breakpoint):");
            ui.label("- 8 hardware breakpoints");
            ui.label("- Can patch flash instructions for debugging");
            ui.separator();

            ui.label("ITM (Instrumentation Trace Macrocell):");
            ui.label("- 32 stimulus ports for software trace");
            ui.label("- Timestamped trace output via SWO");
        });
    }
}