//! Update system.

use hecs::World;
use crate::components::{Position, SignalSource, Renderable};

/// Update system for updating entity state.
pub struct UpdateSystem {
    /// Total elapsed time.
    elapsed_time: f32,
}

impl Default for UpdateSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl UpdateSystem {
    /// Create a new update system.
    pub fn new() -> Self {
        Self { elapsed_time: 0.0 }
    }

    /// Update all entities.
    pub fn update(&mut self, world: &mut World, delta_time: f32) {
        self.elapsed_time += delta_time;
        
        // Update signal sources
        for (_, (signal,)) in world.query_mut::<(&mut SignalSource,)>() {
            signal.update(delta_time);
        }
        
        // Update positions based on animations
        for (_, (pos, renderable)) in world.query_mut::<(&mut Position, &Renderable)>() {
            if renderable.animated {
                // Apply any position animations
                let _ = pos; // Placeholder for animation logic
            }
        }
    }

    /// Get elapsed time.
    pub fn elapsed_time(&self) -> f32 {
        self.elapsed_time
    }
}