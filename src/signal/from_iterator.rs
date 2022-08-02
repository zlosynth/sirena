//! Turn an iterator into a signal.

use super::{Signal, EQUILIBRIUM};

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
