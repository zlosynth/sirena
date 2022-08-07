//! Generate static signal.

use super::Signal;

/// Produces a static signal that yields given value forever
///
/// # Example
///
/// ```rust
/// # #[macro_use]
/// # extern crate approx;
/// # fn main() {
/// use sirena::signal::{self, Signal};
/// let mut signal = signal::constant(2.0);
/// assert_eq!(signal.next(), 2.0);
/// assert_eq!(signal.next(), 2.0);
/// # }
/// ```
pub fn constant(value: f32) -> Constant {
    Constant { value }
}

/// A constant value generator.
#[derive(Clone)]
pub struct Constant {
    value: f32,
}

impl Signal for Constant {
    #[inline]
    fn next(&mut self) -> f32 {
        self.value
    }
}
