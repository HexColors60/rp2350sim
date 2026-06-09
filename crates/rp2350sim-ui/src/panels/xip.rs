//! XIP panel for RP2350 simulator UI.
#![allow(dead_code)]

use egui::{Color32, RichText, Ui, Vec2};

/// XIP state for UI display.
#[derive(Debug, Clone, Default)]
pub struct XipState {
    pub enabled: bool,
    pub cache_enabled: bool,
    pub cache_bypass: bool,
    pub quad_mode: bool,
    pub power_down: bool,
    pub page_size: u32,
    pub cache_hits: u32,
    pub cache_accesses: u32,
    pub stream_addr: u32,
    pub stream_count: u32,
    pub streaming: bool,
    pub fifo_level: usize,
    pub fifo_empty: bool,
    pub fifo_full: bool,
    // Additional fields
    pub clock_div: u32,
    pub status_reg: u32,
    pub ctrl_reg: u32,
    pub flash_size: u32,
    pub boot_mode: String,
    pub last_cmd: u8,
    pub cmd_active: bool,
    pub erase_pending: bool,
    pub write_protected: bool,
    // Cache lines for visualization
    pub cache_lines: Vec<CacheLine>,
    // Command history
    pub cmd_history: Vec<CommandEntry>,
    // Performance metrics
    pub read_latency_ns: u32,
    pub write_latency_ns: u32,
    pub total_reads: u64,
    pub total_writes: u64,
    pub total_bytes_read: u64,
    pub total_bytes_written: u64,
}

/// Cache line state for visualization.
#[derive(Debug, Clone, Default)]
pub struct CacheLine {
    pub valid: bool,
    pub dirty: bool,
    pub tag: u32,
    pub lru: u8,
}

/// Command history entry.
#[derive(Debug, Clone)]
pub struct CommandEntry {
    pub cmd: u8,
    pub addr: Option<u32>,
    pub timestamp: u64,
    pub description: String,
}

