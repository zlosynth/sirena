use super::ring_buffer::RingBuffer;

pub struct CombFilter {
    parameters: CombFilterParameters,
    buffer: RingBuffer,
}

pub struct CombFilterParameters {
    gain: f32,
    delay: usize,
}

impl CombFilter {
    pub fn new(parameters: CombFilterParameters) -> Self {
        Self {
            parameters,
            buffer: RingBuffer::new(),
        }
    }

    pub fn process(&mut self, data: &mut [f32]) {
        for x in data.iter_mut() {
            let echoing_value = self.buffer.peek(-(self.parameters.delay as i32) + 1);
            self.buffer.write(*x + echoing_value * self.parameters.gain);
            *x = echoing_value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_filter() {
        let _comb_filter = CombFilter::new(CombFilterParameters {
            gain: 0.5,
            delay: 2,
        });
    }

    #[test]
    fn process_data() {
        let mut comb_filter = CombFilter::new(CombFilterParameters {
            gain: 0.5,
            delay: 2,
        });
        let mut data: [f32; 8] = [8.0, 0.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0];

        comb_filter.process(&mut data);

        assert_eq!(data, [0.0, 0.0, 8.0, 0.0, 12.0, 0.0, 6.0, 0.0]);
    }
}
