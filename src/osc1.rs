use crate::wavetable_oscillator::{Oscillator, Wavetable, XY0WavetableOscillator};

const MAX_VOICES: u32 = 7;

pub struct Osc1<'a> {
    frequency: f32,
    detune: f32,
    enabled_voices: u32,
    voices: [Voice<'a>; MAX_VOICES as usize],
}

impl<'a> Osc1<'a> {
    pub fn new(
        wavetable_a: &'a Wavetable,
        wavetable_b: &'a Wavetable,
        wavetable_c: &'a Wavetable,
        sample_rate: u32,
    ) -> Self {
        let mut osc1 = Self {
            frequency: 0.0,
            detune: 0.0,
            enabled_voices: 0,
            voices: [
                Voice::new(wavetable_a, wavetable_b, wavetable_c, sample_rate),
                Voice::new(wavetable_a, wavetable_b, wavetable_c, sample_rate),
                Voice::new(wavetable_a, wavetable_b, wavetable_c, sample_rate),
                Voice::new(wavetable_a, wavetable_b, wavetable_c, sample_rate),
                Voice::new(wavetable_a, wavetable_b, wavetable_c, sample_rate),
                Voice::new(wavetable_a, wavetable_b, wavetable_c, sample_rate),
                Voice::new(wavetable_a, wavetable_b, wavetable_c, sample_rate),
            ],
        };
        osc1.set_enabled_voices(1)
            .set_frequency(440.0)
            .set_detune(0.0);
        osc1
    }

    pub fn set_enabled_voices(&mut self, enabled_voices: u32) -> &mut Self {
        assert!((1..=MAX_VOICES).contains(&enabled_voices));
        self.enabled_voices = enabled_voices;
        self.tune_voices();
        self
    }

    pub fn set_detune(&mut self, detune: f32) -> &mut Self {
        self.detune = detune;
        self.tune_voices();
        self
    }

    pub fn set_frequency(&mut self, frequency: f32) -> &mut Self {
        self.frequency = frequency;
        self.tune_voices();
        self
    }

    fn tune_voices(&mut self) {
        let detune_amounts = distribute_detune(self.detune, self.enabled_voices);
        for (i, voice) in self.voices.iter_mut().enumerate() {
            let detuned_frequency = detune_frequency(self.frequency, detune_amounts[i]);
            voice.oscillator.set_frequency(detuned_frequency);
        }
    }

    pub fn set_x(&mut self, x: f32) -> &mut Self {
        for voice in self.voices.iter_mut() {
            voice.oscillator.set_x(x);
        }
        self
    }

    pub fn set_y(&mut self, y: f32) -> &mut Self {
        for voice in self.voices.iter_mut() {
            voice.oscillator.set_y(y);
        }
        self
    }

    pub fn populate(&mut self, buffer: &mut [f32]) {
        self.voices[0].oscillator.populate(buffer);

        for (i, voice) in self.voices[1..].iter_mut().enumerate() {
            if (i + 1) < self.enabled_voices as usize {
                voice.oscillator.add(buffer);
            } else {
                voice.oscillator.dry(buffer);
            }
        }

        for x in buffer.iter_mut() {
            *x /= self.enabled_voices as f32;
        }
    }
}

struct Voice<'a> {
    pub oscillator: XY0WavetableOscillator<'a>,
}

impl<'a> Voice<'a> {
    pub fn new(
        wavetable_a: &'a Wavetable,
        wavetable_b: &'a Wavetable,
        wavetable_c: &'a Wavetable,
        sample_rate: u32,
    ) -> Self {
        Self {
            oscillator: XY0WavetableOscillator::new(
                wavetable_a,
                wavetable_b,
                wavetable_c,
                sample_rate,
            ),
        }
    }
}

