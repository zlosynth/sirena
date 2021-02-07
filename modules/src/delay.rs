use super::ring_buffer::RingBuffer;

pub struct Delay {
    delay: usize,
    buffer: RingBuffer,
}

impl Delay {
    pub fn new() -> Self {
        Self {
            delay: 0,
            buffer: RingBuffer::new(),
        }
    }

    pub fn set_delay(&mut self, delay: usize) -> &mut Self {
        self.delay = delay;
        self
    }

    pub fn process(&mut self, data: &mut [f32]) {
        for x in data.iter_mut() {
            self.buffer.write(*x);
            let echoing_value = self.buffer.peek(-(self.delay as i32));
            *x = echoing_value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_delay() {
        let _delay = Delay::new();
    }

    #[test]
    fn set_delay() {
        let mut delay = Delay::new();

        delay.set_delay(100);
    }

    #[test]
    fn process_data() {
        let mut data: [f32; 8] = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

        let mut comb_filter = Delay::new();
        comb_filter.set_delay(2);

        comb_filter.process(&mut data);

        assert_eq!(data, [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    }
}