/// XIP panel.
#[derive(Debug, Default)]
pub struct XipPanel {
    show_cache_stats: bool,
    show_advanced: bool,
    selected_tab: XipTab,
    cache_scroll: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum XipTab {
    #[default]
    Status,
    Cache,
    Flash,
    Performance,
}

impl XipPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw(&mut self, ui: &mut Ui, state: &mut XipState) {
        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(RichText::new("XIP Controller").strong());
                ui.separator();
                if state.enabled {
                    ui.label(RichText::new("● Enabled").color(Color32::GREEN));
                } else {
                    ui.label(RichText::new("○ Disabled").color(Color32::RED));
                }
                ui.separator();
                if state.cache_enabled {
                    ui.label(RichText::new("Cache: ON").color(Color32::GREEN));
                } else {
                    ui.label(RichText::new("Cache: OFF").color(Color32::GRAY));
                }
            });

            ui.separator();

            // Tabs
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_tab, XipTab::Status, "Status");
                ui.selectable_value(&mut self.selected_tab, XipTab::Cache, "Cache");
                ui.selectable_value(&mut self.selected_tab, XipTab::Flash, "Flash");
                ui.selectable_value(&mut self.selected_tab, XipTab::Performance, "Perf");
            });
            ui.separator();

            match self.selected_tab {
                XipTab::Status => self.draw_status_tab(ui, state),
                XipTab::Cache => self.draw_cache_tab(ui, state),
                XipTab::Flash => self.draw_flash_tab(ui, state),
                XipTab::Performance => self.draw_performance_tab(ui, state),
            }
        });
    }

    fn draw_status_tab(&mut self, ui: &mut Ui, state: &mut XipState) {
        // Quick status indicators
        ui.horizontal(|ui| {
            self.draw_status_indicator(ui, "XIP", state.enabled);
            self.draw_status_indicator(ui, "Cache", state.cache_enabled && !state.cache_bypass);
            self.draw_status_indicator(ui, "Quad", state.quad_mode);
            self.draw_status_indicator(ui, "Power", !state.power_down);
        });

        ui.add_space(8.0);

        // Status flags
        egui::Grid::new("xip_status").spacing([10.0, 4.0]).show(ui, |ui| {
            ui.label(RichText::new("Cache:").strong());
            ui.horizontal(|ui| {
                if state.cache_enabled {
                    if state.cache_bypass {
                        ui.label(RichText::new("Bypass").color(Color32::YELLOW));
                    } else {
                        ui.label(RichText::new("Enabled").color(Color32::GREEN));
                    }
                } else {
                    ui.label(RichText::new("Disabled").color(Color32::RED));
                }
            });
            ui.end_row();

            ui.label(RichText::new("Quad Mode:").strong());
            if state.quad_mode {
                ui.label(RichText::new("Enabled (4-bit)").color(Color32::GREEN));
            } else {
                ui.label(RichText::new("Disabled (1-bit)").color(Color32::DARK_GRAY));
            }
            ui.end_row();

            ui.label(RichText::new("Power:").strong());
            if state.power_down {
                ui.label(RichText::new("Down").color(Color32::RED));
            } else {
                ui.label(RichText::new("Active").color(Color32::GREEN));
            }
            ui.end_row();

            ui.label(RichText::new("Page Size:").strong());
            ui.label(format!("{} bytes", state.page_size));
            ui.end_row();

            ui.label(RichText::new("Clock Div:").strong());
            let freq_mhz = 150 / state.clock_div.max(1);
            ui.label(format!("{} ({} MHz)", state.clock_div, freq_mhz));
            ui.end_row();

            ui.label(RichText::new("Boot Mode:").strong());
            ui.label(&state.boot_mode);
            ui.end_row();
        });

        ui.separator();

        // Memory map visualization
        ui.label(RichText::new("Memory Map").strong());
        self.draw_memory_map(ui, state);

        ui.separator();

        // Streaming status
        ui.label(RichText::new("Streaming").strong());
        egui::Grid::new("xip_stream").spacing([10.0, 4.0]).show(ui, |ui| {
            ui.label("Status:");
            if state.streaming {
                ui.label(RichText::new("▶ Active").color(Color32::YELLOW));
            } else {
                ui.label(RichText::new("⏸ Idle").color(Color32::DARK_GRAY));
            }
            ui.end_row();

            ui.label("Address:");
            ui.monospace(format!("0x{:08X}", state.stream_addr));
            ui.end_row();

            ui.label("Count:");
            ui.monospace(format!("{} bytes", state.stream_count));
            ui.end_row();
        });

        // FIFO visualization
        ui.separator();
        ui.label(RichText::new("FIFO Level").strong());
        self.draw_fifo_bar(ui, state);
    }

    fn draw_status_indicator(&self, ui: &mut Ui, label: &str, active: bool) {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(60.0, 24.0), egui::Sense::hover());
        let bg_color = if active {
            Color32::from_rgb(0, 100, 50)
        } else {
            Color32::from_rgb(50, 50, 60)
        };
        let text_color = if active {
            Color32::GREEN
        } else {
            Color32::GRAY
        };

        ui.painter().rect_filled(rect, 4.0, bg_color);
        ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(1.0, text_color));
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(11.0),
            text_color,
        );
    }

    fn draw_memory_map(&self, ui: &mut Ui, _state: &XipState) {
        let bar_width = ui.available_width() - 10.0;
        let bar_height = 40.0;
        let (rect, _) = ui.allocate_exact_size(Vec2::new(bar_width, bar_height), egui::Sense::hover());

        // Background
        ui.painter().rect_filled(rect, 2.0, Color32::from_rgb(30, 30, 40));

        // Flash region (0x10000000 - 0x15FFFFFF)
        let flash_start = 0.1;
        let flash_end = 0.6;
        let flash_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left() + flash_start * bar_width, rect.top() + 5.0),
            Vec2::new((flash_end - flash_start) * bar_width, 30.0),
        );
        ui.painter().rect_filled(flash_rect, 2.0, Color32::from_rgb(0, 80, 120));
        ui.painter().text(
            egui::pos2(flash_rect.left() + 5.0, flash_rect.top() + 8.0),
            egui::Align2::LEFT_TOP,
            "Flash XIP",
            egui::FontId::proportional(10.0),
            Color32::WHITE,
        );
        ui.painter().text(
            egui::pos2(flash_rect.left() + 5.0, flash_rect.top() + 18.0),
            egui::Align2::LEFT_TOP,
            "0x10000000",
            egui::FontId::proportional(8.0),
            Color32::GRAY,
        );

        // SRAM region (0x20000000 - 0x200BFFFF)
        let sram_start = 0.65;
        let sram_end = 0.75;
        let sram_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left() + sram_start * bar_width, rect.top() + 5.0),
            Vec2::new((sram_end - sram_start) * bar_width, 30.0),
        );
        ui.painter().rect_filled(sram_rect, 2.0, Color32::from_rgb(80, 0, 80));
        ui.painter().text(
            egui::pos2(sram_rect.left() + 5.0, sram_rect.top() + 8.0),
            egui::Align2::LEFT_TOP,
            "SRAM",
            egui::FontId::proportional(10.0),
            Color32::WHITE,
        );

        // ROM region (0x00000000 - 0x0003FFFF)
        let rom_start = 0.02;
        let rom_end = 0.08;
        let rom_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left() + rom_start * bar_width, rect.top() + 5.0),
            Vec2::new((rom_end - rom_start) * bar_width, 30.0),
        );
        ui.painter().rect_filled(rom_rect, 2.0, Color32::from_rgb(80, 60, 0));
        ui.painter().text(
            egui::pos2(rom_rect.left() + 2.0, rom_rect.top() + 12.0),
            egui::Align2::LEFT_TOP,
            "ROM",
            egui::FontId::proportional(9.0),
            Color32::WHITE,
        );
    }

    fn draw_fifo_bar(&self, ui: &mut Ui, state: &XipState) {
        ui.horizontal(|ui| {
            for i in 0..16 {
                let filled = i < state.fifo_level;
                let color = if filled {
                    if state.fifo_full && i == 15 {
                        Color32::from_rgb(255, 100, 0)
                    } else {
                        Color32::from_rgb(0, 150, 0)
                    }
                } else {
                    Color32::from_rgb(50, 50, 50)
                };
                let (rect, _) = ui.allocate_exact_size(
                    Vec2::new(12.0, 20.0),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(rect, 1.0, color);
            }
            ui.label(format!(" {} / 16", state.fifo_level));
        });
    }

    fn draw_cache_tab(&mut self, ui: &mut Ui, state: &mut XipState) {
        // Cache statistics
        egui::Grid::new("xip_cache").spacing([10.0, 4.0]).show(ui, |ui| {
            ui.label(RichText::new("Cache Hits:").strong());
            ui.monospace(format!("{}", state.cache_hits));
            ui.end_row();

            ui.label(RichText::new("Cache Accesses:").strong());
            ui.monospace(format!("{}", state.cache_accesses));
            ui.end_row();

            ui.label(RichText::new("Hit Rate:").strong());
            if state.cache_accesses > 0 {
                let rate = (state.cache_hits as f64 / state.cache_accesses as f64) * 100.0;
                let color = if rate > 80.0 {
                    Color32::GREEN
                } else if rate > 50.0 {
                    Color32::YELLOW
                } else {
                    Color32::RED
                };
                ui.label(RichText::new(format!("{:.1}%", rate)).color(color));
            } else {
                ui.label("N/A");
            }
            ui.end_row();

            ui.label(RichText::new("Miss Rate:").strong());
            if state.cache_accesses > 0 {
                let misses = state.cache_accesses - state.cache_hits;
                let rate = (misses as f64 / state.cache_accesses as f64) * 100.0;
                ui.label(format!("{:.1}%", rate));
            } else {
                ui.label("N/A");
            }
            ui.end_row();
        });

        ui.separator();

        // Cache hit rate visualization
        ui.label(RichText::new("Hit Rate Visualization").strong());
        let bar_width = ui.available_width() - 20.0;
        let bar_height = 24.0;
        let (rect, _) = ui.allocate_exact_size(Vec2::new(bar_width, bar_height), egui::Sense::hover());

        // Background
        ui.painter().rect_filled(rect, 2.0, Color32::from_rgb(40, 40, 50));

        // Hit portion (green)
        if state.cache_accesses > 0 {
            let hit_rate = state.cache_hits as f32 / state.cache_accesses as f32;
            let hit_width = hit_rate * bar_width;
            let hit_rect = egui::Rect::from_min_size(rect.min, Vec2::new(hit_width, bar_height));
            ui.painter().rect_filled(hit_rect, 2.0, Color32::from_rgb(0, 150, 0));

            // Labels
            ui.painter().text(
                egui::pos2(rect.left() + 10.0, rect.top() + 4.0),
                egui::Align2::LEFT_TOP,
                format!("Hits: {:.0}%", hit_rate * 100.0),
                egui::FontId::proportional(12.0),
                Color32::WHITE,
            );
        }

        ui.separator();

        // Cache line visualization
        ui.label(RichText::new("Cache Lines").strong());
        self.draw_cache_lines(ui, state);

        ui.separator();

        // Cache controls
        ui.horizontal(|ui| {
            if ui.button("Reset Stats").clicked() {
                state.cache_hits = 0;
                state.cache_accesses = 0;
            }
            if ui.button("Clear Cache").clicked() {
                // This would trigger a cache clear event
            }
            if ui.button("Invalidate").clicked() {
                // This would trigger a cache invalidate event
            }
        });
    }

    fn draw_cache_lines(&self, ui: &mut Ui, state: &XipState) {
        egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
            egui::Grid::new("cache_lines").spacing([4.0, 2.0]).show(ui, |ui| {
                ui.label(RichText::new("#").size(9.0).strong());
                ui.label(RichText::new("V").size(9.0).strong());
                ui.label(RichText::new("D").size(9.0).strong());
                ui.label(RichText::new("Tag").size(9.0).strong());
                ui.label(RichText::new("LRU").size(9.0).strong());
                ui.end_row();

                for (i, line) in state.cache_lines.iter().enumerate().take(16) {
                    ui.label(RichText::new(format!("{}", i)).size(9.0));

                    // Valid bit
                    let v_color = if line.valid { Color32::GREEN } else { Color32::DARK_GRAY };
                    ui.label(RichText::new(if line.valid { "1" } else { "0" }).color(v_color).size(9.0));

                    // Dirty bit
                    let d_color = if line.dirty { Color32::YELLOW } else { Color32::DARK_GRAY };
                    ui.label(RichText::new(if line.dirty { "1" } else { "0" }).color(d_color).size(9.0));

                    // Tag
                    ui.monospace(RichText::new(format!("{:04X}", line.tag)).size(9.0));

                    // LRU
                    ui.label(RichText::new(format!("{}", line.lru)).size(9.0));

                    ui.end_row();
                }
            });
        });
    }

    fn draw_flash_tab(&mut self, ui: &mut Ui, state: &mut XipState) {
        // Flash information
        egui::Grid::new("xip_flash").spacing([10.0, 4.0]).show(ui, |ui| {
            ui.label(RichText::new("Flash Size:").strong());
            if state.flash_size > 0 {
                let size_mb = state.flash_size as f64 / (1024.0 * 1024.0);
                ui.label(format!("{:.1} MB", size_mb));
            } else {
                ui.label("Unknown");
            }
            ui.end_row();

            ui.label(RichText::new("Status Register:").strong());
            ui.monospace(format!("0x{:02X}", state.status_reg));
            ui.end_row();

            ui.label(RichText::new("Control Register:").strong());
            ui.monospace(format!("0x{:08X}", state.ctrl_reg));
            ui.end_row();

            ui.label(RichText::new("Write Protected:").strong());
            if state.write_protected {
                ui.label(RichText::new("Yes").color(Color32::YELLOW));
            } else {
                ui.label(RichText::new("No").color(Color32::GREEN));
            }
            ui.end_row();
        });

        ui.separator();

        // Command status
        ui.label(RichText::new("Command Status").strong());
        egui::Grid::new("xip_cmd").spacing([10.0, 4.0]).show(ui, |ui| {
            ui.label("Last Command:");
            ui.monospace(format!("0x{:02X}", state.last_cmd));
            ui.label(self.cmd_description(state.last_cmd));
            ui.end_row();

            ui.label("Command Active:");
            if state.cmd_active {
                ui.label(RichText::new("Yes").color(Color32::YELLOW));
            } else {
                ui.label(RichText::new("No").color(Color32::DARK_GRAY));
            }
            ui.end_row();

            ui.label("Erase Pending:");
            if state.erase_pending {
                ui.label(RichText::new("Yes").color(Color32::RED));
            } else {
                ui.label(RichText::new("No").color(Color32::GREEN));
            }
            ui.end_row();
        });

        ui.separator();

        // Flash commands
        ui.label(RichText::new("Quick Commands").strong());
        ui.horizontal(|ui| {
            if ui.button("Read JEDEC ID").clicked() {
                state.last_cmd = 0x9F;
                self.add_cmd_to_history(state, 0x9F, None, "Read JEDEC ID");
            }
            if ui.button("Read Status").clicked() {
                state.last_cmd = 0x05;
                self.add_cmd_to_history(state, 0x05, None, "Read Status");
            }
            if ui.button("Write Enable").clicked() {
                state.last_cmd = 0x06;
                self.add_cmd_to_history(state, 0x06, None, "Write Enable");
            }
        });
        ui.horizontal(|ui| {
            if ui.button("Chip Erase").clicked() {
                state.last_cmd = 0xC7;
                state.erase_pending = true;
                self.add_cmd_to_history(state, 0xC7, None, "Chip Erase");
            }
            if ui.button("Release Power Down").clicked() {
                state.last_cmd = 0xAB;
                state.power_down = false;
                self.add_cmd_to_history(state, 0xAB, None, "Release Power Down");
            }
            if ui.button("Power Down").clicked() {
                state.last_cmd = 0xB9;
                state.power_down = true;
                self.add_cmd_to_history(state, 0xB9, None, "Power Down");
            }
        });

        ui.separator();

        // Status register bit display
        ui.label(RichText::new("Status Register Bits").strong());
        let status = state.status_reg;
        egui::Grid::new("xip_status_bits").spacing([4.0, 2.0]).show(ui, |ui| {
            // Bit 0: WIP (Write In Progress)
            ui.label("WIP:");
            ui.label(if status & 0x01 != 0 { "1" } else { "0" });
            ui.label(if status & 0x01 != 0 { "Writing" } else { "Idle" });
            ui.end_row();

            // Bit 1: WEL (Write Enable Latch)
            ui.label("WEL:");
            ui.label(if status & 0x02 != 0 { "1" } else { "0" });
            ui.label(if status & 0x02 != 0 { "Enabled" } else { "Disabled" });
            ui.end_row();

            // BP bits (Block Protect)
            ui.label("BP:");
            let bp = (status >> 2) & 0x7;
            ui.label(format!("{}", bp));
            ui.label(format!("Protected: {}%", (bp + 1) * 12));
            ui.end_row();

            // SRWD (Status Register Write Disable)
            ui.label("SRWD:");
            ui.label(if status & 0x80 != 0 { "1" } else { "0" });
            ui.label(if status & 0x80 != 0 { "Locked" } else { "Unlocked" });
            ui.end_row();
        });

        ui.separator();

        // Command history
        if !state.cmd_history.is_empty() {
            ui.label(RichText::new("Command History").strong());
            egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                egui::Grid::new("cmd_history").spacing([4.0, 2.0]).show(ui, |ui| {
                    ui.label(RichText::new("Time").size(9.0).strong());
                    ui.label(RichText::new("Cmd").size(9.0).strong());
                    ui.label(RichText::new("Description").size(9.0).strong());
                    ui.end_row();

                    for entry in state.cmd_history.iter().rev().take(10) {
                        ui.label(RichText::new(format!("{}", entry.timestamp)).size(9.0));
                        ui.monospace(RichText::new(format!("0x{:02X}", entry.cmd)).size(9.0));
                        ui.label(RichText::new(&entry.description).size(9.0));
                        ui.end_row();
                    }
                });
            });
        }
    }

    fn draw_performance_tab(&mut self, ui: &mut Ui, state: &mut XipState) {
        // Performance metrics
        egui::Grid::new("xip_perf").spacing([10.0, 4.0]).show(ui, |ui| {
            ui.label(RichText::new("Read Latency:").strong());
            ui.label(format!("{} ns", state.read_latency_ns));
            ui.end_row();

            ui.label(RichText::new("Write Latency:").strong());
            ui.label(format!("{} ns", state.write_latency_ns));
            ui.end_row();

            ui.label(RichText::new("Total Reads:").strong());
            ui.label(format!("{}", state.total_reads));
            ui.end_row();

            ui.label(RichText::new("Total Writes:").strong());
            ui.label(format!("{}", state.total_writes));
            ui.end_row();

            ui.label(RichText::new("Bytes Read:").strong());
            let read_kb = state.total_bytes_read as f64 / 1024.0;
            if read_kb > 1024.0 {
                ui.label(format!("{:.2} MB", read_kb / 1024.0));
            } else {
                ui.label(format!("{:.2} KB", read_kb));
            }
            ui.end_row();

            ui.label(RichText::new("Bytes Written:").strong());
            let write_kb = state.total_bytes_written as f64 / 1024.0;
            if write_kb > 1024.0 {
                ui.label(format!("{:.2} MB", write_kb / 1024.0));
            } else {
                ui.label(format!("{:.2} KB", write_kb));
            }
            ui.end_row();
        });

        ui.separator();

        // Read/Write ratio visualization
        ui.label(RichText::new("Read/Write Ratio").strong());
        let total_ops = state.total_reads + state.total_writes;
        if total_ops > 0 {
            let bar_width = ui.available_width() - 20.0;
            let bar_height = 24.0;
            let (rect, _) = ui.allocate_exact_size(Vec2::new(bar_width, bar_height), egui::Sense::hover());

            // Background
            ui.painter().rect_filled(rect, 2.0, Color32::from_rgb(40, 40, 50));

            // Read portion (blue)
            let read_ratio = state.total_reads as f32 / total_ops as f32;
            let read_width = read_ratio * bar_width;
            let read_rect = egui::Rect::from_min_size(rect.min, Vec2::new(read_width, bar_height));
            ui.painter().rect_filled(read_rect, 2.0, Color32::from_rgb(0, 100, 200));

            // Write portion (orange)
            let write_rect = egui::Rect::from_min_size(
                egui::pos2(rect.left() + read_width, rect.top()),
                Vec2::new(bar_width - read_width, bar_height),
            );
            ui.painter().rect_filled(write_rect, 2.0, Color32::from_rgb(200, 100, 0));

            // Labels
            ui.painter().text(
                egui::pos2(rect.left() + 10.0, rect.top() + 4.0),
                egui::Align2::LEFT_TOP,
                format!("R: {:.0}%", read_ratio * 100.0),
                egui::FontId::proportional(12.0),
                Color32::WHITE,
            );
        }

        ui.separator();

        // Throughput
        ui.label(RichText::new("Throughput").strong());
        egui::Grid::new("xip_throughput").spacing([10.0, 4.0]).show(ui, |ui| {
            // Calculate throughput (simplified)
            let read_throughput = if state.read_latency_ns > 0 {
                let bytes_per_sec = 1_000_000_000.0 / state.read_latency_ns as f64;
                bytes_per_sec / (1024.0 * 1024.0)
            } else {
                0.0
            };

            ui.label("Read:");
            ui.label(format!("{:.2} MB/s theoretical", read_throughput));
            ui.end_row();

            let write_throughput = if state.write_latency_ns > 0 {
                let bytes_per_sec = 1_000_000_000.0 / state.write_latency_ns as f64;
                bytes_per_sec / (1024.0 * 1024.0)
            } else {
                0.0
            };

            ui.label("Write:");
            ui.label(format!("{:.2} MB/s theoretical", write_throughput));
            ui.end_row();
        });

        ui.separator();

        // Reset button
        if ui.button("Reset Performance Counters").clicked() {
            state.total_reads = 0;
            state.total_writes = 0;
            state.total_bytes_read = 0;
            state.total_bytes_written = 0;
        }
    }

    fn cmd_description(&self, cmd: u8) -> &'static str {
        match cmd {
            0x03 => "Read Data",
            0x0B => "Fast Read",
            0x05 => "Read Status",
            0x06 => "Write Enable",
            0x04 => "Write Disable",
            0x9F => "Read JEDEC ID",
            0x20 => "Sector Erase (4KB)",
            0xD8 => "Block Erase (64KB)",
            0xC7 => "Chip Erase",
            0x02 => "Page Program",
            0xAB => "Release Power Down",
            0xB9 => "Power Down",
            0x35 => "Read Status 2",
            0x15 => "Read Status 3",
            0xEB => "Fast Read Quad",
            0x6B => "Fast Read Dual",
            _ => "Unknown",
        }
    }

    fn add_cmd_to_history(&self, state: &mut XipState, cmd: u8, addr: Option<u32>, description: &str) {
        state.cmd_history.push(CommandEntry {
            cmd,
            addr,
            timestamp: state.cmd_history.len() as u64,
            description: description.to_string(),
        });

        // Keep only last 50 commands
        if state.cmd_history.len() > 50 {
            state.cmd_history.remove(0);
        }
    }
}