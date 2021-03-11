#[allow(unused_imports)]
use micromath::F32Ext;

use super::consts::VOICES_LEN;
use crate::xfade;

const BREADTHS: [[f32; VOICES_LEN]; 36] = [
    // start on the center voice
    [0.0, 0.0, 1.0, 0.0, 0.0],
    // extend around center
    [0.0, 1.0, 1.0, 1.0, 0.0],
    [1.0, 1.0, 1.0, 1.0, 1.0],
    // stick around edges
    [1.0, 1.0, 0.0, 1.0, 1.0],
    [1.0, 0.0, 0.0, 0.0, 1.0],
    // single voice
    [0.0, 0.0, 0.0, 0.0, 1.0],
    [0.0, 0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0, 0.0, 0.0],
    // two voices
    [0.0, 0.0, 0.0, 1.0, 1.0],
    [0.0, 0.0, 1.0, 0.0, 1.0],
    [0.0, 0.0, 1.0, 1.0, 0.0],
    [0.0, 1.0, 0.0, 0.0, 1.0],
    [0.0, 1.0, 0.0, 1.0, 0.0],
    [0.0, 1.0, 1.0, 0.0, 0.0],
    [1.0, 0.0, 0.0, 0.0, 1.0],
    [1.0, 0.0, 0.0, 1.0, 0.0],
    [1.0, 0.0, 1.0, 0.0, 0.0],
    [1.0, 1.0, 0.0, 0.0, 0.0],
    // three voices
    [0.0, 0.0, 1.0, 1.0, 1.0],
    [0.0, 1.0, 0.0, 1.0, 1.0],
    [0.0, 1.0, 1.0, 0.0, 1.0],
    [0.0, 1.0, 1.0, 1.0, 0.0],
    [0.0, 1.0, 1.0, 1.0, 1.0],
    [1.0, 0.0, 0.0, 1.0, 1.0],
    [1.0, 0.0, 1.0, 0.0, 1.0],
    [1.0, 0.0, 1.0, 1.0, 0.0],
    [1.0, 1.0, 0.0, 0.0, 1.0],
    [1.0, 1.0, 0.0, 1.0, 0.0],
    [1.0, 1.0, 1.0, 0.0, 0.0],
    // four voices
    [1.0, 0.0, 1.0, 1.0, 1.0],
    [1.0, 1.0, 0.0, 1.0, 1.0],
    [1.0, 1.0, 1.0, 0.0, 1.0],
    [1.0, 1.0, 1.0, 1.0, 0.0],
    // all voices
    [1.0, 1.0, 1.0, 1.0, 1.0],
];

pub fn distribute(breadth: f32) -> [f32; VOICES_LEN] {
    distribute_given(&BREADTHS, breadth)
}

fn distribute_given(breadths: &[[f32; VOICES_LEN]], breadth: f32) -> [f32; VOICES_LEN] {
    let breadths_a = {
        let index_a = (breadth as usize).min(breadths.len() - 1);
        breadths[index_a]
    };

    let breadths_b = {
        let index_b = (breadth as usize + 1).min(breadths.len() - 1);
        breadths[index_b]
    };

    let xfade = breadth.fract();

    [
        xfade::lin(breadths_a[0], breadths_b[0], xfade),
        xfade::lin(breadths_a[1], breadths_b[1], xfade),
        xfade::lin(breadths_a[2], breadths_b[2], xfade),
        xfade::lin(breadths_a[3], breadths_b[3], xfade),
        xfade::lin(breadths_a[4], breadths_b[4], xfade),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breadth_based_on_combinations() {
        const COMBINATIONS: [[f32; 5]; 3] = [
            [0.0, 0.0, 1.0, 0.0, 0.0],
            [0.0, 1.0, 1.0, 1.0, 0.0],
            [1.0, 0.0, 0.0, 0.0, 0.0],
        ];

        let breadths = distribute_given(&COMBINATIONS, 0.0);
        assert_breadths(breadths, 0.0, 0.0, 1.0, 0.0, 0.0);

        let breadths = distribute_given(&COMBINATIONS, 0.5);
        assert_breadths(breadths, 0.0, 0.5, 1.0, 0.5, 0.0);

        let breadths = distribute_given(&COMBINATIONS, 1.0);
        assert_breadths(breadths, 0.0, 1.0, 1.0, 1.0, 0.0);

        let breadths = distribute_given(&COMBINATIONS, 1.5);
        assert_breadths(breadths, 0.5, 0.5, 0.5, 0.5, 0.0);

        let breadths = distribute_given(&COMBINATIONS, 2.0);
        assert_breadths(breadths, 1.0, 0.0, 0.0, 0.0, 0.0);
    }

    fn assert_breadths(breadths: [f32; VOICES_LEN], b1: f32, b2: f32, b3: f32, b4: f32, b5: f32) {
        assert_relative_eq!(breadths[0], b1);
        assert_relative_eq!(breadths[1], b2);
        assert_relative_eq!(breadths[2], b3);
        assert_relative_eq!(breadths[3], b4);
        assert_relative_eq!(breadths[4], b5);
    }
}
