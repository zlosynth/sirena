//! Take `n` samples of a signal.

use super::Signal;

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
