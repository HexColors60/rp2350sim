//! SHA-256 panel for RP2350 simulator UI.

use egui::{Color32, RichText, Ui};

/// SHA-256 panel state.
#[derive(Debug, Clone, Default)]
pub struct Sha256State {
    pub enabled: bool,
    pub busy: bool,
    pub ready: bool,
    pub double_sha: bool,
    pub big_endian: bool,
    pub input_bytes: usize,
    pub hash: [u32; 8],
}

/// SHA-256 panel.
#[derive(Debug, Default)]
pub struct Sha256Panel;

impl Sha256Panel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut Sha256State) {
        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(RichText::new("SHA-256 Accelerator").strong());
                ui.separator();
                if state.busy {
                    ui.label(RichText::new("Processing...").color(Color32::YELLOW));
                } else if state.ready {
                    ui.label(RichText::new("Ready").color(Color32::GREEN));
                } else {
                    ui.label(RichText::new("Idle").color(Color32::DARK_GRAY));
                }
            });

            ui.separator();

            // Status indicators
            ui.horizontal(|ui| {
                status_badge(ui, "ENABLE", state.enabled);
                status_badge(ui, "BUSY", state.busy);
                status_badge(ui, "READY", state.ready);
            });

            ui.separator();

            // Configuration
            ui.group(|ui| {
                ui.label(RichText::new("Configuration").strong());
                
                ui.horizontal(|ui| {
                    status_badge(ui, "DOUBLE_SHA", state.double_sha);
                    status_badge(ui, "BIG_ENDIAN", state.big_endian);
                });
                
                ui.label(format!("Input bytes processed: {}", state.input_bytes));
            });

            ui.separator();

            // Hash result
            ui.group(|ui| {
                ui.label(RichText::new("Hash Result (H0-H7)").strong());
                
                // Display hash as hex values
                egui::Grid::new("sha256_hash").show(ui, |ui| {
                    for i in 0..8 {
                        ui.label(format!("H{}:", i));
                        ui.monospace(RichText::new(format!("0x{:08X}", state.hash[i]))
                            .color(Color32::from_rgb(0, 200, 255)));
                        ui.end_row();
                    }
                });
            });

            ui.separator();

            // Full hash display
            ui.group(|ui| {
                ui.label(RichText::new("Full Hash (256-bit)").strong());
                
                // Display as continuous hex string
                let hash_hex: String = state.hash.iter()
                    .map(|w| format!("{:08x}", w))
                    .collect();
                
                ui.horizontal_wrapped(|ui| {
                    ui.monospace(RichText::new(&hash_hex)
                        .color(Color32::from_rgb(100, 255, 200))
                        .size(12.0));
                });
            });

            ui.separator();

            // Hash visualization
            ui.group(|ui| {
                ui.label(RichText::new("Hash Visualization").strong());
                
                // Display hash as colored blocks
                ui.horizontal(|ui| {
                    for i in 0..8 {
                        let word = state.hash[i];
                        let r = ((word >> 16) & 0xFF) as u8;
                        let g = ((word >> 8) & 0xFF) as u8;
                        let b = (word & 0xFF) as u8;
                        let color = Color32::from_rgb(r, g, b);
                        
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(20.0, 20.0),
                            egui::Sense::hover()
                        );
                        ui.painter().rect_filled(rect, 2.0, color);
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