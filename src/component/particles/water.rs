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
mod tests_liquid {
    use crate::component::{
        grid::{Cell, Grid, ParticleHorizontalDirection, Random, RowUpdateDirection},
        particles::{acid::Acid, particle::Particle, rock::Rock, salt::Salt, sand::Sand},
    };
    use pretty_assertions::assert_eq;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    fn liquid_particle() -> Vec<Particle> {
        vec![
            Particle::from(Water::with_capacity(0)),
            Particle::from(Acid::with_acidity(0)),
        ]
    }

    fn weighted_particle() -> Vec<Particle> {
        vec![Particle::from(Sand::new()), Particle::from(Salt::new())]
    }

    #[test]
    fn test_update_grid_liquid_particle_falls_down_to_last_row_stays_there() {
        /*
         * w -> - -> -
         * -    w    -
         * -    -    w
         */

        for liquid_particle in liquid_particle() {
            let mut g = Grid::new(1, 3);
            g.spawn_particle((0, 0), liquid_particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty().with_cycle(1),
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                    Cell::empty(),
                ],
                *g.get_cells()
            );

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty().with_cycle(1),
                    Cell::empty().with_cycle(2),
                    Cell::new(liquid_particle.clone()).with_cycle(2),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_update_grid_liquid_particle_moves_right_when_bottom_cell_and_left_are_full() {
        /*
         * --- -> ---
         * sw-    s-w
         */
        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let mut g = Grid::new(3, 2);
                g.spawn_particle((0, 1), particle.clone());
                g.spawn_particle((1, 1), liquid_particle.clone());

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::empty(),
                        Cell::empty(),
                        Cell::empty(),
                        Cell::new(particle.clone()),
                        Cell::empty().with_cycle(1),
                        Cell::new(liquid_particle.clone()).with_cycle(1),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_update_grid_liquid_particle_moves_left_when_bottom_cell_and_right_are_full() {
        /*
         * --- -> ---
         * -ws    w-s
         */
        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let mut g = Grid::new(3, 2);
                g.spawn_particle((1, 1), liquid_particle.clone());
                g.spawn_particle((2, 1), particle.clone());

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::empty(),
                        Cell::empty(),
                        Cell::empty(),
                        Cell::new(liquid_particle.clone()).with_cycle(1),
                        Cell::empty().with_cycle(1),
                        Cell::new(particle.clone()),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_update_grid_liquid_particle_moves_left_or_right_when_both_right_and_left_are_empty_forced_right()
     {
        /*
         * --- -> ---
         * -w-    --w
         */
        for liquid_particle in liquid_particle() {
            let mut g = Grid::new(3, 2)
                .with_rand_particle_direction(|_| ParticleHorizontalDirection::Right);

            g.spawn_particle((1, 1), liquid_particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty(),
                    Cell::empty(),
                    Cell::empty(),
                    Cell::empty(),
                    Cell::empty().with_cycle(1),
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_update_grid_liquid_particle_moves_left_or_right_when_both_right_and_left_are_empty_forced_left()
     {
        /*
         * --- -> ---
         * -w-    w--
         */
        for liquid_particle in liquid_particle() {
            let mut g =
                Grid::new(3, 2).with_rand_particle_direction(|_| ParticleHorizontalDirection::Left);

            g.spawn_particle((1, 1), liquid_particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty(),
                    Cell::empty(),
                    Cell::empty(),
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                    Cell::empty().with_cycle(1),
                    Cell::empty(),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_liquid_particle_can_slide_two_cells_to_right() {
        /*
         * --- -> ---
         * w--    --w
         */
        for liquid_particle in liquid_particle() {
            let mut g = Grid::new(3, 2)
                .with_rand_particle_direction(|_| ParticleHorizontalDirection::Right);

            g.spawn_particle((0, 1), liquid_particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty(),
                    Cell::empty(),
                    Cell::empty(),
                    Cell::empty().with_cycle(1),
                    Cell::empty(),
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_liquid_particle_can_slide_two_cells_to_left() {
        /*
         * --- -> ---
         * --w    w--
         */
        for liquid_particle in liquid_particle() {
            let mut g =
                Grid::new(3, 2).with_rand_particle_direction(|_| ParticleHorizontalDirection::Left);

            g.spawn_particle((2, 1), liquid_particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty(),
                    Cell::empty(),
                    Cell::empty(),
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                    Cell::empty(),
                    Cell::empty().with_cycle(1),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_update_grid_liquid_particle_falls_to_bottom_right_when_bottom_cell_and_bottom_left_are_full_and_bottom_right_is_empty()
     {
        /*
         * w- -> --
         * s-    sw
         */
        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let mut g = Grid::new(2, 2);

                g.spawn_particle((0, 0), liquid_particle.clone());
                g.spawn_particle((0, 1), particle.clone());

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::empty().with_cycle(1),
                        Cell::empty(),
                        Cell::new(particle.clone()),
                        Cell::new(liquid_particle.clone()).with_cycle(1),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_update_grid_liquid_particle_falls_bottom_left_when_bottom_cell_and_bottom_right_are_full_and_bottom_left_is_empty()
     {
        /*
         * -w -> --
         * -s    ws
         */
        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let mut g = Grid::new(2, 2);

                g.spawn_particle((1, 0), liquid_particle.clone());
                g.spawn_particle((1, 1), particle.clone());

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::empty(),
                        Cell::empty().with_cycle(1),
                        Cell::new(liquid_particle.clone()).with_cycle(1),
                        Cell::new(particle.clone()),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_update_grid_liquid_particle_falls_bottom_left_or_bottom_right_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_forced_left()
     {
        /*
         * -w- -> ---
         * -s-    ws-
         */
        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let mut g = Grid::new(3, 2)
                    .with_rand_particle_direction(|_| ParticleHorizontalDirection::Left);

                g.spawn_particle((1, 0), liquid_particle.clone());
                g.spawn_particle((1, 1), particle.clone());

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::empty(),
                        Cell::empty().with_cycle(1),
                        Cell::empty(),
                        Cell::new(liquid_particle.clone()).with_cycle(1),
                        Cell::new(particle.clone()),
                        Cell::empty(),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_update_grid_liquid_particle_falls_bottom_left_or_bottom_right_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_forced_right()
     {
        /*
         * -w- -> ---
         * -s-    -sw
         */
        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let mut g = Grid::new(3, 2)
                    .with_rand_particle_direction(|_| ParticleHorizontalDirection::Right);

                g.spawn_particle((1, 0), liquid_particle.clone());
                g.spawn_particle((1, 1), particle.clone());

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::empty(),
                        Cell::empty().with_cycle(1),
                        Cell::empty(),
                        Cell::empty(),
                        Cell::new(particle.clone()),
                        Cell::new(liquid_particle.clone()).with_cycle(1),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_updating_rows_in_forward_order_creates_a_left_bias_on_liquid_particles() {
        /*
         * -ww- => ww-- or w--w
         */
        for liquid_particle in liquid_particle() {
            let mut g = Grid::new(4, 1)
                .with_rand_particle_direction(|_| ParticleHorizontalDirection::Left)
                .with_rand_row_update_direction(|_| RowUpdateDirection::Forward);

            g.spawn_particle((1, 0), liquid_particle.clone());
            g.spawn_particle((2, 0), liquid_particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                    Cell::empty().with_cycle(1),
                    Cell::empty(),
                ],
                *g.get_cells()
            );

            let mut g = Grid::new(4, 1)
                .with_rand_particle_direction(|_| ParticleHorizontalDirection::Right)
                .with_rand_row_update_direction(|_| RowUpdateDirection::Forward);

            g.spawn_particle((1, 0), liquid_particle.clone());
            g.spawn_particle((2, 0), liquid_particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                    Cell::empty().with_cycle(1),
                    Cell::empty().with_cycle(1),
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_updating_rows_in_reverse_order_creates_a_right_bias_on_liquid_particles() {
        /*
         * -ww- => --ww or w--w
         */
        for liquid_particle in liquid_particle() {
            let mut g = Grid::new(4, 1)
                .with_rand_particle_direction(|_| ParticleHorizontalDirection::Right)
                .with_rand_row_update_direction(|_| RowUpdateDirection::Reverse);

            g.spawn_particle((1, 0), liquid_particle.clone());
            g.spawn_particle((2, 0), liquid_particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty(),
                    Cell::empty().with_cycle(1),
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                ],
                *g.get_cells()
            );

            let mut g = Grid::new(4, 1)
                .with_rand_particle_direction(|_| ParticleHorizontalDirection::Left)
                .with_rand_row_update_direction(|_| RowUpdateDirection::Reverse);

            g.spawn_particle((1, 0), liquid_particle.clone());
            g.spawn_particle((2, 0), liquid_particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                    Cell::empty().with_cycle(1),
                    Cell::empty().with_cycle(1),
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_weighted_particle_should_sink_to_bottom_in_liquid_particle() {
        /*
         * -s- -> -w-
         * sws    sss
         */

        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let particle = particle.with_velocity(0);

                let mut g = Grid::new(3, 2)
                    .with_rand_velocity(|_| 0)
                    .with_initial_particle_velocity(0);

                g.spawn_particle((1, 0), particle.clone());
                g.spawn_particle((0, 1), particle.clone());
                g.spawn_particle((1, 1), liquid_particle.clone().with_velocity(0));
                g.spawn_particle((2, 1), particle.clone());

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::empty(),
                        Cell::new(liquid_particle.clone().with_velocity(0)).with_cycle(1),
                        Cell::empty(),
                        Cell::new(particle.clone()),
                        Cell::new(particle.clone().with_velocity(1)).with_cycle(1),
                        Cell::new(particle.clone()),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_weighted_particle_should_sink_to_bottom_left_in_liquid_particle() {
        /*
         * -s- -> -w-
         * wss    sss
         */
        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let particle = particle.with_velocity(0);

                let mut g = Grid::new(3, 2)
                    .with_rand_velocity(|_| 0)
                    .with_initial_particle_velocity(0);

                g.spawn_particle((1, 0), particle.clone());
                g.spawn_particle((0, 1), liquid_particle.clone().with_velocity(0));
                g.spawn_particle((1, 1), particle.clone());
                g.spawn_particle((2, 1), particle.clone());

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::empty(),
                        Cell::new(liquid_particle.clone().with_velocity(0)).with_cycle(1),
                        Cell::empty(),
                        Cell::new(particle.clone().with_velocity(1)).with_cycle(1),
                        Cell::new(particle.clone()),
                        Cell::new(particle.clone()),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_weighted_particle_should_sink_to_bottom_right_in_liquid_particle() {
        /*
         * -s- -> -w-
         * ssw    sss
         */

        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let particle = particle.with_velocity(0);

                let mut g = Grid::new(3, 2)
                    .with_rand_velocity(|_| 0)
                    .with_initial_particle_velocity(0);

                g.spawn_particle((1, 0), particle.clone());
                g.spawn_particle((0, 1), particle.clone());
                g.spawn_particle((1, 1), particle.clone());
                g.spawn_particle((2, 1), liquid_particle.clone().with_velocity(0));

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::empty(),
                        Cell::new(liquid_particle.clone().with_velocity(0)).with_cycle(1),
                        Cell::empty(),
                        Cell::new(particle.clone()),
                        Cell::new(particle.clone()),
                        Cell::new(particle.clone().with_velocity(1)).with_cycle(1),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_weighted_particle_should_sink_in_liquid_particle_but_liquid_particle_should_not_climb_on_the_weighted_particle()
     {
        /*
         * s -> s -> w
         * s    w    s
         * w    s    s
         */
        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let particle = particle.with_velocity(0);

                let mut g = Grid::new(1, 3)
                    .with_rand_velocity(|_| 0)
                    .with_initial_particle_velocity(0);

                g.spawn_particle((0, 0), particle.clone());
                g.spawn_particle((0, 1), particle.clone());
                g.spawn_particle((0, 2), liquid_particle.clone().with_velocity(0));

                assert_eq!(
                    vec![
                        Cell::new(particle.clone()),
                        Cell::new(particle.clone()),
                        Cell::new(liquid_particle.clone().with_velocity(0)),
                    ],
                    *g.get_cells()
                );

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::new(particle.clone()),
                        Cell::new(liquid_particle.clone().with_velocity(0)).with_cycle(1),
                        Cell::new(particle.clone().with_velocity(1)).with_cycle(1),
                    ],
                    *g.get_cells()
                );

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::new(liquid_particle.clone().with_velocity(0)).with_cycle(2),
                        Cell::new(particle.clone().with_velocity(1)).with_cycle(2),
                        Cell::new(particle.clone().with_velocity(0)).with_cycle(1),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_weighted_particle_can_sink_to_left_in_liquid_particle_even_if_the_destination_cell_is_simulated()
     {
        /*
         * ws -> -s -> -w
         * -r    wr    sr
         */

        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let mut g =
                    Grid::new(2, 2).with_rand_row_update_direction(|_| RowUpdateDirection::Forward);

                g.spawn_particle((0, 0), liquid_particle.clone());
                g.spawn_particle((1, 0), particle.clone());
                g.spawn_particle((1, 1), Particle::from(Rock::new()));

                g.update_grid();
                assert_eq!(
                    vec![
                        Cell::empty().with_cycle(1),
                        Cell::new(liquid_particle.clone()).with_cycle(1),
                        Cell::new(particle.clone()).with_cycle(1),
                        Cell::new(Particle::from(Rock::new())),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_weighted_particle_can_sink_to_right_in_liquid_particle_even_if_the_destination_cell_is_simulated()
     {
        /*
         * sw -> s- -> w-
         * r-    rw    rs
         */

        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let mut g =
                    Grid::new(2, 2).with_rand_row_update_direction(|_| RowUpdateDirection::Reverse);

                g.spawn_particle((0, 0), particle.clone());
                g.spawn_particle((1, 0), liquid_particle.clone());
                g.spawn_particle((0, 1), Particle::from(Rock::new()));

                g.update_grid();
                assert_eq!(
                    vec![
                        Cell::new(liquid_particle.clone()).with_cycle(1),
                        Cell::empty().with_cycle(1),
                        Cell::new(Particle::from(Rock::new())),
                        Cell::new(particle.clone()).with_cycle(1),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_if_liquid_particle_did_not_fall_due_to_probability_should_not_flow_either() {
        /*
         * w- -> w-
         * --    --
         */
        for liquid_particle in liquid_particle() {
            static V: &[u8] = &[255]; /*255 won't swap*/
            static V_INDEX: AtomicUsize = AtomicUsize::new(0);
            V_INDEX.store(0, Ordering::SeqCst);
            fn velocity_probability(_: &mut Random) -> u8 {
                let idx = V_INDEX.fetch_add(1, Ordering::SeqCst);
                V[idx]
            }

            let mut g = Grid::new(2, 2).with_rand_velocity(velocity_probability);
            let particle = liquid_particle.clone().with_velocity(0);

            g.spawn_particle((0, 0), particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::new(particle.clone().with_velocity(1)),
                    Cell::empty(),
                    Cell::empty(),
                    Cell::empty(),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_if_liquid_particle_did_not_fall_left_due_to_probability_should_not_flow_either() {
        /*
         * -w -> -w
         * -r    -r
         */
        for liquid_particle in liquid_particle() {
            static V: &[u8] = &[255]; /*255 won't swap*/
            static V_INDEX: AtomicUsize = AtomicUsize::new(0);
            V_INDEX.store(0, Ordering::SeqCst);
            fn velocity_probability(_: &mut Random) -> u8 {
                let idx = V_INDEX.fetch_add(1, Ordering::SeqCst);
                V[idx]
            }

            let mut g = Grid::new(2, 2).with_rand_velocity(velocity_probability);
            let particle = liquid_particle.clone().with_velocity(0);

            g.spawn_particle((1, 0), particle.clone());
            g.spawn_particle((1, 1), Particle::from(Rock::new()));

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty(),
                    Cell::new(particle.clone().with_velocity(1)),
                    Cell::empty(),
                    Cell::new(Particle::from(Rock::new())),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_if_liquid_particle_did_not_fall_right_due_to_probability_should_not_flow_either() {
        /*
         * w- -> w-
         * r-    r-
         */
        for liquid_particle in liquid_particle() {
            static V: &[u8] = &[255]; /*255 won't swap*/
            static V_INDEX: AtomicUsize = AtomicUsize::new(0);
            V_INDEX.store(0, Ordering::SeqCst);
            fn velocity_probability(_: &mut Random) -> u8 {
                let idx = V_INDEX.fetch_add(1, Ordering::SeqCst);
                V[idx]
            }

            let mut g = Grid::new(2, 2).with_rand_velocity(velocity_probability);
            let particle = liquid_particle.clone().with_velocity(0);

            g.spawn_particle((0, 0), particle.clone());
            g.spawn_particle((0, 1), Particle::from(Rock::new()));

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::new(particle.clone().with_velocity(1)),
                    Cell::empty(),
                    Cell::new(Particle::from(Rock::new())),
                    Cell::empty(),
                ],
                *g.get_cells()
            );
        }
    }
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
