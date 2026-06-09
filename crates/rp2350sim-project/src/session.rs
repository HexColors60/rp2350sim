//! Session management.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Session state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Project path.
    pub project_path: Option<PathBuf>,
    /// Last opened firmware.
    pub last_firmware: Option<PathBuf>,
    /// Window state.
    pub window: WindowState,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            project_path: None,
            last_firmware: None,
            window: WindowState::default(),
        }
    }
}

/// Window state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    /// Window width.
    pub width: u32,
    /// Window height.
    pub height: u32,
    /// Window X position.
    pub x: i32,
    /// Window Y position.
    pub y: i32,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            x: 100,
            y: 100,
        }
    }
}

impl Session {
    /// Create a new session.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load session from file.
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let session: Session = ron::from_str(&content)?;
        Ok(session)
    }

    /// Save session to file.
    pub fn save(&self, path: &PathBuf) -> anyhow::Result<()> {
        let content = ron::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}