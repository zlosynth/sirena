use super::consts::VOICES_LEN;

pub fn distribute(wideness: f32) -> [f32; VOICES_LEN] {
    let close_pan = wideness;
    let far_pan = (wideness * 2.0).min(1.0);
    [-far_pan, -close_pan, 0.0, close_pan, far_pan]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center() {
        let pans = distribute(0.0);
        assert_pans(pans, 0.0, 0.0, 0.0, 0.0, 0.0);
    }

    #[test]
    fn evenly_spread_out() {
        let pans = distribute(0.5);
        assert_pans(pans, -1.0, -0.5, 0.0, 0.5, 1.0);
    }

    #[test]
    fn pushed_away() {
        let pans = distribute(1.0);
        assert_pans(pans, -1.0, -1.0, 0.0, 1.0, 1.0);
    }

    fn assert_pans(breadths: [f32; VOICES_LEN], b1: f32, b2: f32, b3: f32, b4: f32, b5: f32) {
        assert_relative_eq!(breadths[0], b1);
        assert_relative_eq!(breadths[1], b2);
        assert_relative_eq!(breadths[2], b3);
        assert_relative_eq!(breadths[3], b4);
        assert_relative_eq!(breadths[4], b5);
    }
}
