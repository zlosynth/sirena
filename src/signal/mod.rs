//! Use the Signal trait to abstract signal as infinite-iterator-like objects.
//!
//! This is based on [dasp-signal](https://github.com/RustAudio/dasp) except
//! this implementation supports `#[no_std]` on stable and is concerned only
//! about mono f32 frames.

mod clip_amp;
mod constant;
mod from_iterator;
mod mul_amp;
mod signal_trait;
mod sine;
mod take;

pub use clip_amp::SignalClipAmp;
pub use constant::constant;
pub use from_iterator::from_iter;
pub use mul_amp::SignalMulAmp;
pub use signal_trait::Signal;
pub use sine::sine;
pub use take::SignalTake;

pub const EQUILIBRIUM: f32 = 0.0;
