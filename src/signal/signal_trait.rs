use super::from_iterator::FromIterator;

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
    /// use sirena::signal::{self, Signal, SignalTake};
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
