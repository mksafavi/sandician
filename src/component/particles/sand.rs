use crate::component::grid::GridAccess;

use super::particle;

#[derive(Clone, PartialEq, Debug)]
pub struct Sand {
    pub weight: u8,
}

impl Default for Sand {
    fn default() -> Self {
        Self::new()
    }
}

impl Sand {
    pub fn new() -> Sand {
        Sand { weight: 0 }
    }
}

impl particle::Updatable for Sand {
    fn update<T: GridAccess>(&self, grid: &mut T, position: (usize, usize)) {
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
    fn test_update_grid_sand_falls_down_when_bottom_cell_is_empty() {
        /*
         * s- -> --
         * --    s-
         */
        let mut g = Grid::new(2, 2);
        g.spawn_particle((0, 0), Particle::new_sand());

        assert_eq!((Cell::new(Some(Particle::new_sand()), 0)), *g.get_cell(0));
        assert_eq!(Cell::new(None, 0), *g.get_cell(1));
        assert_eq!(Cell::new(None, 0), *g.get_cell(2));
        assert_eq!(Cell::new(None, 0), *g.get_cell(3));

        g.update_grid();

        assert_eq!(Cell::new(None, 1), *g.get_cell(0));
        assert_eq!(Cell::new(None, 0), *g.get_cell(1));
        assert_eq!((Cell::new(Some(Particle::new_sand()), 1)), *g.get_cell(2));
        assert_eq!(Cell::new(None, 0), *g.get_cell(3));
    }

    #[test]
    fn test_update_grid_sand_falls_to_bottom_right_when_bottom_cell_and_bottom_left_are_full_and_bottom_right_is_empty()
     {
        /*
         * s- -> --
         * s-    ss
         */
        let mut g = Grid::new(2, 2);

        g.spawn_particle((0, 0), Particle::new_sand());
        g.spawn_particle((0, 1), Particle::new_sand());

        g.update_grid();

        assert_eq!(Cell::new(None, 1), *g.get_cell(0));
        assert_eq!(Cell::new(None, 0), *g.get_cell(1));
        assert_eq!((Cell::new(Some(Particle::new_sand()), 0)), *g.get_cell(2));
        assert_eq!((Cell::new(Some(Particle::new_sand()), 1)), *g.get_cell(3));
    }

    #[test]
    fn test_update_grid_sand_falls_bottom_left_when_bottom_cell_and_bottom_right_are_full_and_bottom_left_is_empty()
     {
        /*
         * -s -> --
         * -s    ss
         */
        let mut g = Grid::new(2, 2);

        g.spawn_particle((1, 0), Particle::new_sand());
        g.spawn_particle((1, 1), Particle::new_sand());

        g.update_grid();

        assert_eq!(Cell::new(None, 0), *g.get_cell(0));
        assert_eq!(Cell::new(None, 1), *g.get_cell(1));
        assert_eq!((Cell::new(Some(Particle::new_sand()), 1)), *g.get_cell(2));
        assert_eq!((Cell::new(Some(Particle::new_sand()), 0)), *g.get_cell(3));
    }

    #[test]
    fn test_update_grid_sand_falls_bottom_left_or_bottom_right_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_forced_left()
     {
        /*
         * -s- -> ---
         * -s-    ss-
         */
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Left), None);

        g.spawn_particle((1, 0), Particle::new_sand());
        g.spawn_particle((1, 1), Particle::new_sand());

        g.update_grid();

        assert_eq!(Cell::new(None, 0), *g.get_cell(0));
        assert_eq!(Cell::new(None, 1), *g.get_cell(1));
        assert_eq!(Cell::new(None, 0), *g.get_cell(2));
        assert_eq!((Cell::new(Some(Particle::new_sand()), 1)), *g.get_cell(3));
        assert_eq!((Cell::new(Some(Particle::new_sand()), 0)), *g.get_cell(4));
        assert_eq!(Cell::new(None, 0), *g.get_cell(5));
    }

    #[test]
    fn test_update_grid_sand_falls_bottom_left_or_bottom_right_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_forced_right()
     {
        /*
         * -s- -> ---
         * -s-    -ss
         */
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Right), None);

        g.spawn_particle((1, 0), Particle::new_sand());
        g.spawn_particle((1, 1), Particle::new_sand());

        g.update_grid();

        assert_eq!(Cell::new(None, 0), *g.get_cell(0));
        assert_eq!(Cell::new(None, 1), *g.get_cell(1));
        assert_eq!(Cell::new(None, 0), *g.get_cell(2));
        assert_eq!(Cell::new(None, 0), *g.get_cell(3));
        assert_eq!((Cell::new(Some(Particle::new_sand()), 0)), *g.get_cell(4));
        assert_eq!((Cell::new(Some(Particle::new_sand()), 1)), *g.get_cell(5));
    }
}
