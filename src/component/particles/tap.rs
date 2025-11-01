use crate::component::{grid::GridAccess, particles::sand::Sand};

use super::{
    drain::Drain,
    particle::{self, Particle},
    salt::Salt,
    water::Water,
};

#[derive(Clone, PartialEq, Debug)]

pub struct Tap {
    particle_id: u8,
}

impl Default for Tap {
    fn default() -> Self {
        Self::new()
    }
}

impl Tap {
    pub fn new() -> Self {
        Self { particle_id: 5 }
    }

    fn new_particle(&self) -> Option<Particle> {
        match self.particle_id {
            0 => Some(particle::Particle::from(Sand::new())),
            1 => Some(particle::Particle::from(Water::new())),
            2 => Some(particle::Particle::from(Salt::new())),
            3 => Some(Particle::Rock),
            4 => Some(particle::Particle::from(Drain::new())),
            _ => None,
        }
    }
}

impl particle::Updatable for Tap {
    fn update<T: GridAccess>(&mut self, grid: &mut T, position: (usize, usize)) {
        for offset in [(0, -1), (-1, 0), (1, 0), (0, 1)] {
            if self.particle_id == 5 {
                if let Ok(i) = grid.get_neighbor_index(position, offset) {
                    if let Some(p) = &grid.get_cell(i).particle {
                        self.particle_id = match p {
                            Particle::Sand(_) => 0,
                            Particle::Water(_) => 1,
                            Particle::Salt(_) => 2,
                            Particle::Rock => 3,
                            Particle::Drain(_) => 4,
                            Particle::Tap(_) => 5,
                        };
                    }
                };
            }
        }

        if let Some(particle) = self.new_particle() {
            for offset in [(0, -1), (-1, 0), (1, 0), (0, 1)] {
                if let Ok(i) = grid.get_neighbor_index(position, offset) {
                    if grid.get_cell_mut(i).particle.is_none() {
                        let cycle = grid.cycle().wrapping_add(1);
                        let cell = grid.get_cell_mut(i);
                        cell.particle = Some(particle.clone());
                        cell.cycle = cycle;
                        grid.get_cell_mut(grid.to_index(position)).cycle = cycle;
                    }
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::component::{
        grid::{Cell, Grid, GridAccess, RowUpdateDirection},
        particles::{
            drain::Drain, particle::Particle, salt::Salt, sand::Sand, tap::Tap, water::Water,
        },
    };

    #[test]
    fn test_update_grid_tap_stays_in_place() {
        /*
         * t- -> t-
         * --    --
         */
        let mut g = Grid::new(2, 2);

        g.spawn_particle((0, 0), Particle::from(Tap::new()));

        assert_eq!(
            vec![
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
                Cell::new(Particle::from(Tap::new())),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_update_grid_tap_shouldnt_emits_tap_particles() {
        /*
         * --r -> --r
         * -tt    -tt
         * ---    --r
         */
        let mut g = Grid::new(3, 3);

        g.spawn_particle((2, 0), Particle::Rock);
        g.spawn_particle((1, 1), Particle::from(Tap::new()));
        g.spawn_particle((2, 1), Particle::from(Tap::new()));
        assert_eq!(
            vec![
                Cell::empty(),
                Cell::empty(),
                Cell::new(Particle::Rock),
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
                Cell::new(Particle::Rock),
                Cell::empty(),
                Cell::new(Particle::from(Tap::new())),
                Cell::new(Particle::from(Tap::new())).with_cycle(1),
                Cell::empty(),
                Cell::empty(),
                Cell::new(Particle::Rock).with_cycle(1),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_update_grid_tap_emits_particle_up_if_touched_by_a_particle() {
        /*
         * -- -> r-
         * tr    tr
         */
        let mut g = Grid::new(2, 2);

        g.spawn_particle((0, 1), Particle::from(Tap::new()));
        g.spawn_particle((1, 1), Particle::Rock);
        assert_eq!(
            vec![
                Cell::empty(),
                Cell::empty(),
                Cell::new(Particle::from(Tap::new())),
                Cell::new(Particle::Rock),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::Rock).with_cycle(1),
                Cell::empty(),
                Cell::new(Particle::from(Tap::new())).with_cycle(1),
                Cell::new(Particle::Rock),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_update_grid_tap_emits_particle_down_if_touched_by_a_particle() {
        /*
         * tr -> tr
         * --    r-
         */
        let mut g = Grid::new(2, 2);

        g.spawn_particle((0, 0), Particle::from(Tap::new()));
        g.spawn_particle((1, 0), Particle::Rock);
        assert_eq!(
            vec![
                Cell::new(Particle::from(Tap::new())),
                Cell::new(Particle::Rock),
                Cell::empty(),
                Cell::empty(),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Tap::new())).with_cycle(1),
                Cell::new(Particle::Rock),
                Cell::new(Particle::Rock).with_cycle(1),
                Cell::empty(),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_update_grid_tap_emits_particle_left_if_touched_by_a_particle() {
        /*
         * -t -> rt
         * -r    -r
         */
        let mut g = Grid::new(2, 2);

        g.spawn_particle((1, 0), Particle::from(Tap::new()));
        g.spawn_particle((1, 1), Particle::Rock);
        assert_eq!(
            vec![
                Cell::empty(),
                Cell::new(Particle::from(Tap::new())),
                Cell::empty(),
                Cell::new(Particle::Rock),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::Rock).with_cycle(1),
                Cell::new(Particle::from(Tap::new())).with_cycle(1),
                Cell::empty(),
                Cell::new(Particle::Rock),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_update_grid_tap_emits_particle_right_if_touched_by_a_particle() {
        /*
         * t- -> tr
         * r-    r-
         */
        let mut g = Grid::new(2, 2);

        g.spawn_particle((0, 0), Particle::from(Tap::new()));
        g.spawn_particle((0, 1), Particle::Rock);
        assert_eq!(
            vec![
                Cell::new(Particle::from(Tap::new())),
                Cell::empty(),
                Cell::new(Particle::Rock),
                Cell::empty(),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Tap::new())).with_cycle(1),
                Cell::new(Particle::Rock).with_cycle(1),
                Cell::new(Particle::Rock),
                Cell::empty(),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_update_grid_tap_emits_any_particle_type_it_touches() {
        /*
         * -- -> p-
         * tp    tp
         */
        for particle in [
            Particle::from(Sand::new()),
            Particle::from(Salt::new()),
            Particle::from(Water::new()),
            Particle::Rock,
        ] {
            let mut g = Grid::new(2, 2);

            g.spawn_particle((0, 1), Particle::from(Tap::new()));
            g.spawn_particle((1, 1), particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::new(particle.clone()).with_cycle(1),
                    Cell::empty(),
                    Cell::new(Particle::from(Tap::new())).with_cycle(1),
                    Cell::new(particle),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_update_grid_tap_emits_drain_particle() {
        /*
         * -- -> d-
         * td    -d
         */
        let mut g = Grid::new_with_rand(2, 2, None, Some(|| RowUpdateDirection::Forward));

        g.spawn_particle((0, 1), Particle::from(Tap::new()));
        g.spawn_particle((1, 1), Particle::from(Drain::new()));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Drain::new())).with_cycle(1),
                Cell::empty(),
                Cell::empty().with_cycle(1),
                Cell::new(Particle::from(Drain::new())).with_cycle(1),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_update_grid_tap_emits_drain_particle_gets_deleted_first() {
        /*
         * -- -> --
         * td    -d
         */
        let mut g = Grid::new_with_rand(2, 2, None, Some(|| RowUpdateDirection::Reverse));

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
}
