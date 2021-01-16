use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gazpatcho::config::*;

use crate::registration::{Module, ModuleInstance};
use crate::samples::{self, Samples};

pub const IN1: &str = "a";
pub const IN2: &str = "b";

pub struct Math;

impl<N, C, P> Module<N, C, P> for Math
where
    N: From<Node>,
    C: From<Consumer>,
    P: From<Producer>,
{
    fn instantiate(&self, _id: String) -> ModuleInstance<N> {
        let formula = Rc::new(RefCell::new("0".parse().unwrap()));
        let node = Node::new(Rc::clone(&formula)).into();
        let widget = Box::new(Widget { formula });
        ModuleInstance::new(node).with_widget(widget)
    }

    fn template(&self) -> NodeTemplate {
        NodeTemplate {
            label: "Math".to_owned(),
            class: "math".to_owned(),
            display_heading: false,
            pins: vec![
                Pin {
                    label: "a".to_owned(),
                    class: IN1.to_owned(),
                    direction: Input,
                },
                Pin {
                    label: "b".to_owned(),
                    class: IN2.to_owned(),
                    direction: Input,
                },
                Pin {
                    label: "Out".to_owned(),
                    class: "out".to_owned(),
                    direction: Output,
                },
            ],
            widgets: vec![TextBox {
                key: "formula".to_owned(),
                capacity: 200,
                size: [200.0, 23.0],
                read_only: false,
            }],
        }
    }

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

pub struct Widget {
    formula: Rc<RefCell<meval::Expr>>,
}

impl crate::registration::Widget for Widget {
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
