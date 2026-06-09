//! NVIC (Nested Vectored Interrupt Controller) panel.
#![allow(dead_code)]

use egui::{Color32, RichText, Ui};
use crate::panels::PeripheralState;

/// NVIC panel.
#[derive(Debug, Default)]
pub struct NvicPanel {
    /// Selected IRQ number for details.
    selected_irq: u8,
    /// Filter text
    filter: String,
}

/// IRQ info
struct IrqInfo {
    irq: u8,
    name: &'static str,
    description: &'static str,
}

impl NvicPanel {
    /// Create a new NVIC panel.
    pub fn new() -> Self {
        Self::default()
    }

    fn get_irqs() -> [IrqInfo; 40] {
        [
            IrqInfo { irq: 0, name: "TIMER0_IRQ_0", description: "Timer 0 alarm 0" },
            IrqInfo { irq: 1, name: "TIMER0_IRQ_1", description: "Timer 0 alarm 1" },
            IrqInfo { irq: 2, name: "TIMER0_IRQ_2", description: "Timer 0 alarm 2" },
            IrqInfo { irq: 3, name: "TIMER0_IRQ_3", description: "Timer 0 alarm 3" },
            IrqInfo { irq: 4, name: "TIMER1_IRQ_0", description: "Timer 1 alarm 0" },
            IrqInfo { irq: 5, name: "TIMER1_IRQ_1", description: "Timer 1 alarm 1" },
            IrqInfo { irq: 6, name: "TIMER1_IRQ_2", description: "Timer 1 alarm 2" },
            IrqInfo { irq: 7, name: "TIMER1_IRQ_3", description: "Timer 1 alarm 3" },
            IrqInfo { irq: 8, name: "PWM_IRQ_WRAP_0", description: "PWM wrap 0" },
            IrqInfo { irq: 9, name: "PWM_IRQ_WRAP_1", description: "PWM wrap 1" },
            IrqInfo { irq: 10, name: "DMA_IRQ_0", description: "DMA interrupt 0" },
            IrqInfo { irq: 11, name: "DMA_IRQ_1", description: "DMA interrupt 1" },
            IrqInfo { irq: 12, name: "USBCTRL_IRQ", description: "USB controller" },
            IrqInfo { irq: 13, name: "PIO0_IRQ_0", description: "PIO0 SM 0-3 IRQ 0" },
            IrqInfo { irq: 14, name: "PIO0_IRQ_1", description: "PIO0 SM 0-3 IRQ 1" },
            IrqInfo { irq: 15, name: "PIO1_IRQ_0", description: "PIO1 SM 0-3 IRQ 0" },
            IrqInfo { irq: 16, name: "PIO1_IRQ_1", description: "PIO1 SM 0-3 IRQ 1" },
            IrqInfo { irq: 17, name: "IO_IRQ_BANK0", description: "GPIO bank 0" },
            IrqInfo { irq: 18, name: "IO_IRQ_QSPI", description: "QSPI GPIO" },
            IrqInfo { irq: 19, name: "SIO_IRQ_PROC0", description: "SIO to core 0" },
            IrqInfo { irq: 20, name: "SIO_IRQ_PROC1", description: "SIO to core 1" },
            IrqInfo { irq: 21, name: "CLOCKS_IRQ", description: "Clocks" },
            IrqInfo { irq: 22, name: "SPI0_IRQ", description: "SPI 0" },
            IrqInfo { irq: 23, name: "SPI1_IRQ", description: "SPI 1" },
            IrqInfo { irq: 24, name: "UART0_IRQ", description: "UART 0" },
            IrqInfo { irq: 25, name: "UART1_IRQ", description: "UART 1" },
            IrqInfo { irq: 26, name: "ADC_IRQ_FIFO", description: "ADC FIFO" },
            IrqInfo { irq: 27, name: "I2C0_IRQ", description: "I2C 0" },
            IrqInfo { irq: 28, name: "I2C1_IRQ", description: "I2C 1" },
            IrqInfo { irq: 29, name: "RTC_IRQ", description: "Real-time clock" },
            IrqInfo { irq: 30, name: "HSTX_IRQ", description: "High-speed TX" },
            IrqInfo { irq: 31, name: "I2S_IRQ", description: "I2S audio" },
            IrqInfo { irq: 32, name: "TBMAN_IRQ", description: "Test bench manager" },
            IrqInfo { irq: 33, name: "TRNG_IRQ", description: "True RNG" },
            IrqInfo { irq: 34, name: "PLL_SYS_IRQ", description: "System PLL" },
            IrqInfo { irq: 35, name: "PLL_USB_IRQ", description: "USB PLL" },
            IrqInfo { irq: 36, name: "CORESIGHT_IRQ", description: "CoreSight debug" },
            IrqInfo { irq: 37, name: "OTP_IRQ", description: "One-time programmable" },
            IrqInfo { irq: 38, name: "POWMAN_IRQ_POW", description: "Power manager power" },
            IrqInfo { irq: 39, name: "POWMAN_IRQ_TIMER", description: "Power manager timer" },
        ]
    }

    /// Check if an IRQ is enabled.
    fn is_enabled(state: &PeripheralState, irq: u8) -> bool {
        let reg = (irq / 32) as usize;
        let bit = irq % 32;
        state.nvic.enabled.get(reg).map_or(false, |r| (*r >> bit) & 1 != 0)
    }

    /// Check if an IRQ is pending.
    fn is_pending(state: &PeripheralState, irq: u8) -> bool {
        let reg = (irq / 32) as usize;
        let bit = irq % 32;
        state.nvic.pending.get(reg).map_or(false, |r| (*r >> bit) & 1 != 0)
    }

