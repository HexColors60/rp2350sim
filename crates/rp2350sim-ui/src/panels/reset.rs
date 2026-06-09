//! Reset controller panel.

use egui::{Color32, RichText, Ui};
use crate::panels::PeripheralState;

/// Reset controller panel state.
#[derive(Debug, Default)]
pub struct ResetPanel {
    /// Selected peripheral for details
    selected_peripheral: Option<usize>,
}

/// Reset peripheral info
struct PeripheralInfo {
    name: &'static str,
    bit: u8,
    wd_bit: i8,
    done_bit: i8,
}

impl ResetPanel {
    /// Create a new Reset panel.
    pub fn new() -> Self {
        Self::default()
    }

    fn get_peripherals() -> [PeripheralInfo; 17] {
        [
            PeripheralInfo { name: "USBCTRL", bit: 0, wd_bit: 0, done_bit: 0 },
            PeripheralInfo { name: "UART0", bit: 1, wd_bit: 1, done_bit: 1 },
            PeripheralInfo { name: "UART1", bit: 2, wd_bit: 2, done_bit: 2 },
            PeripheralInfo { name: "SPI0", bit: 3, wd_bit: 3, done_bit: 3 },
            PeripheralInfo { name: "SPI1", bit: 4, wd_bit: 4, done_bit: 4 },
            PeripheralInfo { name: "I2C0", bit: 5, wd_bit: 5, done_bit: 5 },
            PeripheralInfo { name: "I2C1", bit: 6, wd_bit: 6, done_bit: 6 },
            PeripheralInfo { name: "ADC", bit: 7, wd_bit: 7, done_bit: 7 },
            PeripheralInfo { name: "PWM", bit: 8, wd_bit: 8, done_bit: 8 },
            PeripheralInfo { name: "SIO", bit: 9, wd_bit: 9, done_bit: 9 },
            PeripheralInfo { name: "PIO0", bit: 10, wd_bit: 10, done_bit: 10 },
            PeripheralInfo { name: "PIO1", bit: 11, wd_bit: 11, done_bit: 11 },
            PeripheralInfo { name: "TIMER0", bit: 12, wd_bit: 12, done_bit: 12 },
            PeripheralInfo { name: "TIMER1", bit: 13, wd_bit: 13, done_bit: 13 },
            PeripheralInfo { name: "HSTX", bit: 25, wd_bit: -1, done_bit: -1 },
            PeripheralInfo { name: "TRNG", bit: 28, wd_bit: 14, done_bit: 14 },
            PeripheralInfo { name: "PLL_SYS", bit: 31, wd_bit: 17, done_bit: 17 },
        ]
    }

    /// Draw the panel.
    pub fn draw(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        ui.heading("Reset Controller (RESETS)");
        ui.separator();

        // Register values display
        ui.horizontal(|ui| {
            ui.label(RichText::new("RESET:").strong());
            ui.monospace(format!("0x{:08X}", state.reset.reset_reg));
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("WDSEL:").strong());
            ui.monospace(format!("0x{:08X}", state.reset.wdsel));
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("RESET_DONE:").strong());
            ui.monospace(format!("0x{:08X}", state.reset.reset_done));
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        // Control buttons
        ui.label(RichText::new("Quick Actions:").strong());
        ui.horizontal(|ui| {
            if ui.button("Reset All").clicked() {
                state.reset.reset_reg = 0xFFFFFFFF;
                state.reset.reset_done = 0;
            }
            if ui.button("Release All").clicked() {
                state.reset.reset_reg = 0;
                state.reset.reset_done = 0xFFFFFFFF;
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        // Peripheral list
        ui.label(RichText::new("Peripheral Reset Status:").strong());
        
        egui::ScrollArea::vertical()
            .max_height(250.0)
            .show(ui, |ui| {
                egui::Grid::new("reset_peripherals_grid")
                    .num_columns(5)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Peripheral").strong());
                        ui.label(RichText::new("In Reset").strong());
                        ui.label(RichText::new("WD Sel").strong());
                        ui.label(RichText::new("Done").strong());
                        ui.label(RichText::new("Action").strong());
                        ui.end_row();

                        for (idx, p) in Self::get_peripherals().iter().enumerate() {
                            let in_reset = (state.reset.reset_reg >> p.bit) & 1 != 0;
                            let wd_sel = if p.wd_bit >= 0 { 
                                (state.reset.wdsel >> p.wd_bit) & 1 != 0 
                            } else { 
                                false 
                            };
                            let done = if p.done_bit >= 0 { 
                                (state.reset.reset_done >> p.done_bit) & 1 != 0 
                            } else { 
                                false 
                            };

                            ui.label(p.name);
                            
                            // In Reset indicator
                            ui.label(if in_reset {
                                RichText::new("⬤").color(Color32::RED)
                            } else {
                                RichText::new("○").color(Color32::GREEN)
                            });

                            // WD Select indicator
                            if p.wd_bit >= 0 {
                                ui.label(if wd_sel {
                                    RichText::new("⬤").color(Color32::YELLOW)
                                } else {
                                    RichText::new("○").color(Color32::GRAY)
                                });
                            } else {
                                ui.label("-");
                            };

                            // Done indicator
                            if p.done_bit >= 0 {
                                ui.label(if done {
                                    RichText::new("✓").color(Color32::GREEN)
                                } else {
                                    RichText::new("○").color(Color32::GRAY)
                                });
                            } else {
                                ui.label("-");
                            }

                            // Toggle button
                            if ui.small_button(if in_reset { "Release" } else { "Reset" }).clicked() {
                                state.reset.reset_reg ^= 1 << p.bit;
                                if in_reset {
                                    // Releasing reset - set done after a delay (simulate here)
                                    if p.done_bit >= 0 {
                                        state.reset.reset_done |= 1 << p.done_bit;
                                    }
                                } else {
                                    // Asserting reset - clear done
                                    if p.done_bit >= 0 {
                                        state.reset.reset_done &= !(1 << p.done_bit);
                                    }
                                }
                            }

                            ui.end_row();

                            // Highlight selected
                            if self.selected_peripheral == Some(idx) {
                                ui.style_mut().visuals.selection.bg_fill = Color32::from_rgb(60, 60, 80);
                            }
                        }
                    });
            });

        ui.add_space(8.0);
        ui.separator();
        
        // Legend
        ui.horizontal(|ui| {
            ui.label(RichText::new("Legend:").strong());
            ui.label(RichText::new("⬤").color(Color32::RED));
            ui.label("In Reset");
            ui.label(RichText::new("⬤").color(Color32::YELLOW));
            ui.label("WD Selected");
            ui.label(RichText::new("✓").color(Color32::GREEN));
            ui.label("Reset Done");
        });
    }
}