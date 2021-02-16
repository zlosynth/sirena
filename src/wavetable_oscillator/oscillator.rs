use super::wavetable::Wavetable;

pub struct WavetableOscillator {
    wavetable: Wavetable,
    sample_rate: u32,
    frequency: f32,
    phase: f32,
}

impl WavetableOscillator {
    pub fn new(wavetable: Wavetable, sample_rate: u32) -> Self {
        Self {
            wavetable,
            sample_rate,
            frequency: 440.0,
            phase: 0.0,
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) -> &mut Self {
        self.frequency = frequency;
        self
    }

    pub fn tick(&mut self) -> f32 {
        let interval_in_samples = self.frequency / self.sample_rate as f32;
        let sample = self.wavetable.read(self.phase, interval_in_samples as u32);
        self.phase += interval_in_samples;
        self.phase %= 1.0;
        sample
    }
}

#[cfg(test)]
mod tests {
    use super::super::sine;
    use super::*;

    #[test]
    fn initialize_wavetable_oscillator() {
        let wavetable = Wavetable::new(sine());
        let _wavetable_oscillator = WavetableOscillator::new(wavetable, 44100);
    }

    #[test]
    fn get_first_sample() {
        let wavetable = Wavetable::new(sine());
        let mut wavetable_oscillator = WavetableOscillator::new(wavetable, 44100);

        assert_eq!(wavetable_oscillator.tick(), 0.0);
    }

    #[test]
    fn get_multiple_samples() {
        let wavetable = Wavetable::new(sine());
        let mut wavetable_oscillator = WavetableOscillator::new(wavetable, 8);
        wavetable_oscillator.set_frequency(1.0);

        let first = wavetable_oscillator.tick();
        let second = wavetable_oscillator.tick();
        assert!(second > first);
    }

    #[test]
    fn set_frequency() {
        let three_ticks_frequency_1 = {
            let wavetable = Wavetable::new(sine());
            let mut wavetable_oscillator = WavetableOscillator::new(wavetable, 8);
            wavetable_oscillator.set_frequency(1.0);
            wavetable_oscillator.tick();
            wavetable_oscillator.tick();
            wavetable_oscillator.tick()
        };

        let two_ticks_frequency_2 = {
            let wavetable = Wavetable::new(sine());
            let mut wavetable_oscillator = WavetableOscillator::new(wavetable, 8);
            wavetable_oscillator.set_frequency(2.0);
            wavetable_oscillator.tick();
            wavetable_oscillator.tick()
        };

        assert_relative_eq!(three_ticks_frequency_1, 1.0);
        assert_relative_eq!(three_ticks_frequency_1, two_ticks_frequency_2);
    }

    #[test]
    fn set_sample_rate() {
        let three_ticks_sample_rate_10 = {
            let wavetable = Wavetable::new(sine());
            let mut wavetable_oscillator = WavetableOscillator::new(wavetable, 10);
            wavetable_oscillator.set_frequency(2.0);
            wavetable_oscillator.tick();
            wavetable_oscillator.tick();
            wavetable_oscillator.tick()
        };

        let two_ticks_sample_rate_20 = {
            let wavetable = Wavetable::new(sine());
            let mut wavetable_oscillator = WavetableOscillator::new(wavetable, 20);
            wavetable_oscillator.set_frequency(2.0);
            wavetable_oscillator.tick();
            wavetable_oscillator.tick()
        };

        assert_relative_eq!(three_ticks_sample_rate_10, two_ticks_sample_rate_20);
    }
}
