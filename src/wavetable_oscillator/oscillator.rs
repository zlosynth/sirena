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
