//! Clip sammples yielded by signal.

use super::Signal;

pub trait SignalClipAmp: Signal {
    /// Clips the amplitude of the signal to the given threshold amplitude.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sirena::signal::{self, Signal, SignalClipAmp};
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
}

impl<T> SignalClipAmp for T where T: Signal {}

/// Clips samples yielded by `signal` to the given threshold amplitude.
#[derive(Clone)]
pub struct ClipAmp<S>
where
    S: Signal,
{
    pub(super) signal: S,
    pub(super) threshold: f32,
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
