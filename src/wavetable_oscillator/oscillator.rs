use super::wavetable::Wavetable;

pub struct DoubleWavetableOscillator<'a> {
    wavetable_a: &'a Wavetable,
    wavetable_b: &'a Wavetable,
    wavetable_c: &'a Wavetable,
    sample_rate: u32,
    frequency: f32,
    phase: f32,
    x: f32,
    y: f32,
}

impl<'a> DoubleWavetableOscillator<'a> {
    pub fn new(
        wavetable_a: &'a Wavetable,
        wavetable_b: &'a Wavetable,
        wavetable_c: &'a Wavetable,
        sample_rate: u32,
    ) -> Self {
        Self {
            wavetable_a,
            wavetable_b,
            wavetable_c,
            sample_rate,
            frequency: 440.0,
            phase: 0.0,
            x: 0.0,
            y: 0.0,
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) -> &mut Self {
        self.frequency = frequency;
        self
    }

    pub fn set_x(&mut self, x: f32) -> &mut Self {
        assert!((0.0..=1.0).contains(&x));
        self.x = x;
        self
    }

    pub fn set_y(&mut self, y: f32) -> &mut Self {
        assert!((0.0..=1.0).contains(&y));
        self.y = y;
        self
    }

    pub fn tick(&mut self) -> f32 {
        let interval_in_samples = self.frequency / self.sample_rate as f32;
        let sample_a = self.wavetable_a.read(self.phase, self.frequency);
        let sample_b = self.wavetable_b.read(self.phase, self.frequency);
        let sample_c = self.wavetable_c.read(self.phase, self.frequency);
        self.phase += interval_in_samples;
        self.phase %= 1.0;

        let zero_magnitude = f32::max(0.0, 1.0 - f32::sqrt(self.x + self.y));
        (sample_a * zero_magnitude + sample_b * self.x + sample_c * self.y)
            / (zero_magnitude + self.x + self.y)
    }
}

pub struct WavetableOscillator<'a> {
    wavetable: &'a Wavetable,
    sample_rate: u32,
    frequency: f32,
    phase: f32,
}

