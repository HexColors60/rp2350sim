//! ADC and PWM panel for RP2350 simulator.
#![allow(dead_code)]

use super::{PeripheralEvent, PeripheralState};
use egui::{Color32, RichText, Ui, Vec2};

/// ADC/PWM panel with waveform display.
pub struct AdcPwmPanel {
    selected_adc: usize,
    selected_pwm: usize,
    adc_values: [u16; 4],
    waveform_history: Vec<[u16; 4]>,
    show_waveform: bool,
    selected_slice: usize,
}

impl Default for AdcPwmPanel {
    fn default() -> Self {
        Self {
            selected_adc: 0,
            selected_pwm: 0,
            adc_values: [0; 4],
            waveform_history: Vec::with_capacity(200),
            show_waveform: true,
            selected_slice: 0,
        }
    }
}

impl AdcPwmPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(&self) -> &'static str {
        "ADC/PWM"
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        ui.vertical(|ui| {
            // Tabs
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.show_waveform, true, "ADC");
                ui.selectable_value(&mut self.show_waveform, false, "PWM");
            });
            ui.separator();

            if self.show_waveform {
                self.draw_adc_panel(ui, state);
            } else {
                self.draw_pwm_panel(ui, state);
            }
        });
    }

    fn draw_adc_panel(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        ui.label(RichText::new("ADC Panel").strong());
        ui.separator();

        // Update waveform history
        self.waveform_history.push(state.adc_values);
        if self.waveform_history.len() > 200 {
            self.waveform_history.remove(0);
        }

        // Channel controls
        ui.horizontal(|ui| {
            for ch in 0..4 {
                ui.vertical(|ui| {
                    ui.label(format!("ADC{}", ch));

                    // Value display
                    let value = state.adc_values[ch];
                    let voltage = (value as f32 / 4095.0) * 3.3;
                    ui.label(RichText::new(format!("{:.2}V", voltage)).color(Color32::YELLOW));
                    ui.monospace(format!("0x{:03X}", value));

                    // Slider control
                    let mut slider_val = value;
                    ui.add(egui::Slider::new(&mut slider_val, 0..=4095).text(""));
                    if slider_val != value {
                        state.adc_values[ch] = slider_val;
                        state.events.push(PeripheralEvent::AdcSetValue(ch as u8, slider_val));
                    }

                    // Enable checkbox
                    ui.checkbox(&mut state.adc_enabled[ch], "Enable");
                });
                if ch < 3 {
                    ui.separator();
                }
            }
        });

        ui.add_space(8.0);

        // Waveform display
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Waveform").strong());
                if ui.button("Clear").clicked() {
                    self.waveform_history.clear();
                }
            });
            ui.separator();

            let canvas_size = Vec2::new(ui.available_width(), 150.0);
            let (response, painter) = ui.allocate_painter(canvas_size, egui::Sense::hover());

            let rect = response.rect;
            let width = rect.width();
            let height = rect.height();

            // Draw grid
            for i in 0..5 {
                let y = rect.top() + (height / 4.0) * i as f32;
                painter.line_segment(
                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    egui::Stroke::new(0.5, Color32::from_rgb(50, 50, 60)),
                );
            }

            // Draw waveforms for each channel
            let colors = [
                Color32::GREEN,
                Color32::YELLOW,
                Color32::from_rgb(100, 200, 255),
                Color32::from_rgb(255, 100, 200),
            ];

            for ch in 0..4 {
                if !state.adc_enabled[ch] {
                    continue;
                }

                let points: Vec<egui::Pos2> = self.waveform_history
                    .iter()
                    .enumerate()
                    .map(|(i, values)| {
                        let x = rect.left() + (i as f32 / 200.0) * width;
                        let y = rect.bottom() - (values[ch] as f32 / 4095.0) * height;
                        egui::pos2(x, y)
                    })
                    .collect();

                if points.len() > 1 {
                    painter.add(egui::Shape::line(points, egui::Stroke::new(1.5, colors[ch])));
                }
            }

            // Legend
            for ch in 0..4 {
                if state.adc_enabled[ch] {
                    painter.text(
                        egui::pos2(rect.left() + 10.0 + (ch as f32 * 60.0), rect.top() + 10.0),
                        egui::Align2::LEFT_TOP,
                        format!("CH{}", ch),
                        egui::FontId::proportional(10.0),
                        colors[ch],
                    );
                }
            }
        });

        ui.add_space(8.0);

        // Quick controls
        ui.group(|ui| {
            ui.label(RichText::new("Quick Controls").strong());
            ui.horizontal(|ui| {
                if ui.button("All Zero").clicked() {
                    for ch in 0..4 {
                        state.adc_values[ch] = 0;
                        state.events.push(PeripheralEvent::AdcSetValue(ch as u8, 0));
                    }
                }
                if ui.button("All Mid").clicked() {
                    for ch in 0..4 {
                        state.adc_values[ch] = 2048;
                        state.events.push(PeripheralEvent::AdcSetValue(ch as u8, 2048));
                    }
                }
                if ui.button("All Max").clicked() {
                    for ch in 0..4 {
                        state.adc_values[ch] = 4095;
                        state.events.push(PeripheralEvent::AdcSetValue(ch as u8, 4095));
                    }
                }
                if ui.button("Random").clicked() {
                    // Simple pseudo-random values using timer
                    for ch in 0..4 {
                        let val = ((state.timer_value as u16).wrapping_add(ch as u16 * 1234)) % 4096;
                        state.adc_values[ch] = val;
                        state.events.push(PeripheralEvent::AdcSetValue(ch as u8, val));
                    }
                }
            });
        });
    }

    fn draw_pwm_panel(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        ui.label(RichText::new("PWM Panel (8 Slices, 16 Channels)").strong());
        ui.separator();

        // Slice selector
        ui.horizontal(|ui| {
            ui.label("Slice:");
            for i in 0..8 {
                let label = format!("{}", i);
                if ui.selectable_label(self.selected_slice == i, label).clicked() {
                    self.selected_slice = i;
                }
            }
        });
        ui.separator();

        let slice = self.selected_slice;
        let ch_a = slice * 2;
        let ch_b = slice * 2 + 1;

        // Slice configuration
        ui.group(|ui| {
            ui.label(RichText::new(format!("Slice {} Configuration", slice)).strong());
            ui.horizontal(|ui| {
                // Phase correct
                ui.checkbox(&mut state.pwm_phase_correct[slice], "Phase Correct");
                // Invert outputs
                ui.checkbox(&mut state.pwm_invert_a[slice], "Inv A");
                ui.checkbox(&mut state.pwm_invert_b[slice], "Inv B");
            });

            ui.horizontal(|ui| {
                // Clock divider
                ui.label("Divider:");
                ui.add(egui::DragValue::new(&mut state.pwm_divider[slice]).clamp_range(1..=255));

                // TOP value
                ui.label("TOP:");
                ui.add(egui::DragValue::new(&mut state.pwm_top[slice]).clamp_range(1..=65535));

                // Frequency calculation (assuming 150MHz system clock)
                let div = state.pwm_divider[slice] as f32;
                let top = state.pwm_top[slice] as f32;
                let freq = 150_000_000.0 / (div * top);
                ui.label(RichText::new(format!("≈ {:.1} kHz", freq / 1000.0)).color(Color32::from_rgb(0, 200, 200)));
            });
        });

        ui.add_space(8.0);

        // Channel A and B controls
        egui::Grid::new("pwm_channels").spacing([16.0, 8.0]).show(ui, |ui| {
            ui.label(RichText::new("Channel").strong());
            ui.label(RichText::new("Duty Cycle").strong());
            ui.label(RichText::new("Compare Value").strong());
            ui.label(RichText::new("Enable").strong());
            ui.label(RichText::new("Output").strong());
            ui.end_row();

            // Channel A
            self.draw_channel_row(ui, state, ch_a, 'A', slice);

            // Channel B
            self.draw_channel_row(ui, state, ch_b, 'B', slice);
        });

        ui.add_space(8.0);

        // All slices overview
        ui.collapsing("All Slices Overview", |ui| {
            egui::Grid::new("pwm_overview").spacing([8.0, 4.0]).show(ui, |ui| {
                ui.label(RichText::new("Slice").strong());
                ui.label(RichText::new("A Duty").strong());
                ui.label(RichText::new("B Duty").strong());
                ui.label(RichText::new("Div").strong());
                ui.label(RichText::new("TOP").strong());
                ui.label(RichText::new("Phase").strong());
                ui.end_row();

                for s in 0..8 {
                    ui.label(format!("{}", s));

                    // Channel A
                    let ch_a = s * 2;
                    let duty_a = state.pwm_duty[ch_a];
                    let top = state.pwm_top[s].max(1);
                    let percent_a = (duty_a as f32 / top as f32) * 100.0;
                    ui.label(RichText::new(format!("{:.1}%", percent_a)).color(
                        if state.pwm_enabled[ch_a] { Color32::GREEN } else { Color32::GRAY }
                    ));

                    // Channel B
                    let ch_b = s * 2 + 1;
                    let duty_b = state.pwm_duty[ch_b];
                    let percent_b = (duty_b as f32 / top as f32) * 100.0;
                    ui.label(RichText::new(format!("{:.1}%", percent_b)).color(
                        if state.pwm_enabled[ch_b] { Color32::YELLOW } else { Color32::GRAY }
                    ));

                    // Divider
                    ui.label(format!("{}", state.pwm_divider[s]));

                    // TOP
                    ui.label(format!("{}", state.pwm_top[s]));

                    // Phase correct indicator
                    ui.label(if state.pwm_phase_correct[s] { "✓" } else { "-" });

                    ui.end_row();
                }
            });
        });

        ui.add_space(8.0);

        // Quick controls
        ui.group(|ui| {
            ui.label(RichText::new("Quick Controls").strong());
            ui.horizontal(|ui| {
                if ui.button("All Off").clicked() {
                    for ch in 0..16 {
                        state.pwm_duty[ch] = 0;
                        state.events.push(PeripheralEvent::PwmSetDuty(ch as u8, 0));
                    }
                }
                if ui.button("All 50%").clicked() {
                    for s in 0..8 {
                        let half = state.pwm_top[s] / 2;
                        state.pwm_duty[s * 2] = half;
                        state.pwm_duty[s * 2 + 1] = half;
                        state.events.push(PeripheralEvent::PwmSetDuty((s * 2) as u8, half));
                        state.events.push(PeripheralEvent::PwmSetDuty((s * 2 + 1) as u8, half));
                    }
                }
                if ui.button("All On").clicked() {
                    for s in 0..8 {
                        let top = state.pwm_top[s];
                        state.pwm_duty[s * 2] = top;
                        state.pwm_duty[s * 2 + 1] = top;
                        state.events.push(PeripheralEvent::PwmSetDuty((s * 2) as u8, top));
                        state.events.push(PeripheralEvent::PwmSetDuty((s * 2 + 1) as u8, top));
                    }
                }
                if ui.button("Enable All").clicked() {
                    for ch in 0..16 {
                        state.pwm_enabled[ch] = true;
                    }
                }
                if ui.button("Disable All").clicked() {
                    for ch in 0..16 {
                        state.pwm_enabled[ch] = false;
                    }
                }
            });
        });
    }

    fn draw_channel_row(
        &mut self,
        ui: &mut Ui,
        state: &mut PeripheralState,
        ch: usize,
        label: char,
        slice: usize,
    ) {
        let top = state.pwm_top[slice].max(1);

        ui.label(RichText::new(format!("Channel {}", label)).strong());

        // Duty slider (0 to TOP)
        let mut duty = state.pwm_duty[ch];
        ui.add(egui::Slider::new(&mut duty, 0..=top).text(""));
        if duty != state.pwm_duty[ch] {
            state.pwm_duty[ch] = duty;
            state.events.push(PeripheralEvent::PwmSetDuty(ch as u8, duty));
        }

        // Compare value display
        let percent = (duty as f32 / top as f32) * 100.0;
        ui.horizontal(|ui| {
            ui.monospace(format!("{}/{}", duty, top));
            ui.label(RichText::new(format!("({:.1}%)", percent)).color(Color32::YELLOW));
        });

        // Enable checkbox
        ui.checkbox(&mut state.pwm_enabled[ch], "");

        // Visual output bar
        let bar_width = 100.0;
        let bar_height = 20.0;
        let (rect, _) = ui.allocate_exact_size(Vec2::new(bar_width, bar_height), egui::Sense::hover());

        // Background
        ui.painter().rect_filled(rect, 2.0, Color32::from_rgb(30, 30, 40));

        // Filled portion
        let fill_width = (duty as f32 / top as f32) * bar_width;
        let fill_rect = egui::Rect::from_min_size(rect.min, Vec2::new(fill_width, bar_height));

        let is_inverted = if label == 'A' { state.pwm_invert_a[slice] } else { state.pwm_invert_b[slice] };
        let base_color = if label == 'A' { Color32::from_rgb(0, 150, 255) } else { Color32::from_rgb(255, 150, 0) };
        let fill_color = if state.pwm_enabled[ch] {
            if is_inverted {
                Color32::from_rgb(255, 100, 100) // Red for inverted
            } else {
                base_color
            }
        } else {
            Color32::GRAY
        };
        ui.painter().rect_filled(fill_rect, 2.0, fill_color);

        // Invert indicator
        if is_inverted {
            ui.painter().text(
                egui::pos2(rect.right() - 15.0, rect.top() + 3.0),
                egui::Align2::RIGHT_TOP,
                "INV",
                egui::FontId::proportional(8.0),
                Color32::RED,
            );
        }

        ui.end_row();
    }
}