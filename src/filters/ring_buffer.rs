// TODO: Delete data after reading

const BUFFER_SIZE: usize = crate::SAMPLE_RATE as usize;

pub struct RingBuffer {
    buffer: [f32; BUFFER_SIZE],
    write_index: usize,
}

impl RingBuffer {
    pub fn new() -> Self {
        let buffer = {
            let mut data: [std::mem::MaybeUninit<f32>; BUFFER_SIZE] =
                unsafe { std::mem::MaybeUninit::uninit().assume_init() };
            for elem in &mut data[..] {
                unsafe {
                    std::ptr::write(elem.as_mut_ptr(), 0.0);
                }
            }
            unsafe { std::mem::transmute::<_, [f32; BUFFER_SIZE]>(data) }
        };

        Self {
            buffer,
            write_index: 0,
        }
    }

    pub fn write(&mut self, value: f32) {
        self.write_index %= BUFFER_SIZE;
        self.buffer[self.write_index] = value;
        self.write_index += 1;
    }

    pub fn peek(&self, relative_index: i32) -> f32 {
        let index = (self.write_index as i32 + relative_index - 1)
            .wrapping_rem_euclid(BUFFER_SIZE as i32) as usize;
        self.buffer[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_buffer() {
        let _buffer = RingBuffer::new();
    }

    #[test]
    fn write_to_buffer() {
        let mut buffer = RingBuffer::new();

        buffer.write(1.0);
    }

    #[test]
    fn read_from_buffer() {
        let mut buffer = RingBuffer::new();
        buffer.write(1.0);
        buffer.write(2.0);
        buffer.write(3.0);

        assert_eq!(buffer.peek(0), 3.0);
        assert_eq!(buffer.peek(-1), 2.0);
        assert_eq!(buffer.peek(-2), 1.0);
    }

    #[test]
    fn write_over_capacity() {
        let mut buffer = RingBuffer::new();

        for x in 0..=(BUFFER_SIZE + 1) {
            buffer.write(x as f32);
        }
    }

    #[test]
    fn cross_buffer_end_while_reading() {
        let mut buffer = RingBuffer::new();
        for x in 0..=(BUFFER_SIZE) {
            buffer.write(x as f32);
        }

        assert_eq!(buffer.peek(0) as usize, BUFFER_SIZE);
        assert_eq!(buffer.peek(-1) as usize, BUFFER_SIZE - 1);
    }
}
