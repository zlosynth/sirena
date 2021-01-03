use core::f32::consts::PI;

use graphity::Node;
use sirena::{Buffer, SAMPLE_RATE};

#[derive(Default)]
pub struct Oscillator {
    phase: f32,
    frequency: Buffer,
    result: Buffer,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Input;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Output;

impl Node<Buffer> for Oscillator {
    type Consumer = Input;
    type Producer = Output;

    fn write(&mut self, _input: Input, data: Buffer) {
        self.frequency = data;
    }

    fn read(&self, _output: Output) -> Buffer {
        self.result
    }

    fn tick(&mut self) {
        for (i, result) in self.result.iter_mut().enumerate() {
            *result = sin(self.phase, self.frequency[i]);
            self.phase = self.phase + 1.0;
        }
    }
}

fn sin(phase: f32, frequency: f32) -> f32 {
    (phase * frequency * 2.0 * PI / SAMPLE_RATE).sin()
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
