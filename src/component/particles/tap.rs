use crate::component::grid::GridAccess;

use super::particle::{self, Particle};

#[derive(Clone, PartialEq, Debug)]

pub struct Tap {
    pub particle: Option<Box<Particle>>,
}

impl Default for Tap {
    fn default() -> Self {
        Self::new()
    }
}

impl Tap {
    pub fn new() -> Self {
        Self { particle: None }
    }

    pub fn with_particle(particle: &Particle) -> Self {
        Self {
            particle: Some(Box::new(particle.clone())),
        }
    }
}

impl particle::Updatable for Tap {
    fn update<T: GridAccess>(&self, grid: &mut T, position: (usize, usize)) {
        let mut particle = self.clone();
        if particle.particle.is_none() {
            let mut particle_to_clone = None;
            for y in -1..=1 {
                for x in -1..=1 {
                    if let Ok(i) = grid.get_neighbor_index(position, (x, y)) {
                        if let Some(p) = &grid.get_cell(i).particle {
                            match p {
                                Particle::Tap(..) | Particle::Drain(..) => {}
                                _ => {
                                    particle_to_clone = Some(p.clone());
                                }
                            }
                        }
                    };
                }
            }

            if let Some(p) = particle_to_clone {
                let cell = grid.get_cell_mut(grid.to_index(position));
                particle.particle = Some(Box::new(p.clone()));
                cell.particle = Some(Particle::from(particle.clone()));
            }
        }

        if let Some(particle) = &particle.particle {
            for y in -1..=1 {
                for x in -1..=1 {
                    if let Ok(i) = grid.get_neighbor_index(position, (x, y)) {
                        if grid.get_cell_mut(i).particle.is_none() {
                            let cycle = grid.cycle();
                            let cell = grid.get_cell_mut(i);
                            cell.particle = Some(*particle.clone());
                            cell.cycle = cycle;
                            grid.get_cell_mut(grid.to_index(position)).cycle = cycle;
                        }
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
                let mut g = Grid::new(3, 3);

                g.spawn_particle((1, 1), Particle::from(Tap::new()));
                g.spawn_particle((x, y), Particle::Rock);

                g.update_grid();

                for y in 0..3 {
                    for x in 0..3 {
                        if (x, y) == (1, 1) {
                            continue;
                        }
                        assert_eq!(
                            Some(Particle::Rock),
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
            Particle::Rock,
        ] {
            let mut g = Grid::new(2, 2);

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
        let mut g = Grid::new_with_rand(2, 2, None, Some(|| RowUpdateDirection::Forward));

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
        g.spawn_particle((0, 1), Particle::Rock);

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from({
                    let mut t = Tap::new();
                    t.particle = Some(Box::new(Particle::Rock));
                    t
                })),
                Cell::new(Particle::Rock),
            ],
            *g.get_cells()
        );

        g.despawn_particle((0, 1));
        g.spawn_particle((0, 1), Particle::from(Sand::new()));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from({
                    let mut t = Tap::new();
                    t.particle = Some(Box::new(Particle::Rock));
                    t
                })),
                Cell::new(Particle::from(Sand::new())).with_cycle(1)
            ],
            *g.get_cells()
        );
    }
}
