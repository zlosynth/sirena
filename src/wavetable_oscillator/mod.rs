pub mod oscillator;
pub mod waveshapes;
pub mod wavetable;

pub use oscillator::{DoubleWavetableOscillator, WavetableOscillator};
pub use waveshapes::{saw, sine, triangle};
pub use wavetable::Wavetable;
