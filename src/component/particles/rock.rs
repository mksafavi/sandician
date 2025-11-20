#[derive(Clone, PartialEq, Debug)]
pub struct Rock;

impl Default for Rock {
    fn default() -> Self {
        Self::new()
    }
}

impl Rock {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use crate::component::{
        grid::{Cell, Grid, GridAccess},
        particles::{particle::Particle, rock::Rock, sand::Sand},
    };

    #[test]
    fn test_rock_particles_are_weightless() {
        let particle = Particle::from(Rock::new());
        assert_eq!(0, particle.weight);
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
        g.spawn_particle((0, 1), Particle::from(Rock::new()));
        g.spawn_particle((1, 1), Particle::from(Rock::new()));
        g.spawn_particle((2, 1), Particle::from(Rock::new()));

        assert_eq!(
            vec![
                Cell::empty(),
                Cell::new(Particle::from(Sand::new())),
                Cell::empty(),
                Cell::new(Particle::from(Rock::new())),
                Cell::new(Particle::from(Rock::new())),
                Cell::new(Particle::from(Rock::new())),
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
                Cell::new(Particle::from(Rock::new())),
                Cell::new(Particle::from(Rock::new())),
                Cell::new(Particle::from(Rock::new())),
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
        g.spawn_particle((0, 1), Particle::from(Rock::new()));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty().with_cycle(1),
                Cell::empty(),
                Cell::new(Particle::from(Rock::new())),
                Cell::new(Particle::from(Sand::new())).with_cycle(1),
                Cell::empty(),
                Cell::empty(),
            ],
            *g.get_cells()
        );
    }
}
