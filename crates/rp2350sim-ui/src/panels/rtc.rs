//! RTC panel for RP2350 simulator UI.
#![allow(dead_code)]

use egui::{Color32, RichText, Ui};

/// RTC panel state.
#[derive(Debug, Clone, Default)]
pub struct RtcState {
    pub enabled: bool,
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub day_of_week: u8,
    pub day: u8,
    pub month: u8,
    pub year: u16,
    pub alarm_enabled: bool,
    pub alarm_triggered: bool,
    pub leap_year: bool,
}

/// RTC panel.
#[derive(Debug, Default)]
pub struct RtcPanel {
    set_time_mode: bool,
    edit_second: String,
    edit_minute: String,
    edit_hour: String,
    edit_day: String,
    edit_month: String,
    edit_year: String,
}

impl RtcPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut RtcState) {
        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(RichText::new("Real-Time Clock (RTC)").strong());
                ui.separator();
                if state.enabled {
                    ui.label(RichText::new("Running").color(Color32::GREEN));
                } else {
                    ui.label(RichText::new("Stopped").color(Color32::RED));
                }
            });

            ui.separator();

            // Current time display
            ui.group(|ui| {
                ui.label(RichText::new("Current Time").strong());
                
                // Large time display
                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    ui.label(RichText::new(format!(
                        "{:02}:{:02}:{:02}",
                        state.hour, state.minute, state.second
                    )).size(32.0).color(Color32::from_rgb(0, 255, 200)));
                });

                // Date display
                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    let day_name = match state.day_of_week {
                        0 => "Sun",
                        1 => "Mon",
                        2 => "Tue",
                        3 => "Wed",
                        4 => "Thu",
                        5 => "Fri",
                        6 => "Sat",
                        _ => "???",
                    };
                    ui.label(RichText::new(format!(
                        "{}, {:02}/{:02}/{}",
                        day_name, state.day, state.month, state.year
                    )).size(18.0));
                });

                // Leap year indicator
                if state.leap_year {
                    ui.label(RichText::new("(Leap Year)").color(Color32::YELLOW).size(12.0));
                }
            });

            ui.separator();

            // Detailed time values
            egui::Grid::new("rtc_time").show(ui, |ui| {
                ui.label("Hours:");
                ui.monospace(format!("{:02}", state.hour));
                ui.end_row();

                ui.label("Minutes:");
                ui.monospace(format!("{:02}", state.minute));
                ui.end_row();

                ui.label("Seconds:");
                ui.monospace(format!("{:02}", state.second));
                ui.end_row();

                ui.label("Day of Week:");
                let day_name = match state.day_of_week {
                    0 => "Sunday",
                    1 => "Monday",
                    2 => "Tuesday",
                    3 => "Wednesday",
                    4 => "Thursday",
                    5 => "Friday",
                    6 => "Saturday",
                    _ => "Unknown",
                };
                ui.label(day_name);
                ui.end_row();

                ui.label("Day:");
                ui.monospace(format!("{}", state.day));
                ui.end_row();

                ui.label("Month:");
                ui.monospace(format!("{}", state.month));
                ui.end_row();

                ui.label("Year:");
                ui.monospace(format!("{}", state.year));
                ui.end_row();
            });

            ui.separator();

            // Alarm status
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Alarm").strong());
                    if state.alarm_triggered {
                        ui.label(RichText::new("TRIGGERED!").color(Color32::RED));
                    } else if state.alarm_enabled {
                        ui.label(RichText::new("Armed").color(Color32::YELLOW));
                    } else {
                        ui.label(RichText::new("Disabled").color(Color32::DARK_GRAY));
                    }
                });
            });
        });
    }
}