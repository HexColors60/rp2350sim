//! Display entity creation.

use hecs::World;

use crate::components::{Name, Position, Renderable, Selectable};

/// Create a display entity.
pub fn create_display(world: &mut World, name: &str, width: u32, height: u32) -> hecs::Entity {
    world.spawn((
        Name::new(name),
        Position::default(),
        Renderable::display_with_size(width, height),
        Selectable::new(),
    ))
}