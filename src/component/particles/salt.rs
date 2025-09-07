use crate::component::{
    grid::{GridAccess, ParticleHorizontalDirection, ParticleOperation},
    particles::particle::Particle,
};

pub fn update_salt<T: GridAccess>(grid: &T, position: (usize, usize)) -> Option<ParticleOperation> {
    let neighboring_water =
        (-1..=1)
            .flat_map(|y| (-1..=1).map(move |x| (y, x)))
            .fold(false, |acc, (xo, yo)| {
                let n = match grid.get_neighbor_index(position, (xo, yo)) {
                    Ok(i) => match grid.get_cell(i) {
                        Some(p) => matches!(p.particle, Particle::Water),
                        None => false,
                    },
                    Err(_) => false,
                };
                acc || n
            });
    if neighboring_water {
        return Some(ParticleOperation::Dissolve(Particle::Water));
    }
    let index_bottom = match grid.get_neighbor_index(position, (0, 1)) {
        Ok(i) => match grid.get_cell(i) {
            Some(_) => None,
            None => Some(i),
        },
        Err(_) => None,
    };

    let index_bottom_right = match grid.get_neighbor_index(position, (1, 1)) {
        Ok(i) => match grid.get_cell(i) {
            Some(_) => None,
            None => Some(i),
        },
        Err(_) => None,
    };

    let index_bottom_left = match grid.get_neighbor_index(position, (-1, 1)) {
        Ok(i) => match grid.get_cell(i) {
            Some(_) => None,
            None => Some(i),
        },
        Err(_) => None,
    };

    let index = match (index_bottom_left, index_bottom, index_bottom_right) {
        (None, None, None) => None,
        (None, None, Some(r)) => Some(r),
        (Some(l), None, None) => Some(l),
        (Some(l), None, Some(r)) => match grid.water_direction() {
            ParticleHorizontalDirection::Left => Some(l),
            ParticleHorizontalDirection::Right => Some(r),
        },
        (_, Some(i), _) => Some(i),
    };

    index.map(ParticleOperation::Swap)
}

#[cfg(test)]
mod tests {
    use crate::component::grid::{Cell, Grid};

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
    fn test_update_grid_salt_dissolves_when_touches_water() {
        /*
         * for each neighbor:
         * w-- -> ---
         * -S-    ww-
         * ---    ---
         */
        for y in 0..3 {
            for x in 0..3 {
                if (x, y) == (1, 1) {
                    continue;
                }
                let mut g = Grid::new(3, 3);
                g.spawn_particle(1, 1, Particle::Salt);
                g.spawn_particle(x, y, Particle::Water);

                g.update_grid();

                assert_eq!(Some(Cell::new(Particle::Water)), *g.get_cell(4));
            }
        }
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
