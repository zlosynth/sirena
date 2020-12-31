pub const BUFFER_SIZE: usize = 32;

pub type Buffer = [f32; 32];

pub fn buffer_zeroed() -> Buffer {
    [0.0; 32]
}

pub const SAMPLE_RATE: f32 = 44800.0;
