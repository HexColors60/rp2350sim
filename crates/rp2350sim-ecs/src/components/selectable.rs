//! Selectable component.

use serde::{Deserialize, Serialize};

/// Selectable component for entities that can be selected in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Selectable {
    /// Whether the entity is currently selected.
    pub selected: bool,
    /// Whether the entity can be selected.
    pub enabled: bool,
    /// Selection radius for click detection.
    pub radius: f32,
}

impl Default for Selectable {
    fn default() -> Self {
        Self {
            selected: false,
            enabled: true,
            radius: 20.0,
        }
    }
}

impl Selectable {
    /// Create a new selectable component.
    pub fn new() -> Self {
        Self::default()
    }

    /// Select the entity.
    pub fn select(&mut self) {
        if self.enabled {
            self.selected = true;
        }
    }

    /// Deselect the entity.
    pub fn deselect(&mut self) {
        self.selected = false;
    }

    /// Toggle selection.
    pub fn toggle(&mut self) {
        if self.enabled {
            self.selected = !self.selected;
        }
    }
}