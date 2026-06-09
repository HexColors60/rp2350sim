//! Bus Controller panel for RP2350 simulator.

use egui::{Color32, RichText, Ui};

/// Bus Controller state for the panel.
#[derive(Debug, Clone)]
pub struct BusCtrlState {
    /// Priority for Processor 0 (0-255)
    pub proc0_priority: u8,
    /// Priority for Processor 1 (0-255)
    pub proc1_priority: u8,
    /// Priority for DMA Read (0-255)
    pub dma_r_priority: u8,
    /// Priority for DMA Write (0-255)
    pub dma_w_priority: u8,
    /// Performance counter values
    pub perf_counters: [u32; 4],
    /// Performance event selectors
    pub perf_events: [u8; 4],
    /// Bus is currently contested (multiple masters requesting)
    pub contested: bool,
}

impl Default for BusCtrlState {
    fn default() -> Self {
        Self {
            proc0_priority: 0,
            proc1_priority: 0,
            dma_r_priority: 0,
            dma_w_priority: 0,
            perf_counters: [0; 4],
            perf_events: [0; 4],
            contested: false,
        }
    }
}

/// Bus Controller panel.
pub struct BusCtrlPanel;

impl Default for BusCtrlPanel {
    fn default() -> Self {
        Self
    }
}

impl BusCtrlPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(&self) -> &'static str {
        "Bus Controller"
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut BusCtrlState) {
        ui.vertical(|ui| {
            ui.label(RichText::new("Bus Controller - Bus Arbiter").strong());
            ui.separator();

            // Priority section
            self.draw_priorities(ui, state);

            ui.add_space(8.0);

            // Arbitration section
            self.draw_arbitration(ui, state);

            ui.add_space(8.0);

            // Performance counters section
            self.draw_perf_counters(ui, state);

            ui.add_space(8.0);

            // Status section
            self.draw_status(ui, state);

            ui.add_space(8.0);

            // Actions section
            self.draw_actions(ui, state);

            ui.add_space(8.0);

            // Info section
            self.draw_info(ui);
        });
    }

    fn draw_priorities(&self, ui: &mut Ui, state: &mut BusCtrlState) {
        ui.group(|ui| {
            ui.label(RichText::new("Bus Master Priorities").strong());
            ui.separator();

            // Proc0 priority
            ui.horizontal(|ui| {
                ui.label("Processor 0:");
                let mut priority = state.proc0_priority as u32;
                ui.add(egui::Slider::new(&mut priority, 0..=255));
                state.proc0_priority = priority as u8;
                ui.label(priority_label(priority as u8));
            });

            ui.add_space(4.0);

            // Proc1 priority
            ui.horizontal(|ui| {
                ui.label("Processor 1:");
                let mut priority = state.proc1_priority as u32;
                ui.add(egui::Slider::new(&mut priority, 0..=255));
                state.proc1_priority = priority as u8;
                ui.label(priority_label(priority as u8));
            });

            ui.add_space(4.0);

            // DMA Read priority
            ui.horizontal(|ui| {
                ui.label("DMA Read:");
                let mut priority = state.dma_r_priority as u32;
                ui.add(egui::Slider::new(&mut priority, 0..=255));
                state.dma_r_priority = priority as u8;
                ui.label(priority_label(priority as u8));
            });

            ui.add_space(4.0);

            // DMA Write priority
            ui.horizontal(|ui| {
                ui.label("DMA Write:");
                let mut priority = state.dma_w_priority as u32;
                ui.add(egui::Slider::new(&mut priority, 0..=255));
                state.dma_w_priority = priority as u8;
                ui.label(priority_label(priority as u8));
            });

            ui.add_space(4.0);
            ui.label("Priority range: 0 (lowest) to 255 (highest)");
        });
    }

    fn draw_arbitration(&self, ui: &mut Ui, state: &BusCtrlState) {
        ui.group(|ui| {
            ui.label(RichText::new("Arbitration Winner").strong());
            ui.separator();

            // Determine winner based on priority
            let winner = determine_winner(state);

            ui.horizontal(|ui| {
                ui.label("Current Winner:");
                let (rect, _) = ui.allocate_exact_size(
                    egui::Vec2::new(12.0, 12.0),
                    egui::Sense::hover(),
                );
                ui.painter().circle_filled(rect.center(), 5.0, winner.1);
                ui.label(RichText::new(winner.0).color(winner.1));
            });

            ui.add_space(4.0);

            // Show priority comparison
            ui.label("Priority Comparison:");
            egui::Grid::new("priority_grid")
                .num_columns(3)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Master");
                    ui.label("Priority");
                    ui.label("Status");
                    ui.end_row();

                    // Proc0
                    ui.label("Proc0");
                    ui.monospace(RichText::new(format!("{}", state.proc0_priority))
                        .color(priority_color(state.proc0_priority)));
                    ui.label(if winner.0 == "Processor 0" { "WINNER" } else { "" });
                    ui.end_row();

                    // Proc1
                    ui.label("Proc1");
                    ui.monospace(RichText::new(format!("{}", state.proc1_priority))
                        .color(priority_color(state.proc1_priority)));
                    ui.label(if winner.0 == "Processor 1" { "WINNER" } else { "" });
                    ui.end_row();

                    // DMA Read
                    ui.label("DMA R");
                    ui.monospace(RichText::new(format!("{}", state.dma_r_priority))
                        .color(priority_color(state.dma_r_priority)));
                    ui.label(if winner.0 == "DMA Read" { "WINNER" } else { "" });
                    ui.end_row();

                    // DMA Write
                    ui.label("DMA W");
                    ui.monospace(RichText::new(format!("{}", state.dma_w_priority))
                        .color(priority_color(state.dma_w_priority)));
                    ui.label(if winner.0 == "DMA Write" { "WINNER" } else { "" });
                    ui.end_row();
                });
        });
    }

    fn draw_perf_counters(&self, ui: &mut Ui, state: &mut BusCtrlState) {
        ui.group(|ui| {
            ui.label(RichText::new("Performance Counters").strong());
            ui.separator();

            for i in 0..4 {
                ui.horizontal(|ui| {
                    ui.label(format!("Counter {}:", i));
                    ui.monospace(
                        RichText::new(format!("0x{:08X}", state.perf_counters[i]))
                            .color(Color32::from_rgb(100, 200, 255)),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label(format!("  Event {}:", i));
                    let mut event = state.perf_events[i] as u32;
                    ui.add(egui::DragValue::new(&mut event).clamp_range(0..=15));
                    state.perf_events[i] = event as u8;
                    ui.label(event_name(event as u8));
                });

                ui.add_space(4.0);
            }
        });
    }

    fn draw_status(&self, ui: &mut Ui, state: &BusCtrlState) {
        ui.group(|ui| {
            ui.label(RichText::new("Status").strong());
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Bus Contested:");
                let (rect, _) = ui.allocate_exact_size(
                    egui::Vec2::new(12.0, 12.0),
                    egui::Sense::hover(),
                );
                let color = if state.contested {
                    Color32::YELLOW
                } else {
                    Color32::GREEN
                };
                ui.painter().circle_filled(rect.center(), 5.0, color);
                ui.label(if state.contested { "Yes" } else { "No" });
            });
        });
    }

    fn draw_actions(&self, ui: &mut Ui, state: &mut BusCtrlState) {
        ui.group(|ui| {
            ui.label(RichText::new("Actions").strong());
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Reset Counters").clicked() {
                    state.perf_counters = [0; 4];
                }
                if ui.button("Reset Priorities").clicked() {
                    state.proc0_priority = 0;
                    state.proc1_priority = 0;
                    state.dma_r_priority = 0;
                    state.dma_w_priority = 0;
                }
                if ui.button("Reset All").clicked() {
                    *state = BusCtrlState::default();
                }
            });
        });
    }

    fn draw_info(&self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("Info").strong());
            ui.separator();
            ui.label("The Bus Controller manages bus arbitration:");
            ui.label("- Base address: 0x4003_0000");
            ui.label("- 4 bus masters: Proc0, Proc1, DMA R, DMA W");
            ui.label("- Priority range: 0-255 (higher wins)");
            ui.label("- 4 performance counters with configurable events");
            ui.separator();
            ui.label("Performance Events:");
            ui.label("  0: Read accesses, 1: Write accesses");
            ui.label("  2: Contested accesses, 3: Wait cycles");
        });
    }
}

