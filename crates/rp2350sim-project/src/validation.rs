//! Project validation.

use std::path::PathBuf;

/// Validation result.
#[derive(Debug)]
pub struct ValidationResult {
    /// Whether validation passed.
    pub valid: bool,
    /// Validation errors.
    pub errors: Vec<String>,
    /// Validation warnings.
    pub warnings: Vec<String>,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

impl ValidationResult {
    /// Create a new validation result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an error.
    pub fn add_error(&mut self, error: String) {
        self.valid = false;
        self.errors.push(error);
    }

    /// Add a warning.
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// Validate a project path.
pub fn validate_project_path(path: &PathBuf) -> ValidationResult {
    let mut result = ValidationResult::new();

    if !path.exists() {
        result.add_error(format!("Path does not exist: {}", path.display()));
    }

    if !path.is_dir() {
        result.add_error(format!("Path is not a directory: {}", path.display()));
    }

    result
}

/// Validate a firmware file.
pub fn validate_firmware(path: &PathBuf) -> ValidationResult {
    let mut result = ValidationResult::new();

    if !path.exists() {
        result.add_error(format!("Firmware file does not exist: {}", path.display()));
    }

    if !path.is_file() {
        result.add_error(format!("Firmware path is not a file: {}", path.display()));
    }

    result
}