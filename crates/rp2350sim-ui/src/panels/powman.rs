//! Power Manager panel for RP2350 simulator.

use egui::{Color32, RichText, Ui, Vec2};

/// Power state enumeration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PowerState {
    Active,
    LowPower,
    Sleep,
    DeepSleep,
}

impl PowerState {
    fn name(&self) -> &'static str {
        match self {
            PowerState::Active => "Active",
            PowerState::LowPower => "Low Power",
            PowerState::Sleep => "Sleep",
            PowerState::DeepSleep => "Deep Sleep",
        }
    }

    fn color(&self) -> Color32 {
        match self {
            PowerState::Active => Color32::GREEN,
            PowerState::LowPower => Color32::YELLOW,
            PowerState::Sleep => Color32::BLUE,
            PowerState::DeepSleep => Color32::from_rgb(180, 0, 255), // Purple
        }
    }
}

/// Power Manager state for the panel.
#[derive(Debug, Clone)]
pub struct PowmanState {
    pub power_state: PowerState,
    pub voltage_mv: u32,
    pub bod_enabled: bool,
    pub bod_threshold_mv: u32,
    pub lposc_freq: u32,
    pub wakeup_pending: bool,
    pub sleep_counter: u32,
    pub wakeup_gpio: bool,
    pub wakeup_timer: bool,
    pub wakeup_processor: bool,
}

impl Default for PowmanState {
    fn default() -> Self {
        Self {
            power_state: PowerState::Active,
            voltage_mv: 1100, // Default 1.1V
            bod_enabled: true,
            bod_threshold_mv: 900, // Default 0.9V
            lposc_freq: 32768,
            wakeup_pending: false,
            sleep_counter: 0,
            wakeup_gpio: true,
            wakeup_timer: true,
            wakeup_processor: true,
        }
    }
}

/// Power Manager panel.
pub struct PowmanPanel;

impl Default for PowmanPanel {
    fn default() -> Self {
        Self
    }
}

