// TODO: The maths module should understand nodes. C4 ...
use std::cell::RefCell;
use std::rc::Rc;

use gazpatcho::config as c;
use graphity::Node;
use graphity::{node::ConsumerIndex, node::ProducerIndex, NodeIndex};

pub const CLASS: &str = "math";
pub const IN1: &str = "x";
pub const IN2: &str = "y";
pub const OUT: &str = "out";

pub struct Math {
    formula: Rc<RefCell<meval::Expr>>,
    in1: [f32; 32],
    in2: [f32; 32],
    out: [f32; 32],
}

impl Math {
    pub fn new(formula: Rc<RefCell<meval::Expr>>) -> Self {
        Self {
            formula,
            in1: [0.0; 32],
            in2: [0.0; 32],
            out: [0.0; 32],
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Input {
    In1,
    In2,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Output;

impl Node<[f32; 32]> for Math {
    type Consumer = Input;
    type Producer = Output;

    fn write(&mut self, input: Input, data: [f32; 32]) {
        match input {
            Input::In1 => self.in1 = data,
            Input::In2 => self.in2 = data,
        }
    }

    fn read(&self, _output: Output) -> [f32; 32] {
        self.out
    }

    fn tick(&mut self) {
        let formula = self.formula.borrow().clone();
        if let Ok(formula) = formula.bind2("x", "y") {
            for (i, x) in self.in1.iter().enumerate() {
                self.out[i] = formula(*x as f64, self.in2[i] as f64) as f32;
            }
        }
    }
}

pub fn template() -> c::NodeTemplate {
    c::NodeTemplate {
        label: "Math".to_owned(),
        class: CLASS.to_owned(),
        display_heading: false,
        pins: vec![
            c::Pin {
                label: "x".to_owned(),
                class: IN1.to_owned(),
                direction: c::Input,
            },
            c::Pin {
                label: "y".to_owned(),
                class: IN2.to_owned(),
                direction: c::Input,
            },
            c::Pin {
                label: "Out".to_owned(),
                class: OUT.to_owned(),
                direction: c::Output,
            },
        ],
        widgets: vec![c::TextBox {
            key: "formula".to_owned(),
            capacity: 1000,
            size: [200.0, 23.0],
            read_only: false,
        }],
    }
}

pub fn producer<NI>(source: &NI, pin_class: &str) -> <NI as NodeIndex>::ProducerIndex
where
    NI: NodeIndex,
    NI::Producer: std::convert::From<Output>,
{
    match pin_class {
        "out" => source.producer(Output),
        &_ => unreachable!(),
    }
}

pub fn consumer<NI>(destination: &NI, pin_class: &str) -> <NI as NodeIndex>::ConsumerIndex
where
    NI: NodeIndex,
    NI::Consumer: std::convert::From<Input>,
{
    match pin_class {
        IN1 => destination.consumer(Input::In1),
        IN2 => destination.consumer(Input::In2),
        &_ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum2() {
        let formula = Rc::new(RefCell::new("x + y".parse().unwrap()));

        let mut math = Math::new(formula);

        math.write(Input::In1, [1.0; 32]);
        math.write(Input::In2, [2.0; 32]);
        math.tick();

        assert_eq!(math.read(Output)[0], 3.0);
    }
}