impl<'a> WavetableOscillator<'a> {
    pub fn new(wavetable: &'a Wavetable, sample_rate: u32) -> Self {
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
    use super::super::consts::OVERSAMPLED_WAVETABLE_LENGTH;
    use super::super::{digital_saw, saw, sine, triangle};
    use super::*;
    use crate::spectral_analysis::SpectralAnalysis;

    #[test]
    fn initialize_wavetable_oscillator() {
        const SAMPLE_RATE: u32 = 441000;
        let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
        let _wavetable_oscillator = WavetableOscillator::new(&wavetable, SAMPLE_RATE);
    }

    #[test]
    fn get_first_sample() {
        const SAMPLE_RATE: u32 = 441000;
        let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
        let mut wavetable_oscillator = WavetableOscillator::new(&wavetable, SAMPLE_RATE);

        assert_abs_diff_eq!(wavetable_oscillator.tick(), 0.0, epsilon = 0.01);
    }

    #[test]
    fn get_multiple_samples() {
        const SAMPLE_RATE: u32 = 8;
        let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
        let mut wavetable_oscillator = WavetableOscillator::new(&wavetable, 8);
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
            let mut wavetable_oscillator = WavetableOscillator::new(&wavetable, SAMPLE_RATE);
            wavetable_oscillator.set_frequency(1.0);
            wavetable_oscillator.tick();
            wavetable_oscillator.tick();
            wavetable_oscillator.tick()
        };

        let two_ticks_frequency_2 = {
            const SAMPLE_RATE: u32 = 8;
            let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
            let mut wavetable_oscillator = WavetableOscillator::new(&wavetable, SAMPLE_RATE);
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
            let mut wavetable_oscillator = WavetableOscillator::new(&wavetable, SAMPLE_RATE);
            wavetable_oscillator.set_frequency(4.0);
            wavetable_oscillator.tick();
            wavetable_oscillator.tick()
        };

        let two_ticks_sample_rate_1100 = {
            const SAMPLE_RATE: u32 = 1100;
            let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
            let mut wavetable_oscillator = WavetableOscillator::new(&wavetable, SAMPLE_RATE);
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
        let mut wavetable_oscillator = WavetableOscillator::new(&wavetable, SAMPLE_RATE);
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

    #[test]
    fn dual_oscillator_zero() {
        let signal = dual_oscillator_single_cycle(sine(), digital_saw(), triangle(), 0.0, 0.0);
        assert_signal_eq(&signal, &sine());
    }

    #[test]
    fn dual_oscillator_x() {
        let signal = dual_oscillator_single_cycle(sine(), digital_saw(), triangle(), 1.0, 0.0);

        // ignore the breaking point of the saw wave in the middle
        let expected = digital_saw();
        assert_signal_eq(
            &signal[0..(signal.len() as f32 * 0.49) as usize],
            &expected[0..(expected.len() as f32 * 0.49) as usize],
        );
        assert_signal_eq(
            &signal[(signal.len() as f32 * 0.51) as usize..signal.len() - 1],
            &expected[(expected.len() as f32 * 0.51) as usize..expected.len() - 1],
        );
    }

    #[test]
    fn dual_oscillator_y() {
        let signal = dual_oscillator_single_cycle(sine(), digital_saw(), triangle(), 0.0, 1.0);
        assert_signal_eq(&signal, &triangle());
    }

    fn assert_signal_eq(a: &[f32], b: &[f32]) {
        let ratio = a.len() as f32 / b.len() as f32;

        (0..b.len()).for_each(|i| {
            assert_relative_eq!(
                b[i],
                a[(i as f32 * ratio) as usize],
                max_relative = 0.05,
                epsilon = 0.01
            )
        });
    }

    #[test]
    fn dual_oscillator_blend_zero_and_x_equally() {
        let x = 3.0 / 2.0 - f32::sqrt(5.0) / 2.0;
        let signal = dual_oscillator_single_cycle(sine(), digital_saw(), triangle(), x, 0.0);
        let expected_a = sine();
        let expected_b = digital_saw();

        // ignore the breaking point of the saw wave in the middle
        assert_equal_mix_of_two_signal_eq(
            &signal[0..(signal.len() as f32 * 0.49) as usize],
            &expected_a[0..(expected_a.len() as f32 * 0.49) as usize],
            &expected_b[0..(expected_b.len() as f32 * 0.49) as usize],
        );
        assert_equal_mix_of_two_signal_eq(
            &signal[(signal.len() as f32 * 0.51) as usize..signal.len() - 1],
            &expected_a[(expected_a.len() as f32 * 0.51) as usize..expected_a.len() - 1],
            &expected_b[(expected_b.len() as f32 * 0.51) as usize..expected_b.len() - 1],
        );
    }

    #[test]
    fn dual_oscillator_blend_zero_and_y_equally() {
        let y = 3.0 / 2.0 - f32::sqrt(5.0) / 2.0;
        let signal = dual_oscillator_single_cycle(sine(), digital_saw(), triangle(), 0.0, y);
        assert_equal_mix_of_two_signal_eq(&signal, &sine(), &triangle());
    }

    #[test]
    fn dual_oscillator_blend_x_and_y_equally() {
        let signal = dual_oscillator_single_cycle(sine(), digital_saw(), triangle(), 1.0, 1.0);
        let expected_a = digital_saw();
        let expected_b = triangle();

        // ignore the breaking point of the saw wave in the middle
        assert_equal_mix_of_two_signal_eq(
            &signal[0..(signal.len() as f32 * 0.49) as usize],
            &expected_a[0..(expected_a.len() as f32 * 0.49) as usize],
            &expected_b[0..(expected_b.len() as f32 * 0.49) as usize],
        );
        assert_equal_mix_of_two_signal_eq(
            &signal[(signal.len() as f32 * 0.51) as usize..signal.len() - 1],
            &expected_a[(expected_a.len() as f32 * 0.51) as usize..expected_a.len() - 1],
            &expected_b[(expected_b.len() as f32 * 0.51) as usize..expected_b.len() - 1],
        );
    }

    fn assert_equal_mix_of_two_signal_eq(a: &[f32], b1: &[f32], b2: &[f32]) {
        assert_eq!(b1.len(), b2.len());

        let ratio = a.len() as f32 / b1.len() as f32;

        (0..b1.len()).for_each(|i| {
            assert_relative_eq!(
                (b1[i] + b2[i]) / 2.0,
                a[(i as f32 * ratio) as usize],
                max_relative = 0.05,
                epsilon = 0.01
            )
        });
    }

    #[test]
    fn dual_oscillator_blend_all_three_equally() {
        let x = 2.0 - f32::sqrt(3.0);
        let y = x;
        let signal = dual_oscillator_single_cycle(sine(), digital_saw(), triangle(), x, y);
        let expected_a = sine();
        let expected_b = digital_saw();
        let expected_c = triangle();

        // ignore the breaking point of the saw wave in the middle
        assert_equal_mix_of_three_signal_eq(
            &signal[0..(signal.len() as f32 * 0.49) as usize],
            &expected_a[0..(expected_a.len() as f32 * 0.49) as usize],
            &expected_b[0..(expected_b.len() as f32 * 0.49) as usize],
            &expected_c[0..(expected_c.len() as f32 * 0.49) as usize],
        );
        assert_equal_mix_of_three_signal_eq(
            &signal[(signal.len() as f32 * 0.51) as usize..signal.len() - 1],
            &expected_a[(expected_a.len() as f32 * 0.51) as usize..expected_a.len() - 1],
            &expected_b[(expected_b.len() as f32 * 0.51) as usize..expected_b.len() - 1],
            &expected_c[(expected_c.len() as f32 * 0.51) as usize..expected_c.len() - 1],
        );
    }

    fn assert_equal_mix_of_three_signal_eq(a: &[f32], b1: &[f32], b2: &[f32], b3: &[f32]) {
        assert_eq!(b1.len(), b2.len());
        assert_eq!(b1.len(), b3.len());

        let ratio = a.len() as f32 / b1.len() as f32;

        (0..b1.len()).for_each(|i| {
            assert_relative_eq!(
                (b1[i] + b2[i] + b3[i]) / 3.0,
                a[(i as f32 * ratio) as usize],
                max_relative = 0.05,
                epsilon = 0.01
            )
        });
    }

    fn dual_oscillator_single_cycle(
        wavetable_0: [f32; OVERSAMPLED_WAVETABLE_LENGTH],
        wavetable_x: [f32; OVERSAMPLED_WAVETABLE_LENGTH],
        wavetable_y: [f32; OVERSAMPLED_WAVETABLE_LENGTH],
        x: f32,
        y: f32,
    ) -> [f32; 44100] {
        const SAMPLE_RATE: u32 = 44100;

        let wavetable_0 = Wavetable::new(wavetable_0, SAMPLE_RATE);
        let wavetable_x = Wavetable::new(wavetable_x, SAMPLE_RATE);
        let wavetable_y = Wavetable::new(wavetable_y, SAMPLE_RATE);

        let mut wavetable_oscillator =
            DoubleWavetableOscillator::new(&wavetable_0, &wavetable_x, &wavetable_y, SAMPLE_RATE);

        wavetable_oscillator.set_frequency(1.0);
        wavetable_oscillator.set_x(x);
        wavetable_oscillator.set_y(y);

        let mut signal = [0.0; SAMPLE_RATE as usize];
        for x in signal.iter_mut() {
            *x = wavetable_oscillator.tick();
        }

        signal
    }
}
