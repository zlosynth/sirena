use super::consts::VOICES_LEN;
use crate::tone;

pub fn distribute(frequency: f32, detune: f32) -> [f32; VOICES_LEN] {
    [
        tone::detune_frequency(frequency, detune),
        tone::detune_frequency(frequency, -detune / 2.0),
        frequency,
        tone::detune_frequency(frequency, detune / 2.0),
        tone::detune_frequency(frequency, -detune),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_detune_distribution() {
        const G4: f32 = 391.995;
        const G_SHARP_4: f32 = 415.305;
        const A4: f32 = 440.0;
        const A_SHARP_4: f32 = 466.164;
        const B4: f32 = 493.883;

        let detune = 2.0;
        let detuned = distribute(A4, detune);

        assert_relative_eq!(detuned[0], B4, epsilon = 0.001);
        assert_relative_eq!(detuned[1], G_SHARP_4, epsilon = 0.001);
        assert_relative_eq!(detuned[2], A4, epsilon = 0.001);
        assert_relative_eq!(detuned[3], A_SHARP_4, epsilon = 0.001);
        assert_relative_eq!(detuned[4], G4, epsilon = 0.001);
    }
}
