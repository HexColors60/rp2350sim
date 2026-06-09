//! Application configuration.

use rp2350sim_core::SimulatorConfig;
use serde::{Deserialize, Serialize};

/// Application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Simulator configuration
    #[serde(flatten)]
    pub sim: SimulatorConfig,

    /// Run in headless mode
    pub headless: bool,

    /// Project directory
    #[serde(default)]
    pub project: Option<String>,

    /// Window width
    pub window_width: u32,

    /// Window height
    pub window_height: u32,

    /// Enable VSync
    pub vsync: bool,

    /// Target FPS
    pub target_fps: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sim: SimulatorConfig::default(),
            headless: false,
            project: None,
            window_width: 1280,
            window_height: 800,
            vsync: true,
            target_fps: 60,
        }
    }
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &str) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}