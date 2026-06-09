//! TRNG panel for RP2350 simulator UI.
#![allow(dead_code)]

use egui::{Color32, RichText, Ui};

/// TRNG panel state.
#[derive(Debug, Clone, Default)]
pub struct TrngState {
    pub enabled: bool,
    pub ready: bool,
    pub fifo_empty: bool,
    pub fifo_full: bool,
    pub fifo_level: usize,
    pub sample_count: u32,
    pub last_value: u32,
    pub total_generated: u64,
}

/// TRNG panel.
#[derive(Debug, Default)]
pub struct TrngPanel {
    history: Vec<u32>,
}

impl TrngPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut TrngState) {
        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(RichText::new("True Random Number Generator").strong());
                ui.separator();
                if state.enabled {
                    ui.label(RichText::new("Active").color(Color32::GREEN));
                } else {
                    ui.label(RichText::new("Disabled").color(Color32::RED));
                }
            });

            ui.separator();

            // Status indicators
            ui.horizontal(|ui| {
                status_badge(ui, "ENABLE", state.enabled);
                status_badge(ui, "READY", state.ready);
                status_badge(ui, "IRQ", false);
            });

            ui.separator();

            // FIFO Status
            ui.group(|ui| {
                ui.label(RichText::new("FIFO Status").strong());
                
                ui.horizontal(|ui| {
                    status_badge(ui, "EMPTY", state.fifo_empty);
                    status_badge(ui, "FULL", state.fifo_full);
                });
                
                ui.label(format!("FIFO Level: {}/16", state.fifo_level));
                draw_fifo_bar(ui, state.fifo_level, 16, Color32::from_rgb(150, 100, 255));
            });

            ui.separator();

            // Configuration
            egui::Grid::new("trng_config").show(ui, |ui| {
                ui.label("Sample Count:");
                ui.monospace(format!("{}", state.sample_count));
                ui.end_row();

                ui.label("Total Generated:");
                ui.monospace(format!("{}", state.total_generated));
                ui.end_row();
            });

            ui.separator();

            // Last generated value
            ui.group(|ui| {
                ui.label(RichText::new("Last Generated Value").strong());
                
                ui.horizontal(|ui| {
                    ui.label("Hex:");
                    ui.monospace(RichText::new(format!("0x{:08X}", state.last_value))
                        .color(Color32::from_rgb(0, 200, 255))
                        .size(16.0));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Dec:");
                    ui.monospace(format!("{}", state.last_value));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Bin:");
                    ui.monospace(RichText::new(format!("{:032b}", state.last_value))
                        .size(10.0));
                });
            });

            ui.separator();

            // Randomness visualization
            ui.group(|ui| {
                ui.label(RichText::new("Bit Distribution").strong());
                
                // Show bit distribution as bars
                ui.horizontal(|ui| {
                    for i in 0..32 {
                        let bit = (state.last_value >> i) & 1;
                        let color = if bit == 1 {
                            Color32::from_rgb(100, 255, 100)
                        } else {
                            Color32::from_rgb(50, 50, 50)
                        };
                        
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(6.0, 16.0),
                            egui::Sense::hover()
                        );
                        ui.painter().rect_filled(rect, 1.0, color);
                    }
                });
                ui.label("Bit 0-31 (LSB → MSB)");
            });

            ui.separator();

            // Byte visualization
            ui.group(|ui| {
                ui.label(RichText::new("Byte Values").strong());
                
                ui.horizontal(|ui| {
                    for i in 0..4 {
                        let byte = ((state.last_value >> (i * 8)) & 0xFF) as u8;
                        let intensity = byte as f32 / 255.0;
                        let color = Color32::from_rgb(
                            (intensity * 255.0) as u8,
                            (intensity * 200.0) as u8,
                            (intensity * 150.0) as u8,
                        );
                        
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(40.0, 40.0),
                            egui::Sense::hover()
                        );
                        ui.painter().rect_filled(rect, 4.0, color);
                        
                        // Show byte value
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{}", byte),
                            egui::FontId::default(),
                            Color32::WHITE,
                        );
                    }
                });
                ui.label("Bytes 0-3");
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

/// Draw a FIFO level bar.
fn draw_fifo_bar(ui: &mut Ui, level: usize, max: usize, color: Color32) {
    let available_width = ui.available_width().min(100.0);
    let fill_ratio = level as f32 / max as f32;
    
    let (rect, _) = ui.allocate_exact_size(egui::vec2(available_width, 8.0), egui::Sense::hover());
    
    // Background
    ui.painter().rect_filled(rect, 0.0, Color32::DARK_GRAY);
    
    // Fill
    let fill_width = rect.width() * fill_ratio;
    let fill_rect = egui::Rect::from_min_size(
        rect.min,
        egui::vec2(fill_width, rect.height()),
    );
    ui.painter().rect_filled(fill_rect, 0.0, color);
}