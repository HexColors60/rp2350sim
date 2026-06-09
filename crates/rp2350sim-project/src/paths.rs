//! Path utilities.

use std::path::PathBuf;

/// Project paths.
#[derive(Debug, Clone)]
pub struct ProjectPaths {
    /// Project root directory.
    pub root: PathBuf,
    /// Configuration directory.
    pub config_dir: PathBuf,
    /// Data directory.
    pub data_dir: PathBuf,
}

impl ProjectPaths {
    /// Create project paths from a root directory.
    pub fn new(root: PathBuf) -> Self {
        Self {
            config_dir: root.join("config"),
            data_dir: root.join("data"),
            root,
        }
    }

    /// Get the config file path.
    pub fn config_file(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }

    /// Get the session file path.
    pub fn session_file(&self) -> PathBuf {
        self.data_dir.join("session.ron")
    }
}