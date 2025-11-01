use crate::component::grid::GridAccess;

use super::particle;

#[derive(Clone, PartialEq, Debug)]
pub struct Salt {
    pub weight: u8,
}

impl Default for Salt {
    fn default() -> Self {
        Self::new()
    }
}

impl Salt {
    pub fn new() -> Self {
        Self { weight: 2 }
    }
}

impl particle::Updatable for Salt {
    fn update<T: GridAccess>(&mut self, grid: &mut T, position: (usize, usize)) {
        particle::gravity(grid, position);
    }
}

#[cfg(test)]
mod tests {
    use crate::component::{
        grid::{Cell, Grid, ParticleHorizontalDirection},
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

        assert_eq!(Cell::new(Particle::from(Salt::new())), *g.get_cell(0));
        assert_eq!(Cell::empty(), *g.get_cell(1));
        assert_eq!(Cell::empty(), *g.get_cell(2));
        assert_eq!(Cell::empty(), *g.get_cell(3));

        g.update_grid();

        assert_eq!(Cell::empty().with_cycle(1), *g.get_cell(0));
        assert_eq!(Cell::empty(), *g.get_cell(1));
        assert_eq!(
            Cell::new(Particle::from(Salt::new())).with_cycle(1),
            *g.get_cell(2)
        );
        assert_eq!(Cell::empty(), *g.get_cell(3));
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

        assert_eq!(Cell::empty().with_cycle(1), *g.get_cell(0));
        assert_eq!(Cell::empty(), *g.get_cell(1));
        assert_eq!(Cell::new(Particle::from(Salt::new())), *g.get_cell(2));
        assert_eq!(
            Cell::new(Particle::from(Salt::new())).with_cycle(1),
            *g.get_cell(3)
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

        assert_eq!(Cell::empty(), *g.get_cell(0));
        assert_eq!(Cell::empty().with_cycle(1), *g.get_cell(1));
        assert_eq!(
            Cell::new(Particle::from(Salt::new())).with_cycle(1),
            *g.get_cell(2)
        );
        assert_eq!(Cell::new(Particle::from(Salt::new())), *g.get_cell(3));
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

        assert_eq!(Cell::empty(), *g.get_cell(0));
        assert_eq!(Cell::empty().with_cycle(1), *g.get_cell(1));
        assert_eq!(Cell::empty(), *g.get_cell(2));
        assert_eq!(
            Cell::new(Particle::from(Salt::new())).with_cycle(1),
            *g.get_cell(3)
        );
        assert_eq!(Cell::new(Particle::from(Salt::new())), *g.get_cell(4));
        assert_eq!(Cell::empty(), *g.get_cell(5));
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

        assert_eq!(Cell::empty(), *g.get_cell(0));
        assert_eq!(Cell::empty().with_cycle(1), *g.get_cell(1));
        assert_eq!(Cell::empty(), *g.get_cell(2));
        assert_eq!(Cell::empty(), *g.get_cell(3));
        assert_eq!(Cell::new(Particle::from(Salt::new())), *g.get_cell(4));
        assert_eq!(
            Cell::new(Particle::from(Salt::new())).with_cycle(1),
            *g.get_cell(5)
        );
    }
}
