use graphity::Node;

#[derive(Default)]
pub struct Bank {
    values: [f32; 32],
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Input;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Output;

impl Node<[f32; 32]> for Bank {
    type Consumer = Input;
    type Producer = Output;

    fn write(&mut self, _input: Input, data: [f32; 32]) {
        self.values = data;
    }

    fn read(&self, _output: Output) -> [f32; 32] {
        self.values
    }
}
