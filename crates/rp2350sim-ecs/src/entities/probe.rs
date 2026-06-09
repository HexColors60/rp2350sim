//! Logic probe entity creation.

use hecs::World;

use crate::components::{GpioBind, Name, Position, Renderable, Selectable};

/// Create a logic probe entity.
pub fn create_probe(world: &mut World, pin: u8, name: &str) -> hecs::Entity {
    world.spawn((
        Name::new(name),
        Position::default(),
        Renderable::probe(),
        Selectable::new(),
        GpioBind::input(pin),
    ))
}