//! LED entity creation.

use hecs::World;

use crate::components::{GpioBind, Name, Position, Renderable, Selectable};

/// Create an LED entity.
pub fn create_led(world: &mut World, pin: u8, name: &str) -> hecs::Entity {
    world.spawn((
        Name::new(name),
        Position::default(),
        Renderable::led(),
        Selectable::new(),
        GpioBind::output(pin),
    ))
}