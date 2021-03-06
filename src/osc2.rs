use crate::tone;
use crate::wavetable_oscillator::circular_wavetable_oscillator;
use crate::wavetable_oscillator::{CircularWavetableOscillator, Oscillator, Wavetable};

pub const WAVETABLES_LEN: usize = circular_wavetable_oscillator::MAX_WAVETABLES;

const VOICES_LEN: usize = 5;
const CENTER_VOICE: usize = 2;

pub struct Osc2<'a> {
    detune: f32,
    frequency: f32,
    voices: [Voice<'a>; VOICES_LEN],
}

impl<'a> Osc2<'a> {
    pub fn new(wavetables: [&'a Wavetable; WAVETABLES_LEN], sample_rate: u32) -> Self {
        let mut osc2 = Self {
            detune: 0.0,
            frequency: 0.0,
            voices: [
                Voice::new(wavetables, sample_rate),
                Voice::new(wavetables, sample_rate),
                Voice::new(wavetables, sample_rate),
                Voice::new(wavetables, sample_rate),
                Voice::new(wavetables, sample_rate),
            ],
        };
        osc2.tune_voices();
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
        let detunes = distribute_detune(self.frequency, self.detune);
        self.voices.iter_mut().enumerate().for_each(|(i, v)| {
            v.oscillator.set_frequency(detunes[i]);
        });
    }

    pub fn set_wavetable(&mut self, wavetable: f32) {
        self.voices.iter_mut().for_each(|v| {
            v.oscillator.set_wavetable(wavetable);
        });
    }

    pub fn wavetable(&self) -> f32 {
        self.voices[CENTER_VOICE].oscillator.wavetable()
    }

    pub fn populate(&mut self, buffer: &mut [f32]) {
        self.voices[CENTER_VOICE].oscillator.populate(buffer);

        for (i, voice) in self.voices.iter_mut().enumerate() {
            if i != CENTER_VOICE {
                voice.oscillator.add(buffer);
            }
        }

        for x in buffer.iter_mut() {
            *x /= VOICES_LEN as f32;
        }
    }
}

fn distribute_detune(frequency: f32, detune: f32) -> [f32; VOICES_LEN] {
    [
        tone::detune_frequency(frequency, detune),
        tone::detune_frequency(frequency, -detune / 2.0),
        frequency,
        tone::detune_frequency(frequency, detune / 2.0),
        tone::detune_frequency(frequency, -detune),
    ]
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spectral_analysis::SpectralAnalysis;
    use crate::wavetable_oscillator::{pulse, saw, sine, triangle};

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
    fn equal_detune_distribution() {
        const G4: f32 = 391.995;
        const G_SHARP_4: f32 = 415.305;
        const A4: f32 = 440.0;
        const A_SHARP_4: f32 = 466.164;
        const B4: f32 = 493.883;

        let detune = 2.0;
        let detuned = distribute_detune(A4, detune);

        assert_relative_eq!(detuned[0], B4, epsilon = 0.001);
        assert_relative_eq!(detuned[1], G_SHARP_4, epsilon = 0.001);
        assert_relative_eq!(detuned[CENTER_VOICE], A4, epsilon = 0.001);
        assert_relative_eq!(detuned[3], A_SHARP_4, epsilon = 0.001);
        assert_relative_eq!(detuned[4], G4, epsilon = 0.001);
    }

    #[test]
    fn detuned_voices() {
        let center_frequency = 1000.0;
        let lower_frequency_1 = center_frequency / f32::powf(2.0, 1.0 / 12.0);
        let lower_frequency_2 = center_frequency / f32::powf(2.0, 2.0 / 12.0);
        let higher_frequency_1 = center_frequency * f32::powf(2.0, 1.0 / 12.0);
        let higher_frequency_2 = center_frequency * f32::powf(2.0, 2.0 / 12.0);

        let mut osc2 = Osc2::new(wavetables(), SAMPLE_RATE);
        osc2.set_frequency(center_frequency).set_detune(2.0);

        let off_frequency = (lower_frequency_1 + lower_frequency_2) / 2.0;

        let mut signal = [0.0; SAMPLE_RATE as usize];
        osc2.populate(&mut signal);

        let analysis = SpectralAnalysis::analyze(&signal, SAMPLE_RATE);
        let center_magnitude = analysis.magnitude(center_frequency);
        let lower_magnitude_1 = analysis.magnitude(lower_frequency_1);
        let lower_magnitude_2 = analysis.magnitude(lower_frequency_2);
        let higher_magnitude_1 = analysis.magnitude(higher_frequency_1);
        let higher_magnitude_2 = analysis.magnitude(higher_frequency_2);
        let off_magnitude = analysis.magnitude(off_frequency);

        assert!(center_magnitude / off_magnitude > 10.0);
        assert!(lower_magnitude_1 / off_magnitude > 10.0);
        assert!(lower_magnitude_2 / off_magnitude > 10.0);
        assert!(higher_magnitude_1 / off_magnitude > 10.0);
        assert!(higher_magnitude_2 / off_magnitude > 10.0);
    }
}
