//! Turn an iterator into a signal.

use super::{Signal, EQUILIBRIUM};

/// A type that wraps an Iterator and provides a `Signal` implementation for it.
#[derive(Clone)]
pub struct FromIterator<I>
where
    I: Iterator,
{
    pub(super) iter: I,
    pub(super) next: Option<I::Item>,
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
