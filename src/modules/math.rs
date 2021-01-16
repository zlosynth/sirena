// TODO: The maths module should understand nodes. C4 ...
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gazpatcho::config as c;

use crate::samples::{self, Samples};

pub const CLASS: &str = "math";
pub const IN1: &str = "a";
pub const IN2: &str = "b";
pub const OUT: &str = "out";

pub struct Class;

impl<N, C, P> crate::registration::Module<N, C, P> for Class
where
    N: From<Node>,
    C: From<Consumer>,
    P: From<Producer>,
{
    fn instantiate(&self, _id: String) -> crate::registration::ModuleInstance<N> {
        let formula = Rc::new(RefCell::new("0".parse().unwrap()));
        crate::registration::ModuleInstance::new(
            Node::new(Rc::clone(&formula)).into(),
            Box::new(Module { formula }),
        )
    }

    fn template(&self) -> c::NodeTemplate {
        c::NodeTemplate {
            label: "Math".to_owned(),
            class: CLASS.to_owned(),
            display_heading: false,
            pins: vec![
                c::Pin {
                    label: "a".to_owned(),
                    class: IN1.to_owned(),
                    direction: c::Input,
                },
                c::Pin {
                    label: "b".to_owned(),
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
                capacity: 200,
                size: [200.0, 23.0],
                read_only: false,
            }],
        }
    }

    // TODO: Implement From<&str> trait on Input
    fn consumer(&self, class: &str) -> C {
        match class {
            IN1 => Consumer::In1.into(),
            IN2 => Consumer::In2.into(),
            _ => unreachable!(),
        }
    }

    fn producer(&self, _class: &str) -> P {
        Producer.into()
    }
}

pub struct Module {
    formula: Rc<RefCell<meval::Expr>>,
}

impl crate::registration::Widget for Module {
    fn update(&mut self, data: HashMap<String, gazpatcho::model::Value>) {
        let formula = data.get("formula").unwrap().unwrap_string();
        if let Ok(formula) = formula.parse() {
            *self.formula.borrow_mut() = formula;
        }
    }
}

pub struct Node {
    formula: Rc<RefCell<meval::Expr>>,
    in1: Samples,
    in2: Samples,
    out: Samples,
}

impl Node {
    pub fn new(formula: Rc<RefCell<meval::Expr>>) -> Self {
        Self {
            formula,
            in1: samples::zeroed(),
            in2: samples::zeroed(),
            out: samples::zeroed(),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Consumer {
    In1,
    In2,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Producer;

impl graphity::Node<Samples> for Node {
    type Consumer = Consumer;
    type Producer = Producer;

    fn write(&mut self, consumer: Consumer, data: Samples) {
        match consumer {
            Consumer::In1 => self.in1 = data,
            Consumer::In2 => self.in2 = data,
        }
    }

    fn read(&self, _producer: Producer) -> Samples {
        self.out
    }

    fn tick(&mut self) {
        let formula = self.formula.borrow().clone();
        if let Ok(formula) = formula.bind2("a", "b") {
            for (i, x) in self.in1.iter().enumerate() {
                self.out[i] = formula(*x as f64, self.in2[i] as f64) as f32;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use graphity::Node as _;

    #[test]
    fn sum2() {
        let formula = Rc::new(RefCell::new("a + b".parse().unwrap()));

        let mut math = Node::new(formula);

        math.write(Consumer::In1, samples::value(1.0));
        math.write(Consumer::In2, samples::value(2.0));
        math.tick();

        assert_eq!(math.read(Producer)[0], 3.0);
    }
}
