//! Use the Signal trait to abstract signal as infinite-iterator-like objects.
//!
//! This is based on [dasp-signal](https://github.com/RustAudio/dasp) except
//! this implementation supports `#[no_std]` on stable and is concerned only
//! about mono f32 frames.

mod clip_amp;
mod from_iterator;
mod take;

use clip_amp::ClipAmp;
use from_iterator::FromIterator;
use take::Take;

pub const EQUILIBRIUM: f32 = 0.0;

/// Types that yield values of a PCM signal.
pub trait Signal {
    /// Read the next sample of given signal.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate approx;
    /// # fn main() {
    /// use sirena::signal::{self, Signal};
    /// let frames = [0.1, 0.2];
    /// let mut signal = signal::from_iter(frames);
    /// assert_relative_eq!(signal.next(), 0.1);
    /// assert_relative_eq!(signal.next(), 0.2);
    /// # }
    /// ```
    fn next(&mut self) -> f32;

    /// Borrows a Signal rather than consuming it.
    ///
    /// This is useful to allow applying signal adaptors while still retaining ownership of the
    /// original signal.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate approx;
    /// # fn main() {
    /// use sirena::signal::{self, Signal};
    /// let frames = [0.1, 0.2, 0.3, 0.4, 0.5];
    /// let mut signal = signal::from_iter(frames);
    /// assert_relative_eq!(signal.next(), 0.1);
    /// let mut sub_signal: Vec<_> = signal.by_ref().take(2).collect();
    /// assert_relative_eq!(sub_signal[0], 0.2);
    /// assert_relative_eq!(sub_signal[1], 0.3);
    /// assert_relative_eq!(signal.next(), 0.4);
    /// assert_relative_eq!(signal.next(), 0.5);
    /// # }
    /// ```
    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }

    /// Clips the amplitude of the signal to the given threshold amplitude.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sirena::signal::{self, Signal};
    /// let frames = [0.5, 2.0, -2.0];
    /// let mut signal = signal::from_iter(frames).clip_amp(1.0);
    /// assert_eq!(signal.next(), 0.5);
    /// assert_eq!(signal.next(), 1.0);
    /// assert_eq!(signal.next(), -1.0);
    /// ```
    fn clip_amp(self, threshold: f32) -> ClipAmp<Self>
    where
        Self: Sized,
    {
        ClipAmp {
            signal: self,
            threshold,
        }
    }

    /// Converts the `Signal` into an `Iterator` that will yield the given
    /// number frames before returning `None`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sirena::signal::{self, Signal};
    /// let frames = [0.1, 0.2, 0.3];
    /// let mut signal = signal::from_iter(frames).take(2);
    /// assert_eq!(signal.next(), Some(0.1));
    /// assert_eq!(signal.next(), Some(0.2));
    /// assert_eq!(signal.next(), None);
    /// ```
    fn take(self, n: usize) -> Take<Self>
    where
        Self: Sized,
    {
        Take { signal: self, n }
    }
}

impl<'a, S> Signal for &'a mut S
where
    S: Signal + ?Sized,
{
    #[inline]
    fn next(&mut self) -> f32 {
        (**self).next()
    }
}

/// Create a new `Signal` from the given `Frame`-yielding `Iterator`.
pub fn from_iter<I>(frames: I) -> FromIterator<I::IntoIter>
where
    I: IntoIterator<Item = f32>,
{
    let mut iter = frames.into_iter();
    let next = iter.next();
    FromIterator { iter, next }
}
