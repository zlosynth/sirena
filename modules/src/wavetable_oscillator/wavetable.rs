use core::f32::consts::PI;

pub struct Wavetable {
    wavetable: [f32; 2048],
}

impl Wavetable {
    pub fn new(wavetable: [f32; 2048]) -> Self {
        Wavetable { wavetable }
    }

    pub fn read(&self, phase: f32) -> f32 {
        self.wavetable[(phase * 2048.0) as usize]
    }
}

pub fn sine() -> [f32; 2048] {
    let mut wavetable = [0.0; 2048];
    for (i, x) in wavetable.iter_mut().enumerate() {
        *x = f32::sin(i as f32 / 2048.0 * 2.0 * PI);
    }
    wavetable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_wavetable() {
        let _wavetable = Wavetable::new(sine());
    }

    #[test]
    fn read_value() {
        let wavetable = Wavetable::new(sine());

        let first = wavetable.read(0.0);
        let second = wavetable.read(0.1);
        assert!(second > first);
    }

    #[test]
    fn sine_samples() {
        let wavetable = sine();

        assert_relative_eq!(wavetable[0], 0.0);
        assert_relative_eq!(wavetable[wavetable.len() / 4], 1.0);
    }
}
