#[derive(Clone, PartialEq, Debug)]
pub struct Salt;

impl Default for Salt {
    fn default() -> Self {
        Self::new()
    }
}

impl Salt {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod tests {
    use crate::component::{
        grid::{Cell, Grid, GridAccess, ParticleHorizontalDirection},
        particles::particle::Particle,
    };

    use super::*;

    #[test]
    fn test_update_grid_salt_falls_down_when_bottom_cell_is_empty() {
        /*
         * S- -> --
         * --    S-
         */
        let mut g = Grid::new(2, 2);

        g.spawn_particle((0, 0), Particle::from(Salt::new()));

        assert_eq!(
            vec![
                Cell::new(Particle::from(Salt::new())),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty().with_cycle(1),
                Cell::empty(),
                Cell::new(Particle::from(Salt::new())).with_cycle(1),
                Cell::empty(),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_update_grid_salt_falls_to_bottom_right_when_bottom_cell_and_bottom_left_are_full_and_bottom_right_is_empty()
     {
        /*
         * S- -> --
         * S-    SS
         */
        let mut g = Grid::new(2, 2);

        g.spawn_particle((0, 0), Particle::from(Salt::new()));
        g.spawn_particle((0, 1), Particle::from(Salt::new()));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty().with_cycle(1),
                Cell::empty(),
                Cell::new(Particle::from(Salt::new())),
                Cell::new(Particle::from(Salt::new())).with_cycle(1),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_update_grid_salt_falls_to_bottom_left_when_bottom_cell_and_bottom_right_are_full_and_bottom_left_is_empty()
     {
        /*
         * -S -> --
         * -S    SS
         */
        let mut g = Grid::new(2, 2);

        g.spawn_particle((1, 0), Particle::from(Salt::new()));
        g.spawn_particle((1, 1), Particle::from(Salt::new()));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty(),
                Cell::empty().with_cycle(1),
                Cell::new(Particle::from(Salt::new())).with_cycle(1),
                Cell::new(Particle::from(Salt::new())),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_update_grid_salt_falls_to_bottom_left_or_bottom_right_when_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_forced_left()
     {
        /*
         * -S- -> ---
         * -S-    SS-
         */
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Left), None);

        g.spawn_particle((1, 0), Particle::from(Salt::new()));
        g.spawn_particle((1, 1), Particle::from(Salt::new()));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty(),
                Cell::empty().with_cycle(1),
                Cell::empty(),
                Cell::new(Particle::from(Salt::new())).with_cycle(1),
                Cell::new(Particle::from(Salt::new())),
                Cell::empty(),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_update_grid_salt_falls_to_bottom_left_or_bottom_right_when_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_forced_right()
     {
        /*
         * -S- -> ---
         * -S-    -SS
         */
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Right), None);

        g.spawn_particle((1, 0), Particle::from(Salt::new()));

        g.spawn_particle((1, 1), Particle::from(Salt::new()));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty(),
                Cell::empty().with_cycle(1),
                Cell::empty(),
                Cell::empty(),
                Cell::new(Particle::from(Salt::new())),
                Cell::new(Particle::from(Salt::new())).with_cycle(1),
            ],
            *g.get_cells()
        );
    }
}
