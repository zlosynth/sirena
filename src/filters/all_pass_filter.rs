use super::ring_buffer::RingBuffer;

pub struct AllPassFilter {
    gain: f32,
    delay: usize,
    buffer: RingBuffer,
}

impl AllPassFilter {
    pub fn new() -> Self {
        Self {
            gain: 0.0,
            delay: 0,
            buffer: RingBuffer::new(),
        }
    }

    pub fn set_gain(&mut self, gain: f32) -> &mut Self {
        assert!(
            (0.0..1.0).contains(&gain),
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
            let feed_forward_data = *x * -self.gain;
            let delayed_data = self.buffer.peek(-(self.delay as i32) + 1);
            let feed_back_data = (feed_forward_data + delayed_data) * self.gain;
            self.buffer.write(feed_back_data + *x);
            *x = feed_forward_data + delayed_data;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_filter() {
        let _all_pass_filter = AllPassFilter::new();
    }

    #[test]
    fn set_gain() {
        let mut all_pass_filter = AllPassFilter::new();

        all_pass_filter.set_gain(0.3);
    }

    #[test]
    #[should_panic(expected = "gain must be set between 0 and 1")]
    fn set_invalid_gain_above_the_limit() {
        let mut all_pass_filter = AllPassFilter::new();

        all_pass_filter.set_gain(10.0);
    }

    #[test]
    #[should_panic(expected = "gain must be set between 0 and 1")]
    fn set_invalid_gain_below_the_limit() {
        let mut all_pass_filter = AllPassFilter::new();

        all_pass_filter.set_gain(-0.3);
    }

    #[test]
    fn process_data() {
        let mut data = [8.0, 0.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0];

        let mut all_pass_filter = AllPassFilter::new();
        all_pass_filter.set_gain(0.5).set_delay(2);

        all_pass_filter.process(&mut data);

        assert_eq!(data, [-4.0, 0.0, 2.0, 0.0, 9.0, 0.0, 4.5, 0.0]);
    }
}
