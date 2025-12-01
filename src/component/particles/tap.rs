use crate::component::grid::GridAccess;

use super::particle::{Particle, ParticleKind};

#[derive(Clone, PartialEq, Debug)]
pub struct Tap {
    pub particle_kind: Option<Box<ParticleKind>>,
}

impl Default for Tap {
    fn default() -> Self {
        Self::new()
    }
}

impl Tap {
    pub fn new() -> Self {
        Self {
            particle_kind: None,
        }
    }

    pub fn with_particle(particle: &Particle) -> Self {
        Self {
            particle_kind: Some(Box::new(particle.kind.clone())),
        }
    }

    pub fn update<T: GridAccess>(&self, grid: &mut T, position: (usize, usize)) {
        let mut particle = self.clone();
        if particle.particle_kind.is_none() {
            let mut particle_to_clone = None;
            for y in -1..=1 {
                for x in -1..=1 {
                    if let Ok(i) = grid.get_neighbor_index(position, (x, y))
                        && let Some(p) = &grid.get_cell(i).particle
                        && p.cloneable
                    {
                        particle_to_clone = Some(p.clone());
                    };
                }
            }

            if let Some(p) = particle_to_clone {
                let cell = grid.get_cell_mut(grid.to_index(position));
                particle.particle_kind = Some(Box::new(p.kind.to_default()));
                cell.particle = Some(Particle::from(particle.clone()));
            }
        }

        if let Some(particle_kind) = particle.particle_kind {
            for y in -1..=1 {
                for x in -1..=1 {
                    if let Ok(i) = grid.get_neighbor_index(position, (x, y))
                        && grid.get_cell_mut(i).particle.is_none()
                    {
                        let cycle = grid.cycle();
                        let particle = Particle::from(*particle_kind.clone())
                            .with_velocity(grid.get_particle_initial_velocity())
                            .with_seed(grid.particle_seed());
                        let cell = grid.get_cell_mut(i);
                        cell.particle = Some(particle);
                        cell.cycle = cycle;
                        grid.get_cell_mut(grid.to_index(position)).cycle = cycle;
                    };
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::component::{
        grid::{Cell, Grid, GridAccess, RowUpdateDirection},
        particles::{
            drain::Drain, particle::Particle, rock::Rock, salt::Salt, sand::Sand, tap::Tap,
            water::Water,
        },
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn test_update_grid_tap_should_not_emit_tap_particles() {
        /*
         * --- -> ---
         * -tt    -tt
         * ---    ---
         */
        let mut g = Grid::new(3, 3);

        g.spawn_particle((1, 1), Particle::from(Tap::new()));
        g.spawn_particle((2, 1), Particle::from(Tap::new()));

        assert_eq!(
            vec![
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::new(Particle::from(Tap::new())),
                Cell::new(Particle::from(Tap::new())),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::new(Particle::from(Tap::new())),
                Cell::new(Particle::from(Tap::new())),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_update_grid_tap_emits_particle_on_every_neighbor_if_touched_by_a_particle() {
        /*
         * --- -> rrr
         * -tr    rtr
         * ---    rrr
         */

        for y in 0..3 {
            for x in 0..3 {
                if (x, y) == (1, 1) {
                    continue;
                }
                let mut g = Grid::new(3, 3).with_rand_seed_with_cycle(|_| 127);

                g.spawn_particle((1, 1), Particle::from(Tap::new()));
                g.spawn_particle((x, y), Particle::from(Rock::new()));

                g.update_grid();

                for y in 0..3 {
                    for x in 0..3 {
                        if (x, y) == (1, 1) {
                            continue;
                        }
                        assert_eq!(
                            Some(Particle::from(Rock::new())),
                            g.get_cell(g.to_index((x, y))).particle
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_update_grid_tap_emits_any_particle_type_it_touches() {
        /*
         * -- -> pp
         * tp    tp
         */
        for particle in [
            Particle::from(Sand::new()),
            Particle::from(Salt::new()),
            Particle::from(Water::new()),
            Particle::from(Rock::new()),
        ] {
            let mut g = Grid::new(2, 2).with_rand_seed_with_cycle(|_| 127);

            g.spawn_particle((0, 1), Particle::from(Tap::new()));
            g.spawn_particle((1, 1), particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::new(particle.clone()).with_cycle(1),
                    Cell::new(particle.clone()).with_cycle(1),
                    Cell::new(Particle::from(Tap::with_particle(&particle))).with_cycle(1),
                    Cell::new(particle),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_update_grid_tap_should_not_emit_drain_particles() {
        /*
         * -- -> -- -> --
         * td    td    -d
         */
        let mut g = Grid::new(2, 2).with_rand_row_update_direction(|_| RowUpdateDirection::Forward);

        g.spawn_particle((0, 1), Particle::from(Tap::new()));
        g.spawn_particle((1, 1), Particle::from(Drain::new()));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty(),
                Cell::empty(),
                Cell::empty().with_cycle(1),
                Cell::new(Particle::from(Drain::new())).with_cycle(1),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_update_grid_tap_selects_and_remembers_neighbor_cell_particle() {
        let mut g = Grid::new(1, 2);

        g.spawn_particle((0, 0), Particle::from(Tap::new()));
        g.spawn_particle((0, 1), Particle::from(Rock::new()));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Tap::with_particle(&Particle::from(
                    Rock::new()
                )))),
                Cell::new(Particle::from(Rock::new())),
            ],
            *g.get_cells()
        );

        g.despawn_particle((0, 1));
        g.spawn_particle((0, 1), Particle::from(Sand::new()));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Tap::with_particle(&Particle::from(
                    Rock::new()
                )))),
                Cell::new(Particle::from(Sand::new())).with_cycle(1)
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_update_grid_tap_clones_a_new_particle_with_the_selected_kind_instead_of_the_exact_particle()
     {
        let mut g = Grid::new(1, 2).with_rand_seed_with_cycle(|_| 127);

        g.spawn_particle((0, 0), Particle::from(Tap::new()));
        g.spawn_particle((0, 1), Particle::from(Water::with_capacity(0)));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Tap::with_particle(&Particle::from(
                    Water::new()
                )))),
                Cell::new(Particle::from(Water::with_capacity(0))),
            ],
            *g.get_cells()
        );

        g.despawn_particle((0, 1));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Tap::with_particle(&Particle::from(
                    Water::new()
                ))))
                .with_cycle(2),
                Cell::new(Particle::from(Water::new())).with_cycle(2)
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_tap_clones_a_new_particle_with_the_grid_initial_particle_velocity() {
        let mut g = Grid::new(1, 2)
            .with_rand_seed_with_cycle(|_| 127)
            .with_initial_particle_velocity(111);

        g.spawn_particle((0, 0), Particle::from(Tap::new()));
        g.spawn_particle(
            (0, 1),
            Particle::from(Water::with_capacity(0)).with_velocity(255),
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Tap::with_particle(&Particle::from(
                    Water::new()
                )))),
                Cell::new(Particle::from(Water::with_capacity(0)).with_velocity(254)),
            ],
            *g.get_cells()
        );

        g.despawn_particle((0, 1));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Tap::with_particle(&Particle::from(
                    Water::new()
                ))))
                .with_cycle(2),
                Cell::new(Particle::from(Water::new()).with_velocity(111)).with_cycle(2)
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_tap_clones_a_new_particle_with_a_random_seed() {
        let mut g = Grid::new(1, 2).with_rand_seed_with_cycle(|_| 33);

        g.spawn_particle((0, 0), Particle::from(Tap::new()));
        g.spawn_particle((0, 1), Particle::from(Water::with_capacity(0)));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Tap::with_particle(&Particle::from(
                    Water::new()
                )))),
                Cell::new(Particle::from(Water::with_capacity(0))),
            ],
            *g.get_cells()
        );

        g.despawn_particle((0, 1));

        g.update_grid();

        assert_eq!(33, g.get_cell(1).particle.as_ref().map(|p| p.seed).unwrap());
    }
}
