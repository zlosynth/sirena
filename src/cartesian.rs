use crate::wavetable_oscillator::{saw, sine, triangle, DoubleWavetableOscillator, Wavetable};

pub struct Cartesian {
    oscillator: DoubleWavetableOscillator,
}

impl Cartesian {
    pub fn new(sample_rate: u32) -> Self {
        let wavetable_1 = Wavetable::new(sine(), sample_rate);
        let wavetable_2 = Wavetable::new(triangle(), sample_rate);
        let wavetable_3 = Wavetable::new(saw(), sample_rate);
        let oscillator =
            DoubleWavetableOscillator::new(wavetable_1, wavetable_2, wavetable_3, sample_rate);

        Self { oscillator }
    }

    pub fn set_frequency(&mut self, frequency: f32) -> &mut Self {
        self.oscillator.set_frequency(frequency);
        self
    }

    pub fn tick(&mut self) -> f32 {
        self.oscillator.tick()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize() {
        const SAMPLE_RATE: u32 = 48000;
        let _cartesian = Cartesian::new(SAMPLE_RATE);
    }

    #[test]
    fn tick() {
        const SAMPLE_RATE: u32 = 48000;
        let mut cartesian = Cartesian::new(SAMPLE_RATE);

        let value_1 = cartesian.tick();
        let value_2 = cartesian.tick();

        assert_relative_eq!(value_1, 0.0, epsilon = 0.1);
        assert!(value_2 > value_1);
    }

    #[test]
    fn set_frequency() {
        const SAMPLE_RATE: u32 = 48000;

        let delta_one_tick_frequency_200 = {
            let mut cartesian = Cartesian::new(SAMPLE_RATE);
            cartesian.set_frequency(200.0);

            let original = cartesian.tick();
            let updated = cartesian.tick();

            updated - original
        };

        let delta_two_ticks_frequency_100 = {
            let mut cartesian = Cartesian::new(SAMPLE_RATE);
            cartesian.set_frequency(100.0);

            let original = cartesian.tick();
            cartesian.tick();
            let updated = cartesian.tick();

            updated - original
        };

        assert_relative_eq!(
            delta_one_tick_frequency_200,
            delta_two_ticks_frequency_100,
            max_relative = 0.001
        );
    }
}
