use crate::component::grid::{GridAccess, ParticleHorizontalDirection};

pub fn update_salt<T: GridAccess>(grid: &mut T, position: (usize, usize)) {
    if let Some(index) = grid.is_empty(position, (0, 1)) {
        return grid.swap_particles(grid.to_index(position), index);
    }

    if let Some(index) = match (
        grid.is_empty(position, (-1, 1)),
        grid.is_empty(position, (1, 1)),
    ) {
        (None, None) => None,
        (None, Some(r)) => Some(r),
        (Some(l), None) => Some(l),
        (Some(l), Some(r)) => match grid.water_direction() {
            ParticleHorizontalDirection::Left => Some(l),
            ParticleHorizontalDirection::Right => Some(r),
        },
    } {
        grid.swap_particles(grid.to_index(position), index)
    }
}

#[cfg(test)]
mod tests {
    use crate::component::{
        grid::{Cell, Grid},
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

        g.spawn_particle(0, 0, Particle::Salt);

        assert_eq!(Some(Cell::new(Particle::Salt)), *g.get_cell(0));
        assert_eq!(None, *g.get_cell(1));
        assert_eq!(None, *g.get_cell(2));
        assert_eq!(None, *g.get_cell(3));

        g.update_grid();

        assert_eq!(None, *g.get_cell(0));
        assert_eq!(None, *g.get_cell(1));
        assert_eq!(Some(Cell::new(Particle::Salt)), *g.get_cell(2));
        assert_eq!(None, *g.get_cell(3));
    }

    #[test]
    fn test_update_grid_salt_falls_to_bottom_right_when_bottom_cell_and_bottom_left_are_full_and_bottom_right_is_empty()
     {
        /*
         * S- -> --
         * S-    SS
         */
        let mut g = Grid::new(2, 2);

        g.spawn_particle(0, 0, Particle::Salt);
        g.spawn_particle(0, 1, Particle::Salt);

        g.update_grid();

        assert_eq!(None, *g.get_cell(0));
        assert_eq!(None, *g.get_cell(1));
        assert_eq!(Some(Cell::new(Particle::Salt)), *g.get_cell(2));
        assert_eq!(Some(Cell::new(Particle::Salt)), *g.get_cell(3));
    }

    #[test]
    fn test_update_grid_salt_falls_to_bottom_left_when_bottom_cell_and_bottom_right_are_full_and_bottom_left_is_empty()
     {
        /*
         * -S -> --
         * -S    SS
         */
        let mut g = Grid::new(2, 2);

        g.spawn_particle(1, 0, Particle::Salt);
        g.spawn_particle(1, 1, Particle::Salt);

        g.update_grid();

        assert_eq!(None, *g.get_cell(0));
        assert_eq!(None, *g.get_cell(1));
        assert_eq!(Some(Cell::new(Particle::Salt)), *g.get_cell(2));
        assert_eq!(Some(Cell::new(Particle::Salt)), *g.get_cell(3));
    }

    #[test]
    fn test_update_grid_salt_falls_to_bottom_left_or_bottom_right_when_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_forced_left()
     {
        /*
         * -S- -> ---
         * -S-    SS-
         */
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Left), None);

        g.spawn_particle(1, 0, Particle::Salt);
        g.spawn_particle(1, 1, Particle::Salt);

        g.update_grid();

        assert_eq!(None, *g.get_cell(0));
        assert_eq!(None, *g.get_cell(1));
        assert_eq!(None, *g.get_cell(2));
        assert_eq!(Some(Cell::new(Particle::Salt)), *g.get_cell(3));
        assert_eq!(Some(Cell::new(Particle::Salt)), *g.get_cell(4));
        assert_eq!(None, *g.get_cell(5));
    }

    #[test]
    fn test_update_grid_salt_falls_to_bottom_left_or_bottom_right_when_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_forced_right()
     {
        /*
         * -S- -> ---
         * -S-    -SS
         */
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Right), None);

        g.spawn_particle(1, 0, Particle::Salt);

        g.spawn_particle(1, 1, Particle::Salt);

        g.update_grid();

        assert_eq!(None, *g.get_cell(0));
        assert_eq!(None, *g.get_cell(1));
        assert_eq!(None, *g.get_cell(2));
        assert_eq!(None, *g.get_cell(3));
        assert_eq!(Some(Cell::new(Particle::Salt)), *g.get_cell(4));
        assert_eq!(Some(Cell::new(Particle::Salt)), *g.get_cell(5));
    }
}