fn distribute_detune(detune: f32, enabled_voices: u32) -> [f32; MAX_VOICES as usize] {
    let mut detunes = [0.0; MAX_VOICES as usize];

    let start_index = {
        let odd_voices = enabled_voices % 2 == 1;
        if odd_voices {
            1
        } else {
            0
        }
    };

    let step = if enabled_voices > 1 {
        detune / (enabled_voices / 2) as f32
    } else {
        0.0
    };

    for (i, x) in detunes[start_index..].iter_mut().enumerate() {
        let distance = (i / 2 + 1) as f32;
        let side = match i % 4 {
            0 | 3 => 1.0,
            1 | 2 => -1.0,
            _ => unreachable!(),
        };
        *x = step * distance * side;
    }

    detunes
}

fn detune_frequency(frequency: f32, amount: f32) -> f32 {
    frequency * f32::powf(2.0, amount / 12.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spectral_analysis::SpectralAnalysis;
    use crate::wavetable_oscillator::{digital_saw, sine, triangle};
    use std::f32::consts::PI;

    const SAMPLE_RATE: u32 = 48000;

    lazy_static! {
        static ref SINE_WAVETABLE: Wavetable = Wavetable::new(sine(), SAMPLE_RATE);
        static ref TRIANGLE_WAVETABLE: Wavetable = Wavetable::new(triangle(), SAMPLE_RATE);
        static ref SAW_WAVETABLE: Wavetable = Wavetable::new(digital_saw(), SAMPLE_RATE);
    }

    #[test]
    fn initialize() {
        let _osc1 = Osc1::new(
            &SINE_WAVETABLE,
            &TRIANGLE_WAVETABLE,
            &SAW_WAVETABLE,
            SAMPLE_RATE,
        );
    }

    #[test]
    fn populate() {
        let mut osc1 = Osc1::new(
            &SINE_WAVETABLE,
            &TRIANGLE_WAVETABLE,
            &SAW_WAVETABLE,
            SAMPLE_RATE,
        );

        let mut buffer = [0.0; 2];
        osc1.populate(&mut buffer);

        assert_relative_eq!(buffer[0], 0.0, epsilon = 0.1);
        assert!(buffer[1] > buffer[0]);
    }

    #[test]
    fn set_frequency() {
        let delta_one_tick_frequency_200 = {
            let mut osc1 = Osc1::new(
                &SINE_WAVETABLE,
                &TRIANGLE_WAVETABLE,
                &SAW_WAVETABLE,
                SAMPLE_RATE,
            );
            osc1.set_frequency(200.0);

            let mut buffer = [0.0; 2];
            osc1.populate(&mut buffer);

            buffer[1] - buffer[0]
        };

        let delta_two_ticks_frequency_100 = {
            let mut osc1 = Osc1::new(
                &SINE_WAVETABLE,
                &TRIANGLE_WAVETABLE,
                &SAW_WAVETABLE,
                SAMPLE_RATE,
            );
            osc1.set_frequency(100.0);

            let mut buffer = [0.0; 3];
            osc1.populate(&mut buffer);

            buffer[2] - buffer[0]
        };

        assert_relative_eq!(
            delta_one_tick_frequency_200,
            delta_two_ticks_frequency_100,
            max_relative = 0.001
        );
    }

    #[test]
    fn use_zero_wavetable() {
        let mut osc1 = Osc1::new(
            &SINE_WAVETABLE,
            &TRIANGLE_WAVETABLE,
            &SAW_WAVETABLE,
            SAMPLE_RATE,
        );
        osc1.set_frequency(1.0).set_x(0.0).set_y(0.0);

        let mut buffer = [0.0; SAMPLE_RATE as usize / 8];
        osc1.populate(&mut buffer);
        let value = buffer[buffer.len() - 1];
        assert_relative_eq!(value, f32::sin(0.125 * 2.0 * PI), max_relative = 0.01);

        let mut buffer = [0.0; SAMPLE_RATE as usize / 8];
        osc1.populate(&mut buffer);
        let value = buffer[buffer.len() - 1];
        assert_relative_eq!(value, f32::sin(0.25 * 2.0 * PI), max_relative = 0.01);

        let mut buffer = [0.0; SAMPLE_RATE as usize / 4];
        osc1.populate(&mut buffer);
        let value = buffer[buffer.len() - 1];
        assert_relative_eq!(value, 0.0, epsilon = 0.01);
    }

    #[test]
    fn use_x_wavetable() {
        let mut osc1 = Osc1::new(
            &SINE_WAVETABLE,
            &TRIANGLE_WAVETABLE,
            &SAW_WAVETABLE,
            SAMPLE_RATE,
        );
        osc1.set_frequency(1.0).set_x(1.0).set_y(0.0);

        let mut buffer = [0.0; SAMPLE_RATE as usize / 8];
        osc1.populate(&mut buffer);
        let value = buffer[buffer.len() - 1];
        assert_relative_eq!(value, 0.5, max_relative = 0.01);

        let mut buffer = [0.0; SAMPLE_RATE as usize / 8];
        osc1.populate(&mut buffer);
        let value = buffer[buffer.len() - 1];
        assert_relative_eq!(value, 1.0, max_relative = 0.01);

        let mut buffer = [0.0; SAMPLE_RATE as usize / 4];
        osc1.populate(&mut buffer);
        let value = buffer[buffer.len() - 1];
        assert_relative_eq!(value, 0.0, epsilon = 0.01);
    }

    #[test]
    fn use_y_wavetable() {
        let mut osc1 = Osc1::new(
            &SINE_WAVETABLE,
            &TRIANGLE_WAVETABLE,
            &SAW_WAVETABLE,
            SAMPLE_RATE,
        );
        osc1.set_frequency(1.0).set_x(0.0).set_y(1.0);

        let mut buffer = [0.0; SAMPLE_RATE as usize / 4];
        osc1.populate(&mut buffer);
        let value = buffer[buffer.len() - 1];
        assert_relative_eq!(value, 0.5, epsilon = 0.1);

        let mut buffer = [0.0; SAMPLE_RATE as usize / 4 - 19];
        osc1.populate(&mut buffer);
        let value = buffer[buffer.len() - 1];
        assert_relative_eq!(value, 1.0, max_relative = 0.05);
    }

    #[test]
    fn two_voices() {
        let mut osc1 = Osc1::new(
            &SINE_WAVETABLE,
            &TRIANGLE_WAVETABLE,
            &SAW_WAVETABLE,
            SAMPLE_RATE,
        );
        osc1.set_frequency(1000.0)
            .set_enabled_voices(2)
            .set_detune(2.0);

        let lower_frequency = 1000.0 / f32::powf(2.0, 2.0 / 12.0);
        let higher_frequency = 1000.0 * f32::powf(2.0, 2.0 / 12.0);

        let mut signal = [0.0; SAMPLE_RATE as usize];
        osc1.populate(&mut signal);

        let analysis = SpectralAnalysis::analyze(&signal, SAMPLE_RATE);
        let center_magnitude = analysis.magnitude(1000.0);
        let lower_magnitude = analysis.magnitude(lower_frequency);
        let higher_magnitude = analysis.magnitude(higher_frequency);
        assert!(lower_magnitude / center_magnitude > 10.0);
        assert!(higher_magnitude / center_magnitude > 10.0);
    }

    #[test]
    fn three_voices() {
        let mut osc1 = Osc1::new(
            &SINE_WAVETABLE,
            &TRIANGLE_WAVETABLE,
            &SAW_WAVETABLE,
            SAMPLE_RATE,
        );
        osc1.set_frequency(1000.0)
            .set_enabled_voices(3)
            .set_detune(2.0);

        let lower_frequency = 1000.0 / f32::powf(2.0, 2.0 / 12.0);
        let higher_frequency = 1000.0 * f32::powf(2.0, 2.0 / 12.0);
        let off_frequency = (lower_frequency + higher_frequency) / 2.0;

        let mut signal = [0.0; SAMPLE_RATE as usize];
        osc1.populate(&mut signal);

        let analysis = SpectralAnalysis::analyze(&signal, SAMPLE_RATE);
        let center_magnitude = analysis.magnitude(1000.0);
        let lower_magnitude = analysis.magnitude(lower_frequency);
        let higher_magnitude = analysis.magnitude(higher_frequency);
        let off_magnitude = analysis.magnitude(off_frequency);
        assert!(center_magnitude / off_magnitude > 10.0);
        assert!(lower_magnitude / off_magnitude > 10.0);
        assert!(higher_magnitude / off_magnitude > 10.0);
    }

    #[test]
    #[should_panic]
    fn voices_over_limit() {
        let mut osc1 = Osc1::new(
            &SINE_WAVETABLE,
            &TRIANGLE_WAVETABLE,
            &SAW_WAVETABLE,
            SAMPLE_RATE,
        );
        osc1.set_enabled_voices(20);
    }

    #[test]
    #[should_panic]
    fn voices_under_limit() {
        let mut osc1 = Osc1::new(
            &SINE_WAVETABLE,
            &TRIANGLE_WAVETABLE,
            &SAW_WAVETABLE,
            SAMPLE_RATE,
        );
        osc1.set_enabled_voices(0);
    }

    #[test]
    fn distribute_detune_odd() {
        let detune = 1.0;
        let voices = 5;
        let detunes = distribute_detune(detune, voices);

        assert_relative_eq!(detunes[0], 0.0);
        assert_relative_eq!(detunes[1], 0.5);
        assert_relative_eq!(detunes[2], -0.5);
        assert_relative_eq!(detunes[3], -1.0);
        assert_relative_eq!(detunes[4], 1.0);
    }

    #[test]
    fn distribute_detune_even() {
        let detune = 1.0;
        let voices = 4;
        let detunes = distribute_detune(detune, voices);

        assert_relative_eq!(detunes[0], 0.5);
        assert_relative_eq!(detunes[1], -0.5);
        assert_relative_eq!(detunes[2], -1.0);
        assert_relative_eq!(detunes[3], 1.0);
    }

    #[test]
    fn distribute_detune_full() {
        let detune = 3.0;
        let detunes = distribute_detune(detune, MAX_VOICES);

        assert_relative_eq!(detunes[0], 0.0);
        assert_relative_eq!(detunes[1], 1.0);
        assert_relative_eq!(detunes[2], -1.0);
        assert_relative_eq!(detunes[3], -2.0);
        assert_relative_eq!(detunes[4], 2.0);
        assert_relative_eq!(detunes[5], 3.0);
        assert_relative_eq!(detunes[6], -3.0);
    }

    #[test]
    fn detune_frequency_by_zero() {
        const A4: f32 = 440.0;

        let detuned = detune_frequency(A4, 0.0);

        assert_relative_eq!(detuned, A4, epsilon = 0.001);
    }

    #[test]
    fn detune_frequency_down() {
        const G4: f32 = 391.995;
        const A4: f32 = 440.0;

        let detuned = detune_frequency(A4, -2.0);

        assert_relative_eq!(detuned, G4, epsilon = 0.001);
    }

    #[test]
    fn detune_frequency_up() {
        const A4: f32 = 440.0;
        const B4: f32 = 493.883;

        let detuned = detune_frequency(A4, 2.0);

        assert_relative_eq!(detuned, B4, epsilon = 0.001);
    }

    #[test]
    fn no_clipping() {
        assert_no_clipping(1);
        assert_no_clipping(4);
        assert_no_clipping(7);
    }

    fn assert_no_clipping(voices: u32) {
        let mut osc1 = Osc1::new(
            &SINE_WAVETABLE,
            &TRIANGLE_WAVETABLE,
            &SAW_WAVETABLE,
            SAMPLE_RATE,
        );
        osc1.set_frequency(100.0)
            .set_enabled_voices(voices)
            .set_detune(21.0)
            .set_y(1.0);

        let mut signal = [0.0; SAMPLE_RATE as usize];
        osc1.populate(&mut signal);

        let max = signal.iter().fold(0.0, |a, b| f32::max(a, b.abs()));

        assert!(max < 1.0);
    }
}
