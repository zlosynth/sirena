// TODO: Add PWM, waveform and sync

use core::f32::consts::PI;
use gazpatcho::config as c;
use graphity::Node;

#[derive(Default)]
pub struct VCO {
    phase: f32,
    frequency: [f32; 32],
    result: [f32; 32],
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Input {
    Frequency,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Output;

impl Node<[f32; 32]> for VCO {
    type Consumer = Input;
    type Producer = Output;

    fn write(&mut self, _input: Input, data: [f32; 32]) {
        self.frequency = data;
    }

    fn read(&self, _output: Output) -> [f32; 32] {
        self.result
    }

    fn tick(&mut self) {
        for (i, result) in self.result.iter_mut().enumerate() {
            *result = sin(self.phase / 44800.0, self.frequency[i]);
            self.phase = self.phase + 1.0;
        }
    }
}

fn sin(phase: f32, frequency: f32) -> f32 {
    (phase * frequency * 2.0 * PI).sin()
}

pub fn template() -> c::NodeTemplate {
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
