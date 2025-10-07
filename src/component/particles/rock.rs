use crate::component::grid::GridAccess;

pub fn update_rock<T: GridAccess>(_: &mut T, _: (usize, usize)) {}

#[cfg(test)]
mod tests {
    use crate::component::{
        grid::{Cell, Grid},
        particles::particle::Particle,
    };

    use super::*;

    #[test]
    fn test_update_grid_rock_stays_in_place() {
        /*
         * r- -> r-
         * --    --
         */
        let mut g = Grid::new(2, 2);

        g.spawn_particle(0, 0, Particle::Rock);

        assert_eq!(Cell::new(Some(Particle::Rock), 0), *g.get_cell(0));
        assert_eq!(Cell::new(None, 0), *g.get_cell(1));
        assert_eq!(Cell::new(None, 0), *g.get_cell(2));
        assert_eq!(Cell::new(None, 0), *g.get_cell(3));

        g.update_grid();

        assert_eq!(Cell::new(Some(Particle::Rock), 0), *g.get_cell(0));
        assert_eq!(Cell::new(None, 0), *g.get_cell(1));
        assert_eq!(Cell::new(None, 0), *g.get_cell(2));
        assert_eq!(Cell::new(None, 0), *g.get_cell(3));
    }
}
