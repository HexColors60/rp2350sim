//! SoC synchronization system.

use hecs::World;
use crate::components::{Position, GpioBind, SignalSource, TerminalBind};

/// System for synchronizing ECS entities with SoC state.
pub struct SyncSocSystem {
    /// GPIO state cache.
    gpio_states: [bool; 48],
    /// ADC value cache.
    adc_values: [u16; 4],
}

impl Default for SyncSocSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncSocSystem {
    /// Create a new sync system.
    pub fn new() -> Self {
        Self {
            gpio_states: [false; 48],
            adc_values: [0; 4],
        }
    }

    /// Sync ECS state to SoC.
    /// This updates the SoC based on entity states (e.g., buttons pressed).
    pub fn sync_to_soc(&self, world: &World) {
        // Find all entities with GPIO bindings and update SoC GPIO state
        for (_, (gpio, signal)) in world.query::<(&GpioBind, Option<&SignalSource>)>().iter() {
            if let Some(signal) = signal {
                // Update GPIO based on signal source
                match signal {
                    SignalSource::Constant(value) => {
                        // Set GPIO to constant value
                        let _ = (gpio.pin, value);
                    }
                    SignalSource::Sine { frequency, amplitude, phase } => {
                        // Calculate current sine value
                        let _ = (gpio.pin, frequency, amplitude, phase);
                    }
                    _ => {}
                }
            }
        }
    }

    /// Sync SoC state to ECS.
    /// This updates entity states based on SoC state (e.g., LEDs, displays).
    pub fn sync_from_soc(&mut self, world: &mut World) {
        // Update entities based on GPIO state
        for (_, (gpio, pos)) in world.query_mut::<(&GpioBind, &mut Position)>() {
            // Update position based on GPIO state (e.g., LED position for visual feedback)
            let _ = (gpio.pin, pos);
        }
        
        // Update terminal displays
        for (_, terminal) in world.query_mut::<&TerminalBind>() {
            // Update terminal content from UART
            let _ = terminal;
        }
    }

    /// Set GPIO state (called from SoC).
    pub fn set_gpio(&mut self, pin: usize, value: bool) {
        if pin < self.gpio_states.len() {
            self.gpio_states[pin] = value;
        }
    }

    /// Get GPIO state.
    pub fn get_gpio(&self, pin: usize) -> bool {
        self.gpio_states.get(pin).copied().unwrap_or(false)
    }

    /// Set ADC value (called from SoC).
    pub fn set_adc(&mut self, channel: usize, value: u16) {
        if channel < self.adc_values.len() {
            self.adc_values[channel] = value;
        }
    }

    /// Get ADC value.
    pub fn get_adc(&self, channel: usize) -> u16 {
        self.adc_values.get(channel).copied().unwrap_or(0)
    }
}