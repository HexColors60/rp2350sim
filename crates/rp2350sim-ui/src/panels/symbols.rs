//! Symbols panel for debugging.

use egui::{Color32, RichText, Ui, TextEdit};

/// Symbol information for display.
#[derive(Debug, Clone)]
pub struct SymbolEntry {
    pub name: String,
    pub address: u32,
    pub size: u32,
    pub kind: SymbolKind,
}

/// Symbol kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Variable,
    Section,
    File,
    Other,
}

impl SymbolKind {
    pub fn color(&self) -> Color32 {
        match self {
            SymbolKind::Function => Color32::from_rgb(100, 200, 100),
            SymbolKind::Variable => Color32::from_rgb(100, 150, 255),
            SymbolKind::Section => Color32::from_rgb(200, 150, 100),
            SymbolKind::File => Color32::from_rgb(180, 180, 180),
            SymbolKind::Other => Color32::GRAY,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            SymbolKind::Function => "FUNC",
            SymbolKind::Variable => "VAR",
            SymbolKind::Section => "SEC",
            SymbolKind::File => "FILE",
            SymbolKind::Other => "???",
        }
    }
}

/// Symbols panel state.
#[derive(Debug, Default)]
pub struct SymbolsState {
    /// All symbols.
    pub symbols: Vec<SymbolEntry>,
    /// Filter text.
    pub filter: String,
    /// Address search input.
    pub addr_input: String,
    /// Name search input.
    pub name_input: String,
    /// Selected symbol.
    pub selected: Option<usize>,
    /// Sort column (0=name, 1=address, 2=kind).
    pub sort_column: usize,
    /// Sort ascending.
    pub sort_ascending: bool,
}

impl SymbolsState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load symbols from a symbol table.
    pub fn load_from_table(&mut self, symbols: &rp2350sim_debug::symbols::SymbolTable) {
        self.symbols.clear();

        for sym in symbols.all_symbols() {
            let kind = match sym.kind {
                rp2350sim_debug::symbols::SymbolKind::Function => SymbolKind::Function,
                rp2350sim_debug::symbols::SymbolKind::Variable => SymbolKind::Variable,
                rp2350sim_debug::symbols::SymbolKind::Section => SymbolKind::Section,
                rp2350sim_debug::symbols::SymbolKind::File => SymbolKind::File,
                rp2350sim_debug::symbols::SymbolKind::Other => SymbolKind::Other,
            };

            self.symbols.push(SymbolEntry {
                name: sym.name.clone(),
                address: sym.address,
                size: sym.size,
                kind,
            });
        }

        // Sort by address by default
        self.symbols.sort_by_key(|s| s.address);
    }

    /// Clear all symbols.
    pub fn clear(&mut self) {
        self.symbols.clear();
        self.selected = None;
    }

    /// Get filtered symbols.
    pub fn filtered_symbols(&self) -> Vec<&SymbolEntry> {
        let filter_lower = self.filter.to_lowercase();

        self.symbols
            .iter()
            .filter(|s| {
                if filter_lower.is_empty() {
                    true
                } else {
                    s.name.to_lowercase().contains(&filter_lower)
                        || format!("{:08x}", s.address).contains(&filter_lower)
                }
            })
            .collect()
    }

    /// Find symbol by address.
    pub fn find_by_address(&self, addr: u32) -> Option<&SymbolEntry> {
        // Binary search for nearest symbol
        let idx = self.symbols.partition_point(|s| s.address <= addr);

        if idx == 0 {
            return None;
        }

        let sym = &self.symbols[idx - 1];

        // Check if address is within symbol bounds
        if sym.size > 0 && addr >= sym.address + sym.size {
            return None;
        }

        Some(sym)
    }

    /// Find symbol by name.
    pub fn find_by_name(&self, name: &str) -> Option<&SymbolEntry> {
        let name_lower = name.to_lowercase();
        self.symbols.iter().find(|s| s.name.to_lowercase() == name_lower)
    }

    /// Get symbol count.
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }
}

/// Symbols panel.
#[derive(Debug, Default)]
pub struct SymbolsPanel {
    pub state: SymbolsState,
}

