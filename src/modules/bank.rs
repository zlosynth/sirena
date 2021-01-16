use graphity::Node;

#[derive(Default)]
pub struct Bank {
    values: [f32; 32],
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Consumer;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Producer;

impl Node<[f32; 32]> for Bank {
    type Consumer = Consumer;
    type Producer = Producer;

    fn write(&mut self, _input: Consumer, data: [f32; 32]) {
        self.values = data;
    }

    fn read(&self, _output: Producer) -> [f32; 32] {
        self.values
    }
}
