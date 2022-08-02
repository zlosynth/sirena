//! Generate sine wave signal.

use super::Signal;

/// Produces a signal that yields a sine wave oscillating at the given hz.
///
/// # Example
///
/// ```rust
/// # #[macro_use]
/// # extern crate approx;
/// # fn main() {
/// // Generates a sine wave signal at 1hz to be sampled 4 times per second.
/// use sirena::signal::{self, Signal};
/// let mut signal = signal::sine(4.0, 1.0);
/// assert_eq!(signal.next(), 0.0);
/// assert_eq!(signal.next(), 1.0);
/// signal.next();
/// assert_eq!(signal.next(), -1.0);
/// # }
/// ```
pub fn sine(fs: f32, hz: f32) -> Sine {
    Sine {
        phase: 0.0,
        step: hz / fs,
    }
}

/// A sine wave signal generator.
#[derive(Clone)]
pub struct Sine {
    phase: f32,
    step: f32,
}

impl Signal for Sine {
    #[inline]
    fn next(&mut self) -> f32 {
        const PI_2: f32 = core::f32::consts::PI * 2.0;
        let sample = libm::sinf(PI_2 * self.phase);
        self.phase += self.step;
        sample
    }
}
