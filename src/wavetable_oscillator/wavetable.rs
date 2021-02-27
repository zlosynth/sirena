use super::consts::OVERSAMPLED_WAVETABLE_LENGTH;
use super::waveshapes::sine;
use crate::signal;
use crate::state_variable_filter::{LowPass, StateVariableFilter};

pub struct Wavetable {
    sample_rate: u32,
    wavetable: [f32; 32],
    wavetable_1_4th: [f32; 64],
    wavetable_1_8th: [f32; 128],
    wavetable_1_16th: [f32; 256],
    wavetable_1_32th: [f32; 512],
    wavetable_1_64th: [f32; 1024],
    wavetable_1_128th: [f32; 2048],
}

impl Wavetable {
    pub fn new(
        oversampled_wavetable: [f32; OVERSAMPLED_WAVETABLE_LENGTH],
        sample_rate: u32,
    ) -> Self {
        let wavetable_1_128th = filtered(&oversampled_wavetable, 1024.0);
        let wavetable_1_64th = filtered(&oversampled_wavetable, 256.0);
        let wavetable_1_32th = filtered(&oversampled_wavetable, 64.0);
        let wavetable_1_16th = filtered(&oversampled_wavetable, 16.0);
        let wavetable_1_8th = filtered(&oversampled_wavetable, 4.0);
        let wavetable_1_4th = filtered(&oversampled_wavetable, 1.0);

        let wavetable = {
            let mut wavetable = sine();
            wavetable.iter_mut().for_each(|x| *x *= 0.3);
            wavetable
        };

        Wavetable {
            sample_rate,
            wavetable: undersample_32(wavetable),
            wavetable_1_4th: undersample_64(wavetable_1_4th),
            wavetable_1_8th: undersample_128(wavetable_1_8th),
            wavetable_1_16th: undersample_256(wavetable_1_16th),
            wavetable_1_32th: undersample_512(wavetable_1_32th),
            wavetable_1_64th: undersample_1024(wavetable_1_64th),
            wavetable_1_128th: undersample_2048(wavetable_1_128th),
        }
    }

    pub fn band(&self, frequency: f32) -> BandWavetable {
        let (wavetable_a, wavetable_b, mix): (&[f32], &[f32], f32) = {
            let relative_position = frequency / (self.sample_rate as f32 / 2.0);

            if relative_position < 1.0 / 128.0 {
                let mix = relative_position / (1.0 / 128.0);
                (&self.wavetable_1_128th, &self.wavetable_1_64th, mix)
            } else if relative_position < 1.0 / 64.0 {
                let mix = (relative_position - 1.0 / 128.0) / (1.0 / 128.0);
                (&self.wavetable_1_64th, &self.wavetable_1_32th, mix)
            } else if relative_position < 1.0 / 32.0 {
                let mix = (relative_position - 1.0 / 64.0) / (1.0 / 64.0);
                (&self.wavetable_1_32th, &self.wavetable_1_16th, mix)
            } else if relative_position < 1.0 / 16.0 {
                let mix = (relative_position - 1.0 / 32.0) / (1.0 / 32.0);
                (&self.wavetable_1_16th, &self.wavetable_1_8th, mix)
            } else if relative_position < 1.0 / 8.0 {
                let mix = (relative_position - 1.0 / 16.0) / (1.0 / 16.0);
                (&self.wavetable_1_8th, &self.wavetable_1_4th, mix)
            } else if relative_position < 1.0 / 4.0 {
                let mix = (relative_position - 1.0 / 8.0) / (1.0 / 8.0);
                (&self.wavetable_1_4th, &self.wavetable, mix)
            } else {
                (&self.wavetable, &self.wavetable, 1.0)
            }
        };

        BandWavetable::new(wavetable_a, wavetable_b, mix)
    }

    pub fn read(&self, phase: f32, frequency: f32) -> f32 {
        let band_wavetable = self.band(frequency);
        band_wavetable.read(phase)
    }
}

pub struct BandWavetable<'a> {
    lower: &'a [f32],
    higher: &'a [f32],
    mix: f32,
}

impl<'a> BandWavetable<'a> {
    fn new(lower: &'a [f32], higher: &'a [f32], mix: f32) -> Self {
        Self { lower, higher, mix }
    }

    pub fn read(&self, phase: f32) -> f32 {
        let a = {
            let position = phase * self.lower.len() as f32;
            linear_interpolation(self.lower, position)
        };
        let b = {
            let position = phase * self.higher.len() as f32;
            linear_interpolation(self.higher, position)
        };

        cross_fade(a, b, self.mix)
    }
}

