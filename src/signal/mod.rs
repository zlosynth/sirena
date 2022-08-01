//! Use the Signal trait to abstract signal as infinite-iterator-like objects.
//!
//! This is based on the concepts presented in
//! [dasp-signal](https://github.com/RustAudio/dasp) except this implementation
//! supports `#[no_std]` on stable.

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
}
