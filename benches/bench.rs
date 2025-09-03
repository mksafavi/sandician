use criterion::{criterion_group, criterion_main, Criterion};
use sandsim::component::{particle::Grid, particles::Particle};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("update grid", |b| {
        let y = 1920;
        let x = 1080;
        let mut g = Grid::new(x, y);
        (0..x)
            .zip(0..y / 2)
            .for_each(|(x, y)| g.spawn_particle(x, y, Particle::Sand));
        (0..x)
            .zip(y / 2..y)
            .for_each(|(x, y)| g.spawn_particle(x, y, Particle::Water));
        b.iter(|| {
            g.update_grid();
        });
    });

    c.bench_function("draw grid", |b| {
        let y = 1920;
        let x = 1080;
        let mut g = Grid::new(x, y);
        let mut image = Grid::create_output_frame(x, y);
        (0..x)
            .zip(0..y / 2)
            .for_each(|(x, y)| g.spawn_particle(x, y, Particle::Sand));
        (0..x)
            .zip(y / 2..y)
            .for_each(|(x, y)| g.spawn_particle(x, y, Particle::Water));
        b.iter(|| {
            g.draw_grid(&mut image);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

