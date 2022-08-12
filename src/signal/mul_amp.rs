use super::Signal;

/// Multiplies samples yielded by signal by given signal.
pub trait SignalMulAmp: Signal {
    /// Multiples the amplitude of the signal by another signal.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sirena::signal::{self, Signal, SignalMulAmp};
    /// let a_frames = [0.5, 2.0, -1.0];
    /// let b_frames = [1.0, 1.0, 1.0];
    /// let mut a_signal = signal::from_iter(a_frames);
    /// let mut b_signal = signal::from_iter(b_frames);
    /// let mut signal = a_signal.mul_amp(b_signal);
    /// assert_eq!(signal.next(), 0.5);
    /// assert_eq!(signal.next(), 2.0);
    /// assert_eq!(signal.next(), -1.0);
    /// ```
    fn mul_amp<O>(self, other: O) -> MulAmp<Self, O>
    where
        Self: Sized,
        O: Signal,
    {
        MulAmp {
            signal: self,
            other,
        }
    }
}

impl<T> SignalMulAmp for T where T: Signal {}

/// Clips samples yielded by `signal` to the given threshold amplitude.
#[derive(Clone)]
pub struct MulAmp<S, O>
where
    S: Signal,
    O: Signal,
{
    signal: S,
    other: O,
}

impl<S, O> Signal for MulAmp<S, O>
where
    S: Signal,
    O: Signal,
{
    #[inline]
    fn next(&mut self) -> f32 {
        self.signal.next() * self.other.next()
    }
}
