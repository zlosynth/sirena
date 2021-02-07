use gazpatcho::config::*;

use crate::filters::comb_filter;
use crate::registration::{Module, ModuleInstance};
use crate::samples::{self, Samples};

pub struct CombFilter;

impl<N, C, P> Module<N, C, P> for CombFilter
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
            label: "Comb Filter".to_owned(),
            class: "comb_filter".to_owned(),
            display_heading: true,
            pins: vec![
                Pin {
                    label: "In".to_owned(),
                    class: "in".to_owned(),
                    direction: Input,
                },
                Pin {
                    label: "Delay".to_owned(),
                    class: "delay".to_owned(),
                    direction: Input,
                },
                Pin {
                    label: "Gain".to_owned(),
                    class: "gain".to_owned(),
                    direction: Input,
                },
                Pin {
                    label: "Out".to_owned(),
                    class: "out".to_owned(),
                    direction: Output,
                },
            ],
            widgets: vec![],
        }
    }

    fn consumer(&self, class: &str) -> C {
        match class {
            "in" => Consumer::Input.into(),
            "delay" => Consumer::Delay.into(),
            "gain" => Consumer::Gain.into(),
            _ => unreachable!(),
        }
    }

    fn producer(&self, _class: &str) -> P {
        Producer.into()
    }
}

pub struct Node {
    comb_filter: comb_filter::CombFilter,
    input: Samples,
    delay: Samples,
    gain: Samples,
    out: Samples,
}

#[allow(clippy::new_without_default)]
impl Node {
    pub fn new() -> Self {
        Self {
            comb_filter: comb_filter::CombFilter::new(),
            input: samples::zeroed(),
            delay: samples::zeroed(),
            gain: samples::zeroed(),
            out: samples::zeroed(),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Consumer {
    Input,
    Delay,
    Gain,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Producer;

impl graphity::Node<Samples> for Node {
    type Consumer = Consumer;
    type Producer = Producer;

    fn write(&mut self, consumer: Consumer, data: Samples) {
        match consumer {
            Consumer::Input => self.input = data,
            Consumer::Delay => self.delay = data,
            Consumer::Gain => self.gain = data,
        }
    }

    fn read(&self, _producer: Producer) -> Samples {
        self.out
    }

    fn tick(&mut self) {
        self.comb_filter.set_gain(self.gain[0]);
        self.comb_filter
            .set_delay((self.delay[0] * crate::SAMPLE_RATE as f32) as usize);
        self.out = self.input;
        self.comb_filter.process(&mut self.out);
    }
}