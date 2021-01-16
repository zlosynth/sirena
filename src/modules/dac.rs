use gazpatcho::config as c;
use std::collections::HashMap;

use crate::samples::Samples;

pub struct Class;

impl<N, C, P> crate::registration::Module<N, C, P> for Class
where
    N: From<Node>,
    C: From<Consumer>,
    P: From<Producer>,
{
    fn instantiate(
        &self,
        _id: String,
        _data: HashMap<String, gazpatcho::model::Value>,
    ) -> (Box<dyn crate::Widget>, N) {
        (Box::new(Module), Node::default().into())
    }

    fn template(&self) -> c::NodeTemplate {
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

    fn consumer(&self, _class: &str) -> C {
        Consumer.into()
    }

    fn producer(&self, _class: &str) -> P {
        Producer.into()
    }
}

pub struct Module;

impl crate::registration::Widget for Module {}

#[derive(Default)]
pub struct Node {
    values: Samples,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Consumer;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Producer;

impl graphity::Node<Samples> for Node {
    type Consumer = Consumer;
    type Producer = Producer;

    fn write(&mut self, _consumer: Consumer, data: Samples) {
        self.values = data;
    }

    fn read(&self, _producer: Producer) -> Samples {
        self.values
    }
}
