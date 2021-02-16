pub const WAVETABLE_LENGTH: usize = 2048;
pub const OVERSAMPLING: usize = 4;

pub struct Wavetable {
    wavetable: BandlimitedWavetable,
}

struct BandlimitedWavetable {
    wavetable: [f32; WAVETABLE_LENGTH],
    _minimal_sample_length: u32,
}

impl Wavetable {
    pub fn new(oversampled_wavetable: [f32; WAVETABLE_LENGTH * OVERSAMPLING]) -> Self {
        Wavetable {
            wavetable: BandlimitedWavetable {
                wavetable: undersample(oversampled_wavetable),
                _minimal_sample_length: 3,
            },
        }
    }

    fn wavetable_for_interval(&self, _interval_in_samples: u32) -> &[f32] {
        &self.wavetable.wavetable
    }

    pub fn read(&self, phase: f32, interval_in_samples: u32) -> f32 {
        let position = phase * WAVETABLE_LENGTH as f32;
        let wavetable = self.wavetable_for_interval(interval_in_samples);
        linear_interpolation(wavetable, position)
    }
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
        let _wavetable = Wavetable::new(sine());
    }

    #[test]
    fn read_value() {
        let wavetable = Wavetable::new(sine());

        let first = wavetable.read(0.0, 100);
        let second = wavetable.read(0.1, 100);
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
}
