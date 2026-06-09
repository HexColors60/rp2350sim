//! Memory layout.

mod generic;
mod pico2w;
mod rp2350;

pub use generic::GenericLayout;
pub use pico2w::Pico2WLayout;
pub use rp2350::Rp2350Layout;