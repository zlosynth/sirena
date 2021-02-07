pub fn time_to_frames(time: f32, frame_rate: f32) -> usize {
    assert!(time >= 0.0, "the time must be greater or equal to zero");
    (time * frame_rate) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_time_to_number_of_frames() {
        let frame_rate = 48000.0;
        let time = 0.5;

        assert_eq!(time_to_frames(time, frame_rate), 24000);
    }
}
