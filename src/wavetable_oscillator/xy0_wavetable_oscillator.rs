use super::oscillator::Oscillator;
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
}

impl<'a> Oscillator for XY0WavetableOscillator<'a> {
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

    fn reset_phase(&mut self) -> &mut Self {
        self.phase = 0.0;
        self
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

enum FillMethod {
    Overwrite,
    Add,
    Dry,
}

#[cfg(test)]
mod tests {
    use super::super::tests::{self, SAMPLE_RATE};
    use super::super::{digital_saw, sine, triangle};
    use super::*;

    lazy_static! {
        static ref SINE_WAVETABLE: Wavetable = Wavetable::new(sine(), SAMPLE_RATE);
        static ref TRIANGLE_WAVETABLE: Wavetable = Wavetable::new(triangle(), SAMPLE_RATE);
        static ref SAW_WAVETABLE: Wavetable = Wavetable::new(digital_saw(), SAMPLE_RATE);
    }

    fn wavetable_oscillator() -> XY0WavetableOscillator<'static> {
        wavetable_oscillator_with_sample_rate(SAMPLE_RATE)
    }

    fn wavetable_oscillator_with_sample_rate(sample_rate: u32) -> XY0WavetableOscillator<'static> {
        let mut wavetable_oscillator = XY0WavetableOscillator::new(
            &SINE_WAVETABLE,
            &TRIANGLE_WAVETABLE,
            &SAW_WAVETABLE,
            sample_rate,
        );
        wavetable_oscillator.set_x(0.5).set_y(0.5);
        wavetable_oscillator
    }

    #[test]
    fn initialize() {
        let _wavetable_oscillator = wavetable_oscillator();
    }

    #[test]
    fn get_first_sample() {
        let mut wavetable_oscillator = wavetable_oscillator();
        tests::get_first_sample(&mut wavetable_oscillator);
    }

    #[test]
    fn get_multiple_samples() {
        let mut wavetable_oscillator = wavetable_oscillator();
        tests::get_multiple_samples(&mut wavetable_oscillator);
    }

    #[test]
    fn set_frequency() {
        let mut wavetable_oscillator_a = wavetable_oscillator();
        let mut wavetable_oscillator_b = wavetable_oscillator();
        tests::set_frequency(&mut wavetable_oscillator_a, &mut wavetable_oscillator_b);
    }

    #[test]
    fn get_frequency() {
        let mut wavetable_oscillator = wavetable_oscillator();
        tests::get_frequency(&mut wavetable_oscillator);
    }

    #[test]
    fn set_sample_rate() {
        let mut wavetable_oscillator_a = wavetable_oscillator_with_sample_rate(1000);
        let mut wavetable_oscillator_b = wavetable_oscillator_with_sample_rate(1100);
        tests::set_sample_rate(&mut wavetable_oscillator_a, &mut wavetable_oscillator_b);
    }

    #[test]
    #[ignore] // too slow for regular execution
    fn check_all_notes_for_aliasing() {
        let mut wavetable_oscillator = wavetable_oscillator();
        tests::check_all_fifths_for_aliasing(&mut wavetable_oscillator);
    }

    #[test]
    fn set_amplitude() {
        let mut wavetable_oscillator = wavetable_oscillator();
        tests::set_amplitude(&mut wavetable_oscillator);
    }

    #[test]
    fn get_amplitude() {
        let mut wavetable_oscillator = wavetable_oscillator();
        tests::get_amplitude(&mut wavetable_oscillator);
    }

    #[test]
    fn reset_phase() {
        let mut wavetable_oscillator = wavetable_oscillator();
        tests::reset_phase(&mut wavetable_oscillator);
    }

    #[test]
    fn zero() {
        let signal = xy0_oscillator_single_cycle(0.0, 0.0);
        assert_signal_eq(&signal, &sine());
    }

    #[test]
    fn x() {
        let signal = xy0_oscillator_single_cycle(1.0, 0.0);
        assert_signal_eq(&signal, &triangle());
    }

    #[test]
    fn negative_x() {
        let signal = xy0_oscillator_single_cycle(-1.0, 0.0);

        let expected = {
            let mut inverted_triangle = triangle();
            invert(&mut inverted_triangle);
            inverted_triangle
        };
        assert_signal_eq(&signal, &expected);
    }

    #[test]
    fn y() {
        let signal = xy0_oscillator_single_cycle(0.0, 1.0);

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
    fn negative_y() {
        let signal = xy0_oscillator_single_cycle(0.0, -1.0);

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
        let signal = xy0_oscillator_single_cycle(x, 0.0);
        assert_equal_mix_of_two_signal_eq(&signal, &sine(), &triangle());
    }

    #[test]
    fn blend_zero_and_y_equally() {
        let y = 3.0 / 2.0 - f32::sqrt(5.0) / 2.0;
        let signal = xy0_oscillator_single_cycle(0.0, y);
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
    fn blend_x_and_y_equally() {
        let signal = xy0_oscillator_single_cycle(1.0, 1.0);
        let expected_a = triangle();
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
        let signal = xy0_oscillator_single_cycle(x, y);
        let expected_a = sine();
        let expected_b = triangle();
        let expected_c = digital_saw();

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

    fn xy0_oscillator_single_cycle(x: f32, y: f32) -> [f32; SAMPLE_RATE as usize] {
        let mut wavetable_oscillator = wavetable_oscillator();
        wavetable_oscillator.set_frequency(1.0).set_x(x).set_y(y);

        let mut signal = [0.0; SAMPLE_RATE as usize];
        wavetable_oscillator.populate(&mut signal);

        signal
    }
}
