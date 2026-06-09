//! Project management.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
    pub firmware: Option<String>,
}

impl Project {
    pub fn new(name: impl Into<String>, path: PathBuf) -> Self {
        Self {
            name: name.into(),
            path,
            firmware: None,
        }
    }
}