/// Get color for priority value.
fn priority_color(priority: u8) -> Color32 {
    if priority >= 192 {
        Color32::GREEN
    } else if priority >= 128 {
        Color32::from_rgb(144, 238, 144) // Light green
    } else if priority >= 64 {
        Color32::YELLOW
    } else if priority > 0 {
        Color32::from_rgb(255, 200, 100) // Orange-yellow
    } else {
        Color32::GRAY
    }
}

/// Create a priority label with color.
fn priority_label(priority: u8) -> egui::RichText {
    let text = if priority >= 192 {
        "HIGH"
    } else if priority >= 128 {
        "MEDIUM-HIGH"
    } else if priority >= 64 {
        "MEDIUM"
    } else if priority > 0 {
        "LOW"
    } else {
        "NONE"
    };
    RichText::new(text).color(priority_color(priority))
}

/// Determine the arbitration winner.
fn determine_winner(state: &BusCtrlState) -> (&'static str, Color32) {
    let priorities = [
        (state.proc0_priority, "Processor 0", Color32::from_rgb(100, 200, 255)),
        (state.proc1_priority, "Processor 1", Color32::from_rgb(100, 255, 200)),
        (state.dma_r_priority, "DMA Read", Color32::from_rgb(255, 200, 100)),
        (state.dma_w_priority, "DMA Write", Color32::from_rgb(255, 150, 100)),
    ];

    let max = priorities.iter().max_by_key(|(p, _, _)| p);
    match max {
        Some((p, name, color)) if *p > 0 => (*name, *color),
        _ => ("None (all zero)", Color32::GRAY),
    }
}

/// Get event name for event selector value.
fn event_name(event: u8) -> &'static str {
    match event {
        0 => "Read accesses",
        1 => "Write accesses",
        2 => "Contested",
        3 => "Wait cycles",
        4 => "Proc0 reads",
        5 => "Proc0 writes",
        6 => "Proc1 reads",
        7 => "Proc1 writes",
        8 => "DMA R reads",
        9 => "DMA W writes",
        10 => "Peripheral reads",
        11 => "Peripheral writes",
        12 => "FIFO stalls",
        13 => "Bus lock",
        14 => "Cache hits",
        15 => "Cache misses",
        _ => "Unknown",
    }
}