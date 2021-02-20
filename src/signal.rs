pub fn normalize(data: &mut [f32]) {
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
