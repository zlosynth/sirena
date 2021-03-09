use crate::taper;

pub fn log(a: f32, b: f32, xfade: f32) -> f32 {
    let blend = if a < b {
        1.0 - taper::log(xfade)
    } else {
        taper::log(1.0 - xfade)
    };
    a * blend + b * (1.0 - blend)
}

pub fn lin(a: f32, b: f32, x: f32) -> f32 {
    debug_assert!((0.0..=1.0).contains(&x));

    a * (1.0 - x) + b * x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logarithmic_in() {
        let blended = log(0.0, 1.0, 0.0);
        assert_relative_eq!(blended, 0.0);

        let blended = log(0.0, 1.0, 0.1);
        assert_relative_eq!(blended, 0.040958643);

        let blended = log(0.0, 1.0, 0.2);
        assert_relative_eq!(blended, 0.08618611);

        let blended = log(0.0, 1.0, 0.5);
        assert_relative_eq!(blended, 0.2596373);

        let blended = log(0.0, 1.0, 0.8);
        assert_relative_eq!(blended, 0.5528419);

        let blended = log(0.0, 1.0, 0.9);
        assert_relative_eq!(blended, 0.72124636);

        let blended = log(0.0, 1.0, 1.0);
        assert_relative_eq!(blended, 1.0);
    }

    #[test]
    fn logarithmic_out() {
        let blended = log(1.0, 0.0, 0.0);
        assert_relative_eq!(blended, 1.0);

        let blended = log(1.0, 0.0, 0.1);
        assert_relative_eq!(blended, 0.72124636);

        let blended = log(1.0, 0.0, 0.2);
        assert_relative_eq!(blended, 0.5528419);

        let blended = log(1.0, 0.0, 0.5);
        assert_relative_eq!(blended, 0.2596373);

        let blended = log(1.0, 0.0, 0.8);
        assert_relative_eq!(blended, 0.08618611);

        let blended = log(1.0, 0.0, 0.9);
        assert_relative_eq!(blended, 0.040958643);

        let blended = log(1.0, 0.0, 1.0);
        assert_relative_eq!(blended, 0.0);
    }

    #[test]
    fn logarithmic_equal() {
        let blended = log(1.0, 1.0, 0.0);
        assert_relative_eq!(blended, 1.0);

        let blended = log(1.0, 1.0, 0.5);
        assert_relative_eq!(blended, 1.0);

        let blended = log(1.0, 1.0, 1.0);
        assert_relative_eq!(blended, 1.0);
    }

    #[test]
    fn linear_even() {
        assert_relative_eq!(lin(8.0, 4.0, 0.5), 6.0);
    }

    #[test]
    fn linear_uneven() {
        assert_relative_eq!(lin(10.0, 20.0, 0.2), 12.0);
    }

    #[test]
    fn linear_left_side() {
        assert_relative_eq!(lin(8.0, 4.0, 0.0), 8.0);
    }

    #[test]
    fn linear_right_side() {
        assert_relative_eq!(lin(8.0, 4.0, 1.0), 4.0);
    }

    #[test]
    #[should_panic]
    fn linear_panics_on_x_below_zero() {
        lin(8.0, 4.0, -1.0);
    }

    #[test]
    #[should_panic]
    fn linear_panics_on_x_above_one() {
        lin(8.0, 4.0, 2.0);
    }
}
