use criterion::{Criterion, criterion_group, criterion_main};
use rand::{Rng, SeedableRng, rng, rngs::SmallRng};

fn criterion_benchmark(c: &mut Criterion) {
    const ITERATIONS: u32 = u32::MAX;
    let mut c = c.benchmark_group("rand");
    c.sample_size(50);

    c.bench_function("fastrand random", |b| {
        let mut r = fastrand::Rng::new();

        b.iter(|| {
            for _ in 0..ITERATIONS {
                let _ = r.i32(..);
            }
        });
    });

    c.bench_function("thread random", |b| {
        let mut r = rng();

        b.iter(|| {
            for _ in 0..ITERATIONS {
                let _ = r.random::<i32>();
            }
        });
    });

    c.bench_function("thread random_range", |b| {
        let mut r = rng();

        b.iter(|| {
            for _ in 0..ITERATIONS {
                let _ = r.random_range(i32::MIN..=i32::MAX);
            }
        });
    });

    c.bench_function("small random", |b| {
        let mut r = SmallRng::from_os_rng();

        b.iter(|| {
            for _ in 0..ITERATIONS {
                let _ = r.random::<i32>();
            }
        });
    });

    c.bench_function("small random_range", |b| {
        let mut r = SmallRng::from_os_rng();

        b.iter(|| {
            for _ in 0..ITERATIONS {
                let _ = r.random_range(i32::MIN..=i32::MAX);
            }
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
