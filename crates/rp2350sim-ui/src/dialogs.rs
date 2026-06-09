//! File dialogs for the UI.
#![allow(dead_code)]

use egui::Context;

/// File dialog state.
#[derive(Debug, Clone)]
pub struct FileDialogState {
    /// Current path.
    pub current_path: String,
    /// Selected file.
    pub selected_file: Option<String>,
    /// File filter.
    pub filter: String,
    /// Dialog is open.
    pub is_open: bool,
    /// Dialog title.
    pub title: String,
    /// Dialog mode (open/save).
    pub mode: FileDialogMode,
}

impl Default for FileDialogState {
    fn default() -> Self {
        Self {
            current_path: ".".to_string(),
            selected_file: None,
            filter: "*".to_string(),
            is_open: false,
            title: "Open File".to_string(),
            mode: FileDialogMode::Open,
        }
    }
}

impl FileDialogState {
    /// Create a new file dialog state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Open an open file dialog.
    pub fn open_open(&mut self, title: &str, filter: &str) {
        self.title = title.to_string();
        self.filter = filter.to_string();
        self.mode = FileDialogMode::Open;
        self.is_open = true;
        self.selected_file = None;
    }

    /// Open a save file dialog.
    pub fn open_save(&mut self, title: &str, filter: &str) {
        self.title = title.to_string();
        self.filter = filter.to_string();
        self.mode = FileDialogMode::Save;
        self.is_open = true;
        self.selected_file = None;
    }

    /// Close the dialog.
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Check if a file was selected.
    pub fn selected(&self) -> Option<&str> {
        self.selected_file.as_deref()
    }
}

/// File dialog mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileDialogMode {
    /// Open file dialog.
    Open,
    /// Save file dialog.
    Save,
}

/// File dialog result.
#[derive(Debug, Clone)]
pub struct FileDialogResult {
    /// Selected file path.
    pub path: String,
    /// Whether the user confirmed the selection.
    pub confirmed: bool,
}

/// Simple file dialog implementation.
pub struct FileDialog {
    /// Dialog state.
    state: FileDialogState,
    /// Current directory entries.
    entries: Vec<DirEntry>,
    /// Selected entry index.
    selected_index: Option<usize>,
    /// File name input.
    file_name: String,
}

/// Directory entry.
#[derive(Debug, Clone)]
pub struct DirEntry {
    /// Entry name.
    pub name: String,
    /// Is directory.
    pub is_dir: bool,
    /// File size (if file).
    pub size: Option<u64>,
}

impl FileDialog {
    /// Create a new file dialog.
    pub fn new() -> Self {
        Self {
            state: FileDialogState::new(),
            entries: Vec::new(),
            selected_index: None,
            file_name: String::new(),
        }
    }

    /// Get the dialog state.
    pub fn state(&self) -> &FileDialogState {
        &self.state
    }

    /// Get mutable dialog state.
    pub fn state_mut(&mut self) -> &mut FileDialogState {
        &mut self.state
    }

    /// Open an open file dialog.
    pub fn open_open(&mut self, title: &str, filter: &str) {
        self.state.open_open(title, filter);
        self.refresh_entries();
    }

    /// Open a save file dialog.
    pub fn open_save(&mut self, title: &str, filter: &str) {
        self.state.open_save(title, filter);
        self.refresh_entries();
    }

