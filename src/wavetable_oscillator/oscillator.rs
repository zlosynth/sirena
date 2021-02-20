use super::wavetable::Wavetable;

pub struct WavetableOscillator {
    wavetable: Wavetable,
    sample_rate: u32,
    frequency: f32,
    phase: f32,
}

impl WavetableOscillator {
    pub fn new(wavetable: Wavetable, sample_rate: u32) -> Self {
        Self {
            wavetable,
            sample_rate,
            frequency: 440.0,
            phase: 0.0,
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) -> &mut Self {
        self.frequency = frequency;
        self
    }

    pub fn tick(&mut self) -> f32 {
        let interval_in_samples = self.frequency / self.sample_rate as f32;
        let sample = self.wavetable.read(self.phase, self.frequency);
        self.phase += interval_in_samples;
        self.phase %= 1.0;
        sample
    }
}

#[cfg(test)]
mod tests {
    use super::super::{saw, sine};
    use super::*;
    use crate::spectral_analysis::SpectralAnalysis;

    #[test]
    fn initialize_wavetable_oscillator() {
        const SAMPLE_RATE: u32 = 441000;
        let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
        let _wavetable_oscillator = WavetableOscillator::new(wavetable, SAMPLE_RATE);
    }

    #[test]
    fn get_first_sample() {
        const SAMPLE_RATE: u32 = 441000;
        let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
        let mut wavetable_oscillator = WavetableOscillator::new(wavetable, SAMPLE_RATE);

        assert_abs_diff_eq!(wavetable_oscillator.tick(), 0.0, epsilon = 0.01);
    }

    #[test]
    fn get_multiple_samples() {
        const SAMPLE_RATE: u32 = 8;
        let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
        let mut wavetable_oscillator = WavetableOscillator::new(wavetable, 8);
        wavetable_oscillator.set_frequency(1.0);

        let first = wavetable_oscillator.tick();
        let second = wavetable_oscillator.tick();
        assert!(second > first);
    }

    #[test]
    fn set_frequency() {
        let three_ticks_frequency_1 = {
            const SAMPLE_RATE: u32 = 8;
            let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
            let mut wavetable_oscillator = WavetableOscillator::new(wavetable, SAMPLE_RATE);
            wavetable_oscillator.set_frequency(1.0);
            wavetable_oscillator.tick();
            wavetable_oscillator.tick();
            wavetable_oscillator.tick()
        };

        let two_ticks_frequency_2 = {
            const SAMPLE_RATE: u32 = 8;
            let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
            let mut wavetable_oscillator = WavetableOscillator::new(wavetable, SAMPLE_RATE);
            wavetable_oscillator.set_frequency(2.0);
            wavetable_oscillator.tick();
            wavetable_oscillator.tick()
        };

        assert_relative_eq!(three_ticks_frequency_1, two_ticks_frequency_2);
    }

    #[test]
    fn set_sample_rate() {
        let two_ticks_sample_rate_1000 = {
            const SAMPLE_RATE: u32 = 1000;
            let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
            let mut wavetable_oscillator = WavetableOscillator::new(wavetable, SAMPLE_RATE);
            wavetable_oscillator.set_frequency(4.0);
            wavetable_oscillator.tick();
            wavetable_oscillator.tick()
        };

        let two_ticks_sample_rate_1100 = {
            const SAMPLE_RATE: u32 = 1100;
            let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
            let mut wavetable_oscillator = WavetableOscillator::new(wavetable, SAMPLE_RATE);
            wavetable_oscillator.set_frequency(4.0);
            wavetable_oscillator.tick();
            wavetable_oscillator.tick()
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
        let mut wavetable_oscillator = WavetableOscillator::new(wavetable, SAMPLE_RATE);
        wavetable_oscillator.set_frequency(frequency);

        let mut signal = [0.0; SAMPLE_RATE as usize];
        for x in signal.iter_mut() {
            *x = wavetable_oscillator.tick();
        }

        let mut analysis = SpectralAnalysis::analyze(&signal, SAMPLE_RATE);
        analysis.trash_range(0.0, 1.0);
        let lowest_peak = analysis.lowest_peak(0.04);
        assert_abs_diff_eq!(lowest_peak, frequency, epsilon = 1.0);
    }
}
