#![allow(dead_code)]

//! GPU resources.

use std::collections::HashMap;

/// Resource ID type.
pub type ResourceId = u64;

/// GPU resource manager.
#[derive(Debug, Default)]
pub struct ResourceManager {
    /// Next resource ID.
    next_id: ResourceId,
    /// Active resources.
    resources: HashMap<ResourceId, String>,
}

impl ResourceManager {
    /// Create a new resource manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate a new resource ID.
    pub fn allocate_id(&mut self) -> ResourceId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Register a resource.
    pub fn register(&mut self, name: &str) -> ResourceId {
        let id = self.allocate_id();
        self.resources.insert(id, name.to_string());
        id
    }

    /// Unregister a resource.
    pub fn unregister(&mut self, id: ResourceId) {
        self.resources.remove(&id);
    }

    /// Get a resource name.
    pub fn get_name(&self, id: ResourceId) -> Option<&str> {
        self.resources.get(&id).map(|s| s.as_str())
    }
}