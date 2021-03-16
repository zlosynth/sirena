#[allow(unused_imports)]
use micromath::F32Ext;

use super::oscillator::{Oscillator, StereoOscillator};
use super::wavetable::{BandWavetable, Wavetable};
use crate::tone;
use crate::xfade;

pub const MAX_WAVETABLES: usize = 8;

pub struct CircularWavetableOscillator<'a, 'b> {
    sample_rate: u32,
    frequency: f32,
    amplitude: f32,
    wavetable: f32,
    phase: f32,
    pan: f32,
    wavetables: [&'a Wavetable; MAX_WAVETABLES],
    fm_multiple: f32,
    fm_intensity: f32,
    fm_phase: f32,
    fm_wavetable: &'b Wavetable,
}

impl<'a, 'b> CircularWavetableOscillator<'a, 'b> {
    pub fn new(
        wavetables: [&'a Wavetable; MAX_WAVETABLES],
        fm_wavetable: &'b Wavetable,
        sample_rate: u32,
    ) -> Self {
        Self {
            sample_rate,
            frequency: 440.0,
            amplitude: 1.0,
            wavetable: 0.0,
            phase: 0.0,
            pan: 0.0,
            wavetables,
            fm_multiple: 0.0,
            fm_intensity: 0.0,
            fm_phase: 0.0,
            fm_wavetable,
        }
    }

    fn fill(&mut self, buffers: &mut [&mut [f32]], method: FillMethod, channels: Channels) {
        let wavetable_a_index = self.wavetable as usize;
        let wavetable_b_index = calculate_next_wavetable_index(wavetable_a_index);
        let xfade = self.wavetable.fract();

        let band_wavetable_a = self.wavetables[wavetable_a_index].band(self.frequency);
        let band_wavetable_b = self.wavetables[wavetable_b_index].band(self.frequency);

        let fm_band_wavetable = self.fm_wavetable.band(self.frequency * self.fm_multiple);
        let fm_interval_in_samples = (self.fm_multiple * self.frequency) / self.sample_rate as f32;

        for i in 0..buffers[0].len() {
            let modulation = fm_band_wavetable.read(self.fm_phase) * self.fm_intensity;
            let frequency = tone::detune_frequency(self.frequency, modulation);
            let interval_in_samples = frequency / self.sample_rate as f32;
            match method {
                FillMethod::Overwrite => {
                    let value = mix_and_read_wavetables(
                        &band_wavetable_a,
                        &band_wavetable_b,
                        xfade,
                        self.phase,
                    );
                    match channels {
                        Mono => {
                            buffers[0][i] = value * self.amplitude;
                        }
                        Stereo => {
                            buffers[0][i] = value * self.amplitude * (1.0 - self.pan);
                            buffers[1][i] = value * self.amplitude * self.pan;
                        }
                    }
                }
                FillMethod::Add => {
                    let value = mix_and_read_wavetables(
                        &band_wavetable_a,
                        &band_wavetable_b,
                        xfade,
                        self.phase,
                    );
                    match channels {
                        Mono => {
                            buffers[0][i] += value * self.amplitude;
                        }
                        Stereo => {
                            buffers[0][i] += value * self.amplitude * (1.0 - self.pan);
                            buffers[1][i] += value * self.amplitude * self.pan;
                        }
                    }
                }
                FillMethod::Dry => (),
            }

            self.phase += interval_in_samples;
            self.phase %= 1.0;

            self.fm_phase += fm_interval_in_samples;
            self.fm_phase %= 1.0;
        }
    }

    pub fn set_wavetable(&mut self, wavetable: f32) -> &mut Self {
        self.wavetable = wavetable.rem_euclid(MAX_WAVETABLES as f32);
        self
    }

    pub fn wavetable(&self) -> f32 {
        self.wavetable
    }

    pub fn set_fm_multiple(&mut self, fm_multiple: f32) -> &mut Self {
        self.fm_multiple = fm_multiple;
        self
    }

    pub fn fm_multiple(&self) -> f32 {
        self.fm_multiple
    }

    pub fn set_fm_intensity(&mut self, fm_intensity: f32) -> &mut Self {
        self.fm_intensity = fm_intensity;
        self
    }

    pub fn fm_intensity(&self) -> f32 {
        self.fm_intensity
    }
}

enum Channels {
    Mono,
    Stereo,
}
use Channels::*;

fn calculate_next_wavetable_index(wavetable_index: usize) -> usize {
    if wavetable_index == MAX_WAVETABLES - 1 {
        0
    } else {
        wavetable_index + 1
    }
}

fn mix_and_read_wavetables(
    band_wavetable_a: &BandWavetable,
    band_wavetable_b: &BandWavetable,
    xfade: f32,
    phase: f32,
) -> f32 {
    let value_a = band_wavetable_a.read(phase);
    let value_b = band_wavetable_b.read(phase);
    xfade::lin(value_a, value_b, xfade)
}

enum FillMethod {
    Overwrite,
    Add,
    Dry,
}

impl<'a, 'b> Oscillator for CircularWavetableOscillator<'a, 'b> {
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
        self.fill(&mut [buffer], FillMethod::Add, Mono);
    }

