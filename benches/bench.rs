use criterion::{Criterion, criterion_group, criterion_main};
use sandsim::component::{
    grid::Grid,
    particles::{
        acid::Acid, drain::Drain, particle::Particle, rock::Rock, salt::Salt, sand::Sand, tap::Tap,
        water::Water,
    },
};

fn fill_grid_mixed(g: &mut Grid, (x, y): (usize, usize)) {
    for y in 0..y / 3 {
        for x in 0..x {
            g.spawn_particle((x, y), Particle::from(Sand::new()));
        }
    }
    for y in y / 3..y {
        for x in 0..x {
            g.spawn_particle((x, y), Particle::from(Water::new()));
        }
    }
    for y in 2 * y / 3..y {
        for x in 0..x {
            g.spawn_particle((x, y), Particle::from(Salt::new()));
        }
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let y: usize = 360;
    let x: usize = 250;

    let mut c = c.benchmark_group("bench");
    c.sample_size(50);

    c.bench_function("update grid sand", |b| {
        let mut g = Grid::new(x, y);
        for y in 0..y {
            for x in 0..x {
                g.spawn_particle((x, y), Particle::from(Sand::new()));
            }
        }
        b.iter(|| {
            g.update_grid();
        });
    });

    c.bench_function("update grid water", |b| {
        let mut g = Grid::new(x, y);
        for y in 0..y {
            for x in 0..x {
                g.spawn_particle((x, y), Particle::from(Water::new()));
            }
        }
        b.iter(|| {
            g.update_grid();
        });
    });

    c.bench_function("update grid salt", |b| {
        let mut g = Grid::new(x, y);
        for y in 0..y {
            for x in 0..x {
                g.spawn_particle((x, y), Particle::from(Salt::new()));
            }
        }
        b.iter(|| {
            g.update_grid();
        });
    });

    c.bench_function("update grid salt and water", |b| {
        let mut g = Grid::new(x, y);
        for y in 0..y / 2 {
            for x in 0..x {
                g.spawn_particle((x, y), Particle::from(Salt::new()));
            }
        }
        for y in y / 2..y {
            for x in 0..x {
                g.spawn_particle((x, y), Particle::from(Water::new()));
            }
        }
        b.iter(|| {
            g.update_grid();
        });
    });
    c.bench_function("update grid rock", |b| {
        let mut g = Grid::new(x, y);
        for y in 0..y {
            for x in 0..x {
                g.spawn_particle((x, y), Particle::from(Rock::new()));
            }
        }
        b.iter(|| {
            g.update_grid();
        });
    });

    c.bench_function("update grid drain", |b| {
        let mut g = Grid::new(x, y);
        for y in 0..y {
            for x in 0..x {
                g.spawn_particle((x, y), Particle::from(Drain::new()));
            }
        }
        b.iter(|| {
            g.update_grid();
        });
    });

    c.bench_function("update grid tap", |b| {
        let mut g = Grid::new(x, y);
        for y in 0..y {
            for x in 0..x {
                g.spawn_particle((x, y), Particle::from(Tap::new()));
            }
        }
        b.iter(|| {
            g.update_grid();
        });
    });

    c.bench_function("update grid acid and sand", |b| {
        let mut g = Grid::new(x, y);
        for y in 0..y / 2 {
            for x in 0..x {
                g.spawn_particle((x, y), Particle::from(Acid::new()));
            }
        }
        for y in y / 2..y {
            for x in 0..x {
                g.spawn_particle((x, y), Particle::from(Sand::new()));
            }
        }
        b.iter(|| {
            g.update_grid();
        });
    });

    c.bench_function("update grid", |b| {
        let mut g = Grid::new(x, y);
        fill_grid_mixed(&mut g, (x, y));
        b.iter(|| {
            g.update_grid();
        });
    });

    c.bench_function("draw half grid", |b| {
        let mut g = Grid::new(x, y);
        let mut image = Grid::create_output_frame(x, y);
        for y in 0..y {
            for x in 0..x / 2 {
                g.spawn_particle((x, y), Particle::from(Sand::new()));
            }
        }
        b.iter(|| {
            g.update_grid();
            g.draw_grid(&mut image);
        });
    });

    c.bench_function("draw grid", |b| {
        let mut g = Grid::new(x, y);
        let mut image = Grid::create_output_frame(x, y);
        fill_grid_mixed(&mut g, (x, y));
        b.iter(|| {
            g.update_grid();
            g.draw_grid(&mut image);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
