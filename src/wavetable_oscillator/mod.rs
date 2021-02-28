pub mod circular_wavetable_oscillator;
mod consts;
pub mod oscillator;
pub mod simple_wavetable_oscillator;
pub mod waveshapes;
pub mod wavetable;
pub mod xy0_wavetable_oscillator;

#[cfg(test)]
mod tests;

pub use circular_wavetable_oscillator::CircularWavetableOscillator;
pub use oscillator::Oscillator;
pub use simple_wavetable_oscillator::SimpleWavetableOscillator;
pub use waveshapes::{digital_saw, saw, sine, triangle};
pub use wavetable::Wavetable;
pub use xy0_wavetable_oscillator::XY0WavetableOscillator;
