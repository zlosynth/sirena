//! This module serves as a simple calculator with up to 2 variables on its
//! input.
//!
//! It can be used as a mixer, attenuator, VCO and even an oscillator.

use gazpatcho::config::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::registration::{Module, ModuleInstance};
use crate::samples::{self, Samples};

pub const IN1: &str = "a";
pub const IN2: &str = "b";
pub const FORMULA: &str = "formula";

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
                key: FORMULA.to_owned(),
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
        let formula = data.get(FORMULA).unwrap().unwrap_string();
        if let Ok(formula) = formula.parse::<meval::Expr>() {
            if formula.clone().bind2("a", "b").is_ok() {
                *self.formula.borrow_mut() = formula;
            }
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
    use crate::samples;
    use graphity::Node as _;

    #[test]
    fn instantiate_module() {
        let math = Math;

        let instance =
            <Math as Module<Node, Consumer, Producer>>::instantiate(&math, "id".to_string());

        assert!(instance.widget.is_some());
    }

    #[test]
    fn get_template() {
        let math = Math;

        let template = <Math as Module<Node, Consumer, Producer>>::template(&math);

        assert_eq!(template.class, "math");
    }

    #[test]
    fn access_consumer_in1_by_pin_name() {
        let math = Math;

        let consumer = <Math as Module<Node, Consumer, Producer>>::consumer(&math, IN1);

        assert_eq!(consumer, Consumer::In1);
    }

    #[test]
    fn access_consumer_in2_by_pin_name() {
        let math = Math;

        let consumer = <Math as Module<Node, Consumer, Producer>>::consumer(&math, IN2);

        assert_eq!(consumer, Consumer::In2);
    }

    #[test]
    fn access_producer_by_pin_name() {
        let math = Math;

        let producer =
            <Math as Module<Node, Consumer, Producer>>::producer(&math, "does_not_matter");

        assert_eq!(producer, Producer);
    }

    #[test]
    fn return_0_after_initialization() {
        let math = Math;
        let instance =
            <Math as Module<Node, Consumer, Producer>>::instantiate(&math, "id".to_string());
        let mut node = instance.node;

        node.tick();

        assert_eq!(node.read(Producer)[0], 0.0);
    }

    fn set_formula(widget: &mut dyn crate::registration::Widget, formula: &str) {
        let data: HashMap<_, _> = [(
            FORMULA.to_string(),
            gazpatcho::model::Value::String(formula.to_string()),
        )]
        .iter()
        .cloned()
        .collect();
        widget.update(data);
    }

    #[test]
    fn update_to_a_different_formula() {
        let math = Math;
        let instance =
            <Math as Module<Node, Consumer, Producer>>::instantiate(&math, "id".to_string());
        let mut node = instance.node;
        let mut widget = instance.widget.unwrap();

        set_formula(&mut *widget, "a + b");

        node.write(Consumer::In1, samples::value(1.0));
        node.write(Consumer::In2, samples::value(2.0));
        node.tick();

        assert_eq!(node.read(Producer)[0], 3.0);
    }

    #[test]
    fn return_0_when_set_blank() {
        let math = Math;
        let instance =
            <Math as Module<Node, Consumer, Producer>>::instantiate(&math, "id".to_string());
        let mut node = instance.node;
        let mut widget = instance.widget.unwrap();

        set_formula(&mut *widget, "");

        node.write(Consumer::In1, samples::value(1.0));
        node.write(Consumer::In2, samples::value(2.0));
        node.tick();

        assert_eq!(node.read(Producer)[0], 0.0);
    }

    #[test]
    fn keep_previous_formula_when_the_new_one_has_invalid_syntax() {
        let math = Math;
        let instance =
            <Math as Module<Node, Consumer, Producer>>::instantiate(&math, "id".to_string());
        let mut node = instance.node;
        let mut widget = instance.widget.unwrap();

        set_formula(&mut *widget, "a + b");
        set_formula(&mut *widget, "a ) 10");

        node.write(Consumer::In1, samples::value(1.0));
        node.write(Consumer::In2, samples::value(2.0));
        node.tick();

        assert_eq!(node.read(Producer)[0], 3.0);
    }

    #[test]
    fn keep_previous_formula_when_the_new_one_has_nonexistent_variables() {
        let math = Math;
        let instance =
            <Math as Module<Node, Consumer, Producer>>::instantiate(&math, "id".to_string());
        let mut node = instance.node;
        let mut widget = instance.widget.unwrap();

        set_formula(&mut *widget, "a + b");
        set_formula(&mut *widget, "a + c");

        node.write(Consumer::In1, samples::value(1.0));
        node.write(Consumer::In2, samples::value(2.0));
        node.tick();

        assert_eq!(node.read(Producer)[0], 3.0);
    }

    #[test]
    fn sum_two_values() {
        let formula = Rc::new(RefCell::new("a + b".parse().unwrap()));

        let mut math = Node::new(formula);

        math.write(Consumer::In1, samples::value(1.0));
        math.write(Consumer::In2, samples::value(2.0));
        math.tick();

        assert_eq!(math.read(Producer)[0], 3.0);
    }
}
