mod consts;
pub mod double_wavetable_oscillator;
pub mod simple_wavetable_oscillator;
pub mod waveshapes;
pub mod wavetable;

pub use double_wavetable_oscillator::DoubleWavetableOscillator;
pub use simple_wavetable_oscillator::SimpleWavetableOscillator;
pub use waveshapes::{digital_saw, saw, sine, triangle};
pub use wavetable::Wavetable;
