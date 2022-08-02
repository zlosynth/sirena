//! Clip sammples yielded by signal.

use super::Signal;

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
