use criterion::{black_box, criterion_group, criterion_main, Criterion};

use sirena::osc1::Osc1;
use sirena::wavetable_oscillator::Wavetable;
use sirena::wavetable_oscillator::{saw, sine, triangle};

pub fn osc1_play(osc1: &mut Osc1, buffer: &mut [f32]) {
    osc1.populate(buffer);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("osc1", |b| {
        const SAMPLE_RATE: u32 = 48000;
        let wavetable_a = Wavetable::new(sine(), SAMPLE_RATE);
        let wavetable_b = Wavetable::new(triangle(), SAMPLE_RATE);
        let wavetable_c = Wavetable::new(saw(), SAMPLE_RATE);
        let mut osc1 = Osc1::new(&wavetable_a, &wavetable_b, &wavetable_c, SAMPLE_RATE);
        osc1.set_frequency(440.0)
            .set_enabled_voices(7)
            .set_detune(2.0);
        let mut buffer = [0.0; 64];
        b.iter(|| osc1_play(black_box(&mut osc1), black_box(&mut buffer)));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
