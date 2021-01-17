use core::f32::consts::PI;
use gazpatcho::config::*;

use crate::registration::{Module, ModuleInstance};
use crate::samples::Samples;

pub struct VCO;

impl<N, C, P> Module<N, C, P> for VCO
where
    N: From<Node>,
    C: From<Consumer>,
    P: From<Producer>,
{
    fn instantiate(&self, _id: String) -> ModuleInstance<N> {
        ModuleInstance::new(Node::default().into())
    }

    fn template(&self) -> NodeTemplate {
        NodeTemplate {
            label: "VCO".to_owned(),
            class: "vco".to_owned(),
            display_heading: true,
            pins: vec![
                Pin {
                    label: "Freq".to_owned(),
                    class: "freq".to_owned(),
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

    fn consumer(&self, _class: &str) -> C {
        Consumer::Frequency.into()
    }

    fn producer(&self, _class: &str) -> P {
        Producer.into()
    }
}

#[derive(Default)]
pub struct Node {
    phase: f32,
    frequency: Samples,
    result: Samples,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Consumer {
    Frequency,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Producer;

impl graphity::Node<Samples> for Node {
    type Consumer = Consumer;
    type Producer = Producer;

    fn write(&mut self, _consumer: Consumer, data: Samples) {
        self.frequency = data;
    }

    fn read(&self, _producer: Producer) -> Samples {
        self.result
    }

    fn tick(&mut self) {
        self.phase %= f32::powi(2.0, 24);
        for (i, result) in self.result.iter_mut().enumerate() {
            *result = sin(self.phase / 48000.0, self.frequency[i]);
            self.phase += 1.0;
        }
    }
}

fn sin(phase: f32, frequency: f32) -> f32 {
    (phase * frequency * 2.0 * PI).sin()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_sin() {
        assert_relative_eq!(sin(0.0, 1.0), 0.0);
        assert_relative_eq!(sin(0.25, 1.0), 1.0);
        assert_relative_eq!(sin(0.5, 1.0), 0.0);
        assert_relative_eq!(sin(0.75, 1.0), -1.0);
        assert_abs_diff_eq!(sin(1.0, 1.0), 0.0, epsilon = 0.001);
    }
}
