//! Save state trait.

use crate::Result;

/// Save state trait for serializable components.
pub trait SaveState: Send + Sync {
    /// Save the state to bytes.
    fn save(&self) -> Result<Vec<u8>>;

    /// Load the state from bytes.
    fn load(&mut self, data: &[u8]) -> Result<()>;
}

/// Checkpointable trait.
pub trait Checkpointable: SaveState {
    /// Get the checkpoint name.
    fn checkpoint_name(&self) -> &str;

    /// Get the checkpoint timestamp.
    fn checkpoint_timestamp(&self) -> u64;
}