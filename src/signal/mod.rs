//! Use the Signal trait to abstract signal as infinite-iterator-like objects.
//!
//! This is based on [dasp-signal](https://github.com/RustAudio/dasp) except
//! this implementation supports `#[no_std]` on stable and is concerned only
//! about mono f32 frames.

mod clip_amp;
mod from_iterator;
mod signal_trait;
mod sine;
mod take;

pub use signal_trait::{from_iter, Signal};
pub use sine::sine;

pub const EQUILIBRIUM: f32 = 0.0;
