//! ECS world.

use hecs::World as HecsWorld;

/// ECS world wrapper.
pub struct World {
    world: HecsWorld,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            world: HecsWorld::new(),
        }
    }

    pub fn spawn(&mut self) -> hecs::Entity {
        self.world.spawn(())
    }

    pub fn world(&self) -> &HecsWorld {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut HecsWorld {
        &mut self.world
    }
}