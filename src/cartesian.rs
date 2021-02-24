use crate::wavetable_oscillator::{DoubleWavetableOscillator, Wavetable};

pub struct Cartesian<'a> {
    oscillators: [DoubleWavetableOscillator<'a>; 7],
}

impl<'a> Cartesian<'a> {
    pub fn new(
        wavetable_a: &'a Wavetable,
        wavetable_b: &'a Wavetable,
        wavetable_c: &'a Wavetable,
        sample_rate: u32,
    ) -> Self {
        Self {
            oscillators: [
                DoubleWavetableOscillator::new(wavetable_a, wavetable_b, wavetable_c, sample_rate),
                DoubleWavetableOscillator::new(wavetable_a, wavetable_b, wavetable_c, sample_rate),
                DoubleWavetableOscillator::new(wavetable_a, wavetable_b, wavetable_c, sample_rate),
                DoubleWavetableOscillator::new(wavetable_a, wavetable_b, wavetable_c, sample_rate),
                DoubleWavetableOscillator::new(wavetable_a, wavetable_b, wavetable_c, sample_rate),
                DoubleWavetableOscillator::new(wavetable_a, wavetable_b, wavetable_c, sample_rate),
                DoubleWavetableOscillator::new(wavetable_a, wavetable_b, wavetable_c, sample_rate),
            ],
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) -> &mut Self {
        self.oscillators[0].set_frequency(frequency);
        self
    }

    pub fn set_x(&mut self, x: f32) -> &mut Self {
        self.oscillators[0].set_x(x);
        self
    }

    pub fn set_y(&mut self, y: f32) -> &mut Self {
        self.oscillators[0].set_y(y);
        self
    }

    pub fn tick(&mut self) -> f32 {
        self.oscillators[0].tick()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wavetable_oscillator::{saw, sine, triangle};
    use std::f32::consts::PI;

    #[test]
    fn initialize() {
        const SAMPLE_RATE: u32 = 48000;
        let wavetable_a = Wavetable::new(sine(), SAMPLE_RATE);
        let wavetable_b = Wavetable::new(triangle(), SAMPLE_RATE);
        let wavetable_c = Wavetable::new(saw(), SAMPLE_RATE);
        let _cartesian = Cartesian::new(&wavetable_a, &wavetable_b, &wavetable_c, SAMPLE_RATE);
    }

    #[test]
    fn tick() {
        const SAMPLE_RATE: u32 = 48000;
        let wavetable_a = Wavetable::new(sine(), SAMPLE_RATE);
        let wavetable_b = Wavetable::new(triangle(), SAMPLE_RATE);
        let wavetable_c = Wavetable::new(saw(), SAMPLE_RATE);
        let mut cartesian = Cartesian::new(&wavetable_a, &wavetable_b, &wavetable_c, SAMPLE_RATE);

        let value_1 = cartesian.tick();
        let value_2 = cartesian.tick();

        assert_relative_eq!(value_1, 0.0, epsilon = 0.1);
        assert!(value_2 > value_1);
    }

    #[test]
    fn set_frequency() {
        const SAMPLE_RATE: u32 = 48000;

        let delta_one_tick_frequency_200 = {
            let wavetable_a = Wavetable::new(sine(), SAMPLE_RATE);
            let wavetable_b = Wavetable::new(triangle(), SAMPLE_RATE);
            let wavetable_c = Wavetable::new(saw(), SAMPLE_RATE);
            let mut cartesian =
                Cartesian::new(&wavetable_a, &wavetable_b, &wavetable_c, SAMPLE_RATE);
            cartesian.set_frequency(200.0);

            let original = cartesian.tick();
            let updated = cartesian.tick();

            updated - original
        };

        let delta_two_ticks_frequency_100 = {
            let wavetable_a = Wavetable::new(sine(), SAMPLE_RATE);
            let wavetable_b = Wavetable::new(triangle(), SAMPLE_RATE);
            let wavetable_c = Wavetable::new(saw(), SAMPLE_RATE);
            let mut cartesian =
                Cartesian::new(&wavetable_a, &wavetable_b, &wavetable_c, SAMPLE_RATE);
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

    #[test]
    fn use_zero_wavetable() {
        const SAMPLE_RATE: u32 = 48000;
        let wavetable_a = Wavetable::new(sine(), SAMPLE_RATE);
        let wavetable_b = Wavetable::new(triangle(), SAMPLE_RATE);
        let wavetable_c = Wavetable::new(saw(), SAMPLE_RATE);
        let mut cartesian = Cartesian::new(&wavetable_a, &wavetable_b, &wavetable_c, SAMPLE_RATE);
        cartesian.set_frequency(1.0).set_x(0.0).set_y(0.0);

        for _ in 0..SAMPLE_RATE / 8 - 1 {
            cartesian.tick();
        }
        let value = cartesian.tick();
        assert_relative_eq!(value, f32::sin(0.125 * 2.0 * PI), max_relative = 0.01);

        for _ in 0..SAMPLE_RATE / 8 - 1 {
            cartesian.tick();
        }
        let value = cartesian.tick();
        assert_relative_eq!(value, f32::sin(0.25 * 2.0 * PI), max_relative = 0.01);

        for _ in 0..SAMPLE_RATE / 4 - 1 {
            cartesian.tick();
        }
        let value = cartesian.tick();
        assert_relative_eq!(value, 0.0, epsilon = 0.01);
    }

    #[test]
    fn use_x_wavetable() {
        const SAMPLE_RATE: u32 = 48000;
        let wavetable_a = Wavetable::new(sine(), SAMPLE_RATE);
        let wavetable_b = Wavetable::new(triangle(), SAMPLE_RATE);
        let wavetable_c = Wavetable::new(saw(), SAMPLE_RATE);
        let mut cartesian = Cartesian::new(&wavetable_a, &wavetable_b, &wavetable_c, SAMPLE_RATE);
        cartesian.set_frequency(1.0).set_x(1.0).set_y(0.0);

        for _ in 0..SAMPLE_RATE / 8 - 1 {
            cartesian.tick();
        }
        let value = cartesian.tick();
        assert_relative_eq!(value, 0.5, max_relative = 0.01);

        for _ in 0..SAMPLE_RATE / 8 - 1 {
            cartesian.tick();
        }
        let value = cartesian.tick();
        assert_relative_eq!(value, 1.0, max_relative = 0.01);

        for _ in 0..SAMPLE_RATE / 4 - 1 {
            cartesian.tick();
        }
        let value = cartesian.tick();
        assert_relative_eq!(value, 0.0, epsilon = 0.01);
    }

    #[test]
    fn use_y_wavetable() {
        const SAMPLE_RATE: u32 = 48000;
        let wavetable_a = Wavetable::new(sine(), SAMPLE_RATE);
        let wavetable_b = Wavetable::new(triangle(), SAMPLE_RATE);
        let wavetable_c = Wavetable::new(saw(), SAMPLE_RATE);
        let mut cartesian = Cartesian::new(&wavetable_a, &wavetable_b, &wavetable_c, SAMPLE_RATE);
        cartesian.set_frequency(1.0).set_x(0.0).set_y(1.0);

        for _ in 0..SAMPLE_RATE / 4 - 1 {
            cartesian.tick();
        }
        let value = cartesian.tick();
        assert_relative_eq!(value, 0.5, epsilon = 0.1);

        for _ in 0..SAMPLE_RATE / 4 - 20 {
            cartesian.tick();
        }
        let value = cartesian.tick();
        assert_relative_eq!(value, 1.0, max_relative = 0.05);
    }
}
