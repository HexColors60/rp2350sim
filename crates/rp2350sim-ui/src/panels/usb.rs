//! USB panel for RP2350 simulator.

use super::PeripheralState;
use egui::{Color32, RichText, Ui, Vec2};

/// USB panel with device/host monitoring.
pub struct UsbPanel {
    mode: UsbMode,
    selected_endpoint: usize,
    endpoint_data: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum UsbMode {
    Device,
    Host,
}

impl Default for UsbPanel {
    fn default() -> Self {
        Self {
            mode: UsbMode::Device,
            selected_endpoint: 0,
            endpoint_data: String::new(),
        }
    }
}

impl UsbPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(&self) -> &'static str {
        "USB"
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        ui.vertical(|ui| {
            ui.label(RichText::new("USB Panel").strong());
            ui.separator();

            // Status bar
            ui.horizontal(|ui| {
                let connect_color = if state.usb_connected { Color32::GREEN } else { Color32::RED };
                let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 5.0, connect_color);
                ui.label(if state.usb_connected { "Connected" } else { "Disconnected" });

                ui.separator();

                ui.label(if state.usb_device_mode { "Device Mode" } else { "Host Mode" });
            });

            ui.add_space(8.0);

            // Mode selector
            ui.group(|ui| {
                ui.label(RichText::new("Mode Configuration").strong());
                ui.separator();

                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.mode, UsbMode::Device, "Device Mode");
                    ui.selectable_value(&mut self.mode, UsbMode::Host, "Host Mode");
                });

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    if ui.button("Connect").clicked() {
                        state.usb_connected = true;
                    }
                    if ui.button("Disconnect").clicked() {
                        state.usb_connected = false;
                    }
                    if ui.button("Reset").clicked() {
                        // USB reset
                    }
                });
            });

            ui.add_space(8.0);

            // Endpoints
            ui.group(|ui| {
                ui.label(RichText::new("Endpoints").strong());
                ui.separator();

                // Endpoint tabs
                ui.horizontal(|ui| {
                    for ep in 0..16 {
                        let selected = self.selected_endpoint == ep;
                        if ui.selectable_label(selected, format!("EP{}", ep)).clicked() {
                            self.selected_endpoint = ep;
                        }
                    }
                });

                ui.add_space(4.0);

                // Endpoint configuration
                let ep_type = match self.selected_endpoint {
                    0 => "Control",
                    1..=7 => "Bulk IN",
                    8..=15 => "Bulk OUT",
                    _ => "Unknown",
                };

                ui.horizontal(|ui| {
                    ui.label(format!("Endpoint {}:", self.selected_endpoint));
                    ui.label(RichText::new(ep_type).color(Color32::YELLOW));
                });

                // Endpoint status
                ui.horizontal(|ui| {
                    ui.label("Status:");
                    ui.label("Idle");
                });

                // Data input
                ui.label("Data (hex):");
                ui.add(
                    egui::TextEdit::singleline(&mut self.endpoint_data)
                        .desired_width(ui.available_width())
                        .hint_text("00 01 02 03...")
                );

                ui.horizontal(|ui| {
                    if ui.button("Send").clicked() {
                        // Send data
                    }
                    if ui.button("Clear").clicked() {
                        self.endpoint_data.clear();
                    }
                });
            });

            ui.add_space(8.0);

            // Transfer log
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Transfer Log").strong());
                    if ui.button("Clear").clicked() {
                        // Clear log
                    }
                });
                ui.separator();

                egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                    ui.label(RichText::new("No transfers yet").color(Color32::GRAY));
                });
            });

            ui.add_space(8.0);

            // Device classes
            ui.group(|ui| {
                ui.label(RichText::new("Device Classes").strong());
                ui.separator();

                ui.horizontal(|ui| {
                    ui.checkbox(&mut true, "CDC (Serial)");
                    ui.checkbox(&mut false, "MSC (Mass Storage)");
                    ui.checkbox(&mut false, "HID");
                });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut false, "MIDI");
                    ui.checkbox(&mut false, "Audio");
                    ui.checkbox(&mut false, "Vendor");
                });
            });
        });
    }
}