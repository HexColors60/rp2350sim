//! ECS Systems.

mod input;
mod update;
mod render;
mod sync_soc;

pub use input::InputSystem;
pub use update::UpdateSystem;
pub use render::RenderSystem;
pub use sync_soc::SyncSocSystem;