    /// Refresh directory entries.
    fn refresh_entries(&mut self) {
        self.entries.clear();
        
        let path = std::path::Path::new(&self.state.current_path);
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                let size = if is_dir {
                    None
                } else {
                    entry.metadata().ok().map(|m| m.len())
                };
                
                // Apply filter
                if !is_dir && self.state.filter != "*" {
                    let ext = std::path::Path::new(&name)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");
                    if !self.state.filter.contains(ext) {
                        continue;
                    }
                }
                
                self.entries.push(DirEntry { name, is_dir, size });
            }
        }
        
        // Sort: directories first, then files
        self.entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });
    }

    /// Show the file dialog.
    pub fn show(&mut self, ctx: &Context) -> Option<FileDialogResult> {
        if !self.state.is_open {
            return None;
        }

        let mut result = None;

        egui::Window::new(&self.state.title.clone())
            .default_size(egui::vec2(500.0, 400.0))
            .resizable(true)
            .show(ctx, |ui| {
                // Path bar
                ui.horizontal(|ui| {
                    ui.label("Path:");
                    if ui.text_edit_singleline(&mut self.state.current_path).changed() {
                        self.refresh_entries();
                    }
                    if ui.button("Refresh").clicked() {
                        self.refresh_entries();
                    }
                });

                ui.separator();

                // File list
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, entry) in self.entries.iter().enumerate() {
                        let is_selected = self.selected_index == Some(i);
                        let icon = if entry.is_dir { "📁" } else { "📄" };
                        let label = format!("{} {}", icon, entry.name);
                        
                        if ui.selectable_label(is_selected, &label).clicked() {
                            self.selected_index = Some(i);
                            self.file_name = entry.name.clone();
                        }
                    }
                });

                ui.separator();

                // File name input
                ui.horizontal(|ui| {
                    ui.label("File name:");
                    ui.text_edit_singleline(&mut self.file_name);
                });

                ui.separator();

                // Buttons
                ui.horizontal(|ui| {
                    if ui.button("Open").clicked() {
                        if let Some(idx) = self.selected_index {
                            let entry = &self.entries[idx];
                            if entry.is_dir {
                                // Navigate into directory
                                let new_path = std::path::Path::new(&self.state.current_path)
                                    .join(&entry.name);
                                self.state.current_path = new_path.to_string_lossy().to_string();
                                self.refresh_entries();
                                self.selected_index = None;
                                self.file_name.clear();
                            } else {
                                // Select file
                                let path = std::path::Path::new(&self.state.current_path)
                                    .join(&entry.name);
                                result = Some(FileDialogResult {
                                    path: path.to_string_lossy().to_string(),
                                    confirmed: true,
                                });
                                self.state.is_open = false;
                            }
                        }
                    }
                    
                    if ui.button("Cancel").clicked() {
                        result = Some(FileDialogResult {
                            path: String::new(),
                            confirmed: false,
                        });
                        self.state.is_open = false;
                    }
                });
            });

        result
    }
}

impl Default for FileDialog {
    fn default() -> Self {
        Self::new()
    }
}

/// Firmware loader dialog.
pub struct FirmwareLoaderDialog {
    /// File dialog.
    file_dialog: FileDialog,
    /// Selected firmware path.
    firmware_path: Option<String>,
}

impl FirmwareLoaderDialog {
    /// Create a new firmware loader dialog.
    pub fn new() -> Self {
        Self {
            file_dialog: FileDialog::new(),
            firmware_path: None,
        }
    }

    /// Open the dialog.
    pub fn open(&mut self) {
        self.file_dialog.open_open("Load Firmware", "elf,bin,uf2,hex");
    }

    /// Show the dialog.
    pub fn show(&mut self, ctx: &Context) -> Option<String> {
        if let Some(result) = self.file_dialog.show(ctx) {
            if result.confirmed {
                self.firmware_path = Some(result.path.clone());
                return Some(result.path);
            }
        }
        None
    }
}

impl Default for FirmwareLoaderDialog {
    fn default() -> Self {
        Self::new()
    }
}

/// Save state dialog.
pub struct SaveStateDialog {
    /// File dialog.
    file_dialog: FileDialog,
}

impl SaveStateDialog {
    /// Create a new save state dialog.
    pub fn new() -> Self {
        Self {
            file_dialog: FileDialog::new(),
        }
    }

    /// Open the dialog.
    pub fn open(&mut self) {
        self.file_dialog.open_save("Save State", "rpsim");
    }

    /// Show the dialog.
    pub fn show(&mut self, ctx: &Context) -> Option<String> {
        if let Some(result) = self.file_dialog.show(ctx) {
            if result.confirmed {
                return Some(result.path);
            }
        }
        None
    }
}

impl Default for SaveStateDialog {
    fn default() -> Self {
        Self::new()
    }
}

/// Project open dialog.
pub struct ProjectOpenDialog {
    /// File dialog.
    file_dialog: FileDialog,
}

impl ProjectOpenDialog {
    /// Create a new project open dialog.
    pub fn new() -> Self {
        Self {
            file_dialog: FileDialog::new(),
        }
    }

    /// Open the dialog.
    pub fn open(&mut self) {
        self.file_dialog.open_open("Open Project", "toml");
    }

    /// Show the dialog.
    pub fn show(&mut self, ctx: &Context) -> Option<String> {
        if let Some(result) = self.file_dialog.show(ctx) {
            if result.confirmed {
                return Some(result.path);
            }
        }
        None
    }
}

impl Default for ProjectOpenDialog {
    fn default() -> Self {
        Self::new()
    }
}