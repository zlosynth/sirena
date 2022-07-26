use criterion::{black_box, criterion_group, criterion_main, Criterion};

use sirena::signal;
use sirena::taper;
use sirena::tone;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("log_taper", |b| {
        b.iter(|| taper::log(black_box(0.5)));
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
