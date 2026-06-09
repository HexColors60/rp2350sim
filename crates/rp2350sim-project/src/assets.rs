//! Asset management.

use std::path::PathBuf;

/// Asset manager.
#[derive(Debug)]
pub struct AssetManager {
    /// Asset search paths.
    search_paths: Vec<PathBuf>,
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetManager {
    /// Create a new asset manager.
    pub fn new() -> Self {
        Self {
            search_paths: Vec::new(),
        }
    }

    /// Add a search path.
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Find an asset by name.
    pub fn find(&self, name: &str) -> Option<PathBuf> {
        for path in &self.search_paths {
            let full_path = path.join(name);
            if full_path.exists() {
                return Some(full_path);
            }
        }
        None
    }
}