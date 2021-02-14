use rand::Rng;

pub fn white_noise(buffer: &mut [f32]) {
    let mut rng = rand::thread_rng();
    for i in buffer.iter_mut() {
        *i = rng.gen_range(-1.0..=1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_white_noise() {
        let mut signal = [-10.0; 10];

        white_noise(&mut signal);

        signal.iter().for_each(|x| {
            assert!(*x <= 1.0);
            assert!(*x >= -1.0);
        });
    }
}
