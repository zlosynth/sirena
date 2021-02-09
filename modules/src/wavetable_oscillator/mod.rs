use core::f32::consts::PI;

pub struct WavetableOscillator {
    sample_rate: u32,
    frequency: f32,
    phase: f32,
}

impl WavetableOscillator {
    pub fn new() -> Self {
        Self {
            sample_rate: 1,
            frequency: 1.0,
            phase: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) -> &mut Self {
        self.sample_rate = sample_rate;
        self
    }

    pub fn set_frequency(&mut self, frequency: f32) -> &mut Self {
        self.frequency = frequency;
        self
    }

    pub fn tick(&mut self) -> f32 {
        let sample = f32::sin(self.phase * 2.0 * PI);
        self.phase += self.frequency / self.sample_rate as f32;
        self.phase %= 1.0;
        sample
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_wavetable_oscillator() {
        let _wavetable_oscillator = WavetableOscillator::new();
    }

    #[test]
    fn get_first_sample() {
        let mut wavetable_oscillator = WavetableOscillator::new();

        assert_eq!(wavetable_oscillator.tick(), 0.0);
    }

    #[test]
    fn get_multiple_samples() {
        let mut wavetable_oscillator = WavetableOscillator::new();
        wavetable_oscillator.set_sample_rate(8).set_frequency(1.0);

        let first = wavetable_oscillator.tick();
        let second = wavetable_oscillator.tick();
        assert!(second > first);
    }

    #[test]
    fn set_frequency() {
        let three_ticks_frequency_1 = {
            let mut wavetable_oscillator = WavetableOscillator::new();
            wavetable_oscillator.set_sample_rate(8).set_frequency(1.0);
            wavetable_oscillator.tick();
            wavetable_oscillator.tick();
            wavetable_oscillator.tick()
        };

        let two_ticks_frequency_2 = {
            let mut wavetable_oscillator = WavetableOscillator::new();
            wavetable_oscillator.set_sample_rate(8).set_frequency(2.0);
            wavetable_oscillator.tick();
            wavetable_oscillator.tick()
        };

        assert_relative_eq!(three_ticks_frequency_1, 1.0);
        assert_relative_eq!(three_ticks_frequency_1, two_ticks_frequency_2);
    }

    #[test]
    fn set_sample_rate() {
        let three_ticks_sample_rate_10 = {
            let mut wavetable_oscillator = WavetableOscillator::new();
            wavetable_oscillator.set_sample_rate(20).set_frequency(1.0);
            wavetable_oscillator.tick();
            wavetable_oscillator.tick();
            wavetable_oscillator.tick()
        };

        let two_ticks_sample_rate_20 = {
            let mut wavetable_oscillator = WavetableOscillator::new();
            wavetable_oscillator.set_sample_rate(20).set_frequency(2.0);
            wavetable_oscillator.tick();
            wavetable_oscillator.tick()
        };

        assert_relative_eq!(three_ticks_sample_rate_10, two_ticks_sample_rate_20);
    }
}
