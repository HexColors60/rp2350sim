//! ADC signal sources.

mod constant;
mod sine;
mod square;
mod ramp;

pub use constant::ConstantSource;
pub use sine::SineSource;
pub use square::SquareSource;
pub use ramp::RampSource;

/// Trait for ADC signal sources.
pub trait AdcSource: Send + Sync {
    /// Sample the signal at a given time.
    fn sample(&self, time: f64) -> f32;
    
    /// Reset the source.
    fn reset(&mut self);
}