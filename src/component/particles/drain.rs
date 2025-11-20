use crate::component::grid::GridAccess;

use super::particle;

#[derive(Clone, PartialEq, Debug)]
pub struct Drain;

impl Default for Drain {
    fn default() -> Self {
        Self::new()
    }
}

impl Drain {
    pub fn new() -> Self {
        Self
    }

    pub fn update<T: GridAccess>(&self, grid: &mut T, position: (usize, usize)) {
        for offset in [(0, -1), (-1, 0), (1, 0), (0, 1)] {
            if let Ok(i) = grid.get_neighbor_index(position, offset)
                && let Some(p) = &grid.get_cell(i).particle
            {
                match p.kind {
                    particle::ParticleKind::Drain(..) => (),
                    _ => {
                        let index = grid.to_index(position);
                        grid.dissolve_particles(index, i);
                        return;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::component::{
        grid::{Cell, Grid, GridAccess},
        particles::{drain::Drain, particle::Particle, rock::Rock},
    };

    #[test]
    fn test_tap_particles_are_weightless() {
        let particle = Particle::from(Drain::new());
        assert_eq!(0, particle.weight);
    }

    #[test]
    fn test_update_grid_drain_shouldnt_remove_other_drain_particles() {
        /*
         * dd -> dd
         * dd    dd
         */

        let mut g = Grid::new(2, 2);

        g.spawn_particle((0, 0), Particle::from(Drain::new()));
        g.spawn_particle((1, 0), Particle::from(Drain::new()));
        g.spawn_particle((0, 1), Particle::from(Drain::new()));
        g.spawn_particle((1, 1), Particle::from(Drain::new()));

        //for _ in 0..4 {
        //    g.update_grid();
        //}

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Drain::new())),
                Cell::new(Particle::from(Drain::new())),
                Cell::new(Particle::from(Drain::new())),
                Cell::new(Particle::from(Drain::new())),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_update_grid_drain_removes_particles_around_it() {
        /*
         * rrr -> r-r
         * rdr    -d-
         * rrr    r-r
         */
        let mut g = Grid::new(3, 3);

        g.spawn_particle((1, 1), Particle::from(Drain::new()));
        for y in 0..3 {
            for x in 0..3 {
                if (x, y) == (1, 1) {
                    continue;
                }
                g.spawn_particle((x, y), Particle::from(Rock::new()));
            }
        }

        assert_eq!(
            vec![
                Cell::new(Particle::from(Rock::new())),
                Cell::new(Particle::from(Rock::new())),
                Cell::new(Particle::from(Rock::new())),
                Cell::new(Particle::from(Rock::new())),
                Cell::new(Particle::from(Drain::new())),
                Cell::new(Particle::from(Rock::new())),
                Cell::new(Particle::from(Rock::new())),
                Cell::new(Particle::from(Rock::new())),
                Cell::new(Particle::from(Rock::new())),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Rock::new())),
                Cell::empty().with_cycle(1),
                Cell::new(Particle::from(Rock::new())),
                Cell::new(Particle::from(Rock::new())),
                Cell::new(Particle::from(Drain::new())).with_cycle(1),
                Cell::new(Particle::from(Rock::new())),
                Cell::new(Particle::from(Rock::new())),
                Cell::new(Particle::from(Rock::new())),
                Cell::new(Particle::from(Rock::new())),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Rock::new())),
                Cell::empty().with_cycle(1),
                Cell::new(Particle::from(Rock::new())),
                Cell::empty().with_cycle(2),
                Cell::new(Particle::from(Drain::new())).with_cycle(2),
                Cell::new(Particle::from(Rock::new())),
                Cell::new(Particle::from(Rock::new())),
                Cell::new(Particle::from(Rock::new())),
                Cell::new(Particle::from(Rock::new())),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Rock::new())),
                Cell::empty().with_cycle(1),
                Cell::new(Particle::from(Rock::new())),
                Cell::empty().with_cycle(2),
                Cell::new(Particle::from(Drain::new())).with_cycle(3),
                Cell::empty().with_cycle(3),
                Cell::new(Particle::from(Rock::new())),
                Cell::new(Particle::from(Rock::new())),
                Cell::new(Particle::from(Rock::new())),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Rock::new())),
                Cell::empty().with_cycle(1),
                Cell::new(Particle::from(Rock::new())),
                Cell::empty().with_cycle(2),
                Cell::new(Particle::from(Drain::new())).with_cycle(4),
                Cell::empty().with_cycle(3),
                Cell::new(Particle::from(Rock::new())),
                Cell::empty().with_cycle(4),
                Cell::new(Particle::from(Rock::new())),
            ],
            *g.get_cells()
        );
    }
}
