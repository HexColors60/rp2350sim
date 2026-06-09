//! PLL panel for RP2350 simulator UI.

use egui::{Color32, RichText, Ui};

/// PLL state for UI display.
#[derive(Debug, Clone, Default)]
pub struct PllInstanceState {
    pub enabled: bool,
    pub locked: bool,
    pub bypass: bool,
    pub ref_freq: u32,
    pub vco_freq: u32,
    pub output_freq: u32,
    pub refdiv: u8,
    pub fbdiv: u16,
    pub postdiv1: u8,
    pub postdiv2: u8,
}

/// PLL panel state.
#[derive(Debug, Clone, Default)]
pub struct PllState {
    pub sys_pll: PllInstanceState,
    pub usb_pll: PllInstanceState,
}

/// PLL panel.
#[derive(Debug, Default)]
pub struct PllPanel {
    selected_pll: usize,
}

impl PllPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut PllState) {
        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(RichText::new("PLL Clock Generators").strong());
                ui.separator();
                
                // PLL selector
                if ui.selectable_label(self.selected_pll == 0, "SYS PLL").clicked() {
                    self.selected_pll = 0;
                }
                if ui.selectable_label(self.selected_pll == 1, "USB PLL").clicked() {
                    self.selected_pll = 1;
                }
            });

            ui.separator();

            // Get selected PLL state
            let pll = if self.selected_pll == 0 {
                &state.sys_pll
            } else {
                &state.usb_pll
            };

            // Status indicators
            ui.horizontal(|ui| {
                status_badge(ui, "PWR", pll.enabled);
                status_badge(ui, "LOCK", pll.locked);
                status_badge(ui, "BYPASS", pll.bypass);
            });

            ui.separator();

            // Frequency display
            ui.group(|ui| {
                ui.label(RichText::new("Frequencies").strong());
                
                egui::Grid::new("pll_freq").show(ui, |ui| {
                    ui.label("Reference:");
                    ui.monospace(RichText::new(format!("{} MHz", pll.ref_freq / 1_000_000))
                        .color(Color32::from_rgb(150, 150, 150)));
                    ui.end_row();

                    ui.label("VCO:");
                    ui.monospace(RichText::new(format!("{} MHz", pll.vco_freq / 1_000_000))
                        .color(Color32::from_rgb(255, 200, 100)));
                    ui.end_row();

                    ui.label("Output:");
                    ui.monospace(RichText::new(format!("{} MHz", pll.output_freq / 1_000_000))
                        .color(Color32::from_rgb(0, 255, 200)));
                    ui.end_row();
                });
            });

            ui.separator();

            // Divider configuration
            ui.group(|ui| {
                ui.label(RichText::new("Divider Configuration").strong());
                
                egui::Grid::new("pll_div").show(ui, |ui| {
                    ui.label("REFDIV:");
                    ui.monospace(format!("{}", pll.refdiv));
                    ui.label("(Reference divider)");
                    ui.end_row();

                    ui.label("FBDIV:");
                    ui.monospace(format!("{}", pll.fbdiv));
                    ui.label("(Feedback divider)");
                    ui.end_row();

                    ui.label("POSTDIV1:");
                    ui.monospace(format!("{}", pll.postdiv1));
                    ui.label("(Post divider 1)");
                    ui.end_row();

                    ui.label("POSTDIV2:");
                    ui.monospace(format!("{}", pll.postdiv2));
                    ui.label("(Post divider 2)");
                    ui.end_row();
                });
            });

            ui.separator();

            // Formula display
            ui.group(|ui| {
                ui.label(RichText::new("Frequency Formula").strong());
                ui.label("VCO = (Ref / REFDIV) × FBDIV");
                ui.label("Output = VCO / (POSTDIV1 × POSTDIV2)");
            });

            ui.separator();

            // Both PLLs summary
            ui.group(|ui| {
                ui.label(RichText::new("Summary").strong());
                
                ui.horizontal(|ui| {
                    ui.label("SYS PLL:");
                    if state.sys_pll.locked {
                        ui.label(RichText::new(format!("{} MHz", state.sys_pll.output_freq / 1_000_000))
                            .color(Color32::GREEN));
                    } else {
                        ui.label(RichText::new("Unlocked").color(Color32::RED));
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("USB PLL:");
                    if state.usb_pll.locked {
                        ui.label(RichText::new(format!("{} MHz", state.usb_pll.output_freq / 1_000_000))
                            .color(Color32::GREEN));
                    } else {
                        ui.label(RichText::new("Unlocked").color(Color32::RED));
                    }
                });
            });
        });
    }
}

/// Draw a status badge.
fn status_badge(ui: &mut Ui, label: &str, active: bool) {
    let color = if active { Color32::GREEN } else { Color32::DARK_GRAY };
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
        ui.painter().circle_filled(rect.center(), 4.0, color);
        ui.label(RichText::new(label).size(10.0));
    });
}