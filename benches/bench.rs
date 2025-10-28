use criterion::{Criterion, criterion_group, criterion_main};
use sandsim::component::{grid::Grid, particles::particle::Particle};

fn fill_grid_mixed(g: &mut Grid, (x, y): (usize, usize)) {
    for y in 0..y / 3 {
        for x in 0..x {
            g.spawn_particle((x, y), Particle::new_sand());
        }
    }
    for y in y / 3..y {
        for x in 0..x {
            g.spawn_particle((x, y), Particle::new_water());
        }
    }
    for y in 2 * y / 3..y {
        for x in 0..x {
            g.spawn_particle((x, y), Particle::new_salt());
        }
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let y: usize = 1920;
    let x: usize = 1080;
    c.bench_function("update grid sand", |b| {
        let mut g = Grid::new(x, y);
        for y in 0..y {
            for x in 0..x {
                g.spawn_particle((x, y), Particle::new_sand());
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
                g.spawn_particle((x, y), Particle::new_water());
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
                g.spawn_particle((x, y), Particle::new_salt());
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
                g.spawn_particle((x, y), Particle::new_sand());
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
