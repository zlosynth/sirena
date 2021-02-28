use super::oscillator::Oscillator;
use super::wavetable::Wavetable;

pub struct CircularWavetableOscillator<'a> {
    sample_rate: u32,
    frequency: f32,
    amplitude: f32,
    wavetable: f32,
    phase: f32,
    wavetables: [&'a Wavetable; 8],
}

impl<'a> CircularWavetableOscillator<'a> {
    pub fn new(wavetables: [&'a Wavetable; 8], sample_rate: u32) -> Self {
        Self {
            sample_rate,
            frequency: 440.0,
            amplitude: 1.0,
            wavetable: 0.0,
            phase: 0.0,
            wavetables,
        }
    }

    fn fill(&mut self, buffer: &mut [f32], method: FillMethod) {
        let band_wavetable = self.wavetables[0].band(self.frequency);
        let interval_in_samples = self.frequency / self.sample_rate as f32;

        for x in buffer.iter_mut() {
            match method {
                FillMethod::Overwrite => *x = band_wavetable.read(self.phase) * self.amplitude,
                FillMethod::Add => *x += band_wavetable.read(self.phase) * self.amplitude,
                FillMethod::Dry => (),
            }

            self.phase += interval_in_samples;
            self.phase %= 1.0;
        }
    }
}

enum FillMethod {
    Overwrite,
    Add,
    Dry,
}

impl<'a> Oscillator for CircularWavetableOscillator<'a> {
    fn set_frequency(&mut self, frequency: f32) -> &mut Self {
        self.frequency = frequency;
        self
    }

    fn frequency(&self) -> f32 {
        self.frequency
    }

    fn set_amplitude(&mut self, amplitude: f32) -> &mut Self {
        self.amplitude = amplitude;
        self
    }

    fn amplitude(&self) -> f32 {
        self.amplitude
    }

    fn add(&mut self, buffer: &mut [f32]) {
        self.fill(buffer, FillMethod::Add);
    }

    fn populate(&mut self, buffer: &mut [f32]) {
        self.fill(buffer, FillMethod::Overwrite);
    }

    fn dry(&mut self, buffer: &mut [f32]) {
        self.fill(buffer, FillMethod::Dry);
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{self, SAMPLE_RATE};
    use super::super::{saw, sine};
    use super::*;

    lazy_static! {
        static ref SINE_WAVETABLE: Wavetable = Wavetable::new(sine(), SAMPLE_RATE);
        static ref SAW_WAVETABLE: Wavetable = Wavetable::new(saw(), SAMPLE_RATE);
    }

    #[test]
    fn initialize() {
        let _wavetable_oscillator =
            CircularWavetableOscillator::new([&SINE_WAVETABLE; 8], SAMPLE_RATE);
    }

    #[test]
    fn get_first_sample() {
        let mut wavetable_oscillator =
            CircularWavetableOscillator::new([&SINE_WAVETABLE; 8], SAMPLE_RATE);
        tests::get_first_sample(&mut wavetable_oscillator);
    }

    #[test]
    fn get_multiple_samples() {
        let mut wavetable_oscillator =
            CircularWavetableOscillator::new([&SINE_WAVETABLE; 8], SAMPLE_RATE);
        tests::get_multiple_samples(&mut wavetable_oscillator);
    }

    #[test]
    fn set_frequency() {
        let mut wavetable_oscillator_a =
            CircularWavetableOscillator::new([&SINE_WAVETABLE; 8], SAMPLE_RATE);
        let mut wavetable_oscillator_b =
            CircularWavetableOscillator::new([&SINE_WAVETABLE; 8], SAMPLE_RATE);
        tests::set_frequency(&mut wavetable_oscillator_a, &mut wavetable_oscillator_b);
    }

    #[test]
    fn get_frequency() {
        let mut wavetable_oscillator =
            CircularWavetableOscillator::new([&SINE_WAVETABLE; 8], SAMPLE_RATE);
        tests::get_frequency(&mut wavetable_oscillator);
    }

    #[test]
    fn set_sample_rate() {
        let mut wavetable_oscillator_a =
            CircularWavetableOscillator::new([&SINE_WAVETABLE; 8], 1000);
        let mut wavetable_oscillator_b =
            CircularWavetableOscillator::new([&SINE_WAVETABLE; 8], 1100);
        tests::set_sample_rate(&mut wavetable_oscillator_a, &mut wavetable_oscillator_b);
    }

    #[test]
    #[ignore] // too slow for regular execution
    fn check_all_notes_for_aliasing() {
        let mut wavetable_oscillator =
            CircularWavetableOscillator::new([&SAW_WAVETABLE; 8], SAMPLE_RATE);
        tests::check_all_fifths_for_aliasing(&mut wavetable_oscillator);
    }

    #[test]
    fn set_amplitude() {
        let mut wavetable_oscillator =
            CircularWavetableOscillator::new([&SINE_WAVETABLE; 8], SAMPLE_RATE);
        tests::set_amplitude(&mut wavetable_oscillator);
    }

    #[test]
    fn get_amplitude() {
        let mut wavetable_oscillator =
            CircularWavetableOscillator::new([&SINE_WAVETABLE; 8], SAMPLE_RATE);
        tests::get_amplitude(&mut wavetable_oscillator);
    }
}
