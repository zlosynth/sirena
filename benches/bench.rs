use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("placeholder", |b| {
        b.iter(|| {});
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
