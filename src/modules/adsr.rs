use gazpatcho::config::*;

use crate::registration::{Module, ModuleInstance};
use crate::samples::{self, Samples};

pub struct ADSR;

impl<N, C, P> Module<N, C, P> for ADSR
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
            label: "ADSR".to_owned(),
            class: "adsr".to_owned(),
            display_heading: true,
            pins: vec![
                Pin {
                    label: "Gate".to_owned(),
                    class: "gate".to_owned(),
                    direction: Input,
                },
                Pin {
                    label: "A".to_owned(),
                    class: "attack".to_owned(),
                    direction: Input,
                },
                Pin {
                    label: "D".to_owned(),
                    class: "decay".to_owned(),
                    direction: Input,
                },
                Pin {
                    label: "S".to_owned(),
                    class: "sustain".to_owned(),
                    direction: Input,
                },
                Pin {
                    label: "R".to_owned(),
                    class: "release".to_owned(),
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
            "gate" => Consumer::Gate.into(),
            "attack" => Consumer::Attack.into(),
            "decay" => Consumer::Decay.into(),
            "sustain" => Consumer::Sustain.into(),
            "release" => Consumer::Release.into(),
            _ => unreachable!(),
        }
    }

    fn producer(&self, _class: &str) -> P {
        Producer.into()
    }
}

#[derive(Default)]
pub struct Node {
    phase: Phase,
    value: f32,
    gate: Samples,
    attack: Samples,
    decay: Samples,
    sustain: Samples,
    release: Samples,
    out: Samples,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Consumer {
    Gate,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Producer;

impl graphity::Node<Samples> for Node {
    type Consumer = Consumer;
    type Producer = Producer;

    fn write(&mut self, consumer: Consumer, data: Samples) {
        match consumer {
            Consumer::Gate => self.gate = data,
            Consumer::Attack => self.attack = data,
            Consumer::Decay => self.decay = data,
            Consumer::Sustain => self.sustain = data,
            Consumer::Release => self.release = data,
        }
    }

    fn read(&self, _producer: Producer) -> Samples {
        self.out
    }

    fn tick(&mut self) {
        for (i, out) in self.out.iter_mut().enumerate() {
            match self.phase {
                Phase::Idle => {
                    if self.gate[i] > 0.1 {
                        self.phase = Phase::Attack;
                    }
                }
                Phase::Attack => {
                    self.value += 1.0 / (self.attack[i] * crate::SAMPLE_RATE as f32);
                    if self.value >= 1.0 {
                        self.value = 1.0;
                        self.phase = Phase::Decay;
                    }
                }
                Phase::Decay => {
                    self.value -= 1.0 / (self.decay[i] * crate::SAMPLE_RATE as f32);
                    if self.value <= self.sustain[i] {
                        self.value = self.sustain[i];
                        self.phase = Phase::Sustain;
                    }
                }
                Phase::Sustain => {
                    if self.gate[i] <= 0.1 {
                        self.phase = Phase::Release;
                    }
                }
                Phase::Release => {
                    if self.gate[i] > 0.1 {
                        self.phase = Phase::Attack;
                    } else {
                        self.value -= 1.0 / (self.release[i] * crate::SAMPLE_RATE as f32);
                        if self.value <= 0.0 {
                            self.phase = Phase::Idle;
                            self.value = 0.0;
                        }
                    }
                }
            }
            *out = self.value;
        }

        self.sustain = samples::value(1.0);
    }
}

#[derive(PartialEq)]
enum Phase {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

impl Default for Phase {
    fn default() -> Phase {
        Phase::Idle
    }
}
