//! Memory permissions.

use serde::{Deserialize, Serialize};

/// Memory access permissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Permissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl Permissions {
    pub const fn none() -> Self {
        Self {
            read: false,
            write: false,
            execute: false,
        }
    }

    pub const fn read_only() -> Self {
        Self {
            read: true,
            write: false,
            execute: false,
        }
    }

    pub const fn write_only() -> Self {
        Self {
            read: false,
            write: true,
            execute: false,
        }
    }

    pub const fn read_write() -> Self {
        Self {
            read: true,
            write: true,
            execute: false,
        }
    }

    pub const fn read_execute() -> Self {
        Self {
            read: true,
            write: false,
            execute: true,
        }
    }

    pub const fn all() -> Self {
        Self {
            read: true,
            write: true,
            execute: true,
        }
    }

    pub const fn can_read(&self) -> bool {
        self.read
    }

    pub const fn can_write(&self) -> bool {
        self.write
    }

    pub const fn can_execute(&self) -> bool {
        self.execute
    }
}