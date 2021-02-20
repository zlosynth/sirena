use super::waveshapes::sine;
use crate::signal;
use crate::state_variable_filter::{LowPass, StateVariableFilter};

pub const WAVETABLE_LENGTH: usize = 2048;
pub const OVERSAMPLING: usize = 4;

pub struct Wavetable {
    sample_rate: u32,
    wavetable: [f32; 2048],
    wavetable_1_4th: [f32; 2048],
    wavetable_1_8th: [f32; 2048],
    wavetable_1_16th: [f32; 2048],
    wavetable_1_32th: [f32; 2048],
    wavetable_1_64th: [f32; 2048],
    wavetable_1_128th: [f32; 2048],
}

impl Wavetable {
    pub fn new(
        oversampled_wavetable: [f32; WAVETABLE_LENGTH * OVERSAMPLING],
        sample_rate: u32,
    ) -> Self {
        let wavetable_1_128th = filter(&oversampled_wavetable, 1024.0);
        let wavetable_1_64th = filter(&oversampled_wavetable, 256.0);
        let wavetable_1_32th = filter(&oversampled_wavetable, 64.0);
        let wavetable_1_16th = filter(&oversampled_wavetable, 16.0);
        let wavetable_1_8th = filter(&oversampled_wavetable, 4.0);
        let wavetable_1_4th = filter(&oversampled_wavetable, 1.0);

        let wavetable = {
            let oversampled_wavetable = sine();
            let mut undersampled = undersample(oversampled_wavetable);
            undersampled.iter_mut().for_each(|x| *x *= 0.3);
            undersampled
        };

        Wavetable {
            sample_rate,
            wavetable,
            wavetable_1_4th,
            wavetable_1_8th,
            wavetable_1_16th,
            wavetable_1_32th,
            wavetable_1_64th,
            wavetable_1_128th,
        }
    }

    pub fn read(&self, phase: f32, frequency: f32) -> f32 {
        let position = phase * WAVETABLE_LENGTH as f32;

        let (wavetable_a, wavetable_b, mix) = {
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

        let a = linear_interpolation(wavetable_a, position);
        let b = linear_interpolation(wavetable_b, position);

        cross_fade(a, b, mix)
    }
}

fn filter(
    oversampled_wavetable: &[f32; WAVETABLE_LENGTH * OVERSAMPLING],
    frequency: f32,
) -> [f32; WAVETABLE_LENGTH] {
    let mut oversampled_wavetable = *oversampled_wavetable;

    let mut filter = StateVariableFilter::new((WAVETABLE_LENGTH * OVERSAMPLING * 2) as u32);
    filter
        .set_bandform(LowPass)
        .set_frequency(frequency)
        .set_q_factor(0.7);
    for _ in 0..3 {
        filter.pass(&oversampled_wavetable);
    }
    filter.process(&mut oversampled_wavetable);

    let mut undersampled = undersample(oversampled_wavetable);

    signal::normalize(&mut undersampled);

    undersampled
}

fn cross_fade(a: f32, b: f32, x: f32) -> f32 {
    assert!(x >= 0.0 && x <= 1.0);

    a * (1.0 - x) + b * x
}

fn undersample(data: [f32; WAVETABLE_LENGTH * OVERSAMPLING]) -> [f32; WAVETABLE_LENGTH] {
    let mut undersampled_data = [0.0; WAVETABLE_LENGTH];
    for i in 0..WAVETABLE_LENGTH {
        undersampled_data[i] = data[i * OVERSAMPLING];
    }
    undersampled_data
}

fn linear_interpolation(data: &[f32], position: f32) -> f32 {
    let index = position as usize;
    let remainder = position % 1.0;

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
    fn verify_undersampling() {
        let mut data = [0.0; WAVETABLE_LENGTH * OVERSAMPLING];
        for (i, x) in data.iter_mut().enumerate() {
            *x = i as f32;
        }

        let undersampled_data = undersample(data);

        assert_relative_eq!(undersampled_data[0], 0.0);
        assert_relative_eq!(undersampled_data[1], 4.0);
        assert_relative_eq!(undersampled_data[2], 8.0);
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
