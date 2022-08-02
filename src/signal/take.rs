//! Take `n` samples of a signal.

use super::Signal;

pub trait SignalTake: Signal {
    /// Converts the `Signal` into an `Iterator` that will yield the given
    /// number frames before returning `None`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sirena::signal::{self, Signal, SignalTake};
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

impl<T> SignalTake for T where T: Signal {}

/// An iterator that yields `n` number of frames from the inner `signal`.
#[derive(Clone)]
pub struct Take<S>
where
    S: Signal,
{
    pub(super) signal: S,
    pub(super) n: usize,
}

impl<S> Iterator for Take<S>
where
    S: Signal,
{
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        if self.n == 0 {
            return None;
        }
        self.n -= 1;
        Some(self.signal.next())
    }
}
