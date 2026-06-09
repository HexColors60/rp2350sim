//! GPIO panel for RP2350 simulator.

use super::{PeripheralEvent, PeripheralState, status_indicator};
use egui::{Color32, RichText, Ui, Sense, Vec2};

/// GPIO panel with detailed pin view and configuration.
pub struct GpioPanel {
    selected_pin: Option<usize>,
    view_mode: GpioViewMode,
    show_waveform: bool,
    pin_history: Vec<[bool; 48]>,
    max_history: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum GpioViewMode {
    Grid,
    List,
    Config,
}

impl Default for GpioPanel {
    fn default() -> Self {
        Self {
            selected_pin: None,
            view_mode: GpioViewMode::Grid,
            show_waveform: true,
            pin_history: Vec::new(),
            max_history: 100,
        }
    }
}

impl GpioPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(&self) -> &'static str {
        "GPIO"
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        // Update history
        let current_state: [bool; 48] = state.gpio_values.clone().try_into().unwrap_or([false; 48]);
        self.pin_history.push(current_state);
        if self.pin_history.len() > self.max_history {
            self.pin_history.remove(0);
        }

        ui.vertical(|ui| {
            // Header with view mode selector
            ui.horizontal(|ui| {
                ui.label(RichText::new("GPIO Panel").strong());
                ui.separator();
                ui.selectable_value(&mut self.view_mode, GpioViewMode::Grid, "Grid");
                ui.selectable_value(&mut self.view_mode, GpioViewMode::List, "List");
                ui.selectable_value(&mut self.view_mode, GpioViewMode::Config, "Config");
                ui.separator();
                ui.checkbox(&mut self.show_waveform, "Waveform");
            });
            ui.separator();

            // Quick stats bar
            self.draw_stats_bar(ui, state);

            ui.add_space(4.0);

            match self.view_mode {
                GpioViewMode::Grid => self.draw_grid_view(ui, state),
                GpioViewMode::List => self.draw_list_view(ui, state),
                GpioViewMode::Config => self.draw_config_view(ui, state),
            }
        });
    }

    fn draw_stats_bar(&self, ui: &mut Ui, state: &PeripheralState) {
        ui.horizontal(|ui| {
            let inputs = state.gpio_values.iter().zip(state.gpio_directions.iter())
                .filter(|(_, &dir)| !dir).count();
            let outputs = state.gpio_directions.iter().filter(|&&d| d).count();
            let high = state.gpio_values.iter().filter(|&&v| v).count();
            let interrupts = state.gpio_interrupts.iter().filter(|&&i| i).count();

            // Input/Output counts with visual bars
            self.draw_mini_bar(ui, "IN", inputs, 48, Color32::from_rgb(100, 200, 255));
            self.draw_mini_bar(ui, "OUT", outputs, 48, Color32::from_rgb(255, 150, 100));
            self.draw_mini_bar(ui, "HIGH", high, 48, Color32::GREEN);

            ui.separator();

            if interrupts > 0 {
                ui.label(RichText::new(format!("⚠ {} IRQs", interrupts)).color(Color32::YELLOW));
            }

            ui.separator();

            // Bank status
            for bank in 0..3 {
                let start = bank * 16;
                let end = start + 16;
                let high_count = state.gpio_values[start..end].iter().filter(|&&v| v).count();
                let color = if high_count > 8 { Color32::GREEN } else if high_count > 0 { Color32::GRAY } else { Color32::DARK_GRAY };
                ui.label(RichText::new(format!("B{}", bank)).color(color));
            }
        });
    }

    fn draw_mini_bar(&self, ui: &mut Ui, label: &str, value: usize, max: usize, color: Color32) {
        ui.horizontal(|ui| {
            ui.label(format!("{}:", label));
            let bar_width = 60.0;
            let bar_height = 12.0;
            let (rect, _) = ui.allocate_exact_size(Vec2::new(bar_width, bar_height), Sense::hover());

            // Background
            ui.painter().rect_filled(rect, 2.0, Color32::from_rgb(30, 30, 40));

            // Fill
            let fill_width = (value as f32 / max as f32) * bar_width;
            let fill_rect = egui::Rect::from_min_size(rect.min, Vec2::new(fill_width, bar_height));
            ui.painter().rect_filled(fill_rect, 2.0, color);

            // Text
            ui.label(format!("{}", value));
        });
    }

    fn draw_grid_view(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        // Pin grid - 8 columns
        egui::Grid::new("gpio_grid").spacing([4.0, 4.0]).show(ui, |ui| {
            for row in 0..6 {
                for col in 0..8 {
                    let pin = row * 8 + col;
                    if pin >= 48 {
                        break;
                    }
                    self.draw_pin_cell(ui, state, pin);
                }
                ui.end_row();
            }
        });

        ui.separator();

        // Waveform display for selected pin
        if self.show_waveform {
            if let Some(pin) = self.selected_pin {
                self.draw_pin_waveform(ui, pin);
            } else {
                ui.label(RichText::new("Click a pin to see waveform").color(Color32::GRAY).size(10.0));
            }
        }

        ui.add_space(4.0);

        // Bank controls
        self.draw_bank_controls(ui, state);
    }

    fn draw_pin_waveform(&self, ui: &mut Ui, pin: usize) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("GPIO {} Waveform", pin)).strong().size(11.0));
                if let Some(first) = self.pin_history.first() {
                    let current = first[pin];
                    let color = if current { Color32::GREEN } else { Color32::GRAY };
                    ui.label(RichText::new(if current { "HIGH" } else { "LOW" }).color(color).size(10.0));
                }
            });

            let wave_height = 30.0;
            let wave_width = ui.available_width() - 10.0;
            let (rect, _) = ui.allocate_exact_size(Vec2::new(wave_width, wave_height), Sense::hover());

            // Background
            ui.painter().rect_filled(rect, 2.0, Color32::from_rgb(25, 25, 35));

            // Draw grid lines
            for x in (rect.left() as i32..rect.right() as i32).step_by(20) {
                ui.painter().line_segment(
                    [egui::pos2(x as f32, rect.top()), egui::pos2(x as f32, rect.bottom())],
                    egui::Stroke::new(0.5, Color32::from_rgb(40, 40, 50)),
                );
            }

            // Draw waveform
            if self.pin_history.len() > 1 {
                let y_high = rect.top() + 5.0;
                let y_low = rect.bottom() - 5.0;
                let step = wave_width / (self.pin_history.len() - 1) as f32;

                let mut prev_y = y_low;
                for (i, state) in self.pin_history.iter().enumerate() {
                    let x = rect.left() + i as f32 * step;
                    let y = if state[pin] { y_high } else { y_low };

                    // Vertical transition line
                    if i > 0 && prev_y != y {
                        ui.painter().line_segment(
                            [egui::pos2(x, prev_y), egui::pos2(x, y)],
                            egui::Stroke::new(1.5, Color32::from_rgb(100, 255, 150)),
                        );
                    }

                    // Horizontal line
                    if i > 0 {
                        let prev_x = rect.left() + (i - 1) as f32 * step;
                        ui.painter().line_segment(
                            [egui::pos2(prev_x, prev_y), egui::pos2(x, prev_y)],
                            egui::Stroke::new(1.5, Color32::from_rgb(100, 255, 150)),
                        );
                    }

                    prev_y = y;
                }
            }

            // Labels
            ui.painter().text(
                egui::pos2(rect.left() + 2.0, rect.top() + 2.0),
                egui::Align2::LEFT_TOP,
                "1",
                egui::FontId::proportional(8.0),
                Color32::GRAY,
            );
            ui.painter().text(
                egui::pos2(rect.left() + 2.0, rect.bottom() - 10.0),
                egui::Align2::LEFT_TOP,
                "0",
                egui::FontId::proportional(8.0),
                Color32::GRAY,
            );
        });
    }

    fn draw_bank_controls(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        ui.label(RichText::new("Bank Controls").strong());
        egui::Grid::new("bank_controls").spacing([8.0, 4.0]).show(ui, |ui| {
            for bank in 0..3 {
                let start = bank * 16;
                let end = start + 16;

                ui.label(RichText::new(format!("Bank {}", bank)).strong());

                ui.horizontal(|ui| {
                    if ui.small_button("All High").clicked() {
                        for pin in start..end {
                            if state.gpio_directions[pin] {
                                state.events.push(PeripheralEvent::GpioToggle(pin, true));
                            }
                        }
                    }
                    if ui.small_button("All Low").clicked() {
                        for pin in start..end {
                            if state.gpio_directions[pin] {
                                state.events.push(PeripheralEvent::GpioToggle(pin, false));
                            }
                        }
                    }
                    if ui.small_button("Toggle All").clicked() {
                        for pin in start..end {
                            if state.gpio_directions[pin] {
                                state.events.push(PeripheralEvent::GpioToggle(pin, !state.gpio_values[pin]));
                            }
                        }
                    }
                    if ui.small_button("All Input").clicked() {
                        for pin in start..end {
                            state.events.push(PeripheralEvent::GpioSetDirection(pin, false));
                        }
                    }
                    if ui.small_button("All Output").clicked() {
                        for pin in start..end {
                            state.events.push(PeripheralEvent::GpioSetDirection(pin, true));
                        }
                    }
                });

                ui.end_row();
            }
        });
    }

    fn draw_pin_cell(&mut self, ui: &mut Ui, state: &mut PeripheralState, pin: usize) {
        let value = state.gpio_values[pin];
        let is_output = state.gpio_directions[pin];
        let func = state.gpio_functions[pin];
        let has_irq = state.gpio_interrupts[pin];

        let bg_color = if value {
            Color32::from_rgb(0, 180, 100)
        } else {
            Color32::from_rgb(40, 40, 50)
        };

        let border_color = if has_irq {
            Color32::YELLOW
        } else if self.selected_pin == Some(pin) {
            Color32::from_rgb(255, 200, 0)
        } else if is_output {
            Color32::from_rgb(100, 120, 200)
        } else {
            Color32::from_rgb(50, 50, 60)
        };

        let response = ui.allocate_ui_with_layout(
            Vec2::new(52.0, 52.0),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.set_clip_rect(ui.max_rect());
                let rect = ui.max_rect();

                // Background with gradient effect
                ui.painter().rect_filled(rect, 4.0, bg_color);

                // Highlight effect for high pins
                if value {
                    let highlight = egui::Rect::from_min_size(rect.min, Vec2::new(rect.width(), 8.0));
                    ui.painter().rect_filled(highlight, 4.0, Color32::from_rgb(50, 255, 150));
                }

                ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(2.0, border_color));

                // Pin number
                ui.label(RichText::new(format!("{}", pin)).size(11.0).color(Color32::WHITE).strong());

                // Direction indicator with icon
                let dir_text = if is_output { "▶OUT" } else { "◀IN" };
                let dir_color = if is_output { Color32::from_rgb(150, 180, 255) } else { Color32::GRAY };
                ui.label(RichText::new(dir_text).size(9.0).color(dir_color));

                // Value with large display
                let val_color = if value { Color32::WHITE } else { Color32::DARK_GRAY };
                ui.label(RichText::new(if value { "1" } else { "0" }).size(16.0).color(val_color).strong());

                // Function indicator (if not GPIO)
                if func > 5 {
                    ui.label(RichText::new(format!("F{}", func)).size(8.0).color(Color32::from_rgb(255, 200, 100)));
                }

                // IRQ indicator
                if has_irq {
                    ui.painter().circle_filled(
                        egui::Pos2::new(rect.right() - 8.0, rect.top() + 8.0),
                        5.0,
                        Color32::YELLOW
                    );
                    ui.painter().text(
                        egui::Pos2::new(rect.right() - 8.0, rect.top() + 8.0),
                        egui::Align2::CENTER_CENTER,
                        "!",
                        egui::FontId::proportional(8.0),
                        Color32::BLACK,
                    );
                }
            }
        );

        if response.response.clicked() {
            self.selected_pin = Some(pin);
            if !is_output {
                // Toggle input pin
                state.events.push(PeripheralEvent::GpioToggle(pin, !value));
            }
        }

        if response.response.hovered() {
            self.show_pin_tooltip(ui, state, pin);
        }
    }

    fn show_pin_tooltip(&self, ui: &mut Ui, state: &PeripheralState, pin: usize) {
        egui::show_tooltip_at_pointer(ui.ctx(), egui::Id::new("gpio_tooltip"), |ui| {
            ui.set_min_width(200.0);
            ui.label(RichText::new(format!("GPIO {}", pin)).strong());
            ui.separator();

            let value = state.gpio_values[pin];
            let is_output = state.gpio_directions[pin];
            let pullup = state.gpio_pullups[pin];
            let pulldown = state.gpio_pulldowns[pin];
            let func = state.gpio_functions[pin];
            let has_irq = state.gpio_interrupts[pin];
            let drive = state.gpio_drive_strength[pin];
            let slew_fast = state.gpio_slew_fast[pin];

            egui::Grid::new("tooltip_grid").spacing([8.0, 4.0]).show(ui, |ui| {
                ui.label("Value:");
                let color = if value { Color32::GREEN } else { Color32::GRAY };
                ui.label(RichText::new(if value { "HIGH" } else { "LOW" }).color(color));
                ui.end_row();

                ui.label("Direction:");
                let color = if is_output { Color32::from_rgb(100, 150, 255) } else { Color32::GRAY };
                ui.label(RichText::new(if is_output { "OUTPUT" } else { "INPUT" }).color(color));
                ui.end_row();

                ui.label("Pull:");
                if pullup {
                    ui.label(RichText::new("UP ↑").color(Color32::from_rgb(255, 150, 100)));
                } else if pulldown {
                    ui.label(RichText::new("DOWN ↓").color(Color32::from_rgb(100, 150, 255)));
                } else {
                    ui.label("None");
                }
                ui.end_row();

                ui.label("Function:");
                let func_name = match func {
                    0 => "XIP",
                    1 => "SPI",
                    2 => "UART",
                    3 => "I2C",
                    4 => "PWM",
                    5 => "SIO",
                    6 => "PIO0",
                    7 => "PIO1",
                    8 => "CLOCK",
                    9 => "USB",
                    _ => "GPIO",
                };
                ui.label(format!("ALT{} ({})", func, func_name));
                ui.end_row();

                ui.label("Drive:");
                let drive_ma = match drive {
                    0 => "2mA",
                    1 => "4mA",
                    2 => "8mA",
                    _ => "12mA",
                };
                ui.label(drive_ma);
                ui.end_row();

                ui.label("Slew:");
                ui.label(if slew_fast { "FAST" } else { "SLOW" });
                ui.end_row();
            });

            if has_irq {
                ui.separator();
                ui.label(RichText::new("⚠ IRQ PENDING").color(Color32::YELLOW));
            }

            ui.separator();
            ui.label(RichText::new("Click to toggle/configure").color(Color32::GRAY).size(10.0));
        });
    }

    fn draw_list_view(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        // Filter controls
        ui.horizontal(|ui| {
            ui.label("Filter:");
            if ui.small_button("All").clicked() {
                // Show all
            }
            if ui.small_button("Outputs").clicked() {
                // Filter outputs
            }
            if ui.small_button("Inputs").clicked() {
                // Filter inputs
            }
            if ui.small_button("High").clicked() {
                // Filter high
            }
            if ui.small_button("IRQs").clicked() {
                // Filter IRQs
            }
        });
        ui.separator();

        egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
            egui::Grid::new("gpio_list").spacing([8.0, 4.0]).show(ui, |ui| {
                ui.label(RichText::new("Pin").strong().size(10.0));
                ui.label(RichText::new("Val").strong().size(10.0));
                ui.label(RichText::new("Dir").strong().size(10.0));
                ui.label(RichText::new("Pull").strong().size(10.0));
                ui.label(RichText::new("Func").strong().size(10.0));
                ui.label(RichText::new("Actions").strong().size(10.0));
                ui.end_row();

                for pin in 0..48 {
                    let value = state.gpio_values[pin];
                    let is_output = state.gpio_directions[pin];
                    let pullup = state.gpio_pullups[pin];
                    let pulldown = state.gpio_pulldowns[pin];
                    let func = state.gpio_functions[pin];
                    let has_irq = state.gpio_interrupts[pin];

                    // Pin number with IRQ indicator
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{}", pin)).size(10.0));
                        if has_irq {
                            ui.label(RichText::new("⚠").color(Color32::YELLOW).size(10.0));
                        }
                    });

                    // Value with color
                    let val_color = if value { Color32::GREEN } else { Color32::DARK_GRAY };
                    ui.label(RichText::new(if value { "1" } else { "0" }).color(val_color).size(10.0));

                    // Direction
                    let dir_color = if is_output { Color32::from_rgb(100, 150, 255) } else { Color32::GRAY };
                    ui.label(RichText::new(if is_output { "OUT" } else { "IN" }).color(dir_color).size(10.0));

                    // Pull
                    let pull_text = if pullup { "UP" } else if pulldown { "DN" } else { "-" };
                    ui.label(RichText::new(pull_text).size(10.0));

                    // Function
                    ui.label(RichText::new(format!("F{}", func)).size(10.0));

                    // Actions
                    ui.horizontal(|ui| {
                        if ui.small_button("T").clicked() {
                            state.events.push(PeripheralEvent::GpioToggle(pin, !value));
                        }
                        if ui.small_button("C").clicked() {
                            self.selected_pin = Some(pin);
                            self.view_mode = GpioViewMode::Config;
                        }
                    });

                    ui.end_row();
                }
            });
        });
    }

    fn draw_config_view(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        if let Some(pin) = self.selected_pin {
            self.draw_pin_config(ui, state, pin);
        } else {
            ui.label("Select a pin to configure");
            if ui.button("Back to Grid").clicked() {
                self.view_mode = GpioViewMode::Grid;
            }
        }
    }

    fn draw_pin_config(&mut self, ui: &mut Ui, state: &mut PeripheralState, pin: usize) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("GPIO {} Configuration", pin)).strong());
            if ui.button("Back").clicked() {
                self.view_mode = GpioViewMode::Grid;
            }
        });
        ui.separator();

        let value = state.gpio_values[pin];
        let is_output = state.gpio_directions[pin];
        let pullup = state.gpio_pullups[pin];
        let pulldown = state.gpio_pulldowns[pin];
        let func = state.gpio_functions[pin];
        let has_irq = state.gpio_interrupts[pin];
        let drive = state.gpio_drive_strength[pin];
        let slew_fast = state.gpio_slew_fast[pin];

        egui::Grid::new("config_grid").spacing([10.0, 8.0]).show(ui, |ui| {
            // Current state display
            ui.label(RichText::new("State:").strong());
            ui.horizontal(|ui| {
                status_indicator(ui, if value { "HIGH" } else { "LOW" }, value);
                status_indicator(ui, if is_output { "OUTPUT" } else { "INPUT" }, is_output);
                if has_irq {
                    ui.label(RichText::new("IRQ").color(Color32::YELLOW));
                }
            });
            ui.end_row();

            // Direction control
            ui.label(RichText::new("Direction:").strong());
            ui.horizontal(|ui| {
                if ui.selectable_label(!is_output, "Input").clicked() {
                    state.events.push(PeripheralEvent::GpioSetDirection(pin, false));
                }
                if ui.selectable_label(is_output, "Output").clicked() {
                    state.events.push(PeripheralEvent::GpioSetDirection(pin, true));
                }
            });
            ui.end_row();

            // Pull resistor control
            ui.label(RichText::new("Pull:").strong());
            ui.horizontal(|ui| {
                if ui.selectable_label(!pullup && !pulldown, "None").clicked() {
                    state.gpio_pullups[pin] = false;
                    state.gpio_pulldowns[pin] = false;
                }
                if ui.selectable_label(pullup, "Pull-Up").clicked() {
                    state.gpio_pullups[pin] = true;
                    state.gpio_pulldowns[pin] = false;
                }
                if ui.selectable_label(pulldown, "Pull-Down").clicked() {
                    state.gpio_pullups[pin] = false;
                    state.gpio_pulldowns[pin] = true;
                }
            });
            ui.end_row();

            // Drive strength control
            ui.label(RichText::new("Drive:").strong());
            ui.horizontal(|ui| {
                for (i, ma) in ["2mA", "4mA", "8mA", "12mA"].iter().enumerate() {
                    if ui.selectable_label(drive == i as u8, *ma).clicked() {
                        state.gpio_drive_strength[pin] = i as u8;
                    }
                }
            });
            ui.end_row();

            // Slew rate control
            ui.label(RichText::new("Slew:").strong());
            ui.horizontal(|ui| {
                if ui.selectable_label(!slew_fast, "Slow").clicked() {
                    state.gpio_slew_fast[pin] = false;
                }
                if ui.selectable_label(slew_fast, "Fast").clicked() {
                    state.gpio_slew_fast[pin] = true;
                }
            });
            ui.end_row();

            // Alternate function
            ui.label(RichText::new("Function:").strong());
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_source(format!("func_{}", pin))
                    .selected_text(format!("ALT{}", func))
                    .width(100.0)
                    .show_ui(ui, |ui| {
                        for f in 0..=9 {
                            let func_name = match f {
                                0 => "XIP",
                                1 => "SPI",
                                2 => "UART",
                                3 => "I2C",
                                4 => "PWM",
                                5 => "SIO",
                                6 => "PIO0",
                                7 => "PIO1",
                                8 => "CLOCK",
                                9 => "USB",
                                _ => "GPIO",
                            };
                            ui.selectable_value(&mut state.gpio_functions[pin], f as u8, format!("ALT{} ({})", f, func_name));
                        }
                    });
            });
            ui.end_row();
        });

        ui.add_space(8.0);

        // Quick actions
        ui.separator();
        ui.label(RichText::new("Quick Actions").strong());
        ui.horizontal(|ui| {
            if ui.button("Set High").clicked() {
                if is_output {
                    state.events.push(PeripheralEvent::GpioToggle(pin, true));
                }
            }
            if ui.button("Set Low").clicked() {
                if is_output {
                    state.events.push(PeripheralEvent::GpioToggle(pin, false));
                }
            }
            if ui.button("Toggle").clicked() {
                state.events.push(PeripheralEvent::GpioToggle(pin, !value));
            }
        });

        // Pin navigation
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if pin > 0 && ui.button("◀ Prev").clicked() {
                self.selected_pin = Some(pin - 1);
            }
            if pin < 47 && ui.button("Next ▶").clicked() {
                self.selected_pin = Some(pin + 1);
            }
        });
    }
}