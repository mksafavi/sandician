use super::particle::{Particle, ParticleKind};
use crate::component::grid::GridAccess;

#[derive(Clone, PartialEq, Debug)]
pub struct Water {
    pub solvant_capacity: u8,
}

impl Default for Water {
    fn default() -> Self {
        Self::new()
    }
}

impl Water {
    pub fn new() -> Self {
        Self::with_capacity(3)
    }

    pub fn with_capacity(capacity: u8) -> Self {
        Self {
            solvant_capacity: capacity,
        }
    }

    pub fn update<T: GridAccess>(&self, grid: &mut T, position: (usize, usize)) {
        dissolve_salt(grid, self.solvant_capacity, position);
    }
}

fn dissolve_salt<T: GridAccess>(grid: &mut T, capacity: u8, position: (usize, usize)) -> bool {
    for y in -1..=1 {
        for x in -1..=1 {
            if let Ok(i) = grid.get_neighbor_index(position, (x, y))
                && let Some(p) = &grid.get_cell(i).particle
                && let ParticleKind::Salt(..) = p.kind
                && 0 < capacity
                && 0 < p.health
            {
                let cycle = grid.cycle();
                let cell = grid.get_cell_mut(i);
                if let Some(particle) = &mut cell.particle {
                    particle.health = 0;
                    cell.cycle = cycle;
                }
                let cell = grid.get_cell_mut(grid.to_index(position));
                if let Some(particle) = &cell.particle {
                    cell.particle = Some(
                        Particle::from(Water::with_capacity(capacity - 1))
                            .with_seed(particle.seed)
                            .with_velocity(particle.velocity),
                    );
                    cell.cycle = cycle;
                }
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::component::{
        grid::{Cell, Grid, GridAccess, ParticleHorizontalDirection},
        particles::{
            particle::{Particle, ParticleKind},
            rock::Rock,
            salt::Salt,
            sand::Sand,
            water::Water,
        },
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn test_dissolving_particle_counts_as_being_simulated() {
        /*
         * s -> s
         * w    w
         * S    -
         */

        let mut g = Grid::new(1, 3);
        g.spawn_particle((0, 0), Particle::from(Sand::new()));
        g.spawn_particle((0, 1), Particle::from(Water::new()));
        g.spawn_particle((0, 2), Particle::from(Salt::new()));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Water::with_capacity(2))).with_cycle(1),
                Cell::new(Particle::from(Salt::new()).with_health(0)).with_cycle(1),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_update_grid_water_dissolve_neighboring_salts() {
        /*
         * for each neighbor:
         * Srr -> -rr
         * rwr    rwr
         * rrr    rrr
         */
        for y in 0..3 {
            for x in 0..3 {
                if (x, y) == (1, 1) {
                    continue;
                }
                let mut g = Grid::new(3, 3);
                g.spawn_particle((1, 1), Particle::from(Water::new()));
                g.spawn_particle((x, y), Particle::from(Salt::new()));

                for yr in 0..3 {
                    for xr in 0..3 {
                        if (xr, yr) == (1, 1) || (xr, yr) == (x, y) {
                            continue;
                        }
                        g.spawn_particle((xr, yr), Particle::from(Rock::new()));
                    }
                }

                g.update_grid();

                for yr in 0..3 {
                    for xr in 0..3 {
                        if (xr, yr) == (1, 1) {
                            assert_eq!(
                                Some(Particle::from(Water::with_capacity(2))),
                                g.get_cell(g.to_index((xr, yr))).clone().particle
                            );
                        } else if (xr, yr) == (x, y) {
                            assert_eq!(
                                Some(Particle::from(Salt::new()).with_health(0)),
                                g.get_cell(g.to_index((xr, yr))).clone().particle
                            );
                        } else {
                            assert_eq!(
                                Some(Particle::from(Rock::new())),
                                g.get_cell(g.to_index((xr, yr))).clone().particle
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_update_grid_water_can_only_dissolve_three_salt_particles() {
        let mut g = Grid::new(1, 9).with_rand_velocity(|_| 0);

        let water = Particle::from(Water::new()).with_velocity(0);
        g.spawn_particle((0, 8), water.clone());

        let salt = Particle::from(Salt::new()).with_velocity(0);

        for y in 0..8 {
            g.spawn_particle((0, y), salt.clone());
        }

        assert_eq!(
            vec![
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Water::with_capacity(3))),
            ],
            g.get_cells()
                .iter()
                .map(|c| c.particle.as_ref().map(|p| p.kind.clone()))
                .collect::<Vec<_>>()
        );

        g.update_grid();
        g.update_grid();

        assert_eq!(
            vec![
                None,
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Water::with_capacity(2))),
            ],
            g.get_cells()
                .iter()
                .map(|c| c.particle.as_ref().map(|p| p.kind.clone()))
                .collect::<Vec<_>>()
        );

        g.update_grid();
        g.update_grid();

        assert_eq!(
            vec![
                None,
                None,
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Water::with_capacity(1))),
            ],
            g.get_cells()
                .iter()
                .map(|c| c.particle.as_ref().map(|p| p.kind.clone()))
                .collect::<Vec<_>>()
        );

        g.update_grid();
        g.update_grid();

        assert_eq!(
            vec![
                None,
                None,
                None,
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Water::with_capacity(0))),
            ],
            g.get_cells()
                .iter()
                .map(|c| c.particle.as_ref().map(|p| p.kind.clone()))
                .collect::<Vec<_>>()
        );

        for _ in 0..6 {
            g.update_grid();
        }

        assert_eq!(
            vec![
                None,
                None,
                None,
                Some(ParticleKind::from(Water::with_capacity(0))),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
                Some(ParticleKind::from(Salt::new())),
            ],
            g.get_cells()
                .iter()
                .map(|c| c.particle.as_ref().map(|p| p.kind.clone()))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_salt_water_is_heavier_than_water_and_sinks() {
        /*
         * 0 -> 3
         * 1    2
         * 2    1
         * 3    0
         */
        let mut g = Grid::new(1, 4)
            .with_rand_velocity(|_| 0)
            .with_initial_particle_velocity(1);

        g.spawn_particle(
            (0, 0),
            Particle::from(Water::with_capacity(0)).with_velocity(0),
        );
        g.spawn_particle(
            (0, 1),
            Particle::from(Water::with_capacity(1)).with_velocity(0),
        );
        g.spawn_particle(
            (0, 2),
            Particle::from(Water::with_capacity(2)).with_velocity(0),
        );
        g.spawn_particle((0, 3), Particle::from(Water::new()).with_velocity(0));

        g.update_grid();
        g.update_grid();
        g.update_grid();
        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Water::new()).with_velocity(1)).with_cycle(3),
                Cell::new(Particle::from(Water::with_capacity(2)).with_velocity(1)).with_cycle(4),
                Cell::new(Particle::from(Water::with_capacity(1)).with_velocity(1)).with_cycle(4),
                Cell::new(Particle::from(Water::with_capacity(0)).with_velocity(1)).with_cycle(3),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_water_can_slide_left_into_salt_water() {
        /*
         * Ww -> wW
         */
        let mut g =
            Grid::new(2, 1).with_rand_particle_direction(|_| ParticleHorizontalDirection::Left);

        g.spawn_particle((0, 0), Particle::from(Water::with_capacity(1)));
        g.spawn_particle((1, 0), Particle::from(Water::new()));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Water::new())).with_cycle(1),
                Cell::new(Particle::from(Water::with_capacity(1))).with_cycle(1),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_water_can_slide_right_into_salt_water() {
        /*
         * wW -> Ww
         */
        // TODO: this should be generalized around liquids with different viscosities
        let mut g =
            Grid::new(2, 1).with_rand_particle_direction(|_| ParticleHorizontalDirection::Right);

        g.spawn_particle((0, 0), Particle::from(Water::new()));
        g.spawn_particle((1, 0), Particle::from(Water::with_capacity(1)));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Water::with_capacity(1))).with_cycle(1),
                Cell::new(Particle::from(Water::new())).with_cycle(1),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_update_grid_water_particle_keeps_its_seed_after_dissolving_salts() {
        let mut g = Grid::new(2, 1);

        g.spawn_particle((0, 0), Particle::from(Water::new()).with_seed(111));
        g.spawn_particle((1, 0), Particle::from(Salt::new()));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Water::with_capacity(2)).with_seed(111)).with_cycle(1),
                Cell::new(Particle::from(Salt::new()).with_health(0)).with_cycle(1)
            ],
            *g.get_cells()
        );
    }
}
