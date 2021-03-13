use criterion::{black_box, criterion_group, criterion_main, Criterion};

use sirena::osc1::Osc1;
use sirena::osc2::{Osc2, WAVETABLES_LEN};
use sirena::signal;
use sirena::taper;
use sirena::tone;
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

    c.bench_function("osc2", |b| {
        const SAMPLE_RATE: u32 = 48000;
        let wavetable = Wavetable::new(saw(), SAMPLE_RATE);
        let mut osc2 = Osc2::new([&wavetable; WAVETABLES_LEN], SAMPLE_RATE);
        osc2.set_frequency(440.0).set_breadth(2.0).set_detune(2.0);
        let mut buffer_left = [0.0; 64];
        let mut buffer_right = [0.0; 64];
        b.iter(|| {
            osc2.populate(black_box(&mut [
                &mut buffer_left[..],
                &mut buffer_right[..],
            ]))
        });
    });

    c.bench_function("log_taper", |b| {
        b.iter(|| taper::log(black_box(0.5)));
    });

    c.bench_function("saw_wavetable", |b| {
        b.iter(|| Wavetable::new(black_box(saw()), black_box(44100)));
    });

    c.bench_function("tone", |b| {
        b.iter(|| tone::detune_frequency(black_box(440.0), black_box(2.0)));
    });

    c.bench_function("normalize", |b| {
        b.iter(|| signal::normalize(&mut [0.0, -1.0, 2.0]));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
