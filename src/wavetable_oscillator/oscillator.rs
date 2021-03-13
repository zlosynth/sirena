pub trait Oscillator {
    fn set_frequency(&mut self, frequency: f32) -> &mut Self;
    fn frequency(&self) -> f32;
    fn set_amplitude(&mut self, amplitude: f32) -> &mut Self;
    fn amplitude(&self) -> f32;
    fn reset_phase(&mut self) -> &mut Self;

    fn add(&mut self, buffer: &mut [f32]);
    fn populate(&mut self, buffer: &mut [f32]);
    fn dry(&mut self, buffer: &mut [f32]);
}

pub trait StereoOscillator: Oscillator {
    fn set_pan(&mut self, pan: f32) -> &mut Self;
    fn pan(&self) -> f32;

    fn add_stereo(&mut self, buffer: &mut [&mut [f32]]);
    fn populate_stereo(&mut self, buffer: &mut [&mut [f32]]);
    fn dry_stereo(&mut self, buffer: &mut [&mut [f32]]);
}