fn filtered(
    wavetable: &[f32; OVERSAMPLED_WAVETABLE_LENGTH],
    frequency: f32,
) -> [f32; OVERSAMPLED_WAVETABLE_LENGTH] {
    let mut wavetable = *wavetable;

    let mut filter = StateVariableFilter::new((OVERSAMPLED_WAVETABLE_LENGTH * 2) as u32);
    filter
        .set_bandform(LowPass)
        .set_frequency(frequency)
        .set_q_factor(0.7);
    for _ in 0..3 {
        filter.pass(&wavetable);
    }
    filter.process(&mut wavetable);

    signal::normalize(&mut wavetable);

    wavetable
}

fn cross_fade(a: f32, b: f32, x: f32) -> f32 {
    debug_assert!((0.0..=1.0).contains(&x));

    a * (1.0 - x) + b * x
}

macro_rules! fn_undersample {
    ( $func_name:ident, $target_size:expr ) => {
        fn $func_name(data: [f32; OVERSAMPLED_WAVETABLE_LENGTH]) -> [f32; $target_size] {
            assert!(data.len() >= $target_size);
            assert!(data.len() % $target_size == 0);

            let ratio = data.len() / $target_size;

            let mut undersampled_data = [0.0; $target_size];
            for i in 0..$target_size {
                undersampled_data[i] = data[i * ratio];
            }

            undersampled_data
        }
    };
}

fn_undersample!(undersample_2048, 2048);
fn_undersample!(undersample_1024, 1024);
fn_undersample!(undersample_512, 512);
fn_undersample!(undersample_256, 256);
fn_undersample!(undersample_128, 128);
fn_undersample!(undersample_64, 64);
fn_undersample!(undersample_32, 32);

fn linear_interpolation(data: &[f32], position: f32) -> f32 {
    let index = position as usize;
    let remainder = position.fract();

    let value = data[index];
    let delta_to_next = if index == (data.len() - 1) {
        data[0] - data[index]
    } else {
        data[index + 1] - data[index]
    };

    value + delta_to_next * remainder
}

#[cfg(test)]
mod tests {
    use super::super::sine;
    use super::*;

    #[test]
    fn init_wavetable() {
        const SAMPLE_RATE: u32 = 44100;
        let _wavetable = Wavetable::new(sine(), SAMPLE_RATE);
    }

    #[test]
    fn read_value() {
        const SAMPLE_RATE: u32 = 44100;
        let wavetable = Wavetable::new(sine(), SAMPLE_RATE);

        let first = wavetable.read(0.0, 100.0);
        let second = wavetable.read(0.1, 100.0);
        assert!(second > first);
    }

    #[test]
    fn linear_interpolation_within_range() {
        let data = [0.0, 10.0, 20.0];

        assert_relative_eq!(linear_interpolation(&data, 1.5), 15.0);
    }

    #[test]
    fn linear_interpolation_over_edge() {
        let data = [0.0, 10.0, 20.0];

        assert_relative_eq!(linear_interpolation(&data, 2.5), 10.0);
    }

    #[test]
    fn verify_undersampling_2048() {
        let data = wavetable_ramp();

        let undersampled_data = undersample_2048(data);

        assert_relative_eq!(undersampled_data[0], 0.0);
        assert_relative_eq!(undersampled_data[1], 4.0);
        assert_relative_eq!(undersampled_data[2], 8.0);
    }

    fn wavetable_ramp() -> [f32; OVERSAMPLED_WAVETABLE_LENGTH] {
        let mut data = [0.0; OVERSAMPLED_WAVETABLE_LENGTH];
        for (i, x) in data.iter_mut().enumerate() {
            *x = i as f32;
        }
        data
    }

    #[test]
    fn cross_fade_even() {
        assert_relative_eq!(cross_fade(8.0, 4.0, 0.5), 6.0);
    }

    #[test]
    fn cross_fade_uneven() {
        assert_relative_eq!(cross_fade(10.0, 20.0, 0.2), 12.0);
    }

    #[test]
    fn cross_fade_left_side() {
        assert_relative_eq!(cross_fade(8.0, 4.0, 0.0), 8.0);
    }

    #[test]
    fn cross_fade_right_side() {
        assert_relative_eq!(cross_fade(8.0, 4.0, 1.0), 4.0);
    }

    #[test]
    #[should_panic]
    fn cross_fade_panics_on_x_below_zero() {
        cross_fade(8.0, 4.0, -1.0);
    }

    #[test]
    #[should_panic]
    fn cross_fade_panics_on_x_above_one() {
        cross_fade(8.0, 4.0, 2.0);
    }
}
