//! I2S panel for RP2350 simulator UI.

use egui::{Color32, RichText, Ui};

/// I2S channel state for UI display.
#[derive(Debug, Clone, Default)]
pub struct I2sChannelState {
    pub tx_enabled: bool,
    pub rx_enabled: bool,
    pub tx_pause: bool,
    pub rx_pause: bool,
    pub tx_fifo_level: usize,
    pub rx_fifo_level: usize,
    pub tx_count: u64,
    pub rx_count: u64,
}

/// I2S panel state.
#[derive(Debug, Clone, Default)]
pub struct I2sState {
    pub enabled: bool,
    pub clock_enabled: bool,
    pub sample_rate: u32,
    pub data_width: u8,
    pub channels: u8,
    pub format: String,
    pub channel: I2sChannelState,
    pub interrupt_pending: bool,
}

/// I2S panel.
#[derive(Debug, Default)]
pub struct I2sPanel {
    selected_instance: usize,
}

impl I2sPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut I2sState) {
        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(RichText::new("I2S Audio Controller").strong());
                ui.separator();
                
                // Instance selector
                for i in 0..2 {
                    if ui.selectable_label(self.selected_instance == i, format!("I2S{}", i)).clicked() {
                        self.selected_instance = i;
                    }
                }
            });

            ui.separator();

            // Status indicators
            ui.horizontal(|ui| {
                status_badge(ui, "I2S", state.enabled);
                status_badge(ui, "CLK", state.clock_enabled);
                status_badge(ui, "IRQ", state.interrupt_pending);
            });

            ui.separator();

            // Configuration
            egui::Grid::new("i2s_config").show(ui, |ui| {
                ui.label("Sample Rate:");
                ui.monospace(RichText::new(format!("{} Hz", state.sample_rate)).color(Color32::from_rgb(0, 200, 255)));
                ui.end_row();

                ui.label("Data Width:");
                let width_str = match state.data_width {
                    0 => "8-bit",
                    1 => "16-bit",
                    2 => "24-bit",
                    3 => "32-bit",
                    _ => "Unknown",
                };
                ui.label(width_str);
                ui.end_row();

                ui.label("Channels:");
                ui.label(format!("{}", state.channels));
                ui.end_row();

                ui.label("Format:");
                ui.label(&state.format);
                ui.end_row();
            });

            ui.separator();

            // TX/RX Status
            ui.horizontal(|ui| {
                // TX Section
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("TX").strong());
                        status_badge(ui, "EN", state.channel.tx_enabled);
                        status_badge(ui, "PAUSE", state.channel.tx_pause);
                        
                        ui.separator();
                        
                        ui.label(format!("FIFO: {}/32", state.channel.tx_fifo_level));
                        draw_fifo_bar(ui, state.channel.tx_fifo_level, 32, Color32::from_rgb(100, 150, 255));
                        
                        ui.label(format!("Count: {}", state.channel.tx_count));
                    });
                });

                // RX Section
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("RX").strong());
                        status_badge(ui, "EN", state.channel.rx_enabled);
                        status_badge(ui, "PAUSE", state.channel.rx_pause);
                        
                        ui.separator();
                        
                        ui.label(format!("FIFO: {}/32", state.channel.rx_fifo_level));
                        draw_fifo_bar(ui, state.channel.rx_fifo_level, 32, Color32::from_rgb(100, 255, 150));
                        
                        ui.label(format!("Count: {}", state.channel.rx_count));
                    });
                });
            });

            ui.separator();

            // Audio visualization placeholder
            ui.group(|ui| {
                ui.label(RichText::new("Audio Waveform").strong());
                ui.label("(Audio visualization would appear here)");
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