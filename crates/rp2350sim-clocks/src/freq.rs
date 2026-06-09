//! Frequency utilities.

/// Format frequency as human-readable string.
pub fn format_freq(hz: u64) -> String {
    if hz >= 1_000_000_000 {
        format!("{:.2} GHz", hz as f64 / 1e9)
    } else if hz >= 1_000_000 {
        format!("{:.2} MHz", hz as f64 / 1e6)
    } else if hz >= 1_000 {
        format!("{:.2} kHz", hz as f64 / 1e3)
    } else {
        format!("{} Hz", hz)
    }
}