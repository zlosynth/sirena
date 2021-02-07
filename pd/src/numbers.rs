pub trait Pin: Copy {
    fn pin(self, min: Self, max: Self) -> Self;
}

impl Pin for f32 {
    fn pin(self, min: Self, max: Self) -> Self {
        assert!(min <= max, "min must be lower than max");
        f32::min(f32::max(self, min), max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f32_within_limit() {
        assert_eq!(15.0.pin(10.0, 20.0), 15.0);
    }

    #[test]
    fn f32_under_limit() {
        assert_eq!(1.0.pin(10.0, 20.0), 10.0);
    }

    #[test]
    fn f32_above_limit() {
        assert_eq!(100.0.pin(10.0, 20.0), 20.0);
    }
}
