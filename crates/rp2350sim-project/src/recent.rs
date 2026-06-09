//! Recent projects management.

use std::path::PathBuf;

/// Recent projects list.
#[derive(Debug, Default)]
pub struct RecentProjects {
    /// Recent project paths.
    projects: Vec<PathBuf>,
    /// Maximum number of recent projects.
    max_count: usize,
}

impl RecentProjects {
    /// Create a new recent projects list.
    pub fn new() -> Self {
        Self {
            projects: Vec::new(),
            max_count: 10,
        }
    }

    /// Add a project to the recent list.
    pub fn add(&mut self, path: PathBuf) {
        // Remove if already exists
        self.projects.retain(|p| p != &path);
        // Add to front
        self.projects.insert(0, path);
        // Trim to max count
        if self.projects.len() > self.max_count {
            self.projects.truncate(self.max_count);
        }
    }

    /// Get the recent projects.
    pub fn list(&self) -> &[PathBuf] {
        &self.projects
    }

    /// Clear the recent projects list.
    pub fn clear(&mut self) {
        self.projects.clear();
    }
}