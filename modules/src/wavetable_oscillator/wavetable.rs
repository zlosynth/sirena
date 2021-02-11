use core::f32::consts::PI;

pub struct Wavetable {
    wavetable: [f32; 2048],
}

impl Wavetable {
    pub fn new(wavetable: [f32; 2048]) -> Self {
        Wavetable { wavetable }
    }

    pub fn read(&self, phase: f32) -> f32 {
        let position = phase * 2048.0;
        linear_interpolation(&self.wavetable, position)
    }
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

pub fn sine() -> [f32; 2048] {
    let mut wavetable = [0.0; 2048];
    for (i, x) in wavetable.iter_mut().enumerate() {
        *x = f32::sin(i as f32 / 2048.0 * 2.0 * PI);
    }
    wavetable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_wavetable() {
        let _wavetable = Wavetable::new(sine());
    }

    #[test]
    fn read_value() {
        let wavetable = Wavetable::new(sine());

        let first = wavetable.read(0.0);
        let second = wavetable.read(0.1);
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
    fn sine_samples() {
        let wavetable = sine();

        assert_relative_eq!(wavetable[0], 0.0);
        assert_relative_eq!(wavetable[wavetable.len() / 4], 1.0);
    }
}
