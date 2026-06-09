//! Timer panel for RP2350 simulator.

use super::PeripheralState;
use egui::{Color32, RichText, Ui};

/// Timer panel with system timer display.
pub struct TimerPanel {
    alarm_values: [u64; 4],
    alarm_enabled: [bool; 4],
}

impl Default for TimerPanel {
    fn default() -> Self {
        Self {
            alarm_values: [0; 4],
            alarm_enabled: [false; 4],
        }
    }
}

impl TimerPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(&self) -> &'static str {
        "Timer"
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        ui.vertical(|ui| {
            ui.label(RichText::new("Timer Panel").strong());
            ui.separator();

            // Main timer display
            ui.group(|ui| {
                ui.label(RichText::new("System Timer").strong());
                ui.separator();

                // Current value
                ui.horizontal(|ui| {
                    ui.label("Current Value:");
                    ui.monospace(RichText::new(format!("0x{:016X}", state.timer_value))
                        .color(Color32::YELLOW).size(16.0));
                });

                // Running status
                ui.horizontal(|ui| {
                    ui.label("Status:");
                    let status_color = if state.timer_running { Color32::GREEN } else { Color32::RED };
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                    ui.painter().circle_filled(rect.center(), 5.0, status_color);
                    ui.label(if state.timer_running { "Running" } else { "Stopped" });
                });

                // Control buttons
                ui.horizontal(|ui| {
                    if ui.button("Start").clicked() {
                        state.timer_running = true;
                    }
                    if ui.button("Stop").clicked() {
                        state.timer_running = false;
                    }
                    if ui.button("Reset").clicked() {
                        state.timer_value = 0;
                    }
                });
            });

            ui.add_space(8.0);

            // Alarms
            ui.group(|ui| {
                ui.label(RichText::new("Alarms").strong());
                ui.separator();

                egui::Grid::new("alarms").spacing([8.0, 4.0]).show(ui, |ui| {
                    ui.label(RichText::new("Alarm").strong());
                    ui.label(RichText::new("Value").strong());
                    ui.label(RichText::new("Enable").strong());
                    ui.label(RichText::new("Status").strong());
                    ui.end_row();

                    for i in 0..4 {
                        ui.label(format!("Alarm {}", i));

                        // Value input
                        let mut val_str = format!("{}", self.alarm_values[i]);
                        ui.add(egui::TextEdit::singleline(&mut val_str).desired_width(100.0));
                        if let Ok(v) = val_str.parse::<u64>() {
                            self.alarm_values[i] = v;
                        }

                        // Enable checkbox
                        ui.checkbox(&mut self.alarm_enabled[i], "");

                        // Status
                        let triggered = state.timer_value >= self.alarm_values[i] && self.alarm_enabled[i];
                        let status_color = if triggered { Color32::RED } else { Color32::GRAY };
                        ui.label(RichText::new(if triggered { "TRIGGERED" } else { "Armed" }).color(status_color));

                        ui.end_row();
                    }
                });
            });

            ui.add_space(8.0);

            // Timer info
            ui.group(|ui| {
                ui.label(RichText::new("Timer Info").strong());
                ui.separator();
                ui.label("The system timer is a 64-bit counter that increments at the system clock frequency.");
                ui.label("Alarms can be set to trigger interrupts when the timer reaches a specific value.");
                ui.separator();
                ui.label(format!("Clock: 150 MHz"));
                ui.label(format!("Resolution: 6.67 ns"));
            });
        });
    }
}