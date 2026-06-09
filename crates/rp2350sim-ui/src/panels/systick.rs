//! SysTick timer panel.

use egui::{Color32, RichText, Ui};
use crate::panels::PeripheralState;

/// SysTick timer panel.
#[derive(Debug, Default)]
pub struct SysTickPanel {
    /// Frequency in MHz for calculations
    freq_mhz: u32,
}

impl SysTickPanel {
    /// Create a new SysTick panel.
    pub fn new() -> Self {
        Self { freq_mhz: 150 } // Default 150 MHz
    }

    /// Get ENABLE bit from CSR.
    fn is_enabled(state: &PeripheralState) -> bool {
        (state.systick.csr & 1) != 0
    }

    /// Get TICKINT bit from CSR.
    fn tickint_enabled(state: &PeripheralState) -> bool {
        (state.systick.csr & 2) != 0
    }

    /// Get CLKSOURCE bit from CSR.
    fn clksource(state: &PeripheralState) -> bool {
        (state.systick.csr & 4) != 0
    }

    /// Get COUNTFLAG bit from CSR.
    fn countflag(state: &PeripheralState) -> bool {
        (state.systick.csr & (1 << 16)) != 0
    }

    /// Calculate period in microseconds.
    fn calculate_period_us(rvr: u32, freq_mhz: u32) -> f64 {
        if rvr == 0 || freq_mhz == 0 {
            return 0.0;
        }
        (rvr as f64 + 1.0) / freq_mhz as f64
    }

    /// Draw the panel.
    pub fn draw(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        ui.heading("SysTick Timer");
        ui.separator();

        // Register display
        ui.horizontal(|ui| {
            ui.label(RichText::new("CSR:").strong());
            ui.monospace(format!("0x{:08X}", state.systick.csr));
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("RVR:").strong());
            ui.monospace(format!("0x{:08X}", state.systick.rvr));
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("CVR:").strong());
            ui.monospace(format!("0x{:08X}", state.systick.cvr));
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("CALIB:").strong());
            ui.monospace(format!("0x{:08X}", state.systick.calib));
        });

        ui.add_space(8.0);
        ui.separator();

        // Control bits
        ui.label(RichText::new("Control Bits:").strong());

        let mut enabled = Self::is_enabled(state);
        let mut tickint = Self::tickint_enabled(state);
        let mut clksrc = Self::clksource(state);
        let countflag = Self::countflag(state);

        ui.horizontal(|ui| {
            ui.checkbox(&mut enabled, "ENABLE");
            ui.checkbox(&mut tickint, "TICKINT");
            ui.checkbox(&mut clksrc, "CLKSOURCE");
            ui.label(if countflag { 
                RichText::new("COUNTFLAG: 1").color(Color32::GREEN) 
            } else { 
                RichText::new("COUNTFLAG: 0").color(Color32::GRAY) 
            });
        });

        // Update CSR
        state.systick.csr = (if enabled { 1 } else { 0 })
            | (if tickint { 2 } else { 0 })
            | (if clksrc { 4 } else { 0 })
            | (if countflag { 1 << 16 } else { 0 });

        ui.add_space(8.0);
        ui.separator();

        // Value editors
        ui.label(RichText::new("Values:").strong());

        egui::Grid::new("systick_values")
            .num_columns(2)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
                ui.label("Reload Value (RVR):");
                let mut rvr = state.systick.rvr;
                ui.add(egui::DragValue::new(&mut rvr).clamp_range(0..=0x00FFFFFF));
                state.systick.rvr = rvr & 0x00FFFFFF; // 24-bit
                ui.end_row();

                ui.label("Current Value (CVR):");
                let mut cvr = state.systick.cvr;
                ui.add(egui::DragValue::new(&mut cvr).clamp_range(0..=0x00FFFFFF));
                state.systick.cvr = cvr & 0x00FFFFFF;
                ui.end_row();

                ui.label("CPU Frequency (MHz):");
                ui.add(egui::DragValue::new(&mut self.freq_mhz).clamp_range(1..=500));
                ui.end_row();
            });

        ui.add_space(8.0);
        ui.separator();

        // Calculations
        ui.label(RichText::new("Calculated Period:").strong());
        let period_us = Self::calculate_period_us(state.systick.rvr, self.freq_mhz);
        let period_ms = period_us / 1000.0;
        let freq_hz = if period_us > 0.0 { 1_000_000.0 / period_us } else { 0.0 };

        egui::Grid::new("systick_calc")
            .num_columns(2)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
                ui.label("Period:");
                if period_ms >= 1.0 {
                    ui.label(format!("{:.3} ms", period_ms));
                } else {
                    ui.label(format!("{:.3} µs", period_us));
                }
                ui.end_row();

                ui.label("Frequency:");
                if freq_hz >= 1_000_000.0 {
                    ui.label(format!("{:.3} MHz", freq_hz / 1_000_000.0));
                } else if freq_hz >= 1_000.0 {
                    ui.label(format!("{:.3} kHz", freq_hz / 1_000.0));
                } else {
                    ui.label(format!("{:.3} Hz", freq_hz));
                }
                ui.end_row();
            });

        ui.add_space(8.0);
        ui.separator();

        // Quick actions
        ui.label(RichText::new("Quick Actions:").strong());
        ui.horizontal(|ui| {
            if ui.button("Set 1ms @ 150MHz").clicked() {
                state.systick.rvr = 149999; // 150MHz / 1000 - 1
                self.freq_mhz = 150;
            }
            if ui.button("Set 10ms @ 150MHz").clicked() {
                state.systick.rvr = 1499999;
                self.freq_mhz = 150;
            }
            if ui.button("Set 1ms @ 125MHz").clicked() {
                state.systick.rvr = 124999;
                self.freq_mhz = 125;
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Start").clicked() {
                state.systick.csr |= 1;
            }
            if ui.button("Stop").clicked() {
                state.systick.csr &= !1;
            }
            if ui.button("Reset CVR").clicked() {
                state.systick.cvr = state.systick.rvr;
            }
        });

        ui.add_space(8.0);
        ui.separator();

        // Status indicator
        ui.horizontal(|ui| {
            ui.label(RichText::new("Status:").strong());
            if enabled {
                ui.label(RichText::new("● Running").color(Color32::GREEN));
            } else {
                ui.label(RichText::new("○ Stopped").color(Color32::GRAY));
            }
            if tickint {
                ui.label(RichText::new("IRQ: Enabled").color(Color32::YELLOW));
            }
        });

        // Progress bar for CVR
        if enabled && state.systick.rvr > 0 {
            let progress = 1.0 - (state.systick.cvr as f32 / state.systick.rvr.max(1) as f32);
            ui.add(egui::ProgressBar::new(progress)
                .text(format!("CVR: 0x{:06X}", state.systick.cvr)));
        }
    }
}