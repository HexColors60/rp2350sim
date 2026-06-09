//! Test fixtures

use std::path::PathBuf;

/// Get the path to the golden test directory
pub fn golden_dir() -> PathBuf {
    let root = super::harness::find_project_root();
    root.join("tests").join("golden")
}

/// Get the path to a golden firmware file
pub fn golden_firmware(name: &str) -> PathBuf {
    golden_dir().join("firmware").join(name)
}

/// Get the path to a golden trace file
pub fn golden_trace(name: &str) -> PathBuf {
    golden_dir().join("traces").join(name)
}

/// Get the path to a golden expected output file
pub fn golden_expected(name: &str) -> PathBuf {
    golden_dir().join("expected").join(name)
}

/// Simple blink test firmware (assembled)
/// This is a minimal ARM Thumb program that toggles GPIO 25
pub fn blink_firmware() -> Vec<u8> {
    // Minimal blink program - just a loop
    // In a real scenario, this would be actual compiled firmware
    vec![
        // MOVS R0, #25       ; GPIO pin number
        0x19, 0x20,
        // MOVS R1, #1        ; Value to set
        0x01, 0x21,
        // Loop:
        // B Loop             ; Infinite loop
        0xFE, 0xE7,
    ]
}

/// Simple UART echo test firmware
pub fn uart_echo_firmware() -> Vec<u8> {
    // Minimal UART echo program
    vec![
        // Loop:
        // B Loop
        0xFE, 0xE7,
    ]
}

/// Create a simple test memory image
pub fn create_test_memory() -> Vec<u8> {
    vec![0u8; 1024]
}