//! Formatting utilities.

use std::fmt;

/// Format a byte as a hex string.
pub fn hex_byte(byte: u8) -> String {
    format!("{:02X}", byte)
}

/// Format a 16-bit value as a hex string.
pub fn hex_half(value: u16) -> String {
    format!("{:04X}", value)
}

/// Format a 32-bit value as a hex string.
pub fn hex_word(value: u32) -> String {
    format!("{:08X}", value)
}

/// Format a 64-bit value as a hex string.
pub fn hex_dword(value: u64) -> String {
    format!("{:016X}", value)
}

/// Format bytes as a hex string.
pub fn hex_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ")
}

/// Format a size in human-readable form.
pub fn format_size(size: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;
    const GB: usize = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

/// Format a frequency in human-readable form.
pub fn format_freq(hz: u64) -> String {
    const KHZ: u64 = 1000;
    const MHZ: u64 = KHZ * 1000;
    const GHZ: u64 = MHZ * 1000;

    if hz >= GHZ {
        format!("{:.2} GHz", hz as f64 / GHZ as f64)
    } else if hz >= MHZ {
        format!("{:.2} MHz", hz as f64 / MHZ as f64)
    } else if hz >= KHZ {
        format!("{:.2} kHz", hz as f64 / KHZ as f64)
    } else {
        format!("{} Hz", hz)
    }
}

/// Wrapper for formatting binary data.
pub struct BinaryDisplay<'a>(pub &'a [u8]);

impl<'a> fmt::Display for BinaryDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, byte) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{:02X}", byte)?;
        }
        Ok(())
    }
}

/// Wrapper for formatting a duration.
pub struct DurationDisplay(pub std::time::Duration);

impl fmt::Display for DurationDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let secs = self.0.as_secs();
        let millis = self.0.subsec_millis();
        let micros = self.0.subsec_micros() % 1000;
        let nanos = self.0.subsec_nanos() % 1000;

        if secs > 0 {
            write!(f, "{}.{:03}s", secs, millis)
        } else if millis > 0 {
            write!(f, "{}.{:03}ms", millis, micros)
        } else if micros > 0 {
            write!(f, "{}.{:03}µs", micros, nanos)
        } else {
            write!(f, "{}ns", nanos)
        }
    }
}