    /// Check if an IRQ is active.
    fn is_active(state: &PeripheralState, irq: u8) -> bool {
        let reg = (irq / 32) as usize;
        let bit = irq % 32;
        state.nvic.active.get(reg).map_or(false, |r| (*r >> bit) & 1 != 0)
    }

    /// Get IRQ priority.
    fn get_priority(state: &PeripheralState, irq: u8) -> u8 {
        state.nvic.priority.get(irq as usize).copied().unwrap_or(0)
    }

    /// Toggle IRQ enabled.
    fn toggle_enabled(state: &mut PeripheralState, irq: u8) {
        let reg = (irq / 32) as usize;
        let bit = irq % 32;
        if let Some(r) = state.nvic.enabled.get_mut(reg) {
            *r ^= 1 << bit;
        }
    }

    /// Toggle IRQ pending.
    fn toggle_pending(state: &mut PeripheralState, irq: u8) {
        let reg = (irq / 32) as usize;
        let bit = irq % 32;
        if let Some(r) = state.nvic.pending.get_mut(reg) {
            *r ^= 1 << bit;
        }
    }

    /// Draw the panel.
    pub fn draw(&mut self, ui: &mut Ui, state: &mut PeripheralState) {
        ui.heading("NVIC (Nested Vectored Interrupt Controller)");
        ui.separator();

        // Status summary
        let enabled_count: u32 = state.nvic.enabled.iter().map(|r| r.count_ones()).sum();
        let pending_count: u32 = state.nvic.pending.iter().map(|r| r.count_ones()).sum();
        let active_count: u32 = state.nvic.active.iter().map(|r| r.count_ones()).sum();

        ui.horizontal(|ui| {
            ui.label(RichText::new("Enabled:").strong());
            ui.label(RichText::new(format!("{}", enabled_count)).color(Color32::GREEN));
            ui.label(RichText::new("Pending:").strong());
            ui.label(RichText::new(format!("{}", pending_count)).color(Color32::YELLOW));
            ui.label(RichText::new("Active:").strong());
            ui.label(RichText::new(format!("{}", active_count)).color(Color32::RED));
        });

        ui.horizontal(|ui| {
            ui.label(RichText::new("VTOR:").strong());
            ui.monospace(format!("0x{:08X}", state.nvic.vtor));
        });

        ui.add_space(8.0);
        ui.separator();

        // Quick actions
        ui.horizontal(|ui| {
            if ui.button("Clear All Pending").clicked() {
                state.nvic.pending = [0; 4];
            }
            if ui.button("Disable All").clicked() {
                state.nvic.enabled = [0; 4];
            }
        });

        ui.add_space(8.0);

        // Filter
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.filter);
        });

        ui.add_space(4.0);

        // IRQ list
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                egui::Grid::new("nvic_irqs_grid")
                    .num_columns(6)
                    .spacing([6.0, 4.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("IRQ").strong());
                        ui.label(RichText::new("Name").strong());
                        ui.label(RichText::new("En").strong());
                        ui.label(RichText::new("Pend").strong());
                        ui.label(RichText::new("Act").strong());
                        ui.label(RichText::new("Pri").strong());
                        ui.end_row();

                        for irq_info in Self::get_irqs() {
                            // Filter
                            if !self.filter.is_empty() {
                                let filter_lower = self.filter.to_lowercase();
                                if !irq_info.name.to_lowercase().contains(&filter_lower) 
                                    && !irq_info.description.to_lowercase().contains(&filter_lower) {
                                    continue;
                                }
                            }

                            let irq = irq_info.irq;
                            let enabled = Self::is_enabled(state, irq);
                            let pending = Self::is_pending(state, irq);
                            let active = Self::is_active(state, irq);
                            let priority = Self::get_priority(state, irq);

                            ui.monospace(format!("{:2}", irq));
                            ui.label(irq_info.name);

                            // Enable toggle
                            let en_color = if enabled { Color32::GREEN } else { Color32::GRAY };
                            if ui.small_button(RichText::new(if enabled { "⬤" } else { "○" }).color(en_color)).clicked() {
                                Self::toggle_enabled(state, irq);
                            }

                            // Pending toggle
                            let pend_color = if pending { Color32::YELLOW } else { Color32::GRAY };
                            if ui.small_button(RichText::new(if pending { "⬤" } else { "○" }).color(pend_color)).clicked() {
                                Self::toggle_pending(state, irq);
                            }

                            // Active indicator (read-only)
                            let act_color = if active { Color32::RED } else { Color32::GRAY };
                            ui.label(RichText::new(if active { "⬤" } else { "○" }).color(act_color));

                            // Priority
                            ui.monospace(format!("{}", priority));

                            ui.end_row();
                        }
                    });
            });

        ui.add_space(8.0);
        ui.separator();

        // Selected IRQ details
        if self.selected_irq > 0 {
            let irq = self.selected_irq;
            ui.label(RichText::new(format!("Selected IRQ {} Details:", irq)).strong());
            ui.horizontal(|ui| {
                ui.label("Priority:");
                let mut pri = Self::get_priority(state, irq);
                ui.add(egui::DragValue::new(&mut pri).clamp_range(0..=255));
                if (irq as usize) < state.nvic.priority.len() {
                    state.nvic.priority[irq as usize] = pri;
                }
            });
        }

        // Legend
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Legend:").strong());
            ui.label(RichText::new("⬤").color(Color32::GREEN));
            ui.label("Enabled");
            ui.label(RichText::new("⬤").color(Color32::YELLOW));
            ui.label("Pending");
            ui.label(RichText::new("⬤").color(Color32::RED));
            ui.label("Active");
        });
    }
}