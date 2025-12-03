use super::particle;
use crate::component::grid::GridAccess;

#[derive(Clone, PartialEq, Debug)]
pub struct Acid {
    acidity: u8,
}

impl Default for Acid {
    fn default() -> Self {
        Self::new()
    }
}

impl Acid {
    pub fn new() -> Self {
        Self::with_acidity(40)
    }

    pub fn with_acidity(acidity: u8) -> Self {
        Self { acidity }
    }

    pub fn update<T: GridAccess>(&self, grid: &mut T, position: (usize, usize)) {
        for offset in [(0, 1)] {
            if let Ok(index) = grid.get_neighbor_index(position, offset)
                && let Some(p) = &grid.get_cell(index).particle
                && 0 < self.acidity
            {
                match p.kind {
                    particle::ParticleKind::Acid(..) => (),
                    _ => {
                        let cycle = grid.cycle();
                        let cell = grid.get_cell_mut(index);
                        if let Some(particle) = &mut cell.particle {
                            particle.health = particle.health.saturating_sub(self.acidity);
                            cell.cycle = cycle;
                        }
                        let cell = grid.get_cell_mut(grid.to_index(position));
                        cell.cycle = cycle;
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
        particles::{acid::Acid, particle::Particle, rock::Rock},
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn test_update_grid_acid_shouldnt_remove_other_acid_particles() {
        /*
         * aaa -> aaa
         * aaa    aaa
         * aaa    aaa
         */

        let mut g = Grid::new(3, 3);

        for y in 0..3 {
            for x in 0..3 {
                g.spawn_particle((x, y), Particle::from(Acid::new()));
            }
        }

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Acid::new())),
                Cell::new(Particle::from(Acid::new())),
                Cell::new(Particle::from(Acid::new())),
                Cell::new(Particle::from(Acid::new())),
                Cell::new(Particle::from(Acid::new())),
                Cell::new(Particle::from(Acid::new())),
                Cell::new(Particle::from(Acid::new())),
                Cell::new(Particle::from(Acid::new())),
                Cell::new(Particle::from(Acid::new())),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_acid_gradually_lowers_the_bottom_neighbor_health_to_zero() {
        /* a -> a
         * r    -
         */
        let mut g = Grid::new(1, 2);

        g.spawn_particle((0, 0), Particle::from(Acid::new()));
        g.spawn_particle((0, 1), Particle::from(Rock::new()));

        for _ in 0..7 {
            g.update_grid();
        }

        assert_eq!(
            vec![
                Cell::new(Particle::from(Acid::new())).with_cycle(7),
                Cell::new(Particle::from(Rock::new()).with_health(0)).with_cycle(7),
            ],
            *g.get_cells()
        );
    }
}
