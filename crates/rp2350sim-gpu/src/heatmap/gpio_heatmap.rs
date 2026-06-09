#![allow(dead_code)]

//! GPIO heatmap visualization.

/// GPIO heatmap for visualizing pin activity.
#[derive(Debug)]
pub struct GpioHeatmap {
    /// Number of pins.
    pin_count: usize,
    /// Activity counts per pin.
    activity: Vec<u32>,
}

impl GpioHeatmap {
    /// Create a new GPIO heatmap.
    pub fn new(pin_count: usize) -> Self {
        Self {
            pin_count,
            activity: vec![0; pin_count],
        }
    }

    /// Record activity on a pin.
    pub fn record_activity(&mut self, pin: usize) {
        if pin < self.pin_count {
            self.activity[pin] = self.activity[pin].saturating_add(1);
        }
    }

    /// Get the activity count for a pin.
    pub fn get_activity(&self, pin: usize) -> u32 {
        self.activity.get(pin).copied().unwrap_or(0)
    }

    /// Clear all activity counts.
    pub fn clear(&mut self) {
        for count in &mut self.activity {
            *count = 0;
        }
    }
}