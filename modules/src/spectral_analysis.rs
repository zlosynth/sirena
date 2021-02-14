use rustfft::num_complex::Complex32;
use rustfft::FftPlanner;

pub struct SpectralAnalysis {
    bins: Vec<f32>,
    bin_width: f32,
}

impl SpectralAnalysis {
    pub fn analyze(signal: &[f32], sample_rate: u32) -> Self {
        let magnitude = fft_magnitude(&signal);
        let bins: Vec<f32> = magnitude.iter().take(signal.len() / 2).copied().collect();
        let bin_width = sample_rate as f32 / signal.len() as f32;
        Self { bins, bin_width }
    }

    pub fn magnitude(&self, frequency: f32) -> f32 {
        let bin_index = self.index(frequency);

        if bin_index > self.bins.len() {
            0.0
        } else {
            self.bins[bin_index]
        }
    }

    pub fn mean_magnitude(&self, bottom_frequency: f32, top_frequency: f32) -> f32 {
        assert!(bottom_frequency < top_frequency);

        let bottom_index = self.index(bottom_frequency);
        if bottom_index > self.bins.len() {
            return 0.0;
        }

        let requested_top_index = self.index(top_frequency);
        let actual_top_index = usize::min(requested_top_index, self.bins.len() - 1);

        let sum = (bottom_index..=actual_top_index).fold(0.0, |sum, i| sum + self.bins[i]);

        sum / (requested_top_index - bottom_index) as f32
    }

    fn index(&self, frequency: f32) -> usize {
        (frequency / self.bin_width) as usize
    }
}

fn fft_magnitude(signal: &[f32]) -> Vec<f32> {
    if signal.is_empty() {
        return Vec::new();
    }

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(signal.len());

    let mut complex_buffer: Vec<_> = signal.iter().map(|x| Complex32::new(*x, 0.0)).collect();
    fft.process(&mut complex_buffer);

    complex_buffer.iter().map(|x| x.norm()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noise;
    use std::f32::consts::PI;

    #[test]
    fn fft_magnitude_check() {
        let signal = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

        let magnitude = fft_magnitude(&signal);

        for i in 0..magnitude.len() {
            assert_relative_eq!(magnitude[i], 1.0);
        }
    }

    #[test]
    fn fft_magnitude_check_empty_signal() {
        let magnitude = fft_magnitude(&[]);

        assert_eq!(magnitude.len(), 0);
    }

    #[test]
    fn initialize_analyzer() {
        const SAMPLE_RATE: u32 = 44100;
        let _analysis = SpectralAnalysis::analyze(&[1.0, 0.0], SAMPLE_RATE);
    }

    fn write_sine(buffer: &mut [f32], frequency: f32, sample_rate: u32) {
        for (i, x) in buffer.iter_mut().enumerate() {
            *x = f32::sin((i as f32 / sample_rate as f32) * frequency * 2.0 * PI);
        }
    }

    #[test]
    fn analyze_magnitude_simple_sinusoid() {
        const SAMPLE_RATE: u32 = 11;
        let frequency = 3.0;
        let signal = {
            const LENGTH_IN_SECONDS: u32 = 3;
            let mut signal = [0.0; (SAMPLE_RATE * LENGTH_IN_SECONDS) as usize];
            write_sine(&mut signal, frequency, SAMPLE_RATE);
            signal
        };

        let analysis = SpectralAnalysis::analyze(&signal, SAMPLE_RATE);
        let magnitude = analysis.magnitude(frequency);

        assert!(magnitude > 10.0);
        assert!(analysis.magnitude(frequency - 1.0) < magnitude);
        assert!(analysis.magnitude(frequency + 1.0) < magnitude);
    }

    #[test]
    fn analyze_mean_magnitude_in_range() {
        const SAMPLE_RATE: u32 = 11;
        let ringing_frequency = 3.0;
        let silent_frequency = 1.0;
        let signal = {
            const LENGTH_IN_SECONDS: u32 = 3;
            let mut signal = [0.0; (SAMPLE_RATE * LENGTH_IN_SECONDS) as usize];
            write_sine(&mut signal, ringing_frequency, SAMPLE_RATE);
            signal
        };

        let analysis = SpectralAnalysis::analyze(&signal, SAMPLE_RATE);
        let mean_ringing_magnitude =
            analysis.mean_magnitude(ringing_frequency - 0.5, ringing_frequency + 0.5);
        let mean_silent_magnitude =
            analysis.mean_magnitude(silent_frequency - 0.5, silent_frequency + 0.5);

        dbg!(mean_ringing_magnitude);
        dbg!(mean_silent_magnitude);

        assert!(mean_ringing_magnitude > 5.0);
        assert!(mean_ringing_magnitude > mean_silent_magnitude);
    }

    #[test]
    fn analyze_mean_magnitude_with_bottom_over_the_range() {
        const SAMPLE_RATE: u32 = 3;
        let signal = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0];

        let analysis = SpectralAnalysis::analyze(&signal, SAMPLE_RATE);
        let mean_magnitude = analysis.mean_magnitude(20.0, 21.0);

        assert_relative_eq!(mean_magnitude, 0.0);
    }

    #[test]
    fn analyze_mean_magnitude_with_top_over_the_range() {
        const SAMPLE_RATE: u32 = 3;
        let signal = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0];

        let analysis = SpectralAnalysis::analyze(&signal, SAMPLE_RATE);
        let mean_magnitude = analysis.mean_magnitude(1.0, 21.0);

        assert!(mean_magnitude > 0.01);
    }

    #[test]
    fn analyze_mean_magnitude_of_white_noise() {
        const SAMPLE_RATE: u32 = 1200;

        const SIGNAL_LENGTH_IN_SECONDS: u32 = 1;
        let mut signal = [0.0; SAMPLE_RATE as usize * SIGNAL_LENGTH_IN_SECONDS as usize];
        noise::white_noise(&mut signal);

        let analysis = SpectralAnalysis::analyze(&signal, SAMPLE_RATE);
        let low_mean_magnitude = analysis.mean_magnitude(0.0, 200.0);
        let middle_mean_magnitude = analysis.mean_magnitude(200.0, 400.0);
        let high_mean_magnitude = analysis.mean_magnitude(400.0, 600.0);

        assert_relative_eq!(
            low_mean_magnitude,
            middle_mean_magnitude,
            max_relative = 1.0
        );
        assert_relative_eq!(
            middle_mean_magnitude,
            high_mean_magnitude,
            max_relative = 1.0
        );
    }
}
