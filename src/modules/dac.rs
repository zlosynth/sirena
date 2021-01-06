use gazpatcho::config as c;
use graphity::Node;

#[derive(Default)]
pub struct DAC {
    values: [f32; 32],
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Input;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Output;

impl Node<[f32; 32]> for DAC {
    type Consumer = Input;
    type Producer = Output;

    fn write(&mut self, _input: Input, data: [f32; 32]) {
        self.values = data;
    }

    fn read(&self, _output: Output) -> [f32; 32] {
        self.values
    }
}

pub fn template() -> c::NodeTemplate {
    c::NodeTemplate {
        label: "DAC".to_owned(),
        class: "dac".to_owned(),
        display_heading: true,
        pins: vec![c::Pin {
            label: "In".to_owned(),
            class: "in".to_owned(),
            direction: c::Input,
        }],
        widgets: vec![],
    }
}
