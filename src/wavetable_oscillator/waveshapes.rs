use core::f32::consts::PI;

use crate::signal;

use super::wavetable::{OVERSAMPLING, WAVETABLE_LENGTH};

pub fn sine() -> [f32; WAVETABLE_LENGTH * OVERSAMPLING] {
    let mut wavetable = [0.0; WAVETABLE_LENGTH * OVERSAMPLING];
    for (i, x) in wavetable.iter_mut().enumerate() {
        *x = sin(i as f32, WAVETABLE_LENGTH * OVERSAMPLING);
    }
    wavetable
}

pub fn saw() -> [f32; WAVETABLE_LENGTH * OVERSAMPLING] {
    let harmonics = WAVETABLE_LENGTH / 4 - 1;
    let mut wavetable = [0.0; WAVETABLE_LENGTH * OVERSAMPLING];

    for (i, x) in wavetable.iter_mut().enumerate() {
        *x = sin(i as f32, WAVETABLE_LENGTH * OVERSAMPLING);
        for j in 2..harmonics {
            if j % 2 == 0 {
                *x -= sin(i as f32 * j as f32, WAVETABLE_LENGTH * OVERSAMPLING) / j as f32;
            } else {
                *x += sin(i as f32 * j as f32, WAVETABLE_LENGTH * OVERSAMPLING) / j as f32;
            }
        }
    }

    signal::normalize(&mut wavetable);

    wavetable
}

pub fn digital_saw() -> [f32; WAVETABLE_LENGTH * OVERSAMPLING] {
    let mut wavetable = [0.0; WAVETABLE_LENGTH * OVERSAMPLING];

    for (i, x) in wavetable.iter_mut().enumerate() {
        let phase = i as f32 / (WAVETABLE_LENGTH * OVERSAMPLING) as f32;
        if phase < 0.5 {
            *x = phase * 2.0;
        } else {
            *x = phase * 2.0 - 2.0;
        }
    }

    wavetable
}

pub fn triangle() -> [f32; WAVETABLE_LENGTH * OVERSAMPLING] {
    let harmonics = WAVETABLE_LENGTH / 4 - 1;
    let mut wavetable = [0.0; WAVETABLE_LENGTH * OVERSAMPLING];

    for (i, x) in wavetable.iter_mut().enumerate() {
        *x = sin(i as f32, WAVETABLE_LENGTH * OVERSAMPLING);
        for j in 2..harmonics {
            if j % 4 == 3 {
                *x -=
                    sin(i as f32 * j as f32, WAVETABLE_LENGTH * OVERSAMPLING) / (j as f32).powi(2);
            } else if j % 4 == 1 {
                *x +=
                    sin(i as f32 * j as f32, WAVETABLE_LENGTH * OVERSAMPLING) / (j as f32).powi(2);
            }
        }
    }

    signal::normalize(&mut wavetable);

    wavetable
}

fn sin(phase: f32, wavetable_length: usize) -> f32 {
    f32::sin(phase / (wavetable_length as f32) * 2.0 * PI)
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
}
