//! PLIC (Platform Level Interrupt Controller) panel for RP2350 simulator.

use egui::{Color32, RichText, Ui, Vec2};

/// PLIC panel for RISC-V interrupt controller.
pub struct PlicPanel {
    selected_irq: u32,
    selected_context: usize,
}

impl Default for PlicPanel {
    fn default() -> Self {
        Self {
            selected_irq: 0,
            selected_context: 0,
        }
    }
}

impl PlicPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(&self) -> &'static str {
        "PLIC"
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut PlicState) {
        ui.vertical(|ui| {
            ui.label(RichText::new("PLIC - Platform Level Interrupt Controller").strong());
            ui.separator();

            // Pending interrupts
            self.draw_pending(ui, state);

            ui.add_space(8.0);

            // Context 0
            self.draw_context(ui, state, 0);

            ui.add_space(8.0);

            // Context 1
            self.draw_context(ui, state, 1);

            ui.add_space(8.0);

            // Priority display
            self.draw_priorities(ui, state);

            ui.add_space(8.0);

            // Actions
            self.draw_actions(ui, state);

            ui.add_space(8.0);

            // Info
            self.draw_info(ui);
        });
    }

    fn draw_pending(&self, ui: &mut Ui, state: &PlicState) {
        ui.group(|ui| {
            ui.label(RichText::new("Pending Interrupts (IRQ 0-31)").strong());
            ui.separator();

            // Draw bits 0-31 from first pending word
            let pending_word = state.pending[0];
            self.draw_irq_bits(ui, pending_word, None, "IRQ");

            ui.add_space(4.0);

            // Summary
            let pending_count = (0..32).filter(|i| (pending_word & (1 << i)) != 0).count();
            ui.horizontal(|ui| {
                ui.label("Pending count:");
                ui.label(RichText::new(format!("{}", pending_count)).color(Color32::RED));
            });

            ui.add_space(4.0);

            // Full pending register display
            ui.label("Pending registers:");
            for (i, word) in state.pending.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("  Word {}:", i));
                    ui.monospace(RichText::new(format!("0x{:08X}", word))
                        .color(Color32::from_rgb(100, 200, 255)));
                });
            }
        });
    }

    fn draw_context(&mut self, ui: &mut Ui, state: &mut PlicState, ctx: usize) {
        ui.group(|ui| {
            ui.label(RichText::new(format!("Context {} (Core {})", ctx, ctx)).strong());
            ui.separator();

            // Enabled interrupts for this context
            ui.label("Enabled IRQs (0-31):");
            let enabled_word = state.enabled[ctx][0];
            self.draw_irq_bits(ui, enabled_word, None, "IRQ");

            ui.add_space(4.0);

            // Threshold
            ui.horizontal(|ui| {
                ui.label("Threshold:");
                let threshold = state.threshold[ctx];
                let mut thresh_value = threshold;
                ui.add(egui::DragValue::new(&mut thresh_value).clamp_range(0..=7));
                if thresh_value != threshold {
                    state.threshold[ctx] = thresh_value;
                }
                ui.label("(0-7, only IRQs with priority > threshold trigger)");
            });

            ui.add_space(4.0);

            // Claimed IRQ
            ui.horizontal(|ui| {
                ui.label("Claimed IRQ:");
                let claimed = state.claimed[ctx];
                if claimed > 0 {
                    ui.label(RichText::new(format!("IRQ {}", claimed)).color(Color32::YELLOW));
                } else {
                    ui.label(RichText::new("None").color(Color32::GRAY));
                }
            });

            // Show combined status for pending+enabled IRQs
            ui.add_space(4.0);
            ui.label("Active (Pending + Enabled):");
            let active_word = state.pending[0] & enabled_word;
            self.draw_irq_bits(ui, active_word, Some(Color32::RED), "IRQ");
        });
    }

    fn draw_priorities(&self, ui: &mut Ui, state: &mut PlicState) {
        ui.group(|ui| {
            ui.label(RichText::new("Interrupt Priorities (IRQ 0-15)").strong());
            ui.separator();

            // Display priorities for first 16 IRQs in a grid
            egui::Grid::new("priorities_grid")
                .num_columns(4)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    for irq in 0..16 {
                        let priority = state.priority[irq];
                        ui.label(format!("IRQ {}:", irq));
                        
                        // Color based on priority
                        let color = match priority {
                            0 => Color32::GRAY,
                            1..=2 => Color32::from_rgb(100, 200, 100),
                            3..=4 => Color32::YELLOW,
                            5..=6 => Color32::from_rgb(255, 165, 0),
                            7 => Color32::RED,
                            _ => Color32::WHITE,
                        };
                        
                        ui.monospace(RichText::new(format!("{}", priority)).color(color));
                        
                        if (irq + 1) % 4 == 0 {
                            ui.end_row();
                        }
                    }
                });

            ui.add_space(4.0);
            ui.label("Priority range: 0 (disabled) to 7 (highest)");
        });
    }

    fn draw_actions(&mut self, ui: &mut Ui, state: &mut PlicState) {
        ui.group(|ui| {
            ui.label(RichText::new("Actions").strong());
            ui.separator();

            // IRQ selection
            ui.horizontal(|ui| {
                ui.label("Select IRQ:");
                ui.add(egui::DragValue::new(&mut self.selected_irq).clamp_range(0..=127));
            });

            ui.add_space(4.0);

            // Context selection
            ui.horizontal(|ui| {
                ui.label("Target Context:");
                ui.radio_value(&mut self.selected_context, 0, "Context 0");
                ui.radio_value(&mut self.selected_context, 1, "Context 1");
            });

            ui.add_space(8.0);

            // Pending control
            ui.horizontal(|ui| {
                if ui.button("Set Pending").clicked() {
                    let irq = self.selected_irq as usize;
                    let word = irq / 32;
                    let bit = irq % 32;
                    if word < 4 {
                        state.pending[word] |= 1 << bit;
                    }
                }
                if ui.button("Clear Pending").clicked() {
                    let irq = self.selected_irq as usize;
                    let word = irq / 32;
                    let bit = irq % 32;
                    if word < 4 {
                        state.pending[word] &= !(1 << bit);
                    }
                }
            });

            ui.add_space(4.0);

            // Enable control
            ui.horizontal(|ui| {
                if ui.button("Enable IRQ").clicked() {
                    let irq = self.selected_irq as usize;
                    let ctx = self.selected_context;
                    let word = irq / 32;
                    let bit = irq % 32;
                    if word < 4 {
                        state.enabled[ctx][word] |= 1 << bit;
                    }
                }
                if ui.button("Disable IRQ").clicked() {
                    let irq = self.selected_irq as usize;
                    let ctx = self.selected_context;
                    let word = irq / 32;
                    let bit = irq % 32;
                    if word < 4 {
                        state.enabled[ctx][word] &= !(1 << bit);
                    }
                }
            });

            ui.add_space(4.0);

            // Claim/Complete
            ui.horizontal(|ui| {
                if ui.button("Claim IRQ").clicked() {
                    let ctx = self.selected_context;
                    // Find highest priority pending and enabled IRQ
                    let active = state.pending[0] & state.enabled[ctx][0];
                    if active != 0 {
                        // Find highest set bit (highest IRQ number for simplicity)
                        let irq = 31 - active.leading_zeros();
                        state.claimed[ctx] = irq;
                        // Clear pending bit
                        state.pending[0] &= !(1 << irq);
                    }
                }
                if ui.button("Complete IRQ").clicked() {
                    let ctx = self.selected_context;
                    state.claimed[ctx] = 0;
                }
            });

            ui.add_space(4.0);

            // Priority control
            ui.horizontal(|ui| {
                ui.label("Set Priority:");
                let irq = self.selected_irq as usize;
                let mut priority = state.priority.get(irq).copied().unwrap_or(0);
                ui.add(egui::DragValue::new(&mut priority).clamp_range(0..=7));
                if irq < 32 {
                    state.priority[irq] = priority;
                }
            });

            ui.add_space(8.0);

            // Reset
            if ui.button("Reset All").clicked() {
                *state = PlicState::default();
            }
        });
    }

    fn draw_info(&self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("Info").strong());
            ui.separator();
            ui.label("The PLIC manages external interrupts for RISC-V cores:");
            ui.label("- Base address: 0x1000_0000");
            ui.label("- 128 interrupt sources (IRQ 0-127)");
            ui.label("- 2 contexts (dual-core support)");
            ui.label("- 3-bit priority (0-7, 0 = disabled)");
            ui.separator();
            ui.label("Claim/Complete mechanism:");
            ui.label("1. Core reads claim register to get highest priority IRQ");
            ui.label("2. Core services the interrupt");
            ui.label("3. Core writes IRQ number to complete register");
            ui.separator();
            ui.label("Threshold filtering:");
            ui.label("Only IRQs with priority > threshold will trigger");
        });
    }

    fn draw_irq_bits(&self, ui: &mut Ui, word: u32, highlight_color: Option<Color32>, label: &str) {
        ui.horizontal_wrapped(|ui| {
            for bit in 0..32 {
                let is_set = (word & (1 << bit)) != 0;
                let color = highlight_color.unwrap_or_else(|| {
                    if is_set {
                        Color32::RED
                    } else {
                        Color32::DARK_GRAY
                    }
                });

                let (rect, _) = ui.allocate_exact_size(
                    Vec2::new(16.0, 16.0),
                    egui::Sense::hover()
                );

                // Draw bit indicator
                ui.painter().rect_filled(
                    rect.shrink(1.0),
                    2.0,
                    if is_set { color } else { Color32::from_rgb(40, 40, 50) }
                );

                // Show bit number on hover
                if is_set {
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        format!("{}", bit % 10),
                        egui::FontId::proportional(9.0),
                        Color32::WHITE
                    );
                }
            }
        });

        ui.add_space(2.0);
        ui.label(format!("{} 0-31 bits (hover to see values)", label));
    }
}

/// PLIC state for the panel.
#[derive(Debug, Clone)]
pub struct PlicState {
    /// Pending interrupt bits (128 bits = 4 words)
    pub pending: [u32; 4],
    /// Enabled interrupt bits per context [context][word]
    pub enabled: [[u32; 4]; 2],
    /// Priority for each IRQ (showing first 32)
    pub priority: [u32; 32],
    /// Threshold per context
    pub threshold: [u32; 2],
    /// Currently claimed IRQ per context
    pub claimed: [u32; 2],
}

impl Default for PlicState {
    fn default() -> Self {
        Self {
            pending: [0; 4],
            enabled: [[0; 4]; 2],
            priority: [0; 32],
            threshold: [0; 2],
            claimed: [0; 2],
        }
    }
}