    fn populate(&mut self, buffer: &mut [f32]) {
        self.fill(&mut [buffer], FillMethod::Overwrite, Mono);
    }

    fn dry(&mut self, buffer: &mut [f32]) {
        self.fill(&mut [buffer], FillMethod::Dry, Mono);
    }
}

impl<'a, 'b> StereoOscillator for CircularWavetableOscillator<'a, 'b> {
    fn set_pan(&mut self, pan: f32) -> &mut Self {
        self.pan = (pan + 1.0) / 2.0;
        self
    }

    fn pan(&self) -> f32 {
        self.pan * 2.0 - 1.0
    }

    fn add_stereo(&mut self, buffer: &mut [&mut [f32]]) {
        self.fill(buffer, FillMethod::Add, Stereo);
    }

    fn populate_stereo(&mut self, buffer: &mut [&mut [f32]]) {
        self.fill(buffer, FillMethod::Overwrite, Stereo);
    }

    fn dry_stereo(&mut self, buffer: &mut [&mut [f32]]) {
        self.fill(buffer, FillMethod::Dry, Stereo);
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{self, SAMPLE_RATE};
    use super::super::{saw, sine, triangle};
    use super::*;

    lazy_static! {
        static ref SINE_WAVETABLE: Wavetable = Wavetable::new(sine(), SAMPLE_RATE);
        static ref TRIANGLE_WAVETABLE: Wavetable = Wavetable::new(triangle(), SAMPLE_RATE);
        static ref SAW_WAVETABLE: Wavetable = Wavetable::new(saw(), SAMPLE_RATE);
    }

    #[test]
    fn initialize() {
        let _wavetable_oscillator = CircularWavetableOscillator::new(
            [&SINE_WAVETABLE; MAX_WAVETABLES],
            &SINE_WAVETABLE,
            SAMPLE_RATE,
        );
    }

    #[test]
    fn get_first_sample() {
        let mut wavetable_oscillator = CircularWavetableOscillator::new(
            [&SINE_WAVETABLE; MAX_WAVETABLES],
            &SINE_WAVETABLE,
            SAMPLE_RATE,
        );
        tests::get_first_sample(&mut wavetable_oscillator);
    }

    #[test]
    fn get_multiple_samples() {
        let mut wavetable_oscillator = CircularWavetableOscillator::new(
            [&SINE_WAVETABLE; MAX_WAVETABLES],
            &SINE_WAVETABLE,
            SAMPLE_RATE,
        );
        tests::get_multiple_samples(&mut wavetable_oscillator);
    }

    #[test]
    fn set_frequency() {
        let mut wavetable_oscillator_a = CircularWavetableOscillator::new(
            [&SINE_WAVETABLE; MAX_WAVETABLES],
            &SINE_WAVETABLE,
            SAMPLE_RATE,
        );
        let mut wavetable_oscillator_b = CircularWavetableOscillator::new(
            [&SINE_WAVETABLE; MAX_WAVETABLES],
            &SINE_WAVETABLE,
            SAMPLE_RATE,
        );
        tests::set_frequency(&mut wavetable_oscillator_a, &mut wavetable_oscillator_b);
    }

    #[test]
    fn get_frequency() {
        let mut wavetable_oscillator = CircularWavetableOscillator::new(
            [&SINE_WAVETABLE; MAX_WAVETABLES],
            &SINE_WAVETABLE,
            SAMPLE_RATE,
        );
        tests::get_frequency(&mut wavetable_oscillator);
    }

    #[test]
    fn get_pan() {
        let mut wavetable_oscillator = CircularWavetableOscillator::new(
            [&SINE_WAVETABLE; MAX_WAVETABLES],
            &SINE_WAVETABLE,
            SAMPLE_RATE,
        );
        tests::get_pan(&mut wavetable_oscillator);
    }

    #[test]
    fn set_sample_rate() {
        let mut wavetable_oscillator_a = CircularWavetableOscillator::new(
            [&SINE_WAVETABLE; MAX_WAVETABLES],
            &SINE_WAVETABLE,
            1000,
        );
        let mut wavetable_oscillator_b = CircularWavetableOscillator::new(
            [&SINE_WAVETABLE; MAX_WAVETABLES],
            &SINE_WAVETABLE,
            1100,
        );
        tests::set_sample_rate(&mut wavetable_oscillator_a, &mut wavetable_oscillator_b);
    }

    #[test]
    #[ignore] // too slow for regular execution
    fn check_all_notes_for_aliasing() {
        let mut wavetable_oscillator = CircularWavetableOscillator::new(
            [&SINE_WAVETABLE; MAX_WAVETABLES],
            &SINE_WAVETABLE,
            SAMPLE_RATE,
        );

        wavetable_oscillator.set_wavetable(0.0);
        tests::check_all_fifths_for_aliasing(&mut wavetable_oscillator);

        wavetable_oscillator.set_wavetable(0.5);
        tests::check_all_fifths_for_aliasing(&mut wavetable_oscillator);

        wavetable_oscillator.set_wavetable(1.0);
        tests::check_all_fifths_for_aliasing(&mut wavetable_oscillator);
    }

    #[test]
    fn set_amplitude() {
        let mut wavetable_oscillator = CircularWavetableOscillator::new(
            [&SINE_WAVETABLE; MAX_WAVETABLES],
            &SINE_WAVETABLE,
            SAMPLE_RATE,
        );
        tests::set_amplitude(&mut wavetable_oscillator);
    }

    #[test]
    fn get_amplitude() {
        let mut wavetable_oscillator = CircularWavetableOscillator::new(
            [&SINE_WAVETABLE; MAX_WAVETABLES],
            &SINE_WAVETABLE,
            SAMPLE_RATE,
        );
        tests::get_amplitude(&mut wavetable_oscillator);
    }

    #[test]
    fn reset_phase() {
        let mut wavetable_oscillator = CircularWavetableOscillator::new(
            [&SINE_WAVETABLE; MAX_WAVETABLES],
            &SINE_WAVETABLE,
            SAMPLE_RATE,
        );
        tests::reset_phase(&mut wavetable_oscillator);
    }

    #[test]
    fn stereo_panning() {
        let mut wavetable_oscillator = CircularWavetableOscillator::new(
            [&SINE_WAVETABLE; MAX_WAVETABLES],
            &SINE_WAVETABLE,
            SAMPLE_RATE,
        );
        tests::stereo_panning(&mut wavetable_oscillator);
    }

    #[test]
    fn set_wavetable() {
        let mut wavetable_oscillator = CircularWavetableOscillator::new(
            [
                &SINE_WAVETABLE,
                &TRIANGLE_WAVETABLE,
                &SAW_WAVETABLE,
                &SAW_WAVETABLE,
                &SAW_WAVETABLE,
                &SAW_WAVETABLE,
                &SAW_WAVETABLE,
                &SAW_WAVETABLE,
            ],
            &SINE_WAVETABLE,
            SAMPLE_RATE,
        );
        wavetable_oscillator.set_frequency(1.0);
        let mut signal = [0.0; SAMPLE_RATE as usize];

        wavetable_oscillator.set_wavetable(0.0);
        wavetable_oscillator.populate(&mut signal);
        assert_signal_eq(&signal, &sine());

        wavetable_oscillator.set_wavetable(1.0);
        wavetable_oscillator.populate(&mut signal);
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
    fn get_wavetable() {
        let mut wavetable_oscillator = CircularWavetableOscillator::new(
            [&SINE_WAVETABLE; MAX_WAVETABLES],
            &SINE_WAVETABLE,
            SAMPLE_RATE,
        );
        assert!(3.0 < MAX_WAVETABLES as f32, "invalid test parameter");
        wavetable_oscillator.set_wavetable(3.0);
        assert_relative_eq!(wavetable_oscillator.wavetable(), 3.0);
    }

    #[test]
    fn roll_over_top_wavetable() {
        let mut wavetable_oscillator = CircularWavetableOscillator::new(
            [&SINE_WAVETABLE; MAX_WAVETABLES],
            &SINE_WAVETABLE,
            SAMPLE_RATE,
        );
        let set_wavetable = MAX_WAVETABLES as f32 + 1.0;
        let expected_wavetable = 1.0;
        wavetable_oscillator.set_wavetable(set_wavetable);
        assert_relative_eq!(wavetable_oscillator.wavetable(), expected_wavetable);
    }

    #[test]
    fn check_bellow_bottom_wavetable() {
        let mut wavetable_oscillator = CircularWavetableOscillator::new(
            [&SINE_WAVETABLE; MAX_WAVETABLES],
            &SINE_WAVETABLE,
            SAMPLE_RATE,
        );
        let set_wavetable = -1.0;
        let expected_wavetable = MAX_WAVETABLES as f32 - 1.0;
        wavetable_oscillator.set_wavetable(set_wavetable);
        assert_relative_eq!(wavetable_oscillator.wavetable(), expected_wavetable);
    }

    #[test]
    fn blend_between_wavetables() {
        let mut wavetable_oscillator = CircularWavetableOscillator::new(
            [
                &SINE_WAVETABLE,
                &TRIANGLE_WAVETABLE,
                &SAW_WAVETABLE,
                &SAW_WAVETABLE,
                &SAW_WAVETABLE,
                &SAW_WAVETABLE,
                &SAW_WAVETABLE,
                &SAW_WAVETABLE,
            ],
            &SINE_WAVETABLE,
            SAMPLE_RATE,
        );
        wavetable_oscillator.set_frequency(1.0);
        let mut signal = [0.0; SAMPLE_RATE as usize];

        wavetable_oscillator.set_wavetable(0.5);
        wavetable_oscillator.populate(&mut signal);
        assert_equal_mix_of_two_signal_eq(&signal, &sine(), &triangle());
    }

    #[test]
    fn blend_between_wavetables_over_top() {
        let mut wavetable_oscillator = CircularWavetableOscillator::new(
            [
                &SINE_WAVETABLE,
                &SAW_WAVETABLE,
                &SAW_WAVETABLE,
                &SAW_WAVETABLE,
                &SAW_WAVETABLE,
                &SAW_WAVETABLE,
                &SAW_WAVETABLE,
                &TRIANGLE_WAVETABLE,
            ],
            &SINE_WAVETABLE,
            SAMPLE_RATE,
        );
        wavetable_oscillator.set_frequency(1.0);
        let mut signal = [0.0; SAMPLE_RATE as usize];

        wavetable_oscillator.set_wavetable(7.5);
        wavetable_oscillator.populate(&mut signal);
        assert_equal_mix_of_two_signal_eq(&signal, &sine(), &triangle());
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
    fn get_next_wavetable_index() {
        let next_wavetable_index = calculate_next_wavetable_index(1);
        assert_eq!(next_wavetable_index, 2);
    }

    #[test]
    fn get_next_wavetable_index_over_edge() {
        let next_wavetable_index = calculate_next_wavetable_index(MAX_WAVETABLES - 1);
        assert_eq!(next_wavetable_index, 0);
    }

    #[test]
    fn cross_fade_two_bandlimited_wavetables() {
        let band_wavetable_a = SINE_WAVETABLE.band(1.0);
        let band_wavetable_b = TRIANGLE_WAVETABLE.band(1.0);

        let xfade = 0.3;
        let phase = 0.1;
        let value = mix_and_read_wavetables(&band_wavetable_a, &band_wavetable_b, xfade, phase);

        let expected =
            band_wavetable_a.read(phase) * (1.0 - xfade) + band_wavetable_b.read(phase) * xfade;
        assert_relative_eq!(value, expected);
    }

    #[test]
    fn set_fm_multiple() {
        let mut wavetable_oscillator = CircularWavetableOscillator::new(
            [&SINE_WAVETABLE; MAX_WAVETABLES],
            &SINE_WAVETABLE,
            SAMPLE_RATE,
        );
        wavetable_oscillator.set_fm_multiple(0.01);
        assert_relative_eq!(wavetable_oscillator.fm_multiple(), 0.01);
    }

    #[test]
    fn set_fm_intensity() {
        let mut wavetable_oscillator = CircularWavetableOscillator::new(
            [&SINE_WAVETABLE; MAX_WAVETABLES],
            &SINE_WAVETABLE,
            SAMPLE_RATE,
        );
        wavetable_oscillator.set_fm_intensity(0.1);
        assert_relative_eq!(wavetable_oscillator.fm_intensity(), 0.1);
    }
}