impl SymbolsPanel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Draw the symbols panel.
    pub fn show(&mut self, ui: &mut Ui, current_pc: Option<u32>) -> Option<SymbolEvent> {
        let mut event = None;

        ui.horizontal(|ui| {
            ui.label(RichText::new("Symbols").strong());
            ui.label(RichText::new(format!("({})", self.state.len())).color(Color32::GRAY));
        });

        ui.separator();

        // Search/filter
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.add(TextEdit::singleline(&mut self.state.filter).desired_width(150.0));

            if ui.button("Clear").clicked() {
                self.state.filter.clear();
            }
        });

        ui.separator();

        // Symbol list - collect filtered symbols to avoid borrow issues
        let filtered: Vec<(usize, String, u32, u32, SymbolKind)> = self.state.symbols
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                let filter_lower = self.state.filter.to_lowercase();
                if filter_lower.is_empty() {
                    true
                } else {
                    s.name.to_lowercase().contains(&filter_lower)
                        || format!("{:08x}", s.address).contains(&filter_lower)
                }
            })
            .map(|(idx, s)| (idx, s.name.clone(), s.address, s.size, s.kind))
            .collect();

        let selected = self.state.selected;
        let mut new_selected = None;
        let mut new_event = None;

        egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.set_min_width(ui.available_width());
                ui.label(RichText::new("Name").strong().color(Color32::from_rgb(0, 200, 255)));
                ui.add_space(80.0);
                ui.label(RichText::new("Address").strong().color(Color32::from_rgb(0, 200, 255)));
                ui.add_space(50.0);
                ui.label(RichText::new("Size").strong().color(Color32::from_rgb(0, 200, 255)));
                ui.add_space(30.0);
                ui.label(RichText::new("Type").strong().color(Color32::from_rgb(0, 200, 255)));
            });

            ui.separator();

            for (idx, sym_name, sym_addr, sym_size, sym_kind) in filtered.iter() {
                let _is_selected = selected == Some(*idx);
                let is_current_pc = current_pc.map_or(false, |pc| {
                    pc >= *sym_addr && (*sym_size == 0 || pc < sym_addr + sym_size)
                });

                let response = ui.horizontal(|ui| {
                    ui.set_min_width(ui.available_width());

                    // Highlight current PC
                    if is_current_pc {
                        ui.label(RichText::new("▶").color(Color32::GREEN));
                    } else {
                        ui.add_space(10.0);
                    }

                    // Name
                    ui.label(RichText::new(sym_name).color(Color32::WHITE));

                    ui.add_space(10.0);

                    // Address
                    ui.label(RichText::new(format!("0x{:08X}", sym_addr)).color(Color32::LIGHT_GRAY).monospace());

                    ui.add_space(10.0);

                    // Size
                    if *sym_size > 0 {
                        ui.label(RichText::new(format!("{}", sym_size)).color(Color32::GRAY));
                    } else {
                        ui.label(RichText::new("-").color(Color32::DARK_GRAY));
                    }

                    ui.add_space(10.0);

                    // Type
                    ui.label(RichText::new(sym_kind.name()).color(sym_kind.color()));
                });

                if response.response.clicked() {
                    new_selected = Some(*idx);
                    new_event = Some(SymbolEvent::Selected(*sym_addr));
                }

                if response.response.double_clicked() {
                    new_event = Some(SymbolEvent::GotoAddress(*sym_addr));
                }
            }
        });

        // Apply changes after the closure
        if let Some(idx) = new_selected {
            self.state.selected = Some(idx);
        }
        if new_event.is_some() {
            event = new_event;
        }

        // Address lookup
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Go to addr:");
            ui.add(TextEdit::singleline(&mut self.state.addr_input).desired_width(80.0).hint_text("0x..."));

            if ui.button("Go").clicked() {
                if let Some(addr) = parse_hex_address(&self.state.addr_input) {
                    event = Some(SymbolEvent::GotoAddress(addr));
                }
            }
        });

        // Name lookup
        ui.horizontal(|ui| {
            ui.label("Find name:");
            ui.add(TextEdit::singleline(&mut self.state.name_input).desired_width(100.0));

            if ui.button("Find").clicked() {
                if let Some(sym) = self.state.find_by_name(&self.state.name_input) {
                    event = Some(SymbolEvent::GotoAddress(sym.address));
                }
            }
        });

        event
    }
}

/// Parse a hex address string.
fn parse_hex_address(s: &str) -> Option<u32> {
    let s = s.trim();
    let s = s.strip_prefix("0x").unwrap_or(s);
    u32::from_str_radix(s, 16).ok()
}

/// Symbol panel event.
#[derive(Debug, Clone, Copy)]
pub enum SymbolEvent {
    /// Symbol selected.
    Selected(u32),
    /// Go to address.
    GotoAddress(u32),
}