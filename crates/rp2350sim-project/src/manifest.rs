//! Project manifest.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Project manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Project name.
    pub name: String,
    /// Project version.
    pub version: String,
    /// Firmware path.
    pub firmware: Option<PathBuf>,
    /// Configuration path.
    pub config: Option<PathBuf>,
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            name: "Untitled Project".to_string(),
            version: "1.0.0".to_string(),
            firmware: None,
            config: None,
        }
    }
}

impl Manifest {
    /// Create a new manifest.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Self::default()
        }
    }

    /// Load manifest from file.
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let manifest: Manifest = toml::from_str(&content)?;
        Ok(manifest)
    }

    /// Save manifest to file.
    pub fn save(&self, path: &PathBuf) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}