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
}
