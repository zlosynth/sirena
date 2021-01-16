use graphity::Node;

use crate::samples::Samples;

#[derive(Default)]
pub struct Bank {
    values: Samples,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Consumer;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Producer;

impl Node<Samples> for Bank {
    type Consumer = Consumer;
    type Producer = Producer;

    fn write(&mut self, _input: Consumer, data: Samples) {
        self.values = data;
    }

    fn read(&self, _output: Producer) -> Samples {
        self.values
    }
}
