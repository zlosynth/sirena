pub type Samples = [f32; 32];

pub fn zeroed() -> Samples {
    [0.0; 32]
}

pub fn value(value: f32) -> Samples {
    [value; 32]
}
