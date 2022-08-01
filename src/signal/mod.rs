//! Use the Signal trait to abstract signal as infinite-iterator-like objects.
//!
//! This is based on the concepts presented in
//! [dasp-signal](https://github.com/RustAudio/dasp) except this implementation
//! supports `#[no_std]` on stable.

/// Types that yield values of a PCM signal.
pub trait Signal {
    fn next(&mut self) -> f32;
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
}
