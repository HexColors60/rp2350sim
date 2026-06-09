//! System Information panel for RP2350 simulator.

use egui::{Color32, RichText, Ui};

/// System Information panel for RP2350.
pub struct SysinfoPanel;

impl Default for SysinfoPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl SysinfoPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn name(&self) -> &'static str {
        "Sysinfo"
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut SysinfoState) {
        ui.vertical(|ui| {
            ui.label(RichText::new("System Information Panel").strong());
            ui.separator();

            // Chip Identification
            self.draw_chip_id(ui, state);

            ui.add_space(8.0);

            // Platform
            self.draw_platform(ui, state);

            ui.add_space(8.0);

            // Device ID
            self.draw_device_id(ui, state);

            ui.add_space(8.0);

            // Memory Layout
            self.draw_memory_layout(ui, state);

            ui.add_space(8.0);

            // Clock
            self.draw_clock(ui, state);

            ui.add_space(8.0);

            // Info
            self.draw_info(ui);
        });
    }

    fn draw_chip_id(&self, ui: &mut Ui, state: &SysinfoState) {
        ui.group(|ui| {
            ui.label(RichText::new("Chip Identification").strong());
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Chip ID:");
                ui.monospace(
                    RichText::new(format!("0x{:08X}", state.chip_id))
                        .color(Color32::from_rgb(100, 200, 255)),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Revision:");
                let revision_name = match state.revision {
                    0 => "A0",
                    1 => "A1",
                    2 => "A2",
                    3 => "B0",
                    4 => "B1",
                    _ => "Unknown",
                };
                ui.monospace(
                    RichText::new(format!("{} (v{})", revision_name, state.revision))
                        .color(Color32::from_rgb(200, 200, 100)),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Package:");
                let package_name = match state.package {
                    1 => "QFN-60",
                    2 => "QFN-80",
                    3 => "WLCSPI",
                    _ => "Unknown",
                };
                ui.label(package_name);
            });
        });
    }

    fn draw_platform(&self, ui: &mut Ui, state: &SysinfoState) {
        ui.group(|ui| {
            ui.label(RichText::new("Platform").strong());
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Type:");
                let (platform_name, color) = match state.platform {
                    1 => ("ASIC", Color32::from_rgb(100, 255, 100)),
                    2 => ("FPGA", Color32::from_rgb(255, 200, 100)),
                    3 => ("Simulation", Color32::from_rgb(100, 200, 255)),
                    _ => ("Unknown", Color32::GRAY),
                };
                ui.label(RichText::new(platform_name).color(color).strong());
            });
        });
    }

    fn draw_device_id(&self, ui: &mut Ui, state: &SysinfoState) {
        let hi = (state.device_id >> 32) as u32;
        let lo = state.device_id as u32;

        ui.group(|ui| {
            ui.label(RichText::new("Device ID").strong());
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Unique ID:");
                ui.monospace(
                    RichText::new(format!("{:08X}{:08X}", hi, lo))
                        .color(Color32::from_rgb(150, 255, 150))
                        .size(14.0),
                );
            });

            ui.horizontal(|ui| {
                ui.label("          ");
                ui.monospace(
                    RichText::new(format!("Hi: 0x{:08X}", hi))
                        .color(Color32::GRAY)
                        .size(11.0),
                );
            });

            ui.horizontal(|ui| {
                ui.label("          ");
                ui.monospace(
                    RichText::new(format!("Lo: 0x{:08X}", lo))
                        .color(Color32::GRAY)
                        .size(11.0),
                );
            });
        });
    }

    fn draw_memory_layout(&self, ui: &mut Ui, state: &SysinfoState) {
        ui.group(|ui| {
            ui.label(RichText::new("Memory Layout").strong());
            ui.separator();

            egui::Grid::new("memory_grid")
                .num_columns(3)
                .spacing([40.0, 4.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Region").strong());
                    ui.label(RichText::new("Base").strong());
                    ui.label(RichText::new("Size").strong());
                    ui.end_row();

                    // Boot RAM
                    ui.label("Boot RAM");
                    ui.monospace(format!("0x{:08X}", state.bootram_base));
                    ui.monospace(format!("{} KB", state.bootram_size / 1024));
                    ui.end_row();

                    // SRAM
                    ui.label("SRAM");
                    ui.monospace(format!("0x{:08X}", state.sram_base));
                    ui.monospace(format!("{} KB", state.sram_size / 1024));
                    ui.end_row();

                    // Flash
                    ui.label("Flash");
                    ui.monospace(format!("0x{:08X}", state.flash_base));
                    ui.monospace(format!("{} KB", state.flash_size / 1024));
                    ui.end_row();
                });
        });
    }

    fn draw_clock(&self, ui: &mut Ui, state: &SysinfoState) {
        ui.group(|ui| {
            ui.label(RichText::new("Reference Clock").strong());
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Frequency:");
                let mhz = state.refclock_freq / 1_000_000;
                let khz = (state.refclock_freq % 1_000_000) / 1_000;
                ui.monospace(
                    RichText::new(format!("{}.{} MHz", mhz, khz))
                        .color(Color32::from_rgb(255, 200, 100)),
                );
            });
        });
    }

    fn draw_info(&self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("Info").strong());
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Base Address:");
                ui.monospace("0x4000_8000");
            });

            ui.add_space(4.0);

            ui.label(RichText::new("Note:").color(Color32::YELLOW));
            ui.label("Most SYSINFO registers are read-only and reflect");
            ui.label("hardware configuration determined at manufacturing time.");
        });
    }
}

/// System Information state for the panel.
#[derive(Debug, Clone)]
pub struct SysinfoState {
    /// Chip identifier (0x0000_2350 for RP2350).
    pub chip_id: u32,
    /// Silicon revision (0=A0, 1=A1, 2=A2, 3=B0, 4=B1).
    pub revision: u32,
    /// Platform type (1=ASIC, 2=FPGA, 3=Simulation).
    pub platform: u32,
    /// Package type (1=QFN60, 2=QFN80, 3=WLCSPI).
    pub package: u32,
    /// 64-bit unique device identifier.
    pub device_id: u64,
    /// Reference clock frequency in Hz (typically 12 MHz).
    pub refclock_freq: u32,
    /// Boot RAM base address.
    pub bootram_base: u32,
    /// Boot RAM size in bytes.
    pub bootram_size: u32,
    /// SRAM base address.
    pub sram_base: u32,
    /// SRAM size in bytes.
    pub sram_size: u32,
    /// Flash base address.
    pub flash_base: u32,
    /// Flash size in bytes.
    pub flash_size: u32,
}

impl Default for SysinfoState {
    fn default() -> Self {
        Self {
            chip_id: 0x0000_2350,
            revision: 3, // B0
            platform: 3, // Simulation
            package: 1,  // QFN-60
            device_id: 0xDEADBEEF_CAFE1234,
            refclock_freq: 12_000_000, // 12 MHz
            bootram_base: 0x20000000,
            bootram_size: 4 * 1024, // 4 KB
            sram_base: 0x20000000,
            sram_size: 512 * 1024, // 512 KB
            flash_base: 0x10000000,
            flash_size: 16 * 1024 * 1024, // 16 MB
        }
    }
}
