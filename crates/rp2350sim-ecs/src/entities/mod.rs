//! Entity creation helpers.

mod led;
mod button;
mod probe;
mod display;

pub use led::create_led;
pub use button::create_button;
pub use probe::create_probe;
pub use display::create_display;

use hecs::World;

/// Create common entities in the world.
pub fn create_default_entities(world: &mut World) {
    // Create default LED on GPIO 25 (Pico onboard LED)
    create_led(world, 25, "Onboard LED");
}