//! GUI abstraction layer for RP2350 Simulator.
//!
//! This module provides a common interface for different GUI backends:
//! - `macroquad`: Default macroquad + egui backend
//! - `bevy`: Bevy game engine with egui integration
//! - `winapi`: Native Windows API with wgpu/egui

use crate::app::App;
use crate::config::Config;

/// GUI backend trait.
pub trait GuiBackend: Sized {
    /// Initialize the GUI backend.
    fn init(config: &Config) -> anyhow::Result<Self>;

    /// Run the main GUI loop.
    fn run(&mut self, app: &mut App) -> anyhow::Result<()>;

    /// Get the backend name.
    fn name() -> &'static str;
}

/// GUI event type.
#[derive(Debug, Clone)]
pub enum GuiEvent {
    /// Window resize event.
    Resize { width: u32, height: u32 },
    /// Window close requested.
    CloseRequested,
    /// Mouse moved.
    MouseMoved { x: f32, y: f32 },
    /// Mouse button pressed.
    MousePressed { button: MouseButton, x: f32, y: f32 },
    /// Mouse button released.
    MouseReleased { button: MouseButton, x: f32, y: f32 },
    /// Mouse wheel scrolled.
    MouseWheel { delta: f32 },
    /// Key pressed.
    KeyPressed { key: Key },
    /// Key released.
    KeyReleased { key: Key },
    /// Text input.
    TextInput { char: char },
}

/// Mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Key code (abstracted).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    // Numbers
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    // Special keys
    Space, Enter, Tab, Escape, Backspace, Delete, Insert,
    Home, End, PageUp, PageDown,
    // Arrow keys
    ArrowUp, ArrowDown, ArrowLeft, ArrowRight,
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    // Modifiers
    Shift, Control, Alt, Super,
    // Other
    Unknown,
}

/// GUI configuration.
#[derive(Debug, Clone)]
pub struct GuiConfig {
    pub window_title: String,
    pub window_width: u32,
    pub window_height: u32,
    pub vsync: bool,
    pub resizable: bool,
    pub high_dpi: bool,
}

impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            window_title: "RP2350 Simulator".to_string(),
            window_width: 1280,
            window_height: 800,
            vsync: true,
            resizable: true,
            high_dpi: true,
        }
    }
}

impl From<&Config> for GuiConfig {
    fn from(config: &Config) -> Self {
        Self {
            window_title: "RP2350 Simulator".to_string(),
            window_width: config.window_width,
            window_height: config.window_height,
            vsync: config.vsync,
            resizable: true,
            high_dpi: true,
        }
    }
}

// Conditional module imports based on feature flags
#[cfg(feature = "gui-macroquad")]
pub mod macroquad_gui;

#[cfg(feature = "gui-bevy")]
pub mod bevy_gui;

#[cfg(feature = "gui-winapi")]
pub mod winapi_gui;

/// Create the appropriate GUI backend based on compile-time features.
#[cfg(feature = "gui-macroquad")]
pub fn create_backend(config: &Config) -> anyhow::Result<impl GuiBackend> {
    macroquad_gui::MacroquadBackend::init(config)
}

#[cfg(all(feature = "gui-bevy", not(feature = "gui-macroquad")))]
pub fn create_backend(config: &Config) -> anyhow::Result<impl GuiBackend> {
    bevy_gui::BevyBackend::init(config)
}

#[cfg(all(feature = "gui-winapi", not(any(feature = "gui-macroquad", feature = "gui-bevy"))))]
pub fn create_backend(config: &Config) -> anyhow::Result<impl GuiBackend> {
    winapi_gui::WinapiBackend::init(config)
}

/// Get the current GUI backend name.
pub fn current_backend_name() -> &'static str {
    #[cfg(feature = "gui-macroquad")]
    { "macroquad" }
    
    #[cfg(all(feature = "gui-bevy", not(feature = "gui-macroquad")))]
    { "bevy" }
    
    #[cfg(all(feature = "gui-winapi", not(any(feature = "gui-macroquad", feature = "gui-bevy"))))]
    { "winapi" }
    
    #[cfg(not(any(feature = "gui-macroquad", feature = "gui-bevy", feature = "gui-winapi")))]
    { "headless" }
}