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

    pub fn set_frequency(&mut self, frequency: f32) -> &mut Self {
        self.frequency = frequency;
        self
    }

    pub fn frequency(&self) -> f32 {
        self.frequency
    }

    pub fn set_amplitude(&mut self, amplitude: f32) -> &mut Self {
        self.amplitude = amplitude;
        self
    }

    pub fn add(&mut self, buffer: &mut [f32]) {
        self.fill(buffer, FillMethod::Add);
    }

    pub fn populate(&mut self, buffer: &mut [f32]) {
        self.fill(buffer, FillMethod::Overwrite);
    }

    pub fn dry(&mut self, buffer: &mut [f32]) {
        self.fill(buffer, FillMethod::Dry);
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

#[cfg(test)]
mod tests {
    use super::super::{saw, sine};
    use super::*;
    use crate::spectral_analysis::SpectralAnalysis;

    #[test]
    fn initialize() {
        const SAMPLE_RATE: u32 = 44100;
        let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
        let _wavetable_oscillator = CircularWavetableOscillator::new([&wavetable; 8], SAMPLE_RATE);
    }

    #[test]
    fn get_first_sample() {
        const SAMPLE_RATE: u32 = 44100;
        let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
        let mut wavetable_oscillator =
            CircularWavetableOscillator::new([&wavetable; 8], SAMPLE_RATE);
        wavetable_oscillator.set_frequency(100.0);

        let mut buffer = [0.0];
        wavetable_oscillator.populate(&mut buffer);

        assert_abs_diff_eq!(buffer[0], 0.0, epsilon = 0.01);
    }

    #[test]
    fn get_multiple_samples() {
        const SAMPLE_RATE: u32 = 8;
        let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
        let mut wavetable_oscillator =
            CircularWavetableOscillator::new([&wavetable; 8], SAMPLE_RATE);
        wavetable_oscillator.set_frequency(1.0);

        let mut buffer = [0.0; 2];
        wavetable_oscillator.populate(&mut buffer);

        assert!(buffer[1] > buffer[0]);
    }

    #[test]
    fn set_frequency() {
        let three_ticks_frequency_1 = {
            const SAMPLE_RATE: u32 = 8;
            let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
            let mut wavetable_oscillator =
                CircularWavetableOscillator::new([&wavetable; 8], SAMPLE_RATE);
            wavetable_oscillator.set_frequency(1.0);
            let mut buffer = [0.0; 3];
            wavetable_oscillator.populate(&mut buffer);
            buffer[2]
        };

        let two_ticks_frequency_2 = {
            const SAMPLE_RATE: u32 = 8;
            let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
            let mut wavetable_oscillator =
                CircularWavetableOscillator::new([&wavetable; 8], SAMPLE_RATE);
            wavetable_oscillator.set_frequency(2.0);
            let mut buffer = [0.0; 2];
            wavetable_oscillator.populate(&mut buffer);
            buffer[1]
        };

        assert_relative_eq!(three_ticks_frequency_1, two_ticks_frequency_2);
    }

    #[test]
    fn get_frequency() {
        const SAMPLE_RATE: u32 = 8;
        let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
        let mut wavetable_oscillator =
            CircularWavetableOscillator::new([&wavetable; 8], SAMPLE_RATE);
        wavetable_oscillator.set_frequency(110.0);

        assert_eq!(wavetable_oscillator.frequency(), 110.0);
    }

    #[test]
    fn set_sample_rate() {
        let two_ticks_sample_rate_1000 = {
            const SAMPLE_RATE: u32 = 1000;
            let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
            let mut wavetable_oscillator =
                CircularWavetableOscillator::new([&wavetable; 8], SAMPLE_RATE);
            wavetable_oscillator.set_frequency(4.0);
            let mut buffer = [0.0; 2];
            wavetable_oscillator.populate(&mut buffer);
            buffer[1]
        };

        let two_ticks_sample_rate_1100 = {
            const SAMPLE_RATE: u32 = 1100;
            let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
            let mut wavetable_oscillator =
                CircularWavetableOscillator::new([&wavetable; 8], SAMPLE_RATE);
            wavetable_oscillator.set_frequency(4.0);
            let mut buffer = [0.0; 2];
            wavetable_oscillator.populate(&mut buffer);
            buffer[1]
        };

        assert!(two_ticks_sample_rate_1000 > two_ticks_sample_rate_1100);
    }

    #[test]
    #[ignore] // too slow for regular execution
    fn check_all_notes_for_aliasing() {
        let notes: Vec<_> = (1..)
            .step_by(5)
            .map(|i| 27.5 * f32::powf(2.0, i as f32 / 12.0))
            .take_while(|x| *x < 22000.0)
            .collect();

        for note in notes.into_iter() {
            check_note_for_aliasing(note);
        }
    }

    fn check_note_for_aliasing(frequency: f32) {
        const SAMPLE_RATE: u32 = 44100;
        let wavetable = Wavetable::new(saw(), SAMPLE_RATE);
        let mut wavetable_oscillator =
            CircularWavetableOscillator::new([&wavetable; 8], SAMPLE_RATE);
        wavetable_oscillator.set_frequency(frequency);

        let mut signal = [0.0; SAMPLE_RATE as usize];
        wavetable_oscillator.populate(&mut signal);

        let mut analysis = SpectralAnalysis::analyze(&signal, SAMPLE_RATE);
        analysis.trash_range(0.0, 1.0);
        let lowest_peak = analysis.lowest_peak(0.04);
        assert_abs_diff_eq!(lowest_peak, frequency, epsilon = 1.0);
    }

    #[test]
    fn set_amplitude() {
        const SAMPLE_RATE: u32 = 44100;
        let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
        let mut wavetable_oscillator =
            CircularWavetableOscillator::new([&wavetable; 8], SAMPLE_RATE);
        wavetable_oscillator.set_frequency(1.0);

        let mut buffer = [0.0; SAMPLE_RATE as usize];

        wavetable_oscillator.set_amplitude(2.0);
        wavetable_oscillator.populate(&mut buffer);
        let max = buffer.iter().fold(0.0, |a, b| f32::max(a, b.abs()));
        assert_relative_eq!(max, 2.0, max_relative = 0.001);

        wavetable_oscillator.set_amplitude(3.0);
        wavetable_oscillator.populate(&mut buffer);
        let max = buffer.iter().fold(0.0, |a, b| f32::max(a, b.abs()));
        assert_relative_eq!(max, 3.0, max_relative = 0.001);
    }
}
