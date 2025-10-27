use super::particle::{self, Particle};
use crate::component::grid::{GridAccess, ParticleHorizontalDirection};

pub fn update_water<T: GridAccess>(grid: &mut T, solute: u8, position: (usize, usize)) {
    let index_top_left = match grid.get_neighbor_index(position, (-1, -1)) {
        Ok(i) => {
            let c = grid.get_cell(i);
            match c.particle {
                Some(Particle::Salt) => {
                    if 0 < solute {
                        let index = grid.to_index(position);
                        grid.get_cell_mut(index).particle =
                            Some(Particle::Water { solute: solute - 1 });
                        grid.dissolve_particles(index, i);
                        return;
                    }
                    match grid.is_simulated(c) {
                        true => None,
                        false => Some(i),
                    }
                }
                Some(Particle::Sand) => match grid.is_simulated(c) {
                    true => None,
                    false => Some(i),
                },
                _ => None,
            }
        }
        Err(_) => None,
    };

    let index_top = match grid.get_neighbor_index(position, (0, -1)) {
        Ok(i) => {
            let c = grid.get_cell(i);
            match c.particle {
                Some(Particle::Salt) => {
                    if 0 < solute {
                        let index = grid.to_index(position);
                        grid.get_cell_mut(index).particle =
                            Some(Particle::Water { solute: solute - 1 });
                        grid.dissolve_particles(index, i);
                        return;
                    }
                    match grid.is_simulated(c) {
                        true => None,
                        false => Some(i),
                    }
                }
                Some(Particle::Sand) => match grid.is_simulated(c) {
                    true => None,
                    false => Some(i),
                },
                _ => None,
            }
        }
        Err(_) => None,
    };

    let index_top_right = match grid.get_neighbor_index(position, (1, -1)) {
        Ok(i) => {
            let c = grid.get_cell(i);
            match c.particle {
                Some(Particle::Salt) => {
                    if 0 < solute {
                        let index = grid.to_index(position);
                        grid.get_cell_mut(index).particle =
                            Some(Particle::Water { solute: solute - 1 });
                        grid.dissolve_particles(index, i);
                        return;
                    }
                    match grid.is_simulated(c) {
                        true => None,
                        false => Some(i),
                    }
                }
                Some(Particle::Sand) => match grid.is_simulated(c) {
                    true => None,
                    false => Some(i),
                },
                _ => None,
            }
        }
        Err(_) => None,
    };

    let index_left = match grid.get_neighbor_index(position, (-1, 0)) {
        Ok(i) => {
            let c = grid.get_cell(i);
            match &c.particle {
                Some(p) => {
                    if 0 < solute && *p == Particle::Salt {
                        let index = grid.to_index(position);
                        grid.get_cell_mut(index).particle =
                            Some(Particle::Water { solute: solute - 1 });
                        grid.dissolve_particles(index, i);
                        return;
                    }
                    None
                }
                None => Some(i),
            }
        }
        Err(_) => None,
    };

    let index_right = match grid.get_neighbor_index(position, (1, 0)) {
        Ok(i) => {
            let c = grid.get_cell(i);
            match &c.particle {
                Some(p) => {
                    if 0 < solute && *p == Particle::Salt {
                        let index = grid.to_index(position);
                        grid.get_cell_mut(index).particle =
                            Some(Particle::Water { solute: solute - 1 });
                        grid.dissolve_particles(index, i);
                        return;
                    }
                    None
                }
                None => Some(i),
            }
        }
        Err(_) => None,
    };

    let index_bottom_left = match grid.get_neighbor_index(position, (-1, 1)) {
        Ok(i) => {
            let c = grid.get_cell(i);
            match &c.particle {
                Some(p) => {
                    if 0 < solute && *p == Particle::Salt {
                        let index = grid.to_index(position);
                        grid.get_cell_mut(index).particle =
                            Some(Particle::Water { solute: solute - 1 });
                        grid.dissolve_particles(index, i);
                        return;
                    }
                    None
                }
                None => Some(i),
            }
        }
        Err(_) => None,
    };

    let index_bottom = match grid.get_neighbor_index(position, (0, 1)) {
        Ok(i) => {
            let c = grid.get_cell(i);
            match &c.particle {
                Some(p) => {
                    if 0 < solute && *p == Particle::Salt {
                        let index = grid.to_index(position);
                        grid.get_cell_mut(index).particle =
                            Some(Particle::Water { solute: solute - 1 });
                        grid.dissolve_particles(index, i);
                        return;
                    }
                    None
                }
                None => Some(i),
            }
        }
        Err(_) => None,
    };

    let index_bottom_right = match grid.get_neighbor_index(position, (1, 1)) {
        Ok(i) => {
            let c = grid.get_cell(i);
            match &c.particle {
                Some(p) => {
                    if 0 < solute && *p == Particle::Salt {
                        let index = grid.to_index(position);
                        grid.get_cell_mut(index).particle =
                            Some(Particle::Water { solute: solute - 1 });
                        grid.dissolve_particles(index, i);
                        return;
                    }
                    None
                }
                None => Some(i),
            }
        }
        Err(_) => None,
    };

    if sink_in_water(grid, position) {
        return;
    }

    if particle::gravity(grid, position) {
        return;
    }

    if slide_water(grid, position) {
        return;
    }
}

