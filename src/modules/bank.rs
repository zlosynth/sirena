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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::samples;
    use graphity::Node as _;

    #[test]
    fn what_was_written_can_be_read_back() {
        let mut bank = Node::new();

        bank.write(Consumer, samples::value(1.0));

        assert_eq!(bank.read(Producer), samples::value(1.0));
    }
}
