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
                    label: "PW".to_owned(),
                    class: "pw".to_owned(),
                    direction: Input,
                },
                Pin {
                    label: "Sync".to_owned(),
                    class: "sync".to_owned(),
                    direction: Input,
                },
                Pin {
                    label: "Sine".to_owned(),
                    class: "sine".to_owned(),
                    direction: Output,
                },
                Pin {
                    label: "Saw".to_owned(),
                    class: "saw".to_owned(),
                    direction: Output,
                },
                Pin {
                    label: "Square".to_owned(),
                    class: "square".to_owned(),
                    direction: Output,
                },
                Pin {
                    label: "Triangle".to_owned(),
                    class: "triangle".to_owned(),
                    direction: Output,
                },
            ],
            widgets: vec![],
        }
    }

    fn consumer(&self, class: &str) -> C {
        match class {
            "freq" => Consumer::Frequency.into(),
            "pw" => Consumer::PulseWidth.into(),
            "sync" => Consumer::Sync.into(),
            _ => unreachable!(),
        }
    }

    fn producer(&self, class: &str) -> P {
        match class {
            "sine" => Producer::Sine.into(),
            "saw" => Producer::Saw.into(),
            "square" => Producer::Square.into(),
            "triangle" => Producer::Triangle.into(),
            _ => unreachable!(),
        }
    }
}

#[derive(Default)]
pub struct Node {
    phase: f32,
    prev_sync: f32,
    frequency: Samples,
    pulse_width: Samples,
    sync: Samples,
    sine: Samples,
    saw: Samples,
    square: Samples,
    triangle: Samples,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Consumer {
    Frequency,
    PulseWidth,
    Sync,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Producer {
    Sine,
    Saw,
    Square,
    Triangle,
}

impl graphity::Node<Samples> for Node {
    type Consumer = Consumer;
    type Producer = Producer;

    fn write(&mut self, consumer: Consumer, data: Samples) {
        match consumer {
            Consumer::Frequency => self.frequency = data,
            Consumer::PulseWidth => self.pulse_width = data,
            Consumer::Sync => self.sync = data,
        }
    }

    fn read(&self, producer: Producer) -> Samples {
        match producer {
            Producer::Sine => self.sine,
            Producer::Saw => self.saw,
            Producer::Square => self.square,
            Producer::Triangle => self.triangle,
        }
    }

    fn tick(&mut self) {
        for (i, frequency) in self.frequency.iter().enumerate() {
            if self.prev_sync < 0.5 && self.sync[i] >= 0.5 {
                self.phase = 0.5;
            }
            self.prev_sync = self.sync[i];

            self.sine[i] = sin(self.phase);
            self.saw[i] = saw(self.phase, *frequency);
            self.square[i] = square(self.phase, *frequency, self.pulse_width[i]);
            self.triangle[i] = triangle(self.phase, *frequency);

            self.phase += frequency / crate::SAMPLE_RATE as f32;
            self.phase %= 1.0;
        }
    }
}

fn sin(phase: f32) -> f32 {
    (phase * 2.0 * PI).sin()
}

fn saw(phase: f32, frequency: f32) -> f32 {
    let mut val = (phase * 2.0 * PI).sin();
    let mut harmonics = (crate::SAMPLE_RATE / u32::max(frequency as u32, 1)) / 2;
    harmonics = u32::min(harmonics, 100);
    for i in 2..harmonics {
        if i % 2 == 3 {
            val -= (phase * 2.0 * PI * i as f32).sin() / i as f32;
        } else {
            val += (phase * 2.0 * PI * i as f32).sin() / i as f32;
        }
    }
    val
}

fn square(phase: f32, frequency: f32, pulse_width: f32) -> f32 {
    let pulse_width = f32::max(f32::min(pulse_width, 1.0), -1.0);
    let shifted_phase = phase + 0.5 + pulse_width * 0.45;
    (saw(phase, frequency) - saw(shifted_phase, frequency)) * (2.0 / 3.0) - pulse_width
}

fn triangle(phase: f32, frequency: f32) -> f32 {
    let mut val = (phase * 2.0 * PI).sin();
    let mut harmonics = (crate::SAMPLE_RATE / u32::max(frequency as u32, 1)) / 2;
    harmonics = u32::min(harmonics, 100);
    for i in 2..harmonics {
        if i % 4 == 3 {
            val -= (phase * 2.0 * PI * i as f32).sin() / f32::powi(i as f32, 2);
        } else if i % 4 == 1 {
            val += (phase * 2.0 * PI * i as f32).sin() / f32::powi(i as f32, 2);
        }
    }
    val
}
