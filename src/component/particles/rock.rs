#[cfg(test)]
mod tests {
    use crate::component::{
        grid::{Cell, Grid, GridAccess},
        particles::particle::Particle,
    };

    #[test]
    fn test_update_grid_rock_stays_in_place() {
        /*
         * r- -> r-
         * --    --
         */
        let mut g = Grid::new(2, 2);

        g.spawn_particle((0, 0), Particle::Rock);

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
