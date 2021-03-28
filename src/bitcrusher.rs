#[allow(unused_imports)]
use micromath::F32Ext;

pub struct Bitcrusher {
    rate: u32,
    resolution_multiple: f32,
    last_sampled: f32,
    since_sample: u32,
}

impl Bitcrusher {
    pub fn new() -> Self {
        Self {
            rate: 1,
            resolution_multiple: 1.0,
            last_sampled: 0.0,
            since_sample: 0,
        }
    }

    pub fn set_rate(&mut self, rate: u32) -> &mut Self {
        assert!(rate >= 1);
        self.rate = rate;
        self
    }

    pub fn set_resolution(&mut self, resolution: i32) -> &mut Self {
        assert!((1..=24).contains(&resolution));
        self.resolution_multiple = f32::powi(2.0, resolution as i32);
        self
    }

    pub fn process(&mut self, buffer: &mut [f32]) {
        buffer.iter_mut().for_each(|x| {
            if self.since_sample == 0 {
                let multiplied = *x * self.resolution_multiple;
                let truncated = multiplied.trunc();
                let divided = truncated / self.resolution_multiple;
                self.last_sampled = divided;
            }

            *x = self.last_sampled;

            self.since_sample += 1;
            self.since_sample %= self.rate;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize() {
        let _bitcrusher = Bitcrusher::new();
    }

    #[test]
    fn leave_incact() {
        let mut bitcrusher = Bitcrusher::new();
        bitcrusher.set_rate(1).set_resolution(24);

        let mut buffer = [10.0, 0.123456];
        let original_buffer = buffer;

        bitcrusher.process(&mut buffer);

        assert_eq!(&buffer, &original_buffer);
    }

    #[test]
    #[should_panic]
    fn resolution_over_range() {
        let mut bitcrusher = Bitcrusher::new();
        bitcrusher.set_resolution(25);
    }

    #[test]
    #[should_panic]
    fn resolution_under_range() {
        let mut bitcrusher = Bitcrusher::new();
        bitcrusher.set_resolution(0);
    }

    #[test]
    #[should_panic]
    fn rate_under_range() {
        let mut bitcrusher = Bitcrusher::new();
        bitcrusher.set_rate(0);
    }

    #[test]
    fn lower_resolution_to_5_bits() {
        let mut bitcrusher = Bitcrusher::new();
        bitcrusher.set_rate(1).set_resolution(5);

        let mut buffer = [10.0, 0.123456];

        bitcrusher.process(&mut buffer);

        assert_eq!(buffer, [10.0, 0.09375]);
    }

    #[test]
    fn lower_resolution_to_minimum() {
        let mut bitcrusher = Bitcrusher::new();
        bitcrusher.set_rate(1).set_resolution(1);

        let mut buffer = [0.0, 0.7, 1.0, -0.7, -1.0];

        bitcrusher.process(&mut buffer);

        assert_eq!(buffer, [0.0, 0.5, 1.0, -0.5, -1.0]);
    }

    #[test]
    fn lower_rate() {
        let mut bitcrusher = Bitcrusher::new();
        bitcrusher.set_rate(2);

        let mut buffer = [10.0, 1.0, 2.0, 3.0];

        bitcrusher.process(&mut buffer);

        assert_buffer_eq(&buffer, &[10.0, 10.0, 2.0, 2.0]);
    }

    #[test]
    fn lower_rate_accross_buffers() {
        let mut bitcrusher = Bitcrusher::new();
        bitcrusher.set_rate(2);

        let mut buffer_a = [10.0, 1.0, 2.0];
        let mut buffer_b = [3.0, 4.0, 5.0];

        bitcrusher.process(&mut buffer_a);
        bitcrusher.process(&mut buffer_b);

        assert_buffer_eq(&buffer_a, &[10.0, 10.0, 2.0]);
        assert_buffer_eq(&buffer_b, &[2.0, 4.0, 4.0]);
    }

    fn assert_buffer_eq(buffer: &[f32], expected: &[f32]) {
        for i in 0..buffer.len() {
            assert_relative_eq!(buffer[i], expected[i]);
        }
    }
}
