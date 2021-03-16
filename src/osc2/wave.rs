#[allow(unused_imports)]
use micromath::F32Ext;

use super::consts::VOICES_LEN;

pub fn distribute(center: f32, radius: f32) -> [f32; VOICES_LEN] {
    let center = center.rem_euclid(VOICES_LEN as f32);
    [
        center - radius,
        center - radius / 2.0,
        center,
        center + radius / 2.0,
        center + radius,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stay_on_center() {
        let center = 1.0;
        let radius = 0.0;
        let distributed = distribute(center, radius);

        assert_waves(distributed, 1.0, 1.0, 1.0, 1.0, 1.0);
    }

    #[test]
    fn spread() {
        let center = 2.0;
        let radius = 1.0;
        let distributed = distribute(center, radius);

        assert_waves(distributed, 1.0, 1.5, 2.0, 2.5, 3.0);
    }

    fn assert_waves(waves: [f32; VOICES_LEN], b1: f32, b2: f32, b3: f32, b4: f32, b5: f32) {
        assert_relative_eq!(waves[0], b1);
        assert_relative_eq!(waves[1], b2);
        assert_relative_eq!(waves[2], b3);
        assert_relative_eq!(waves[3], b4);
        assert_relative_eq!(waves[4], b5);
    }
}
