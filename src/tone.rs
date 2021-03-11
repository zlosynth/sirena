#[allow(unused_imports)]
use micromath::F32Ext;

pub fn detune_frequency(frequency: f32, amount: f32) -> f32 {
    frequency * f32::powf(2.0, amount / 12.0)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
