use core::f32::consts::PI;

pub struct Wavetable {}

impl Wavetable {
    pub fn new() -> Self {
        Wavetable {}
    }

    pub fn read(&self, phase: f32) -> f32 {
        f32::sin(phase * 2.0 * PI)
    }
}
