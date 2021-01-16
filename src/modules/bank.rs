//! Bank provides a simple way to store written data and offer them to readers.

use crate::samples::Samples;

#[derive(Default)]
pub struct Node {
    values: Samples,
}

impl Node {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Consumer;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Producer;

impl graphity::Node<Samples> for Node {
    type Consumer = Consumer;
    type Producer = Producer;

    fn write(&mut self, _input: Consumer, data: Samples) {
        self.values = data;
    }

    fn read(&self, _output: Producer) -> Samples {
        self.values
    }
}
