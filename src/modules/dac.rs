//! DAC module represents the final audio sink in the UI.
//!
//! It is a visual representation of the system output. It does not handle the
//! playback on its own though. Instead, it is expected to be wired up
//! internally into the [`Bank`](../../bank/index.html) node and collected from
//! there, to be fed into the playback.

use gazpatcho::config::*;

use crate::registration::{Module, ModuleInstance};
use crate::samples::Samples;

pub struct DAC;

impl<N, C, P> Module<N, C, P> for DAC
where
    N: From<Node>,
    C: From<Consumer>,
    P: From<Producer>,
{
    fn instantiate(&self, _id: String) -> ModuleInstance<N> {
        ModuleInstance::new(Node::new().into())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::samples;
    use graphity::Node as _;

    #[test]
    fn instantiate_module() {
        let dac = DAC;

        let instance =
            <DAC as Module<Node, Consumer, Producer>>::instantiate(&dac, "id".to_string());

        assert!(instance.widget.is_none());
    }

    #[test]
    fn get_template() {
        let dac = DAC;

        let template = <DAC as Module<Node, Consumer, Producer>>::template(&dac);

        assert_eq!(template.class, "dac");
    }

    #[test]
    fn access_consumer_by_pin_name() {
        let dac = DAC;

        let consumer = <DAC as Module<Node, Consumer, Producer>>::consumer(&dac, "in");

        assert_eq!(consumer, Consumer);
    }

    #[test]
    fn access_producer_by_pin_name() {
        let dac = DAC;

        let producer = <DAC as Module<Node, Consumer, Producer>>::producer(&dac, "does_not_matter");

        assert_eq!(producer, Producer);
    }

    #[test]
    fn instantiate_node() {
        let _node = Node::new();
    }

    #[test]
    fn write_into_node() {
        let mut node = Node::new();

        node.write(Consumer, samples::value(1.0));
    }

    #[test]
    fn read_from_node() {
        let node = Node::new();

        assert_eq!(node.read(Producer), samples::zeroed());
    }

    #[test]
    fn read_data_written_to_node() {
        let mut node = Node::new();

        node.write(Consumer, samples::value(1.0));
        let data = node.read(Producer);

        assert_eq!(data, samples::value(1.0));
    }
}
