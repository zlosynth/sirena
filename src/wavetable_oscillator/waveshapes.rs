use core::f32::consts::PI;

pub fn sine() -> [f32; 2048 * 4] {
    let mut wavetable = [0.0; 2048 * 4];
    for (i, x) in wavetable.iter_mut().enumerate() {
        *x = f32::sin(i as f32 / (2048.0 * 4.0) * 2.0 * PI);
    }
    wavetable
}

pub fn saw(harmonics: u32) -> [f32; 2048 * 4] {
    let mut wavetable = [0.0; 2048 * 4];

    for (i, x) in wavetable.iter_mut().enumerate() {
        *x = f32::sin(i as f32 / (2048.0 * 4.0) * 2.0 * PI);
        for j in 2..harmonics {
            if j % 2 == 0 {
                *x -= f32::sin(i as f32 / (2048.0 * 4.0) * 2.0 * PI * j as f32) / j as f32;
            } else {
                *x += f32::sin(i as f32 / (2048.0 * 4.0) * 2.0 * PI * j as f32) / j as f32;
            }
        }
    }

    normalize(&mut wavetable);

    wavetable
}

fn normalize(data: &mut [f32]) {
    let ratio = normalization_ratio(data);
    for x in data.iter_mut() {
        *x *= ratio;
    }
}

fn normalization_ratio(data: &[f32]) -> f32 {
    let max = data.iter().fold(0.0, |a, b| f32::max(a, *b));
    let min = data.iter().fold(0.0, |a, b| f32::min(a, *b));
    let max_delta = f32::max(max, f32::abs(min));
    1.0 / max_delta
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_samples() {
        let wavetable = sine();

        assert_relative_eq!(wavetable[0], 0.0);
        assert_relative_eq!(wavetable[wavetable.len() / 4], 1.0);
    }

    #[test]
    fn saw_samples() {
        let wavetable = saw(368);

        assert_relative_eq!(wavetable[0], 0.0);

        let peak_phase = (wavetable.len() as f32 * 0.499) as usize;
        assert_relative_eq!(wavetable[peak_phase], 1.0, max_relative = 0.05);
    }

    #[test]
    fn normalize_waveform() {
        let mut data = [0.0, 2.0, -4.0];

        normalize(&mut data);

        assert_relative_eq!(data[0], 0.0);
        assert_relative_eq!(data[1], 0.5);
        assert_relative_eq!(data[2], -1.0);
    }

    #[test]
    fn normalization_ratio_infered_from_max() {
        let data = [0.0, 2.0];

        let ratio = normalization_ratio(&data);

        assert_relative_eq!(ratio, 0.5);
    }

    #[test]
    fn normalization_ratio_infered_from_min() {
        let data = [0.0, -2.0];

        let ratio = normalization_ratio(&data);

        assert_relative_eq!(ratio, 0.5);
    }

    #[test]
    fn normalization_ratio_greater_than_one() {
        let data = [0.0, 0.5];

        let ratio = normalization_ratio(&data);

        assert_relative_eq!(ratio, 2.0);
    }

    #[test]
    fn normalization_ratio_lower_than_one() {
        let data = [0.0, 2.0];

        let ratio = normalization_ratio(&data);

        assert_relative_eq!(ratio, 0.5);
    }
}
