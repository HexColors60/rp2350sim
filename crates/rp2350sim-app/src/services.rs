#![allow(dead_code)]

//! Service management.

use std::collections::HashMap;

/// Service registry.
pub struct Services {
    services: HashMap<String, Box<dyn std::any::Any>>,
}

impl Default for Services {
    fn default() -> Self {
        Self::new()
    }
}

impl Services {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn register<S: 'static>(&mut self, name: &str, service: S) {
        self.services.insert(name.to_string(), Box::new(service));
    }

    pub fn get<S: 'static>(&self, name: &str) -> Option<&S> {
        self.services.get(name)?.downcast_ref::<S>()
    }
}