// TODO: Add PWM, waveform and sync

use core::f32::consts::PI;
use gazpatcho::config as c;
use std::collections::HashMap;

pub struct Class;

impl<N, C, P> crate::registration::ModuleClass<N, C, P> for Class
where
    N: From<Node>,
    C: From<Consumer>,
    P: From<Producer>,
{
    fn instantiate(
        &self,
        _data: HashMap<String, gazpatcho::model::Value>,
    ) -> Box<dyn crate::Module<N>> {
        Box::new(Module)
    }

    fn template(&self) -> c::NodeTemplate {
        c::NodeTemplate {
            label: "VCO".to_owned(),
            class: "vco".to_owned(),
            display_heading: true,
            pins: vec![
                c::Pin {
                    label: "Freq".to_owned(),
                    class: "freq".to_owned(),
                    direction: c::Input,
                },
                c::Pin {
                    label: "Out".to_owned(),
                    class: "out".to_owned(),
                    direction: c::Output,
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
    phase: f32,
    frequency: [f32; 32],
    result: [f32; 32],
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Consumer {
    Frequency,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Producer;

impl graphity::Node<[f32; 32]> for Node {
    type Consumer = Consumer;
    type Producer = Producer;

    fn write(&mut self, _consumer: Consumer, data: [f32; 32]) {
        self.frequency = data;
    }

    fn read(&self, _producer: Producer) -> [f32; 32] {
        self.result
    }

    fn tick(&mut self) {
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
