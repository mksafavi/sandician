#[cfg(test)]
mod tests {
    use crate::component::{
        grid::{Cell, Grid, GridAccess},
        particles::{particle::Particle, sand::Sand},
    };

    #[test]
    fn test_update_grid_rock_stays_in_place() {
        /*
         * r- -> r-
         * --    --
         */
        let mut g = Grid::new(2, 2);

        g.spawn_particle((0, 0), Particle::Rock);

        assert_eq!(Cell::new(Particle::Rock), *g.get_cell(0));
        assert_eq!(Cell::empty(), *g.get_cell(1));
        assert_eq!(Cell::empty(), *g.get_cell(2));
        assert_eq!(Cell::empty(), *g.get_cell(3));

        g.update_grid();

        assert_eq!(Cell::new(Particle::Rock), *g.get_cell(0));
        assert_eq!(Cell::empty(), *g.get_cell(1));
        assert_eq!(Cell::empty(), *g.get_cell(2));
        assert_eq!(Cell::empty(), *g.get_cell(3));
    }

    #[test]
    fn test_update_grid_particles_should_not_swap_with_rock() {
        /*
         * -s- -> -s-
         * rrr    rrr
         * ---    ---
         */
        let mut g = Grid::new(3, 3);

        g.spawn_particle((1, 0), Particle::from(Sand::new()));
        g.spawn_particle((0, 1), Particle::Rock);
        g.spawn_particle((1, 1), Particle::Rock);
        g.spawn_particle((2, 1), Particle::Rock);

        assert_eq!(
            vec![
                Cell::empty(),
                Cell::new(Particle::from(Sand::new())),
                Cell::empty(),
                Cell::new(Particle::Rock),
                Cell::new(Particle::Rock),
                Cell::new(Particle::Rock),
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
                Cell::new(Particle::from(Sand::new())),
                Cell::empty(),
                Cell::new(Particle::Rock),
                Cell::new(Particle::Rock),
                Cell::new(Particle::Rock),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_update_grid_particles_move_past_rock() {
        /*
         * s- -> --
         * r-    rs
         * --    --
         */
        let mut g = Grid::new(2, 3);

        g.spawn_particle((0, 0), Particle::from(Sand::new()));
        g.spawn_particle((0, 1), Particle::Rock);

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty().with_cycle(1),
                Cell::empty(),
                Cell::new(Particle::Rock),
                Cell::new(Particle::from(Sand::new())).with_cycle(1),
                Cell::empty(),
                Cell::empty(),
            ],
            *g.get_cells()
        );
    }
}
