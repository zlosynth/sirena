//! Use the Signal trait to abstract signal as infinite-iterator-like objects.
//!
//! This is based on the concepts presented in
//! [dasp-signal](https://github.com/RustAudio/dasp) except this implementation
//! supports `#[no_std]` on stable.

const EQUILIBRIUM: f32 = 0.0;

/// Types that yield values of a PCM signal.
pub trait Signal {
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

/// Clips samples yielded by `signal` to the given threshold amplitude.
#[derive(Clone)]
pub struct ClipAmp<S>
where
    S: Signal,
{
    signal: S,
    threshold: f32,
}

impl<S> Signal for ClipAmp<S>
where
    S: Signal,
{
    #[inline]
    fn next(&mut self) -> f32 {
        self.signal.next().clamp(-self.threshold, self.threshold)
    }
}

/// An iterator that yields `n` number of frames from the inner `signal`.
#[derive(Clone)]
pub struct Take<S>
where
    S: Signal,
{
    signal: S,
    n: usize,
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

/// Create a new `Signal` from the given `Frame`-yielding `Iterator`.
pub fn from_iter<I>(frames: I) -> FromIterator<I::IntoIter>
where
    I: IntoIterator<Item = f32>,
{
    let mut iter = frames.into_iter();
    let next = iter.next();
    FromIterator { iter, next }
}

/// A type that wraps an Iterator and provides a `Signal` implementation for it.
#[derive(Clone)]
pub struct FromIterator<I>
where
    I: Iterator,
{
    iter: I,
    next: Option<I::Item>,
}

impl<I> Signal for FromIterator<I>
where
    I: Iterator<Item = f32>,
{
    #[inline]
    fn next(&mut self) -> f32 {
        match self.next.take() {
            Some(frame) => {
                self.next = self.iter.next();
                frame
            }
            None => EQUILIBRIUM,
        }
    }
}
