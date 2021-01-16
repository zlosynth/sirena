use crate::samples::Samples;
use crate::ui::template::*;

pub struct Module;

impl<N, C, P> crate::registration::Module<N, C, P> for Module
where
    N: From<Node>,
    C: From<Consumer>,
    P: From<Producer>,
{
    fn instantiate(&self, _id: String) -> crate::registration::ModuleInstance<N> {
        let node = Node::new();
        crate::registration::ModuleInstance::new(node.into())
    }

    fn template(&self) -> NodeTemplate {
        NodeTemplate {
            label: "DAC".to_owned(),
            class: "dac".to_owned(),
            display_heading: true,
            pins: vec![Pin {
                label: "In".to_owned(),
                class: "in".to_owned(),
                direction: Input,
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

    fn write(&mut self, _consumer: Consumer, data: Samples) {
        self.values = data;
    }

    fn read(&self, _producer: Producer) -> Samples {
        self.values
    }
}
