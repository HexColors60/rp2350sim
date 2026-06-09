//! Name component.

use serde::{Deserialize, Serialize};

/// Entity name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Name(pub String);

impl Name {
    /// Create a new name.
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}