//! DMA panel for RP2350 simulator UI.

use egui::{Color32, RichText, Ui};

/// DMA channel state for UI display.
#[derive(Debug, Clone, Default)]
pub struct DmaChannelState {
    pub enabled: bool,
    pub busy: bool,
    pub read_addr: u32,
    pub write_addr: u32,
    pub trans_count: u32,
    pub data_size: u8,
    pub incr_read: bool,
    pub incr_write: bool,
    pub chain_to: u8,
    pub irq_quiet: bool,
    pub bswap: bool,
}

/// DMA panel state.
#[derive(Debug, Clone, Default)]
pub struct DmaState {
    pub channels: [DmaChannelState; 12],
    pub global_enable: bool,
    pub interrupt_status: u32,
}

/// DMA panel.
#[derive(Debug, Default)]
pub struct DmaPanel {
    selected_channel: usize,
}

impl DmaPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut DmaState) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("DMA Controller").strong());
                ui.separator();
                if state.global_enable {
                    ui.label(RichText::new("Enabled").color(Color32::GREEN));
                } else {
                    ui.label(RichText::new("Disabled").color(Color32::RED));
                }
            });

            ui.separator();

            // Channel selector
            ui.horizontal(|ui| {
                ui.label("Channel:");
                for i in 0..12 {
                    let ch = &state.channels[i];
                    let label = if ch.busy {
                        format!("{}*", i)
                    } else {
                        format!("{}", i)
                    };
                    let color = if ch.enabled {
                        if ch.busy {
                            Color32::YELLOW
                        } else {
                            Color32::GREEN
                        }
                    } else {
                        Color32::DARK_GRAY
                    };
                    if ui.selectable_label(self.selected_channel == i, RichText::new(label).color(color)).clicked() {
                        self.selected_channel = i;
                    }
                }
            });

            ui.separator();

            // Selected channel details
            let ch_idx = self.selected_channel;
            let ch = &state.channels[ch_idx];

            egui::Grid::new("dma_channel").show(ui, |ui| {
                ui.label("Channel:");
                ui.label(format!("{}", ch_idx));
                ui.end_row();

                ui.label("Status:");
                ui.horizontal(|ui| {
                    if ch.enabled {
                        ui.label(RichText::new("EN").color(Color32::GREEN));
                    }
                    if ch.busy {
                        ui.label(RichText::new("BUSY").color(Color32::YELLOW));
                    }
                    if !ch.enabled && !ch.busy {
                        ui.label(RichText::new("IDLE").color(Color32::DARK_GRAY));
                    }
                });
                ui.end_row();

                ui.label("Read Address:");
                ui.monospace(format!("0x{:08X}", ch.read_addr));
                ui.end_row();

                ui.label("Write Address:");
                ui.monospace(format!("0x{:08X}", ch.write_addr));
                ui.end_row();

                ui.label("Transfer Count:");
                ui.monospace(format!("{}", ch.trans_count));
                ui.end_row();

                ui.label("Data Size:");
                let size_str = match ch.data_size {
                    0 => "Byte",
                    1 => "Halfword",
                    2 => "Word",
                    _ => "Unknown",
                };
                ui.label(size_str);
                ui.end_row();

                ui.label("Address Mode:");
                ui.horizontal(|ui| {
                    if ch.incr_read {
                        ui.label("Incr Read");
                    }
                    if ch.incr_write {
                        ui.label("Incr Write");
                    }
                    if !ch.incr_read && !ch.incr_write {
                        ui.label("Fixed");
                    }
                });
                ui.end_row();

                ui.label("Chain To:");
                if ch.chain_to < 12 {
                    ui.label(format!("Channel {}", ch.chain_to));
                } else {
                    ui.label("None");
                }
                ui.end_row();

                ui.label("Options:");
                ui.horizontal(|ui| {
                    if ch.bswap {
                        ui.label("BSWAP");
                    }
                    if ch.irq_quiet {
                        ui.label("IRQ_QUIET");
                    }
                });
                ui.end_row();
            });

            ui.separator();

            // Interrupt status
            ui.horizontal(|ui| {
                ui.label("Interrupt Status:");
                for i in 0..12 {
                    if (state.interrupt_status & (1 << i)) != 0 {
                        ui.label(RichText::new(format!("{}", i)).color(Color32::RED));
                    }
                }
                if state.interrupt_status == 0 {
                    ui.label("None");
                }
            });
        });
    }
}