//! Reset trait.

/// Resettable trait for devices that can be reset.
pub trait Resettable {
    /// Reset the device to its initial state.
    fn reset(&mut self);
}

/// Reset controller trait.
pub trait ResetController: Send + Sync {
    /// Request a reset.
    fn request_reset(&mut self);

    /// Check if a reset is pending.
    fn is_reset_pending(&self) -> bool;

    /// Clear the reset request.
    fn clear_reset(&mut self);
}