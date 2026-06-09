//! RP2350 Project Management

pub mod assets;
pub mod firmware;
pub mod manifest;
pub mod paths;
pub mod project;
pub mod recent;
pub mod session;
pub mod validation;

pub use assets::AssetManager;
pub use firmware::{Firmware, FirmwareFormat};
pub use manifest::Manifest;
pub use paths::ProjectPaths;
pub use project::Project;
pub use recent::RecentProjects;
pub use session::Session;
pub use validation::{ValidationResult, validate_project_path, validate_firmware};