impl PowmanPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(&self) -> &'static str {
        "Power Manager"
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut PowmanState) {
        ui.vertical(|ui| {
            ui.label(RichText::new("Power Manager Panel").strong());
            ui.separator();

            // Power State display
            self.draw_power_state(ui, state);

            ui.add_space(8.0);

            // Voltage Regulator
            self.draw_voltage_regulator(ui, state);

            ui.add_space(8.0);

            // Brown-out Detection
            self.draw_bod(ui, state);

            ui.add_space(8.0);

            // Sleep Modes
            self.draw_sleep_modes(ui, state);

            ui.add_space(8.0);

            // Wake-up Sources
            self.draw_wakeup_sources(ui, state);

            ui.add_space(8.0);

            // Info
            self.draw_info(ui, state);
        });
    }

    fn draw_power_state(&self, ui: &mut Ui, state: &PowmanState) {
        ui.group(|ui| {
            ui.label(RichText::new("Power State").strong());
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Current State:");
                let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
                ui.painter()
                    .circle_filled(rect.center(), 5.0, state.power_state.color());
                ui.label(RichText::new(state.power_state.name()).color(state.power_state.color()));
            });

            ui.horizontal(|ui| {
                ui.label("Sleep Counter:");
                ui.monospace(
                    RichText::new(format!("{}", state.sleep_counter))
                        .color(Color32::from_rgb(100, 200, 255)),
                );
            });

            if state.wakeup_pending {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Wake-up Pending!").color(Color32::YELLOW));
                });
            }
        });
    }

    fn draw_voltage_regulator(&self, ui: &mut Ui, state: &mut PowmanState) {
        ui.group(|ui| {
            ui.label(RichText::new("Voltage Regulator").strong());
            ui.separator();

            // Voltage display
            let voltage_v = state.voltage_mv as f32 / 1000.0;
            ui.horizontal(|ui| {
                ui.label("Current Voltage:");
                ui.monospace(
                    RichText::new(format!("{:.3} V", voltage_v))
                        .color(Color32::from_rgb(0, 255, 150)),
                );
            });

            // Voltage progress bar (800mV - 1350mV range)
            let min_mv = 800;
            let max_mv = 1350;
            let voltage_ratio = (state.voltage_mv - min_mv) as f32 / (max_mv - min_mv) as f32;

            ui.add_space(4.0);
            let (rect, _) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), 20.0), egui::Sense::hover());
            ui.painter()
                .rect_filled(rect, 4.0, Color32::from_rgb(40, 40, 50));
            let filled_width = rect.width() * voltage_ratio.clamp(0.0, 1.0);
            let filled_rect =
                egui::Rect::from_min_size(rect.min, egui::vec2(filled_width, rect.height()));

            // Color gradient based on voltage level
            let voltage_color = if state.voltage_mv < 900 {
                Color32::RED
            } else if state.voltage_mv < 1000 {
                Color32::YELLOW
            } else if state.voltage_mv < 1200 {
                Color32::GREEN
            } else {
                Color32::from_rgb(100, 200, 255)
            };

            ui.painter().rect_filled(filled_rect, 4.0, voltage_color);
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("{:.3} V", voltage_v),
                egui::FontId::default(),
                Color32::WHITE,
            );

            ui.add_space(4.0);

            // Voltage slider
            ui.horizontal(|ui| {
                ui.label("Set Voltage:");
                let mut voltage = state.voltage_mv as i32;
                ui.add(egui::Slider::new(&mut voltage, 800..=1350).text("mV"));
                state.voltage_mv = voltage as u32;
            });

            // Preset voltages
            ui.horizontal(|ui| {
                if ui.small_button("0.85V").clicked() {
                    state.voltage_mv = 850;
                }
                if ui.small_button("1.0V").clicked() {
                    state.voltage_mv = 1000;
                }
                if ui.small_button("1.1V").clicked() {
                    state.voltage_mv = 1100;
                }
                if ui.small_button("1.2V").clicked() {
                    state.voltage_mv = 1200;
                }
                if ui.small_button("1.35V").clicked() {
                    state.voltage_mv = 1350;
                }
            });
        });
    }

    fn draw_bod(&self, ui: &mut Ui, state: &mut PowmanState) {
        ui.group(|ui| {
            ui.label(RichText::new("Brown-out Detection").strong());
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Enabled:");
                ui.checkbox(&mut state.bod_enabled, "");
            });

            ui.add_space(4.0);

            // BOD threshold slider
            ui.horizontal(|ui| {
                ui.label("Threshold:");
                let mut threshold = state.bod_threshold_mv as i32;
                ui.add(egui::Slider::new(&mut threshold, 700..=1100).text("mV"));
                state.bod_threshold_mv = threshold as u32;
            });

            // Status indicator
            let bod_status = if state.voltage_mv < state.bod_threshold_mv {
                (Color32::RED, "BROWN-OUT!")
            } else if state.bod_enabled {
                (Color32::GREEN, "OK")
            } else {
                (Color32::GRAY, "Disabled")
            };

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Status:");
                let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 5.0, bod_status.0);
                ui.label(RichText::new(bod_status.1).color(bod_status.0));
            });
        });
    }

    fn draw_sleep_modes(&self, ui: &mut Ui, state: &mut PowmanState) {
        ui.group(|ui| {
            ui.label(RichText::new("Sleep Modes").strong());
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Enter Low Power").clicked() {
                    state.power_state = PowerState::LowPower;
                }
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                if ui.button("Enter Sleep").clicked() {
                    state.power_state = PowerState::Sleep;
                    state.sleep_counter += 1;
                }
                if ui.button("Enter Deep Sleep").clicked() {
                    state.power_state = PowerState::DeepSleep;
                    state.sleep_counter += 1;
                }
            });

            ui.add_space(4.0);

            // Wake button (only enabled when in sleep mode)
            let can_wake = state.power_state == PowerState::Sleep
                || state.power_state == PowerState::DeepSleep;
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(can_wake, egui::Button::new("Wake Up"))
                    .clicked()
                {
                    state.power_state = PowerState::Active;
                    state.wakeup_pending = false;
                }
            });
        });
    }

    fn draw_wakeup_sources(&self, ui: &mut Ui, state: &mut PowmanState) {
        ui.group(|ui| {
            ui.label(RichText::new("Wake-up Sources").strong());
            ui.separator();

            ui.horizontal(|ui| {
                ui.checkbox(&mut state.wakeup_gpio, "GPIO Wake");
            });

            ui.horizontal(|ui| {
                ui.checkbox(&mut state.wakeup_timer, "Timer Wake");
            });

            ui.horizontal(|ui| {
                ui.checkbox(&mut state.wakeup_processor, "Processor Wake");
            });

            ui.add_space(4.0);

            // Trigger wake-up button (simulate external wake)
            if state.power_state == PowerState::Sleep || state.power_state == PowerState::DeepSleep
            {
                ui.horizontal(|ui| {
                    if ui.button("Trigger GPIO Wake").clicked() {
                        if state.wakeup_gpio {
                            state.wakeup_pending = true;
                        }
                    }
                    if ui.button("Trigger Timer Wake").clicked() {
                        if state.wakeup_timer {
                            state.wakeup_pending = true;
                        }
                    }
                });
            }
        });
    }

    fn draw_info(&self, ui: &mut Ui, state: &PowmanState) {
        ui.group(|ui| {
            ui.label(RichText::new("Info").strong());
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("LPOSC Frequency:");
                ui.monospace(
                    RichText::new(format!("{} Hz", state.lposc_freq))
                        .color(Color32::from_rgb(100, 200, 255)),
                );
            });

            ui.separator();

            ui.label("The Power Manager (POWMAN) controls power states and voltage.");
            ui.label("Base Address: 0x5004_4000");
            ui.separator();
            ui.label("States:");
            ui.label("  Active - Full power, all clocks running");
            ui.label("  Low Power - Reduced clocks, fast wake-up");
            ui.label("  Sleep - Core powered down, peripherals active");
            ui.label("  Deep Sleep - Minimal power, slow wake-up");
        });
    }
}
