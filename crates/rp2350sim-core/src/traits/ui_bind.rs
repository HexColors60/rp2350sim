//! UI binding trait.

use crate::DeviceId;

/// UI bindable trait for devices that can be visualized.
pub trait UiBindable: Send + Sync {
    /// Get the device ID for UI purposes.
    fn ui_id(&self) -> DeviceId;

    /// Get the display name for the UI.
    fn ui_name(&self) -> &str;

    /// Get the UI category.
    fn ui_category(&self) -> UiCategory;
}

/// UI category for organizing devices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiCategory {
    Cpu,
    Memory,
    Gpio,
    Communication,
    Analog,
    Timing,
    Usb,
    Pio,
    Wireless,
    Debug,
    Other,
}

/// UI panel provider trait.
pub trait UiPanelProvider: Send + Sync {
    /// Get the panel name.
    fn panel_name(&self) -> &str;

    /// Check if the panel is visible.
    fn is_visible(&self) -> bool;

    /// Set the panel visibility.
    fn set_visible(&mut self, visible: bool);
}