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
            if let Ok(index) = grid.get_neighbor_index(position, offset)
                && let Some(p) = &grid.get_cell(index).particle
                && 0 < p.health
            {
                match p.kind {
                    particle::ParticleKind::Drain(..) => (),
                    _ => {
                        let cycle = grid.cycle();
                        let cell = grid.get_cell_mut(index);
                        if let Some(particle) = &mut cell.particle {
                            particle.health = particle.health.saturating_sub(40);
                            cell.cycle = cycle;
                        }
                        let cell = grid.get_cell_mut(grid.to_index(position));
                        cell.cycle = cycle;
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
    use pretty_assertions::assert_eq;

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
    fn test_update_grid_drain_gradually_lowers_the_neighbor_particles_health_to_zero() {
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

        for _ in 0..4 {
            for _ in 0..7 {
                g.update_grid();
            }
            g.update_grid();
        }

        assert_eq!(
            vec![
                Cell::new(Particle::from(Rock::new())),
                Cell::empty().with_cycle(8),
                Cell::new(Particle::from(Rock::new())),
                Cell::empty().with_cycle(15),
                Cell::new(Particle::from(Drain::new())).with_cycle(28),
                Cell::empty().with_cycle(22),
                Cell::new(Particle::from(Rock::new())),
                Cell::empty().with_cycle(29),
                Cell::new(Particle::from(Rock::new())),
            ],
            *g.get_cells()
        );
    }
}
