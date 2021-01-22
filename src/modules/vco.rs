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
    frequency: Samples,
    pulse_width: Samples,
    sine: Samples,
    saw: Samples,
    square: Samples,
    triangle: Samples,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Consumer {
    Frequency,
    PulseWidth,
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
        self.phase %= f32::powi(2.0, 24);
        for (i, frequency) in self.frequency.iter().enumerate() {
            let phase = self.phase / crate::SAMPLE_RATE as f32;
            self.sine[i] = sin(phase, *frequency);
            self.saw[i] = saw(phase, *frequency);
            self.square[i] = square(phase, *frequency, self.pulse_width[i]);
            self.triangle[i] = triangle(phase, *frequency);
            self.phase += 1.0;
        }
    }
}

fn sin(phase: f32, frequency: f32) -> f32 {
    (phase * frequency * 2.0 * PI).sin()
}

fn saw(phase: f32, frequency: f32) -> f32 {
    let mut val = (phase * frequency * 2.0 * PI).sin();
    let mut harmonics = crate::SAMPLE_RATE / u32::max(frequency as u32, 1) / 3 - 1;
    harmonics = u32::min(harmonics, 100);
    for i in 2..harmonics {
        if i % 2 == 3 {
            val -= (phase * frequency * 2.0 * PI * i as f32).sin() / i as f32;
        } else {
            val += (phase * frequency * 2.0 * PI * i as f32).sin() / i as f32;
        }
    }
    val
}

fn square(phase: f32, frequency: f32, pulse_width: f32) -> f32 {
    let pulse_width = f32::max(f32::min(pulse_width + 1.0, 2.0), 0.0) * 0.9 + 0.1;
    let shifted_phase = phase + ((1.0 / frequency) / 2.0) * (2.0 - pulse_width);
    (saw(phase, frequency) - saw(shifted_phase, frequency)) * (2.0 / 3.0) + (pulse_width - 1.0)
}

fn triangle(phase: f32, frequency: f32) -> f32 {
    let mut val = (phase * frequency * 2.0 * PI).sin();
    let mut harmonics = crate::SAMPLE_RATE / u32::max(frequency as u32, 1) / 3 - 1;
    harmonics = u32::min(harmonics, 100);
    for i in 2..harmonics {
        if i % 4 == 3 {
            val -= (phase * frequency * 2.0 * PI * i as f32).sin() / f32::powi(i as f32, 2);
        } else if i % 4 == 1 {
            val += (phase * frequency * 2.0 * PI * i as f32).sin() / f32::powi(i as f32, 2);
        }
    }
    val
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
