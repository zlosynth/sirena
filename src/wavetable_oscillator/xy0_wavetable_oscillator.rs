use super::wavetable::Wavetable;

pub struct XY0WavetableOscillator<'a> {
    wavetable_0: &'a Wavetable,
    wavetable_x: &'a Wavetable,
    wavetable_y: &'a Wavetable,
    sample_rate: u32,
    frequency: f32,
    amplitude: f32,
    phase: f32,
    x: f32,
    y: f32,
}

impl<'a> XY0WavetableOscillator<'a> {
    pub fn new(
        wavetable_0: &'a Wavetable,
        wavetable_x: &'a Wavetable,
        wavetable_y: &'a Wavetable,
        sample_rate: u32,
    ) -> Self {
        Self {
            wavetable_0,
            wavetable_x,
            wavetable_y,
            sample_rate,
            frequency: 440.0,
            amplitude: 1.0,
            phase: 0.0,
            x: 0.0,
            y: 0.0,
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

    pub fn set_x(&mut self, x: f32) -> &mut Self {
        assert!((-1.0..=1.0).contains(&x));
        self.x = x;
        self
    }

    pub fn set_y(&mut self, y: f32) -> &mut Self {
        assert!((-1.0..=1.0).contains(&y));
        self.y = y;
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
        let band_wavetable_0 = self.wavetable_0.band(self.frequency);
        let band_wavetable_x = self.wavetable_x.band(self.frequency);
        let band_wavetable_y = self.wavetable_y.band(self.frequency);
        let interval_in_samples = self.frequency / self.sample_rate as f32;
        let zero_magnitude = f32::max(0.0, 1.0 - f32::sqrt(self.x.abs() + self.y.abs()));

        for x in buffer.iter_mut() {
            let sample_a = band_wavetable_0.read(self.phase);
            let sample_b = band_wavetable_x.read(self.phase);
            let sample_c = band_wavetable_y.read(self.phase);

            let sample = (sample_a * zero_magnitude + sample_b * self.x + sample_c * self.y)
                / (zero_magnitude + self.x.abs() + self.y.abs());

            match method {
                FillMethod::Overwrite => *x = sample * self.amplitude,
                FillMethod::Add => *x += sample * self.amplitude,
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
    use super::super::consts::OVERSAMPLED_WAVETABLE_LENGTH;
    use super::super::{digital_saw, saw, sine, triangle};
    use super::*;
    use crate::spectral_analysis::SpectralAnalysis;

    #[test]
    fn initialize() {
        const SAMPLE_RATE: u32 = 44100;
        let wavetable_0 = Wavetable::new(sine(), SAMPLE_RATE);
        let wavetable_x = Wavetable::new(triangle(), SAMPLE_RATE);
        let wavetable_y = Wavetable::new(saw(), SAMPLE_RATE);
        let _wavetable_oscillator =
            XY0WavetableOscillator::new(&wavetable_0, &wavetable_x, &wavetable_y, SAMPLE_RATE);
    }

    #[test]
    fn get_first_sample() {
        const SAMPLE_RATE: u32 = 44100;
        let wavetable_0 = Wavetable::new(sine(), SAMPLE_RATE);
        let wavetable_x = Wavetable::new(triangle(), SAMPLE_RATE);
        let wavetable_y = Wavetable::new(saw(), SAMPLE_RATE);
        let mut wavetable_oscillator =
            XY0WavetableOscillator::new(&wavetable_0, &wavetable_x, &wavetable_y, SAMPLE_RATE);
        wavetable_oscillator
            .set_frequency(100.0)
            .set_x(0.5)
            .set_y(0.5);

        let mut buffer = [0.0];
        wavetable_oscillator.populate(&mut buffer);

        assert_abs_diff_eq!(buffer[0], 0.0, epsilon = 0.01);
    }

    #[test]
    fn get_multiple_samples() {
        const SAMPLE_RATE: u32 = 8;
        let wavetable_0 = Wavetable::new(sine(), SAMPLE_RATE);
        let wavetable_x = Wavetable::new(triangle(), SAMPLE_RATE);
        let wavetable_y = Wavetable::new(saw(), SAMPLE_RATE);
        let mut wavetable_oscillator =
            XY0WavetableOscillator::new(&wavetable_0, &wavetable_x, &wavetable_y, SAMPLE_RATE);
        wavetable_oscillator
            .set_frequency(1.0)
            .set_x(0.5)
            .set_y(0.5);

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
                XY0WavetableOscillator::new(&wavetable, &wavetable, &wavetable, SAMPLE_RATE);
            wavetable_oscillator.set_frequency(1.0);
            let mut buffer = [0.0; 3];
            wavetable_oscillator.populate(&mut buffer);
            buffer[2]
        };

        let two_ticks_frequency_2 = {
            const SAMPLE_RATE: u32 = 8;
            let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
            let mut wavetable_oscillator =
                XY0WavetableOscillator::new(&wavetable, &wavetable, &wavetable, SAMPLE_RATE);
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
            XY0WavetableOscillator::new(&wavetable, &wavetable, &wavetable, SAMPLE_RATE);
        wavetable_oscillator.set_frequency(110.0);

        assert_eq!(wavetable_oscillator.frequency(), 110.0);
    }

    #[test]
    fn set_sample_rate() {
        let two_ticks_sample_rate_1000 = {
            const SAMPLE_RATE: u32 = 1000;
            let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
            let mut wavetable_oscillator =
                XY0WavetableOscillator::new(&wavetable, &wavetable, &wavetable, SAMPLE_RATE);
            wavetable_oscillator.set_frequency(4.0);
            let mut buffer = [0.0; 2];
            wavetable_oscillator.populate(&mut buffer);
            buffer[1]
        };

        let two_ticks_sample_rate_1100 = {
            const SAMPLE_RATE: u32 = 1100;
            let wavetable = Wavetable::new(sine(), SAMPLE_RATE);
            let mut wavetable_oscillator =
                XY0WavetableOscillator::new(&wavetable, &wavetable, &wavetable, SAMPLE_RATE);
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
        let wavetable_0 = Wavetable::new(sine(), SAMPLE_RATE);
        let wavetable_x = Wavetable::new(triangle(), SAMPLE_RATE);
        let wavetable_y = Wavetable::new(saw(), SAMPLE_RATE);
        let mut wavetable_oscillator =
            XY0WavetableOscillator::new(&wavetable_0, &wavetable_x, &wavetable_y, SAMPLE_RATE);
        wavetable_oscillator
            .set_frequency(frequency)
            .set_x(0.5)
            .set_y(0.5);

        let mut signal = [0.0; SAMPLE_RATE as usize];
        wavetable_oscillator.populate(&mut signal);

        let mut analysis = SpectralAnalysis::analyze(&signal, SAMPLE_RATE);
        analysis.trash_range(0.0, 1.0);
        let lowest_peak = analysis.lowest_peak(0.04);
        assert_abs_diff_eq!(lowest_peak, frequency, epsilon = 1.0);
    }

    #[test]
    fn zero() {
        let signal = xy0_oscillator_single_cycle(sine(), digital_saw(), triangle(), 0.0, 0.0);
        assert_signal_eq(&signal, &sine());
    }

    #[test]
    fn x() {
        let signal = xy0_oscillator_single_cycle(sine(), digital_saw(), triangle(), 1.0, 0.0);

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
    fn negative_x() {
        let signal = xy0_oscillator_single_cycle(sine(), digital_saw(), triangle(), -1.0, 0.0);

        // ignore the breaking point of the saw wave in the middle
        let expected = {
            let mut inverted_saw = digital_saw();
            invert(&mut inverted_saw);
            inverted_saw
        };
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
    fn y() {
        let signal = xy0_oscillator_single_cycle(sine(), digital_saw(), triangle(), 0.0, 1.0);
        assert_signal_eq(&signal, &triangle());
    }

    #[test]
    fn negative_y() {
        let signal = xy0_oscillator_single_cycle(sine(), digital_saw(), triangle(), 0.0, -1.0);
        let expected = {
            let mut inverted_triangle = triangle();
            invert(&mut inverted_triangle);
            inverted_triangle
        };
        assert_signal_eq(&signal, &expected);
    }

    fn invert(data: &mut [f32]) {
        data.iter_mut().for_each(|x| *x *= -1.0);
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
    fn blend_zero_and_x_equally() {
        let x = 3.0 / 2.0 - f32::sqrt(5.0) / 2.0;
        let signal = xy0_oscillator_single_cycle(sine(), digital_saw(), triangle(), x, 0.0);
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
    fn blend_zero_and_y_equally() {
        let y = 3.0 / 2.0 - f32::sqrt(5.0) / 2.0;
        let signal = xy0_oscillator_single_cycle(sine(), digital_saw(), triangle(), 0.0, y);
        assert_equal_mix_of_two_signal_eq(&signal, &sine(), &triangle());
    }

    #[test]
    fn blend_x_and_y_equally() {
        let signal = xy0_oscillator_single_cycle(sine(), digital_saw(), triangle(), 1.0, 1.0);
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
    fn blend_all_three_equally() {
        let x = 2.0 - f32::sqrt(3.0);
        let y = x;
        let signal = xy0_oscillator_single_cycle(sine(), digital_saw(), triangle(), x, y);
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

    fn xy0_oscillator_single_cycle(
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
            XY0WavetableOscillator::new(&wavetable_0, &wavetable_x, &wavetable_y, SAMPLE_RATE);

        wavetable_oscillator.set_frequency(1.0);
        wavetable_oscillator.set_x(x);
        wavetable_oscillator.set_y(y);

        let mut signal = [0.0; SAMPLE_RATE as usize];
        wavetable_oscillator.populate(&mut signal);

        signal
    }

    #[test]
    fn set_amplitude() {
        const SAMPLE_RATE: u32 = 44100;
        let wavetable_0 = Wavetable::new(sine(), SAMPLE_RATE);
        let wavetable_x = Wavetable::new(triangle(), SAMPLE_RATE);
        let wavetable_y = Wavetable::new(saw(), SAMPLE_RATE);
        let mut wavetable_oscillator =
            XY0WavetableOscillator::new(&wavetable_0, &wavetable_x, &wavetable_y, SAMPLE_RATE);
        wavetable_oscillator.set_frequency(1.0).set_x(1.0);

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