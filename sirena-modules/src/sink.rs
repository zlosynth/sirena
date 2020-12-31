use graphity::Node;
use sirena::Buffer;

#[derive(Default)]
pub struct Sink(Buffer);

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Input;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Output;

impl Node<Buffer> for Sink {
    type Consumer = Input;
    type Producer = Output;

    fn write(&mut self, _input: Input, data: Buffer) {
        self.0 = data;
    }

    fn read(&self, _output: Output) -> Buffer {
        self.0
    }
}
