// TODO: The maths module should understand nodes. C4 ...
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gazpatcho::config as c;
use graphity::{node::ConsumerIndex, node::ProducerIndex, NodeIndex};

pub const CLASS: &str = "math";
pub const IN1: &str = "x";
pub const IN2: &str = "y";
pub const OUT: &str = "out";

pub struct Class;

impl<N, C, P> crate::registration::ModuleClass<N, C, P> for Class
where
    N: From<Node>,
    C: From<Consumer>,
    P: From<Producer>,
{
    fn instantiate(
        &self,
        data: HashMap<String, gazpatcho::model::Value>,
    ) -> Box<dyn crate::Module<N>> {
        let formula = data.get("formula").unwrap().unwrap_string();
        let formula = if let Ok(formula) = formula.parse() {
            formula
        } else {
            "0".parse().unwrap()
        };
        let formula = Rc::new(RefCell::new(formula));
        Box::new(Module { formula })
    }

    fn template(&self) -> c::NodeTemplate {
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

    // TODO: Implement From<&str> trait on Input
    fn consumer(&self, class: &str) -> C {
        match class {
            "x" => Consumer::In1.into(),
            "y" => Consumer::In2.into(),
            _ => unreachable!(),
        }
    }

    fn producer(&self, class: &str) -> P {
        Producer.into()
    }
}

pub struct Module {
    formula: Rc<RefCell<meval::Expr>>,
}

impl<N> crate::registration::Module<N> for Module
where
    N: From<Node>,
{
    fn take_node(&mut self) -> N {
        Node::new(Rc::clone(&self.formula)).into()
    }

    fn update(&mut self, data: HashMap<String, gazpatcho::model::Value>) {
        let formula = data.get("formula").unwrap().unwrap_string();
        if let Ok(formula) = formula.parse() {
            *self.formula.borrow_mut() = formula;
        }
    }
}

pub struct Node {
    formula: Rc<RefCell<meval::Expr>>,
    in1: [f32; 32],
    in2: [f32; 32],
    out: [f32; 32],
}

impl Node {
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
pub enum Consumer {
    In1,
    In2,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Producer;

impl graphity::Node<[f32; 32]> for Node {
    type Consumer = Consumer;
    type Producer = Producer;

    fn write(&mut self, consumer: Consumer, data: [f32; 32]) {
        match consumer {
            Consumer::In1 => self.in1 = data,
            Consumer::In2 => self.in2 = data,
        }
    }

    fn read(&self, _producer: Producer) -> [f32; 32] {
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

#[cfg(test)]
mod tests {
    use super::*;
    use graphity::Node as _;

    #[test]
    fn sum2() {
        let formula = Rc::new(RefCell::new("x + y".parse().unwrap()));

        let mut math = Node::new(formula);

        math.write(Consumer::In1, [1.0; 32]);
        math.write(Consumer::In2, [2.0; 32]);
        math.tick();

        assert_eq!(math.read(Producer)[0], 3.0);
    }
}
