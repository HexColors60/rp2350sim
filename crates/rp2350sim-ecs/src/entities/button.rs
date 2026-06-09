//! Button entity creation.

use hecs::World;

use crate::components::{GpioBind, Name, Position, Renderable, Selectable};

/// Create a button entity.
pub fn create_button(world: &mut World, pin: u8, name: &str) -> hecs::Entity {
    world.spawn((
        Name::new(name),
        Position::default(),
        Renderable::button(),
        Selectable::new(),
        GpioBind::input(pin),
    ))
}