fn sink_in_water<T: GridAccess>(grid: &mut T, position: (usize, usize)) -> bool {
    let index_top_left = match grid.get_neighbor_index(position, (-1, -1)) {
        Ok(i) => {
            let c = grid.get_cell(i);
            match c.particle {
                Some(Particle::Salt | Particle::Sand) => match grid.is_simulated(c) {
                    true => None,
                    false => Some(i),
                },
                _ => None,
            }
        }
        Err(_) => None,
    };
    let index_top = match grid.get_neighbor_index(position, (0, -1)) {
        Ok(i) => {
            let c = grid.get_cell(i);
            match c.particle {
                Some(Particle::Salt | Particle::Sand) => match grid.is_simulated(c) {
                    true => None,
                    false => Some(i),
                },
                _ => None,
            }
        }
        Err(_) => None,
    };
    let index_top_right = match grid.get_neighbor_index(position, (1, -1)) {
        Ok(i) => {
            let c = grid.get_cell(i);
            match c.particle {
                Some(Particle::Salt | Particle::Sand) => match grid.is_simulated(c) {
                    true => None,
                    false => Some(i),
                },
                _ => None,
            }
        }
        Err(_) => None,
    };
    let top_index = match (index_top_left, index_top, index_top_right) {
        (None, None, None) => None,
        (_, Some(i), _) => Some(i),
        (None, None, Some(i)) => Some(i),
        (Some(i), None, None) => Some(i),
        (Some(l), None, Some(r)) => match grid.water_direction() {
            ParticleHorizontalDirection::Left => Some(l),
            ParticleHorizontalDirection::Right => Some(r),
        },
    };

    if let Some(index) = top_index {
        grid.swap_particles(grid.to_index(position), index);
        true
    } else {
        false
    }
}

