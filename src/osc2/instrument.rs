use super::breadth;
use super::detune;
use super::pan;
use super::wave;
use crate::wavetable_oscillator::circular_wavetable_oscillator;
use crate::wavetable_oscillator::{
    CircularWavetableOscillator, Oscillator, StereoOscillator, Wavetable,
};

pub const WAVETABLES_LEN: usize = circular_wavetable_oscillator::MAX_WAVETABLES;

const VOICES_LEN: usize = 5;
const CENTER_VOICE: usize = 2;

pub struct Osc2<'a, 'b> {
    detune: f32,
    frequency: f32,
    breadth: f32,
    pan_combination: f32,
    wavetable: f32,
    wavetable_spread: f32,
    total_amplitude: f32,
    voices: [Voice<'a, 'b>; VOICES_LEN],
}

// TODO: Pass FM wavetable from the outside
impl<'a, 'b> Osc2<'a, 'b> {
    pub fn new(
        wavetables: [&'a Wavetable; WAVETABLES_LEN],
        fm_wavetable: &'b Wavetable,
        sample_rate: u32,
    ) -> Self {
        let mut osc2 = Self {
            detune: 0.0,
            frequency: 0.0,
            breadth: 0.0,
            pan_combination: 0.0,
            wavetable: 0.0,
            wavetable_spread: 0.0,
            total_amplitude: 0.0,
            voices: [
                Voice::new(wavetables, fm_wavetable, sample_rate),
                Voice::new(wavetables, fm_wavetable, sample_rate),
                Voice::new(wavetables, fm_wavetable, sample_rate),
                Voice::new(wavetables, fm_wavetable, sample_rate),
                Voice::new(wavetables, fm_wavetable, sample_rate),
            ],
        };
        osc2.tune_voices();
        osc2.amplify_voices();
        osc2.shape_voices();
        osc2.set_pan_combination(0.0);
        osc2
    }

    pub fn set_frequency(&mut self, frequency: f32) -> &mut Self {
        self.frequency = frequency;
        self.tune_voices();
        self.amplify_voices();
        self
    }

    pub fn frequency(&self) -> f32 {
        self.frequency
    }

    pub fn set_detune(&mut self, detune: f32) -> &mut Self {
        self.detune = detune;
        self.tune_voices();
        self.amplify_voices();
        self
    }

    pub fn detune(&self) -> f32 {
        self.detune
    }

    pub fn set_breadth(&mut self, breadth: f32) -> &mut Self {
        assert!(breadth >= 0.0);
        self.breadth = breadth;
        self.amplify_voices();
        self
    }

    pub fn breadth(&self) -> f32 {
        self.breadth
    }

    pub fn set_pan_combination(&mut self, pan_combination: f32) -> &mut Self {
        self.pan_combination = pan_combination;
        let pans = pan::distribute(self.pan_combination);
        for (i, voice) in self.voices.iter_mut().enumerate() {
            voice.oscillator.set_pan(pans[i]);
        }
        self
    }

    pub fn pan_combination(&self) -> f32 {
        self.pan_combination
    }

    pub fn set_wavetable(&mut self, wavetable: f32) -> &mut Self {
        self.wavetable = wavetable;
        self.shape_voices();
        self
    }

    pub fn wavetable(&self) -> f32 {
        self.wavetable
    }

    pub fn set_wavetable_spread(&mut self, spread: f32) -> &mut Self {
        self.wavetable_spread = spread;
        self.shape_voices();
        self
    }

    pub fn wavetable_spread(&self) -> f32 {
        self.wavetable_spread
    }

    pub fn set_fm_multiple(&mut self, multiple: f32) -> &mut Self {
        self.voices.iter_mut().for_each(|v| {
            v.oscillator.set_fm_multiple(multiple);
        });
        self
    }

    pub fn set_fm_intensity(&mut self, intensity: f32) -> &mut Self {
        self.voices.iter_mut().for_each(|v| {
            v.oscillator.set_fm_intensity(intensity);
        });
        self
    }

    pub fn reset_phase(&mut self) -> &mut Self {
        self.voices.iter_mut().for_each(|v| {
            v.oscillator.reset_phase();
        });
        self
    }

    pub fn populate(&mut self, buffers: &mut [&mut [f32]]) {
        self.voices[CENTER_VOICE]
            .oscillator
            .populate_stereo(buffers);

        for (i, voice) in self.voices.iter_mut().enumerate() {
            if i != CENTER_VOICE {
                voice.oscillator.add_stereo(buffers);
            }
        }

        for buffer in buffers.iter_mut() {
            for x in buffer.iter_mut() {
                *x /= self.total_amplitude;
            }
        }
    }

    fn tune_voices(&mut self) {
        let detunes = detune::distribute(self.frequency, self.detune);
        self.voices.iter_mut().enumerate().for_each(|(i, v)| {
            v.oscillator.set_frequency(detunes[i]);
        });
    }

    fn amplify_voices(&mut self) {
        let mut breadths = breadth::distribute(self.breadth);

        self.voices.iter_mut().enumerate().for_each(|(i, v)| {
            let frequency = v.oscillator.frequency();
            if frequency < 20.0 {
                breadths[i] *= (frequency - 15.0).max(0.0) / 5.0;
            }
            v.oscillator.set_amplitude(breadths[i]);
        });

        self.total_amplitude = calculate_total_amplitude(&breadths);
    }

    fn shape_voices(&mut self) {
        let waves = wave::distribute(self.wavetable, self.wavetable_spread);
        self.voices.iter_mut().enumerate().for_each(|(i, v)| {
            v.oscillator.set_wavetable(waves[i]);
        });
    }
}

fn calculate_total_amplitude(amplitudes: &[f32]) -> f32 {
    let amplitudes_sum = amplitudes.iter().fold(0.0, |a, b| a + b);
    if amplitudes_sum > VOICES_LEN as f32 {
        VOICES_LEN as f32
    } else {
        amplitudes_sum + (VOICES_LEN as f32 - amplitudes_sum) * 0.8
    }
}

struct Voice<'a, 'b> {
    oscillator: CircularWavetableOscillator<'a, 'b>,
}

impl<'a, 'b> Voice<'a, 'b> {
    pub fn new(
        wavetables: [&'a Wavetable; WAVETABLES_LEN],
        fm_wavetable: &'b Wavetable,
        sample_rate: u32,
    ) -> Self {
        Self {
            oscillator: CircularWavetableOscillator::new(wavetables, fm_wavetable, sample_rate),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spectral_analysis::SpectralAnalysis;
    use crate::tone;
    use crate::wavetable_oscillator::{pulse, saw, sine, triangle};
    use proptest::prelude::*;

    const SAMPLE_RATE: u32 = 48000;

    lazy_static! {
        static ref SINE_WAVETABLE: Wavetable = Wavetable::new(sine(), SAMPLE_RATE);
        static ref TRIANGLE_WAVETABLE: Wavetable = Wavetable::new(triangle(), SAMPLE_RATE);
        static ref SAW_WAVETABLE: Wavetable = Wavetable::new(saw(), SAMPLE_RATE);
        static ref PULSE_WAVETABLE: Wavetable = Wavetable::new(pulse(0.5), SAMPLE_RATE);
    }

    fn wavetables() -> [&'static Wavetable; WAVETABLES_LEN] {
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
        let _osc2 = Osc2::new(wavetables(), &SINE_WAVETABLE, SAMPLE_RATE);
    }

    #[test]
    fn populate_buffer() {
        let mut osc2 = Osc2::new(wavetables(), &SINE_WAVETABLE, SAMPLE_RATE);
        osc2.set_frequency(440.0);

        let mut buffer_left = [0.0; 8];
        let mut buffer_right = [0.0; 8];
        osc2.populate(&mut [&mut buffer_left[..], &mut buffer_right[..]]);

        buffer_left
            .iter()
            .find(|x| **x > 0.0)
            .expect("there must be at least one value over zero");
        buffer_right
            .iter()
            .find(|x| **x > 0.0)
            .expect("there must be at least one value over zero");
    }

    #[test]
    fn set_frequency() {
        let mut osc2 = Osc2::new(wavetables(), &SINE_WAVETABLE, SAMPLE_RATE);
        osc2.set_frequency(880.0);
        assert_eq!(osc2.frequency(), 880.0);
    }

    #[test]
    fn set_wavetable() {
        let mut osc2 = Osc2::new(wavetables(), &SINE_WAVETABLE, SAMPLE_RATE);
        osc2.set_wavetable(2.1);
        assert_eq!(osc2.wavetable(), 2.1);
    }

    #[test]
    fn set_wavetable_spread() {
        let mut osc2 = Osc2::new(wavetables(), &SINE_WAVETABLE, SAMPLE_RATE);
        osc2.set_wavetable_spread(0.1);
        assert_eq!(osc2.wavetable_spread(), 0.1);
    }

    #[test]
    fn set_detune() {
        let mut osc2 = Osc2::new(wavetables(), &SINE_WAVETABLE, SAMPLE_RATE);
        osc2.set_detune(2.0);
        assert_eq!(osc2.detune(), 2.0);
    }

    #[test]
    fn set_breadth() {
        let mut osc2 = Osc2::new(wavetables(), &SINE_WAVETABLE, SAMPLE_RATE);
        osc2.set_breadth(0.5);
        assert_eq!(osc2.breadth(), 0.5);
    }

    #[test]
    fn set_pan_combination() {
        let mut osc2 = Osc2::new(wavetables(), &SINE_WAVETABLE, SAMPLE_RATE);
        osc2.set_pan_combination(0.5);
        assert_eq!(osc2.pan_combination(), 0.5);
    }

    #[test]
    fn reset_phase() {
        let mut osc2 = Osc2::new(wavetables(), &SINE_WAVETABLE, SAMPLE_RATE);
        osc2.set_frequency(440.0);

        let mut signal_a_left = [0.0; 20];
        let mut signal_a_right = [0.0; 20];
        osc2.populate(&mut [&mut signal_a_left[..], &mut signal_a_right[..]]);

        osc2.reset_phase();
        let mut signal_b_left = [0.0; 20];
        let mut signal_b_right = [0.0; 20];
        osc2.populate(&mut [&mut signal_b_left[..], &mut signal_b_right[..]]);

        for i in 0..20 {
            assert_relative_eq!(signal_a_left[i], signal_b_left[i]);
            assert_relative_eq!(signal_a_right[i], signal_b_right[i]);
        }
    }

    #[test]
    fn detuned_voices_full_breadth() {
        let mut osc2 = Osc2::new(wavetables(), &SINE_WAVETABLE, SAMPLE_RATE);
        osc2.set_frequency(1000.0).set_detune(2.0).set_breadth(2.0);

        let magnitudes = voice_magnitudes_linear_detune(&mut osc2);
        assert!(magnitudes.lowest / magnitudes.off > 10.0);
        assert!(magnitudes.lower / magnitudes.off > 10.0);
        assert!(magnitudes.center / magnitudes.off > 10.0);
        assert!(magnitudes.higher / magnitudes.off > 10.0);
        assert!(magnitudes.highest / magnitudes.off > 10.0);
    }

    struct VoiceMagnitudes {
        lowest: f32,
        lower: f32,
        center: f32,
        higher: f32,
        highest: f32,
        off: f32,
    }

    fn voice_magnitudes_linear_detune(osc2: &mut Osc2) -> VoiceMagnitudes {
        let center_frequency = osc2.frequency();
        let lower_frequency = osc2.frequency() / f32::powf(2.0, (osc2.detune() / 2.0) / 12.0);
        let lowest_frequency = osc2.frequency() / f32::powf(2.0, osc2.detune() / 12.0);
        let higher_frequency = osc2.frequency() * f32::powf(2.0, (osc2.detune() / 2.0) / 12.0);
        let highest_frequency = osc2.frequency() * f32::powf(2.0, osc2.detune() / 12.0);
        let off_frequency = (lower_frequency + lowest_frequency) / 2.0;

        let mut signal_left = [0.0; SAMPLE_RATE as usize];
        let mut signal_right = [0.0; SAMPLE_RATE as usize];
        osc2.populate(&mut [&mut signal_left[..], &mut signal_right[..]]);

        let analysis = SpectralAnalysis::analyze(&signal_left, SAMPLE_RATE);

        VoiceMagnitudes {
            center: analysis.magnitude(center_frequency),
            lower: analysis.magnitude(lower_frequency),
            lowest: analysis.magnitude(lowest_frequency),
            higher: analysis.magnitude(higher_frequency),
            highest: analysis.magnitude(highest_frequency),
            off: analysis.magnitude(off_frequency),
        }
    }

    #[test]
    fn no_aliasing_of_high_detuned_voices_with_sine() {
        const FREQUENCY: f32 = 23400.0;
        const DETUNE: f32 = 7.0;

        let mut osc2 = Osc2::new(wavetables(), &SINE_WAVETABLE, SAMPLE_RATE);
        osc2.set_frequency(FREQUENCY).set_detune(DETUNE);

        let lowest_expected = tone::detune_frequency(FREQUENCY, -DETUNE);

        assert_no_aliasing(osc2, lowest_expected);
    }

    #[test]
    fn no_aliasing_of_high_detuned_voices_with_sine_and_voices_over_limits() {
        const FREQUENCY: f32 = 18200.0;
        const DETUNE: f32 = 12.0;
        const BREADTH: f32 = 10.0;

        let mut osc2 = Osc2::new(wavetables(), &SINE_WAVETABLE, SAMPLE_RATE);
        osc2.set_frequency(FREQUENCY)
            .set_detune(DETUNE)
            .set_breadth(BREADTH);

        let lowest_expected = tone::detune_frequency(FREQUENCY, -DETUNE);

        assert_no_aliasing(osc2, lowest_expected);
    }

    fn assert_no_aliasing(mut osc2: Osc2, lowest_expected: f32) {
        let mut buffer_left = [0.0; SAMPLE_RATE as usize];
        let mut buffer_right = [0.0; SAMPLE_RATE as usize];
        osc2.populate(&mut [&mut buffer_left[..], &mut buffer_right[..]]);

        assert_no_aliasing_in_buffer(&buffer_left, lowest_expected, SAMPLE_RATE);
        assert_no_aliasing_in_buffer(&buffer_right, lowest_expected, SAMPLE_RATE);
    }

    fn assert_no_aliasing_in_buffer(buffer: &[f32], lowest_expected: f32, sample_rate: u32) {
        let mut analysis = SpectralAnalysis::analyze(&buffer, sample_rate);
        analysis.trash_range(0.0, 1.0);

        let lowest_peak = analysis.lowest_peak(0.04);
        if relative_eq!(lowest_peak, 0.0) {
            return;
        }

        assert!(
            lowest_peak >= lowest_expected - 1.0,
            "expected >= {}, obtained {}",
            lowest_expected,
            lowest_peak
        );
    }

    #[test]
    fn total_amplitude_single_voice() {
        let breadths = [0.0, 0.0, 1.0, 0.0, 0.0];
        let amplitude = calculate_total_amplitude(&breadths);
        assert_relative_eq!(amplitude, 1.0 + 0.8 * 4.0);
    }

    #[test]
    fn total_amplitude_over_limit() {
        let breadths = [2.0, 2.0, 1.0, 2.0, 2.0];
        let amplitude = calculate_total_amplitude(&breadths);
        assert_relative_eq!(amplitude, VOICES_LEN as f32);
    }

    #[test]
    fn zero_pan_combination() {
        let mut osc2 = Osc2::new(wavetables(), &SINE_WAVETABLE, SAMPLE_RATE);
        osc2.set_frequency(440.0).set_pan_combination(0.0);

        let mut signal_a_left = [0.0; 20];
        let mut signal_a_right = [0.0; 20];
        osc2.populate(&mut [&mut signal_a_left[..], &mut signal_a_right[..]]);

        for i in 0..signal_a_left.len() {
            assert_relative_eq!(signal_a_left[i], signal_a_right[i]);
        }
    }

    #[derive(Debug)]
    struct Osc2Config {
        frequency: f32,
        detune: f32,
        wavetable: f32,
        breadth: f32,
    }

    prop_compose! {
        fn arbitrary_config()
            (
                frequency in 11.0f32..24000.0,
                detune in -13.0f32..13.0,
                wavetable in -16.0f32..16.0,
                breadth in 0.0f32..36.0,
            )
            -> Osc2Config
        {
            Osc2Config {
                frequency,
                detune,
                wavetable,
                breadth,
            }
        }
    }

    proptest! {
        #[test]
        #[ignore] // too slow for regular execution
        fn no_clipping(config in arbitrary_config()) {
        let mut osc2 = Osc2::new(wavetables(), &SINE_WAVETABLE, SAMPLE_RATE);
            osc2
                .set_frequency(config.frequency)
                .set_detune(config.detune)
                .set_breadth(config.breadth)
                .set_wavetable(config.wavetable);

            let mut buffer_right = [0.0; SAMPLE_RATE as usize];
            let mut buffer_left = [0.0; SAMPLE_RATE as usize];
            osc2.populate(&mut [&mut buffer_left[..], &mut buffer_right[..]]);

            prop_assert!(buffer_left
                .iter()
                .find(|x| **x > 1.0).is_none());
            prop_assert!(buffer_right
                .iter()
                .find(|x| **x > 1.0).is_none());
        }

        #[test]
        #[ignore] // too slow for regular execution
        fn no_aliasing(config in arbitrary_config()) {
        let mut osc2 = Osc2::new(wavetables(), &SINE_WAVETABLE, SAMPLE_RATE);
            osc2
                .set_frequency(config.frequency)
                .set_detune(config.detune)
                .set_breadth(config.breadth)
                .set_wavetable(config.wavetable);

            let lowest_expected = tone::detune_frequency(config.frequency, -config.detune.abs());
            if lowest_expected < 0.0 {
                return Err(TestCaseError::Reject("voices go below zero hz".into()));
            }

            assert_no_aliasing(osc2, lowest_expected);
        }
    }
}
