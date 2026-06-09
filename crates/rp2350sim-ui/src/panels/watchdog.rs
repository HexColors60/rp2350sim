//! Watchdog panel for RP2350 simulator.

use egui::{Color32, RichText, Ui, Vec2};

/// Watchdog panel with timer display and configuration.
pub struct WatchdogPanel {
    load_value: u32,
    auto_reload: bool,
}

impl Default for WatchdogPanel {
    fn default() -> Self {
        Self {
            load_value: 0x000FFFFF,
            auto_reload: false,
        }
    }
}

impl WatchdogPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(&self) -> &'static str {
        "Watchdog"
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut WatchdogState) {
        ui.vertical(|ui| {
            ui.label(RichText::new("Watchdog Panel").strong());
            ui.separator();

            // Status display
            self.draw_status(ui, state);

            ui.add_space(8.0);

            // Counter display
            self.draw_counter(ui, state);

            ui.add_space(8.0);

            // Configuration
            self.draw_config(ui, state);

            ui.add_space(8.0);

            // Actions
            self.draw_actions(ui, state);

            ui.add_space(8.0);

            // Info
            self.draw_info(ui);
        });
    }

    fn draw_status(&self, ui: &mut Ui, state: &WatchdogState) {
        ui.group(|ui| {
            ui.label(RichText::new("Status").strong());
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("State:");
                let status_color = if state.enabled {
                    if state.counter == 0 {
                        Color32::RED
                    } else {
                        Color32::GREEN
                    }
                } else {
                    Color32::GRAY
                };
                let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 5.0, status_color);

                let status_text = if !state.enabled {
                    "Disabled"
                } else if state.counter == 0 {
                    "TIMEOUT!"
                } else {
                    "Running"
                };
                ui.label(RichText::new(status_text).color(status_color));
            });

            ui.horizontal(|ui| {
                ui.label("Enabled:");
                ui.label(if state.enabled { "Yes" } else { "No" });
            });

            ui.horizontal(|ui| {
                ui.label("Timeout Events:");
                ui.label(format!("{}", state.timeout_count));
            });
        });
    }

    fn draw_counter(&self, ui: &mut Ui, state: &WatchdogState) {
        ui.group(|ui| {
            ui.label(RichText::new("Counter").strong());
            ui.separator();

            // Counter value with progress bar
            let counter_ratio = if state.load > 0 {
                state.counter as f32 / state.load as f32
            } else {
                0.0
            };

            ui.horizontal(|ui| {
                ui.label("Current:");
                ui.monospace(RichText::new(format!("0x{:08X}", state.counter))
                    .color(Color32::YELLOW).size(14.0));
            });

            // Progress bar
            let progress_color = if counter_ratio < 0.25 {
                Color32::RED
            } else if counter_ratio < 0.5 {
                Color32::YELLOW
            } else {
                Color32::GREEN
            };

            ui.add_space(4.0);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 20.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 4.0, Color32::from_rgb(40, 40, 50));
            let filled_width = rect.width() * counter_ratio;
            let filled_rect = egui::Rect::from_min_size(
                rect.min,
                egui::vec2(filled_width, rect.height())
            );
            ui.painter().rect_filled(filled_rect, 4.0, progress_color);
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("{:.1}%", counter_ratio * 100.0),
                egui::FontId::default(),
                Color32::WHITE
            );

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Load Value:");
                ui.monospace(RichText::new(format!("0x{:08X}", state.load))
                    .color(Color32::from_rgb(100, 200, 255)));
            });
        });
    }

    fn draw_config(&mut self, ui: &mut Ui, state: &mut WatchdogState) {
        ui.group(|ui| {
            ui.label(RichText::new("Configuration").strong());
            ui.separator();

            // Enable toggle
            ui.horizontal(|ui| {
                ui.label("Enable:");
                if ui.checkbox(&mut state.enabled, "").changed() {
                    if state.enabled && state.counter == 0 {
                        state.counter = state.load;
                    }
                }
            });

            ui.add_space(4.0);

            // Load value
            ui.horizontal(|ui| {
                ui.label("Load Value:");
                ui.add(egui::DragValue::new(&mut self.load_value));
                if ui.button("Apply").clicked() {
                    state.load = self.load_value;
                }
            });

            ui.add_space(4.0);

            // Preset values
            ui.label("Presets:");
            ui.horizontal(|ui| {
                if ui.small_button("1M cycles").clicked() {
                    self.load_value = 1_000_000;
                    state.load = self.load_value;
                }
                if ui.small_button("10M cycles").clicked() {
                    self.load_value = 10_000_000;
                    state.load = self.load_value;
                }
                if ui.small_button("100M cycles").clicked() {
                    self.load_value = 100_000_000;
                    state.load = self.load_value;
                }
            });

            ui.add_space(4.0);

            // Auto reload
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.auto_reload, "Auto-reload on timeout");
            });
        });
    }

    fn draw_actions(&self, ui: &mut Ui, state: &mut WatchdogState) {
        ui.group(|ui| {
            ui.label(RichText::new("Actions").strong());
            ui.separator();

            ui.horizontal(|ui| {
                // Feed button (reload counter)
                if ui.button("Feed (Reload)").clicked() {
                    state.counter = state.load;
                }

                // Reset button
                if ui.button("Reset").clicked() {
                    state.enabled = false;
                    state.counter = 0;
                    state.load = 0;
                    state.timeout_count = 0;
                }
            });

            ui.add_space(4.0);

            // Tick controls
            ui.horizontal(|ui| {
                if ui.button("Tick").clicked() {
                    if state.enabled && state.counter > 0 {
                        state.counter -= 1;
                        if state.counter == 0 {
                            state.timeout_count += 1;
                            if self.auto_reload {
                                state.counter = state.load;
                            }
                        }
                    }
                }
                if ui.button("Tick x100").clicked() {
                    for _ in 0..100 {
                        if state.enabled && state.counter > 0 {
                            state.counter -= 1;
                            if state.counter == 0 {
                                state.timeout_count += 1;
                                if self.auto_reload {
                                    state.counter = state.load;
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                }
                if ui.button("Tick x1000").clicked() {
                    for _ in 0..1000 {
                        if state.enabled && state.counter > 0 {
                            state.counter -= 1;
                            if state.counter == 0 {
                                state.timeout_count += 1;
                                if self.auto_reload {
                                    state.counter = state.load;
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                }
            });
        });
    }

    fn draw_info(&self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("Info").strong());
            ui.separator();
            ui.label("The Watchdog Timer (WDT) is a hardware timer that can reset");
            ui.label("the system if the software fails to feed it within the");
            ui.label("specified timeout period.");
            ui.separator();
            ui.label("Usage:");
            ui.label("1. Set the load value (timeout period)");
            ui.label("2. Enable the watchdog");
            ui.label("3. Feed periodically to prevent timeout");
            ui.separator();
            ui.label("At 150 MHz, 1M cycles = ~6.67ms");
        });
    }
}

/// Watchdog state for the panel.
#[derive(Debug, Clone)]
pub struct WatchdogState {
    pub enabled: bool,
    pub counter: u32,
    pub load: u32,
    pub timeout_count: u64,
}

impl Default for WatchdogState {
    fn default() -> Self {
        Self {
            enabled: false,
            counter: 0,
            load: 0,
            timeout_count: 0,
        }
    }
}