fn slide_water<T: GridAccess>(grid: &mut T, position: (usize, usize)) -> bool {
    let index_left = match grid.get_neighbor_index(position, (-1, 0)) {
        Ok(i) => {
            let c = grid.get_cell(i);
            match &c.particle {
                Some(_) => None,
                None => Some(i),
            }
        }
        Err(_) => None,
    };

    let index_right = match grid.get_neighbor_index(position, (1, 0)) {
        Ok(i) => {
            let c = grid.get_cell(i);
            match &c.particle {
                Some(_) => None,
                None => Some(i),
            }
        }
        Err(_) => None,
    };

    let index = match (index_left, index_right) {
        (None, None) => None,
        (None, Some(i)) => Some(i),
        (Some(i), None) => Some(i),
        (Some(l), Some(r)) => match grid.water_direction() {
            ParticleHorizontalDirection::Left => Some(l),
            ParticleHorizontalDirection::Right => Some(r),
        },
    };

    if let Some(index) = index {
        grid.swap_particles(grid.to_index(position), index);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::component::{
        grid::{Cell, Grid, RowUpdateDirection},
        particles::particle::Particle,
    };

    use super::*;

    #[test]
    fn test_update_grid_water_falls_down_to_last_row_stays_there() {
        /*
         * w -> - -> -
         * -    w    -
         * -    -    w
         */
        let mut g = Grid::new(1, 3);
        g.spawn_particle(0, 0, Particle::new_water());

        g.update_grid();
        assert_eq!(Cell::new(None, 1), *g.get_cell(0));
        assert_eq!(Cell::new(Some(Particle::new_water()), 1), *g.get_cell(1));
        assert_eq!(Cell::new(None, 0), *g.get_cell(2));

        g.update_grid();
        assert_eq!(Cell::new(None, 1), *g.get_cell(0));
        assert_eq!(Cell::new(None, 2), *g.get_cell(1));
        assert_eq!(Cell::new(Some(Particle::new_water()), 2), *g.get_cell(2));
    }

    #[test]
    fn test_update_grid_water_moves_right_when_bottom_cell_and_left_are_full() {
        /*
         * --- -> ---
         * sw-    s-w
         */
        let mut g = Grid::new(3, 2);
        g.spawn_particle(0, 1, Particle::Sand);
        g.spawn_particle(1, 1, Particle::new_water());

        g.update_grid();

        assert_eq!(Cell::new(None, 0), *g.get_cell(0));
        assert_eq!(Cell::new(None, 0), *g.get_cell(1));
        assert_eq!(Cell::new(None, 0), *g.get_cell(2));
        assert_eq!(Cell::new(Some(Particle::Sand), 0), *g.get_cell(3));
        assert_eq!(Cell::new(None, 1), *g.get_cell(4));
        assert_eq!(Cell::new(Some(Particle::new_water()), 1), *g.get_cell(5));
    }

    #[test]
    fn test_update_grid_water_moves_left_when_bottom_cell_and_right_are_full() {
        /*
         * --- -> ---
         * -ws    w-s
         */
        let mut g = Grid::new(3, 2);
        g.spawn_particle(1, 1, Particle::new_water());
        g.spawn_particle(2, 1, Particle::Sand);

        g.update_grid();

        assert_eq!(Cell::new(None, 0), *g.get_cell(0));
        assert_eq!(Cell::new(None, 0), *g.get_cell(1));
        assert_eq!(Cell::new(None, 0), *g.get_cell(2));
        assert_eq!(Cell::new(Some(Particle::new_water()), 1), *g.get_cell(3));
        assert_eq!(Cell::new(None, 1), *g.get_cell(4));
        assert_eq!(Cell::new(Some(Particle::Sand), 0), *g.get_cell(5));
    }

    #[test]
    fn test_update_grid_water_moves_left_or_right_when_both_right_and_left_are_empty_forced_right()
    {
        /*
         * --- -> ---
         * -w-    --w
         */
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Right), None);

        g.spawn_particle(1, 1, Particle::new_water());

        g.update_grid();

        assert_eq!(Cell::new(None, 0), *g.get_cell(0));
        assert_eq!(Cell::new(None, 0), *g.get_cell(1));
        assert_eq!(Cell::new(None, 0), *g.get_cell(2));
        assert_eq!(Cell::new(None, 0), *g.get_cell(3));
        assert_eq!(Cell::new(None, 1), *g.get_cell(4));
        assert_eq!(Cell::new(Some(Particle::new_water()), 1), *g.get_cell(5));
    }

    #[test]
    fn test_update_grid_water_moves_left_or_right_when_both_right_and_left_are_empty_forced_left() {
        /*
         * --- -> ---
         * -w-    w--
         */
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Left), None);

        g.spawn_particle(1, 1, Particle::new_water());

        g.update_grid();

        assert_eq!(Cell::new(None, 0), *g.get_cell(0));
        assert_eq!(Cell::new(None, 0), *g.get_cell(1));
        assert_eq!(Cell::new(None, 0), *g.get_cell(2));
        assert_eq!(Cell::new(Some(Particle::new_water()), 1), *g.get_cell(3));
        assert_eq!(Cell::new(None, 1), *g.get_cell(4));
        assert_eq!(Cell::new(None, 0), *g.get_cell(5));
    }

    #[test]
    fn test_update_grid_water_falls_to_bottom_right_when_bottom_cell_and_bottom_left_are_full_and_bottom_right_is_empty()
     {
        /*
         * w- -> --
         * s-    sw
         */
        let mut g = Grid::new(2, 2);

        g.spawn_particle(0, 0, Particle::new_water());
        g.spawn_particle(0, 1, Particle::Sand);

        g.update_grid();

        assert_eq!(Cell::new(None, 1), *g.get_cell(0));
        assert_eq!(Cell::new(None, 0), *g.get_cell(1));
        assert_eq!(Cell::new(Some(Particle::Sand), 0), *g.get_cell(2));
        assert_eq!(Cell::new(Some(Particle::new_water()), 1), *g.get_cell(3));
    }

    #[test]
    fn test_update_grid_water_falls_bottom_left_when_bottom_cell_and_bottom_right_are_full_and_bottom_left_is_empty()
     {
        /*
         * -w -> --
         * -s    ws
         */
        let mut g = Grid::new(2, 2);

        g.spawn_particle(1, 0, Particle::new_water());
        g.spawn_particle(1, 1, Particle::Sand);

        g.update_grid();

        assert_eq!(Cell::new(None, 0), *g.get_cell(0));
        assert_eq!(Cell::new(None, 1), *g.get_cell(1));
        assert_eq!(Cell::new(Some(Particle::new_water()), 1), *g.get_cell(2));
        assert_eq!(Cell::new(Some(Particle::Sand), 0), *g.get_cell(3));
    }

    #[test]
    fn test_update_grid_water_falls_bottom_left_or_bottom_right_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_forced_left()
     {
        /*
         * -w- -> ---
         * -s-    ws-
         */
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Left), None);

        g.spawn_particle(1, 0, Particle::new_water());
        g.spawn_particle(1, 1, Particle::Sand);

        g.update_grid();

        assert_eq!(Cell::new(None, 0), *g.get_cell(0));
        assert_eq!(Cell::new(None, 1), *g.get_cell(1));
        assert_eq!(Cell::new(None, 0), *g.get_cell(2));
        assert_eq!(Cell::new(Some(Particle::new_water()), 1), *g.get_cell(3));
        assert_eq!(Cell::new(Some(Particle::Sand), 0), *g.get_cell(4));
        assert_eq!(Cell::new(None, 0), *g.get_cell(5));
    }

    #[test]
    fn test_update_grid_water_falls_bottom_left_or_bottom_right_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_forced_right()
     {
        /*
         * -w- -> ---
         * -s-    -sw
         */
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Right), None);

        g.spawn_particle(1, 0, Particle::new_water());
        g.spawn_particle(1, 1, Particle::Sand);

        g.update_grid();

        assert_eq!(Cell::new(None, 0), *g.get_cell(0));
        assert_eq!(Cell::new(None, 1), *g.get_cell(1));
        assert_eq!(Cell::new(None, 0), *g.get_cell(2));
        assert_eq!(Cell::new(None, 0), *g.get_cell(3));
        assert_eq!(Cell::new(Some(Particle::Sand), 0), *g.get_cell(4));
        assert_eq!(Cell::new(Some(Particle::new_water()), 1), *g.get_cell(5));
    }

    #[test]
    fn test_updating_rows_in_forward_order_creates_a_left_bias_on_water() {
        /*
         * -ww- => ww-- or w--w
         */
        let mut g = Grid::new_with_rand(
            4,
            1,
            Some(|| ParticleHorizontalDirection::Left),
            Some(|| RowUpdateDirection::Forward),
        );

        g.spawn_particle(1, 0, Particle::new_water());
        g.spawn_particle(2, 0, Particle::new_water());

        g.update_grid();

        assert_eq!(Cell::new(Some(Particle::new_water()), 1), *g.get_cell(0));
        assert_eq!(Cell::new(Some(Particle::new_water()), 1), *g.get_cell(1));
        assert_eq!(Cell::new(None, 1), *g.get_cell(2));
        assert_eq!(Cell::new(None, 0), *g.get_cell(3));

        let mut g = Grid::new_with_rand(
            4,
            1,
            Some(|| ParticleHorizontalDirection::Right),
            Some(|| RowUpdateDirection::Forward),
        );

        g.spawn_particle(1, 0, Particle::new_water());
        g.spawn_particle(2, 0, Particle::new_water());

        g.update_grid();

        assert_eq!(Cell::new(Some(Particle::new_water()), 1), *g.get_cell(0));
        assert_eq!(Cell::new(None, 1), *g.get_cell(1));
        assert_eq!(Cell::new(None, 1), *g.get_cell(2));
        assert_eq!(Cell::new(Some(Particle::new_water()), 1), *g.get_cell(3));
    }

    #[test]
    fn test_updating_rows_in_reverse_order_creates_a_right_bias_on_water() {
        /*
         * -ww- => --ww or w--w
         */
        let mut g = Grid::new_with_rand(
            4,
            1,
            Some(|| ParticleHorizontalDirection::Right),
            Some(|| RowUpdateDirection::Reverse),
        );

        g.spawn_particle(1, 0, Particle::new_water());
        g.spawn_particle(2, 0, Particle::new_water());

        g.update_grid();

        assert_eq!(Cell::new(None, 0), *g.get_cell(0));
        assert_eq!(Cell::new(None, 1), *g.get_cell(1));
        assert_eq!(Cell::new(Some(Particle::new_water()), 1), *g.get_cell(2));
        assert_eq!(Cell::new(Some(Particle::new_water()), 1), *g.get_cell(3));

        let mut g = Grid::new_with_rand(
            4,
            1,
            Some(|| ParticleHorizontalDirection::Left),
            Some(|| RowUpdateDirection::Reverse),
        );

        g.spawn_particle(1, 0, Particle::new_water());
        g.spawn_particle(2, 0, Particle::new_water());

        g.update_grid();

        assert_eq!(Cell::new(Some(Particle::new_water()), 1), *g.get_cell(0));
        assert_eq!(Cell::new(None, 1), *g.get_cell(1));
        assert_eq!(Cell::new(None, 1), *g.get_cell(2));
        assert_eq!(Cell::new(Some(Particle::new_water()), 1), *g.get_cell(3));
    }

    #[test]
    fn test_sand_should_sink_to_bottom_in_water() {
        /*
         * -s- -> -w-
         * sws    sss
         */

        let mut g = Grid::new(3, 2);

        g.spawn_particle(1, 0, Particle::Sand);
        g.spawn_particle(0, 1, Particle::Sand);
        g.spawn_particle(1, 1, Particle::new_water());
        g.spawn_particle(2, 1, Particle::Sand);

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(None, 0),
                Cell::new(Some(Particle::new_water()), 1),
                Cell::new(None, 0),
                Cell::new(Some(Particle::Sand), 0),
                Cell::new(Some(Particle::Sand), 1),
                Cell::new(Some(Particle::Sand), 0),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_sand_should_sink_to_bottom_left_in_water() {
        /*
         * -s- -> -w-
         * wss    sss
         */
        let mut g = Grid::new(3, 2);

        g.spawn_particle(1, 0, Particle::Sand);
        g.spawn_particle(0, 1, Particle::new_water());
        g.spawn_particle(1, 1, Particle::Sand);
        g.spawn_particle(2, 1, Particle::Sand);

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(None, 0),
                Cell::new(Some(Particle::new_water()), 1),
                Cell::new(None, 0),
                Cell::new(Some(Particle::Sand), 1),
                Cell::new(Some(Particle::Sand), 0),
                Cell::new(Some(Particle::Sand), 0),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_sand_should_sink_to_bottom_right_in_water() {
        /*
         * -s- -> -w-
         * ssw    sss
         */
        let mut g = Grid::new(3, 2);

        g.spawn_particle(1, 0, Particle::Sand);
        g.spawn_particle(0, 1, Particle::Sand);
        g.spawn_particle(1, 1, Particle::Sand);
        g.spawn_particle(2, 1, Particle::new_water());

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(None, 0),
                Cell::new(Some(Particle::new_water()), 1),
                Cell::new(None, 0),
                Cell::new(Some(Particle::Sand), 0),
                Cell::new(Some(Particle::Sand), 0),
                Cell::new(Some(Particle::Sand), 1),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_sand_should_sink_in_water_but_water_should_not_climb_sands() {
        /*
         * s -> s -> w
         * s    w    s
         * w    s    s
         */
        let mut g = Grid::new_with_rand(1, 3, Some(|| ParticleHorizontalDirection::Right), None);

        g.spawn_particle(0, 0, Particle::Sand);
        g.spawn_particle(0, 1, Particle::Sand);
        g.spawn_particle(0, 2, Particle::new_water());

        assert_eq!(
            vec![
                Cell::new(Some(Particle::Sand), 0),
                Cell::new(Some(Particle::Sand), 0),
                Cell::new(Some(Particle::new_water()), 0),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Some(Particle::Sand), 0),
                Cell::new(Some(Particle::new_water()), 1),
                Cell::new(Some(Particle::Sand), 1),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Some(Particle::new_water()), 2),
                Cell::new(Some(Particle::Sand), 2),
                Cell::new(Some(Particle::Sand), 1),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_solving_particle_counts_as_being_simulated() {
        /*
         * s -> s
         * w    w
         * S    -
         */

        let mut g = Grid::new(1, 3);
        g.spawn_particle(0, 0, Particle::Sand);
        g.spawn_particle(0, 1, Particle::new_water());
        g.spawn_particle(0, 2, Particle::Salt);

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Some(Particle::Sand), 0),
                Cell::new(Some(Particle::Water { solute: 2 }), 1),
                Cell::new(None, 1),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_update_grid_water_dissolve_neighboring_salts() {
        /*
         * for each neighbor:
         * S-- -> ---
         * -w-    -w-
         * ---    ---
         */
        for y in 0..3 {
            for x in 0..3 {
                if (x, y) == (1, 1) {
                    continue;
                }
                let mut g = Grid::new(3, 3);
                g.spawn_particle(1, 1, Particle::new_water());
                g.spawn_particle(x, y, Particle::Salt);

                g.update_grid();

                assert_eq!(Some(Particle::Water { solute: 2 }), g.get_cell(4).particle);

                for y in 0..3 {
                    for x in 0..3 {
                        if (x, y) == (1, 1) {
                            continue;
                        }
                        assert_eq!(None, g.get_cell(g.to_index((x, y))).clone().particle);
                    }
                }
            }
        }
    }

    #[test]
    fn test_update_grid_water_can_only_dissolve_three_salt_particles() {
        let mut g = Grid::new(3, 3);
        g.spawn_particle(1, 1, Particle::new_water());
        for y in 0..3 {
            for x in 0..3 {
                if (x, y) == (1, 1) {
                    continue;
                }
                g.spawn_particle(x, y, Particle::Salt);
            }
        }

        assert_eq!(
            vec![
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Water { solute: 3 }), 0),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(None, 1),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Water { solute: 2 }), 1),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(None, 1),
                Cell::new(None, 2),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Water { solute: 1 }), 2),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(None, 1),
                Cell::new(None, 2),
                Cell::new(None, 3),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Water { solute: 0 }), 3),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
            ],
            *g.get_cells()
        );

        for _ in 0..6 {
            g.update_grid();
            assert_eq!(
                vec![
                    Cell::new(None, 1),
                    Cell::new(None, 2),
                    Cell::new(None, 3),
                    Cell::new(Some(Particle::Salt), 0),
                    Cell::new(Some(Particle::Water { solute: 0 }), 3),
                    Cell::new(Some(Particle::Salt), 0),
                    Cell::new(Some(Particle::Salt), 0),
                    Cell::new(Some(Particle::Salt), 0),
                    Cell::new(Some(Particle::Salt), 0),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_update_grid_salt_sink_in_water_when_capacity_is_zero() {
        let mut g = Grid::new(1, 5);

        g.spawn_particle(0, 0, Particle::Salt);
        g.spawn_particle(0, 1, Particle::Salt);
        g.spawn_particle(0, 2, Particle::Salt);
        g.spawn_particle(0, 3, Particle::Salt);
        g.spawn_particle(0, 4, Particle::new_water());

        assert_eq!(
            vec![
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Water { solute: 3 }), 0),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(None, 1),
                Cell::new(Some(Particle::Salt), 1),
                Cell::new(Some(Particle::Salt), 1),
                Cell::new(Some(Particle::Salt), 1),
                Cell::new(Some(Particle::Water { solute: 2 }), 1),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(None, 1),
                Cell::new(None, 2),
                Cell::new(Some(Particle::Salt), 2),
                Cell::new(Some(Particle::Salt), 2),
                Cell::new(Some(Particle::Water { solute: 1 }), 2),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(None, 1),
                Cell::new(None, 2),
                Cell::new(None, 3),
                Cell::new(Some(Particle::Salt), 3),
                Cell::new(Some(Particle::Water { solute: 0 }), 3),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(None, 1),
                Cell::new(None, 2),
                Cell::new(None, 3),
                Cell::new(Some(Particle::Water { solute: 0 }), 4),
                Cell::new(Some(Particle::Salt), 4),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_salt_should_sink_to_bottom_in_water() {
        /*
         * -S- -> -w-
         * SwS    SSS
         */

        let mut g = Grid::new(3, 2);

        g.spawn_particle(1, 0, Particle::Salt);
        g.spawn_particle(0, 1, Particle::Salt);
        g.spawn_particle(1, 1, Particle::Water { solute: 0 });
        g.spawn_particle(2, 1, Particle::Salt);

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(None, 0),
                Cell::new(Some(Particle::Water { solute: 0 }), 1),
                Cell::new(None, 0),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 1),
                Cell::new(Some(Particle::Salt), 0),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_salt_should_sink_to_bottom_left_in_water() {
        /*
         * -S- -> -w-
         * wSS    SSS
         */
        let mut g = Grid::new(3, 2);

        g.spawn_particle(1, 0, Particle::Salt);
        g.spawn_particle(0, 1, Particle::Water { solute: 0 });
        g.spawn_particle(1, 1, Particle::Salt);
        g.spawn_particle(2, 1, Particle::Salt);

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(None, 0),
                Cell::new(Some(Particle::Water { solute: 0 }), 1),
                Cell::new(None, 0),
                Cell::new(Some(Particle::Salt), 1),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_salt_should_sink_to_bottom_right_in_water() {
        /*
         * -S- -> -w-
         * SSw    SSS
         */
        let mut g = Grid::new(3, 2);

        g.spawn_particle(1, 0, Particle::Salt);
        g.spawn_particle(0, 1, Particle::Salt);
        g.spawn_particle(1, 1, Particle::Salt);
        g.spawn_particle(2, 1, Particle::Water { solute: 0 });

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(None, 0),
                Cell::new(Some(Particle::Water { solute: 0 }), 1),
                Cell::new(None, 0),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 1),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_salt_should_sink_in_water_but_water_should_not_climb_salts() {
        /*
         * S -> S -> w
         * S    w    S
         * w    S    S
         */
        let mut g = Grid::new_with_rand(1, 3, Some(|| ParticleHorizontalDirection::Right), None);

        g.spawn_particle(0, 0, Particle::Salt);
        g.spawn_particle(0, 1, Particle::Salt);
        g.spawn_particle(0, 2, Particle::Water { solute: 0 });

        assert_eq!(
            vec![
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Water { solute: 0 }), 0),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Some(Particle::Salt), 0),
                Cell::new(Some(Particle::Water { solute: 0 }), 1),
                Cell::new(Some(Particle::Salt), 1),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Some(Particle::Water { solute: 0 }), 2),
                Cell::new(Some(Particle::Salt), 2),
                Cell::new(Some(Particle::Salt), 1),
            ],
            *g.get_cells()
        );
    }
}
