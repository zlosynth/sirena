use super::ring_buffer::RingBuffer;

pub struct AllPassFilter {
    parameters: AllPassFilterParameters,
    buffer: RingBuffer,
}

pub struct AllPassFilterParameters {
    gain: f32,
    delay: usize,
}

impl AllPassFilter {
    pub fn new(parameters: AllPassFilterParameters) -> Self {
        Self {
            parameters,
            buffer: RingBuffer::new(),
        }
    }

    pub fn process(&mut self, data: &mut [f32]) {
        for x in data.iter_mut() {
            let feed_forward_data = *x * -self.parameters.gain;
            let delayed_data = self.buffer.peek(-(self.parameters.delay as i32) + 1);
            let feed_back_data = (feed_forward_data + delayed_data) * self.parameters.gain;
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
        let _all_pass_filter = AllPassFilter::new(AllPassFilterParameters {
            gain: 0.5,
            delay: 2,
        });
    }

    #[test]
    fn process_data() {
        let mut all_pass_filter = AllPassFilter::new(AllPassFilterParameters {
            gain: 0.5,
            delay: 2,
        });
        let mut data = [8.0, 0.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0];

        all_pass_filter.process(&mut data);

        assert_eq!(data, [-4.0, 0.0, 2.0, 0.0, 9.0, 0.0, 4.5, 0.0]);
    }
}
