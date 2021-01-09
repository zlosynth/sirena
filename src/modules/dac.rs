use gazpatcho::config as c;

pub struct Class;

impl<N, C, P> crate::registration::ModuleClass<N, C, P> for Class
where
    N: From<Node>,
    C: From<Consumer>,
    P: From<Producer>,
{
    fn instantiate(&self) -> Box<dyn crate::Module<N>> {
        Box::new(Module)
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

impl<N> crate::registration::Module<N> for Module
where
    N: From<Node>,
{
    fn take_node(&mut self) -> N {
        Node::default().into()
    }
}

#[derive(Default)]
pub struct Node {
    values: [f32; 32],
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Consumer;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Producer;

impl graphity::Node<[f32; 32]> for Node {
    type Consumer = Consumer;
    type Producer = Producer;

    fn write(&mut self, _consumer: Consumer, data: [f32; 32]) {
        self.values = data;
    }

    fn read(&self, _producer: Producer) -> [f32; 32] {
        self.values
    }
}
