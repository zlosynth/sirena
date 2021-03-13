#[allow(unused_imports)]
use micromath::F32Ext;

use super::consts::VOICES_LEN;
use crate::xfade;

const PANS: [[f32; VOICES_LEN]; 6] = [
    [0.0, 0.0, 0.0, 0.0, 0.0],
    [-1.0, -0.5, 0.0, 0.5, 1.0],
    [0.5, -1.0, 0.0, 1.0, -0.5],
    [0.5, 0.0, -1.0, 1.0, -0.5],
    [1.0, -1.0, 1.0, -1.0, -1.0],
    [-1.0, 1.0, -1.0, 1.0, 1.0],
];

pub fn distribute(pan_combination: f32) -> [f32; VOICES_LEN] {
    distribute_given(&PANS, pan_combination)
}

fn distribute_given(pans: &[[f32; VOICES_LEN]], pan_combination: f32) -> [f32; VOICES_LEN] {
    let pans_a = {
        let index_a = (pan_combination as usize).min(pans.len() - 1);
        pans[index_a]
    };

    let pans_b = {
        let index_b = (pan_combination as usize + 1).min(pans.len() - 1);
        pans[index_b]
    };

    let xfade = pan_combination.fract();

    [
        xfade::lin(pans_a[0], pans_b[0], xfade),
        xfade::lin(pans_a[1], pans_b[1], xfade),
        xfade::lin(pans_a[2], pans_b[2], xfade),
        xfade::lin(pans_a[3], pans_b[3], xfade),
        xfade::lin(pans_a[4], pans_b[4], xfade),
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

        let pans = distribute_given(&COMBINATIONS, 0.0);
        assert_pans(pans, 0.0, 0.0, 1.0, 0.0, 0.0);

        let pans = distribute_given(&COMBINATIONS, 0.5);
        assert_pans(pans, 0.0, 0.5, 1.0, 0.5, 0.0);

        let pans = distribute_given(&COMBINATIONS, 1.0);
        assert_pans(pans, 0.0, 1.0, 1.0, 1.0, 0.0);

        let pans = distribute_given(&COMBINATIONS, 1.5);
        assert_pans(pans, 0.5, 0.5, 0.5, 0.5, 0.0);

        let pans = distribute_given(&COMBINATIONS, 2.0);
        assert_pans(pans, 1.0, 0.0, 0.0, 0.0, 0.0);
    }

    fn assert_pans(pans: [f32; VOICES_LEN], b1: f32, b2: f32, b3: f32, b4: f32, b5: f32) {
        assert_relative_eq!(pans[0], b1);
        assert_relative_eq!(pans[1], b2);
        assert_relative_eq!(pans[2], b3);
        assert_relative_eq!(pans[3], b4);
        assert_relative_eq!(pans[4], b5);
    }
}
