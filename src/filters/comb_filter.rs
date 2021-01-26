use super::ring_buffer::RingBuffer;

pub struct CombFilter {
    gain: f32,
    delay: usize,
    buffer: RingBuffer,
}

impl CombFilter {
    pub fn new() -> Self {
        Self {
            gain: 0.0,
            delay: 0,
            buffer: RingBuffer::new(),
        }
    }

    pub fn set_gain(&mut self, gain: f32) -> &mut Self {
        assert!(
            gain >= 0.0 && gain < 1.0,
            "gain must be set between 0 and 1"
        );
        self.gain = gain;
        self
    }

    pub fn set_delay(&mut self, delay: usize) -> &mut Self {
        self.delay = delay;
        self
    }

    pub fn process(&mut self, data: &mut [f32]) {
        for x in data.iter_mut() {
            let echoing_value = self.buffer.peek(-(self.delay as i32) + 1);
            self.buffer.write(*x + echoing_value * self.gain);
            *x = echoing_value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_filter() {
        let _comb_filter = CombFilter::new();
    }

    #[test]
    fn set_gain() {
        let mut comb_filter = CombFilter::new();

        comb_filter.set_gain(0.3);
    }

    #[test]
    #[should_panic(expected = "gain must be set between 0 and 1")]
    fn set_invalid_gain_above_the_limit() {
        let mut comb_filter = CombFilter::new();

        comb_filter.set_gain(10.0);
    }

    #[test]
    #[should_panic(expected = "gain must be set between 0 and 1")]
    fn set_invalid_gain_below_the_limit() {
        let mut comb_filter = CombFilter::new();

        comb_filter.set_gain(-0.3);
    }

    #[test]
    fn process_data() {
        let mut data: [f32; 8] = [8.0, 0.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0];

        let mut comb_filter = CombFilter::new();
        comb_filter.set_delay(2).set_gain(0.5);

        comb_filter.process(&mut data);

        assert_eq!(data, [0.0, 0.0, 8.0, 0.0, 12.0, 0.0, 6.0, 0.0]);
    }
}
