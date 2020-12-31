use graphity::Node;
use sirena::{Buffer, BUFFER_SIZE};

pub struct Generator(Buffer);

impl Generator {
    pub fn new(value: f32) -> Self {
        Self([value; BUFFER_SIZE])
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Input {}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Output;

impl Node<Buffer> for Generator {
    type Consumer = Input;
    type Producer = Output;

    fn read(&self, _output: Output) -> Buffer {
        self.0
    }
}
