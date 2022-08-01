//! Use the Signal trait to abstract signal as infinite-iterator-like objects.
//!
//! This is based on the concepts presented in
//! [dasp-signal](https://github.com/RustAudio/dasp) except this implementation
//! supports `#[no_std]` on stable.

const EQUILIBRIUM: f32 = 0.0;

/// Types that yield values of a PCM signal.
pub trait Signal {
    fn next(&mut self) -> f32;

    /// Clips the amplitude of the signal to the given threshold amplitude.
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

#[cfg(test)]
mod tests {
    use super::*;

    struct StableSignal(f32);

    impl Signal for StableSignal {
        fn next(&mut self) -> f32 {
            self.0
        }
    }

    #[test]
    fn given_stable_signal_it_yields_expected_value() {
        let mut signal = StableSignal(1.0);
        assert_relative_eq!(signal.next(), 1.0);
        assert_relative_eq!(signal.next(), 1.0);
    }

    #[test]
    fn given_clip_amp_when_signal_is_with_limit_it_stays_intact() {
        let mut signal = StableSignal(0.5).clip_amp(1.0);
        assert_relative_eq!(signal.next(), 0.5);
    }

    #[test]
    fn given_clip_amp_when_signal_is_outside_limit_it_gets_clipped() {
        let mut signal = StableSignal(2.0).clip_amp(1.0);
        assert_relative_eq!(signal.next(), 1.0);
    }

    #[test]
    fn given_from_iter_when_called_on_an_iterator_it_returns_signal() {
        let array = [0.0, 1.0, 2.0];
        let mut signal = from_iter(array.into_iter());
        assert_relative_eq!(signal.next(), 0.0);
        assert_relative_eq!(signal.next(), 1.0);
    }
}
