use super::oscillator::Oscillator;
use crate::spectral_analysis::SpectralAnalysis;

pub const SAMPLE_RATE: u32 = 44100;

pub fn get_first_sample(oscillator: &mut impl Oscillator) {
    oscillator.set_frequency(100.0);

    let mut buffer = [0.0];
    oscillator.populate(&mut buffer);

    assert_abs_diff_eq!(buffer[0], 0.0, epsilon = 0.01);
}

pub fn get_multiple_samples(oscillator: &mut impl Oscillator) {
    oscillator.set_frequency(1.0);

    let mut buffer = [0.0; 2];
    oscillator.populate(&mut buffer);

    assert!(buffer[1] > buffer[0]);
}

pub fn set_frequency(oscillator_a: &mut impl Oscillator, oscillator_b: &mut impl Oscillator) {
    let two_thousand_ticks_frequency_1 = {
        oscillator_a.set_frequency(1.0);
        let mut buffer = [0.0; 2000];
        oscillator_a.populate(&mut buffer);
        buffer[1999]
    };

    let one_thousand_ticks_frequency_2 = {
        oscillator_b.set_frequency(2.0);
        let mut buffer = [0.0; 1000];
        oscillator_b.populate(&mut buffer);
        buffer[999]
    };

    assert_relative_eq!(
        two_thousand_ticks_frequency_1,
        one_thousand_ticks_frequency_2,
        max_relative = 0.001,
    );
}

pub fn get_frequency(oscillator: &mut impl Oscillator) {
    oscillator.set_frequency(110.0);

    assert_eq!(oscillator.frequency(), 110.0);
}

pub fn set_sample_rate(
    oscillator_sample_rate_1000: &mut impl Oscillator,
    oscillator_sample_rate_1100: &mut impl Oscillator,
) {
    let two_ticks_sample_rate_1000 = {
        oscillator_sample_rate_1000.set_frequency(4.0);
        let mut buffer = [0.0; 2];
        oscillator_sample_rate_1000.populate(&mut buffer);
        buffer[1]
    };

    let two_ticks_sample_rate_1100 = {
        oscillator_sample_rate_1100.set_frequency(4.0);
        let mut buffer = [0.0; 2];
        oscillator_sample_rate_1100.populate(&mut buffer);
        buffer[1]
    };

    assert!(two_ticks_sample_rate_1000 > two_ticks_sample_rate_1100);
}

pub fn set_amplitude(oscillator: &mut impl Oscillator) {
    oscillator.set_frequency(1.0);

    let mut buffer = [0.0; SAMPLE_RATE as usize];

    oscillator.set_amplitude(2.0);
    oscillator.populate(&mut buffer);
    let max = buffer.iter().fold(0.0, |a, b| f32::max(a, b.abs()));
    assert_relative_eq!(max, 2.0, max_relative = 0.001);

    oscillator.set_amplitude(3.0);
    oscillator.populate(&mut buffer);
    let max = buffer.iter().fold(0.0, |a, b| f32::max(a, b.abs()));
    assert_relative_eq!(max, 3.0, max_relative = 0.001);
}

pub fn get_amplitude(oscillator: &mut impl Oscillator) {
    oscillator.set_frequency(110.0);

    assert_eq!(oscillator.frequency(), 110.0);
}

pub fn check_all_fifths_for_aliasing(oscillator: &mut impl Oscillator) {
    let notes: Vec<_> = (1..)
        .step_by(5)
        .map(|i| 27.5 * f32::powf(2.0, i as f32 / 12.0))
        .take_while(|x| *x < 22000.0)
        .collect();

    for note in notes.into_iter() {
        check_note_for_aliasing(oscillator, note);
    }
}

fn check_note_for_aliasing(oscillator: &mut impl Oscillator, frequency: f32) {
    oscillator.set_frequency(frequency);

    let mut signal = [0.0; SAMPLE_RATE as usize];
    oscillator.populate(&mut signal);

    let mut analysis = SpectralAnalysis::analyze(&signal, SAMPLE_RATE);
    analysis.trash_range(0.0, 1.0);

    let lowest_peak = analysis.lowest_peak(0.04);
    assert_abs_diff_eq!(lowest_peak, frequency, epsilon = 1.0);
}
