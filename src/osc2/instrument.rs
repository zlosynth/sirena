use super::breadth;
use super::detune;
use crate::wavetable_oscillator::circular_wavetable_oscillator;
use crate::wavetable_oscillator::{CircularWavetableOscillator, Oscillator, Wavetable};

pub const WAVETABLES_LEN: usize = circular_wavetable_oscillator::MAX_WAVETABLES;

const VOICES_LEN: usize = 5;
const CENTER_VOICE: usize = 2;

pub struct Osc2<'a> {
    detune: f32,
    frequency: f32,
    breadth: f32,
    total_amplitude: f32,
    voices: [Voice<'a>; VOICES_LEN],
}

impl<'a> Osc2<'a> {
    pub fn new(wavetables: [&'a Wavetable; WAVETABLES_LEN], sample_rate: u32) -> Self {
        let mut osc2 = Self {
            detune: 0.0,
            frequency: 0.0,
            breadth: 0.0,
            total_amplitude: 0.0,
            voices: [
                Voice::new(wavetables, sample_rate),
                Voice::new(wavetables, sample_rate),
                Voice::new(wavetables, sample_rate),
                Voice::new(wavetables, sample_rate),
                Voice::new(wavetables, sample_rate),
            ],
        };
        osc2.tune_voices();
        osc2.amplify_voices();
        osc2
    }

    pub fn set_frequency(&mut self, frequency: f32) -> &mut Self {
        self.frequency = frequency;
        self.tune_voices();
        self
    }

    pub fn frequency(&self) -> f32 {
        self.frequency
    }

    pub fn set_detune(&mut self, detune: f32) -> &mut Self {
        self.detune = detune;
        self.tune_voices();
        self
    }

    pub fn detune(&self) -> f32 {
        self.detune
    }

    fn tune_voices(&mut self) {
        let detunes = detune::distribute(self.frequency, self.detune);
        self.voices.iter_mut().enumerate().for_each(|(i, v)| {
            v.set_frequency(detunes[i]);
        });
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

    fn amplify_voices(&mut self) {
        let breadths = breadth::distribute(self.breadth);

        self.voices.iter_mut().enumerate().for_each(|(i, v)| {
            v.oscillator.set_amplitude(breadths[i]);
        });

        self.total_amplitude = calculate_total_amplitude(&breadths);
    }

    pub fn set_wavetable(&mut self, wavetable: f32) {
        self.voices.iter_mut().for_each(|v| {
            v.oscillator.set_wavetable(wavetable);
        });
    }

    pub fn wavetable(&self) -> f32 {
        self.voices[CENTER_VOICE].oscillator.wavetable()
    }

    pub fn reset_phase(&mut self) -> &mut Self {
        self.voices.iter_mut().for_each(|v| {
            v.oscillator.reset_phase();
        });
        self
    }

    pub fn populate(&mut self, buffer: &mut [f32]) {
        self.voices[CENTER_VOICE].oscillator.populate(buffer);

        for (i, voice) in self.voices.iter_mut().enumerate() {
            if i != CENTER_VOICE {
                voice.oscillator.add(buffer);
            }
        }

        for x in buffer.iter_mut() {
            *x /= self.total_amplitude;
        }
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

struct Voice<'a> {
    oscillator: CircularWavetableOscillator<'a>,
}

impl<'a> Voice<'a> {
    pub fn new(wavetables: [&'a Wavetable; WAVETABLES_LEN], sample_rate: u32) -> Self {
        Self {
            oscillator: CircularWavetableOscillator::new(wavetables, sample_rate),
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) -> &mut Self {
        self.oscillator.set_frequency(frequency);
        self
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
        let _osc2 = Osc2::new(wavetables(), SAMPLE_RATE);
    }

    #[test]
    fn populate_buffer() {
        let mut osc2 = Osc2::new(wavetables(), SAMPLE_RATE);
        osc2.set_frequency(440.0);

        let mut buffer = [0.0; 8];
        osc2.populate(&mut buffer);

        buffer
            .iter()
            .find(|x| **x > 0.0)
            .expect("there must be at least one value over zero");
    }

    #[test]
    fn set_frequency() {
        let mut osc2 = Osc2::new(wavetables(), SAMPLE_RATE);
        osc2.set_frequency(880.0);
        assert_eq!(osc2.frequency(), 880.0);
    }

    #[test]
    fn set_wavetable() {
        let mut osc2 = Osc2::new(wavetables(), SAMPLE_RATE);
        osc2.set_wavetable(2.1);
        assert_eq!(osc2.wavetable(), 2.1);
    }

    #[test]
    fn set_detune() {
        let mut osc2 = Osc2::new(wavetables(), SAMPLE_RATE);
        osc2.set_detune(2.0);
        assert_eq!(osc2.detune(), 2.0);
    }

    #[test]
    fn set_breadth() {
        let mut osc2 = Osc2::new(wavetables(), SAMPLE_RATE);
        osc2.set_breadth(0.5);
        assert_eq!(osc2.breadth(), 0.5);
    }

    #[test]
    fn reset_phase() {
        let mut osc2 = Osc2::new(wavetables(), SAMPLE_RATE);
        osc2.set_frequency(440.0);

        let mut signal_a = [0.0; 20];
        osc2.populate(&mut signal_a);

        osc2.reset_phase();
        let mut signal_b = [0.0; 20];
        osc2.populate(&mut signal_b);

        for i in 0..20 {
            assert_relative_eq!(signal_a[i], signal_b[i]);
        }
    }

    #[test]
    fn detuned_voices_full_breadth() {
        let mut osc2 = Osc2::new(wavetables(), SAMPLE_RATE);
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

        let mut signal = [0.0; SAMPLE_RATE as usize];
        osc2.populate(&mut signal);

        let analysis = SpectralAnalysis::analyze(&signal, SAMPLE_RATE);

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

        let mut osc2 = Osc2::new(wavetables(), SAMPLE_RATE);
        osc2.set_frequency(FREQUENCY).set_detune(DETUNE);

        let lowest_expected = tone::detune_frequency(FREQUENCY, -DETUNE);

        assert_no_aliasing(osc2, lowest_expected);
    }

    fn assert_no_aliasing(mut osc2: Osc2, lowest_expected: f32) {
        let mut buffer = [0.0; SAMPLE_RATE as usize];
        osc2.populate(&mut buffer);

        let mut analysis = SpectralAnalysis::analyze(&buffer, SAMPLE_RATE);
        analysis.trash_range(0.0, 1.0);

        let lowest_peak = analysis.lowest_peak(0.04);
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
                frequency in 0.0f32..24000.0,
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
            let mut osc2 = Osc2::new(wavetables(), SAMPLE_RATE);
            osc2
                .set_frequency(config.frequency)
                .set_detune(config.detune)
                .set_breadth(config.breadth)
                .set_wavetable(config.wavetable);

            let mut buffer = [0.0; SAMPLE_RATE as usize];
            osc2.populate(&mut buffer);

            prop_assert!(buffer
                .iter()
                .find(|x| **x > 1.0).is_none());
        }

        #[test]
        #[ignore] // too slow for regular execution
        fn no_aliasing(config in arbitrary_config()) {
            let mut osc2 = Osc2::new(wavetables(), SAMPLE_RATE);
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
