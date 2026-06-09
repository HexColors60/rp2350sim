//! UART diff utilities.

use crate::DiffResult;

/// UART state for diff comparison.
#[derive(Debug, Clone)]
pub struct UartState {
    /// TX buffer.
    pub tx_buffer: Vec<u8>,
    /// RX buffer.
    pub rx_buffer: Vec<u8>,
    /// Baud rate.
    pub baud_rate: u32,
}

impl Default for UartState {
    fn default() -> Self {
        Self::new()
    }
}

impl UartState {
    /// Create a new UART state.
    pub fn new() -> Self {
        Self {
            tx_buffer: Vec::new(),
            rx_buffer: Vec::new(),
            baud_rate: 115200,
        }
    }
}

/// Compare UART states.
pub fn diff_uart(a: &UartState, b: &UartState) -> Vec<DiffResult> {
    let mut results = Vec::new();
    
    // Compare TX buffers
    if a.tx_buffer != b.tx_buffer {
        results.push(DiffResult::DataMismatch {
            name: "TX buffer".to_string(),
            expected: a.tx_buffer.clone(),
            actual: b.tx_buffer.clone(),
        });
    }
    
    // Compare RX buffers
    if a.rx_buffer != b.rx_buffer {
        results.push(DiffResult::DataMismatch {
            name: "RX buffer".to_string(),
            expected: a.rx_buffer.clone(),
            actual: b.rx_buffer.clone(),
        });
    }
    
    // Compare baud rates
    if a.baud_rate != b.baud_rate {
        results.push(DiffResult::Mismatch {
            expected: a.baud_rate as u64,
            actual: b.baud_rate as u64,
        });
    }
    
    results
}