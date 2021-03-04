use crate::wavetable_oscillator::{CircularWavetableOscillator, Oscillator, Wavetable};

pub struct Osc2<'a> {
    oscillator: CircularWavetableOscillator<'a>,
}

impl<'a> Osc2<'a> {
    pub fn new(wavetables: [&'a Wavetable; 8], sample_rate: u32) -> Self {
        Self {
            oscillator: CircularWavetableOscillator::new(wavetables, sample_rate),
        }
    }

    pub fn populate(&mut self, buffer: &mut [f32]) {
        self.oscillator.populate(buffer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wavetable_oscillator::{pulse, saw, sine, triangle};

    const SAMPLE_RATE: u32 = 48000;

    lazy_static! {
        static ref SINE_WAVETABLE: Wavetable = Wavetable::new(sine(), SAMPLE_RATE);
        static ref TRIANGLE_WAVETABLE: Wavetable = Wavetable::new(triangle(), SAMPLE_RATE);
        static ref SAW_WAVETABLE: Wavetable = Wavetable::new(saw(), SAMPLE_RATE);
        static ref PULSE_WAVETABLE: Wavetable = Wavetable::new(pulse(0.5), SAMPLE_RATE);
    }

    fn wavetables() -> [&'static Wavetable; 8] {
        [
            &SINE_WAVETABLE,
            &TRIANGLE_WAVETABLE,
            &SAW_WAVETABLE,
            &PULSE_WAVETABLE,
            &SINE_WAVETABLE,
            &TRIANGLE_WAVETABLE,
            &SAW_WAVETABLE,
            &PULSE_WAVETABLE,
        ]
    }

    #[test]
    fn initialize() {
        let _osc2 = Osc2::new(wavetables(), SAMPLE_RATE);
    }

    #[test]
    fn populate_buffer() {
        let mut osc2 = Osc2::new(wavetables(), SAMPLE_RATE);

        let mut buffer = [0.0; 8];
        osc2.populate(&mut buffer);

        buffer
            .iter()
            .find(|x| **x > 0.0)
            .expect("there must be at least one value over zero");
    }
}
