//! Terminal binding component.

use serde::{Deserialize, Serialize};

/// Terminal binding component for connecting entities to UART terminals.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalBind {
    /// UART instance number (0 or 1).
    pub uart: u8,
    /// Whether to echo received characters.
    pub echo: bool,
    /// Whether to append newlines.
    pub append_newline: bool,
}

impl TerminalBind {
    /// Create a new terminal binding.
    pub fn new(uart: u8) -> Self {
        Self {
            uart,
            echo: true,
            append_newline: true,
        }
    }

    /// Create a binding for UART0.
    pub fn uart0() -> Self {
        Self::new(0)
    }

    /// Create a binding for UART1.
    pub fn uart1() -> Self {
        Self::new(1)
    }
}