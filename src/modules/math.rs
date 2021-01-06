use std::cell::RefCell;
use std::rc::Rc;

use gazpatcho::config as c;
use graphity::Node;

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
        class: "math".to_owned(),
        display_heading: false,
        pins: vec![
            c::Pin {
                label: "x".to_owned(),
                class: "x".to_owned(),
                direction: c::Input,
            },
            c::Pin {
                label: "y".to_owned(),
                class: "y".to_owned(),
                direction: c::Input,
            },
            c::Pin {
                label: "Out".to_owned(),
                class: "out".to_owned(),
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
