#[allow(unused_imports)]
use micromath::F32Ext;

pub fn linear(data: &[f32], position: f32) -> f32 {
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
    use super::*;

    #[test]
    fn linear_interpolation_within_range() {
        let data = [0.0, 10.0, 20.0];

        assert_relative_eq!(linear(&data, 1.5), 15.0);
    }

    #[test]
    fn linear_interpolation_over_edge() {
        let data = [0.0, 10.0, 20.0];

        assert_relative_eq!(linear(&data, 2.5), 10.0);
    }
}
