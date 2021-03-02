use core::f32::consts::PI;

use crate::signal;

use super::consts::OVERSAMPLED_WAVETABLE_LENGTH;

pub fn sine() -> [f32; OVERSAMPLED_WAVETABLE_LENGTH] {
    let mut wavetable = [0.0; OVERSAMPLED_WAVETABLE_LENGTH];
    for (i, x) in wavetable.iter_mut().enumerate() {
        *x = sin(i as f32);
    }
    wavetable
}

pub fn saw() -> [f32; OVERSAMPLED_WAVETABLE_LENGTH] {
    let harmonics = OVERSAMPLED_WAVETABLE_LENGTH / 4 / 4 - 1;
    let mut wavetable = [0.0; OVERSAMPLED_WAVETABLE_LENGTH];

    for (i, x) in wavetable.iter_mut().enumerate() {
        *x = sin(i as f32);
        for j in 2..harmonics {
            if j % 2 == 0 {
                *x -= sin(i as f32 * j as f32) / j as f32;
            } else {
                *x += sin(i as f32 * j as f32) / j as f32;
            }
        }
    }

    signal::normalize(&mut wavetable);

    wavetable
}

pub fn digital_saw() -> [f32; OVERSAMPLED_WAVETABLE_LENGTH] {
    let mut wavetable = [0.0; OVERSAMPLED_WAVETABLE_LENGTH];

    for (i, x) in wavetable.iter_mut().enumerate() {
        let phase = i as f32 / (OVERSAMPLED_WAVETABLE_LENGTH) as f32;
        if phase < 0.5 {
            *x = phase * 2.0;
        } else {
            *x = phase * 2.0 - 2.0;
        }
    }

    wavetable
}

pub fn triangle() -> [f32; OVERSAMPLED_WAVETABLE_LENGTH] {
    let harmonics = OVERSAMPLED_WAVETABLE_LENGTH / 4 / 4 - 1;
    let mut wavetable = [0.0; OVERSAMPLED_WAVETABLE_LENGTH];

    for (i, x) in wavetable.iter_mut().enumerate() {
        *x = sin(i as f32);
        for j in 2..harmonics {
            if j % 4 == 3 {
                *x -= sin(i as f32 * j as f32) / (j as f32).powi(2);
            } else if j % 4 == 1 {
                *x += sin(i as f32 * j as f32) / (j as f32).powi(2);
            }
        }
    }

    signal::normalize(&mut wavetable);

    wavetable
}

pub fn pulse(width: f32) -> [f32; OVERSAMPLED_WAVETABLE_LENGTH] {
    debug_assert!((0.0..1.0).contains(&width));

    let width_in_samples = width * OVERSAMPLED_WAVETABLE_LENGTH as f32;

    let harmonics = OVERSAMPLED_WAVETABLE_LENGTH / 4 / 4 - 1;
    let mut wavetable = [0.0; OVERSAMPLED_WAVETABLE_LENGTH];

    for (i, x) in wavetable.iter_mut().enumerate() {
        // offset to make sure it starts with the positive pulse
        let phase = i as f32 + (0.5 - width) * OVERSAMPLED_WAVETABLE_LENGTH as f32;
        let shifted_phase = phase + width_in_samples;

        *x = sin(phase);
        *x -= sin(shifted_phase);

        for j in 2..harmonics {
            if j % 2 == 0 {
                *x -= sin(phase * j as f32) / j as f32;
                *x += sin(shifted_phase * j as f32) / j as f32;
            } else {
                *x += sin(phase * j as f32) / j as f32;
                *x -= sin(shifted_phase * j as f32) / j as f32;
            }
        }

        // center the waveshape
        *x *= 2.0 / 3.0;
        *x += (width - 0.5) * 2.0;
    }

    signal::normalize(&mut wavetable);

    wavetable
}

fn sin(phase: f32) -> f32 {
    f32::sin(phase / (OVERSAMPLED_WAVETABLE_LENGTH as f32) * 2.0 * PI)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_samples() {
        let wavetable = sine();

        assert_relative_eq!(wavetable[0], 0.0);
        assert_relative_eq!(wavetable[wavetable.len() / 4], 1.0);
    }

    #[test]
    fn saw_samples() {
        let wavetable = saw();

        assert_relative_eq!(wavetable[0], 0.0);

        let peak_phase = (wavetable.len() as f32 * 0.499) as usize;
        assert_relative_eq!(wavetable[peak_phase], 1.0, max_relative = 0.05);
    }

    #[test]
    fn digital_saw_samples() {
        let wavetable = digital_saw();

        assert_relative_eq!(wavetable[0], 0.0);

        let mid_peak_phase = (wavetable.len() as f32 * 0.25) as usize;
        assert_relative_eq!(wavetable[mid_peak_phase], 0.5, max_relative = 0.05);

        let peak_phase = (wavetable.len() as f32 * 0.499) as usize;
        assert_relative_eq!(wavetable[peak_phase], 1.0, max_relative = 0.05);

        let dip_phase = (wavetable.len() as f32 * 0.501) as usize;
        assert_relative_eq!(wavetable[dip_phase], -1.0, max_relative = 0.05);

        assert_abs_diff_eq!(wavetable[wavetable.len() - 1], 0.0, epsilon = 0.05);
    }

    #[test]
    fn triangle_samples() {
        let wavetable = triangle();

        assert_relative_eq!(wavetable[0], 0.0);

        let peak_phase = (wavetable.len() as f32 * 0.25) as usize;
        assert_relative_eq!(wavetable[peak_phase], 1.0, max_relative = 0.05);
    }

    #[test]
    fn pulse_samples_width_0_5() {
        assert_pulse_samples(0.5);
    }

    #[test]
    fn pulse_samples_width_0_8() {
        assert_pulse_samples(0.8);
    }

    #[test]
    fn pulse_samples_width_0_3() {
        assert_pulse_samples(0.3);
    }

    fn assert_pulse_samples(width: f32) {
        let wavetable = pulse(width);

        let peak_beginning = (wavetable.len() as f32 * 0.1) as usize;
        let peak_middle = (wavetable.len() as f32 * width * 0.5) as usize;
        let peak_end = (wavetable.len() as f32 * width * 0.9) as usize;
        let dip_beginning = (wavetable.len() as f32 * (1.0 - (1.0 - width) * 0.9)) as usize;
        let dip_middle = (wavetable.len() as f32 * (1.0 - (1.0 - width) * 0.5)) as usize;
        let dip_end = (wavetable.len() as f32 * (1.0 - (1.0 - width) * 0.1)) as usize;

        assert_relative_eq!(wavetable[0], 0.0, epsilon = 0.1);
        assert_relative_eq!(wavetable[peak_beginning], 0.9, epsilon = 0.1);
        assert_relative_eq!(wavetable[peak_middle], 0.9, epsilon = 0.1);
        assert_relative_eq!(wavetable[peak_end], 0.9, epsilon = 0.1);
        assert_relative_eq!(wavetable[dip_beginning], -0.9, epsilon = 0.1);
        assert_relative_eq!(wavetable[dip_middle], -0.9, epsilon = 0.1);
        assert_relative_eq!(wavetable[dip_end], -0.9, epsilon = 0.1);
    }
}
