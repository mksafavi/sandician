use super::particle::Particle;
use crate::component::grid::{GridAccess, ParticleHorizontalDirection};

pub fn update_water<T: GridAccess>(grid: &mut T, solute: u8, position: (usize, usize)) {
    let _index_top_left = match grid.get_neighbor_index(position, (-1, -1)) {
        Ok(i) => match grid.get_cell(i) {
            Some(c) => {
                if 0 < solute {
                    if c.particle == Particle::Salt {
                        let index = grid.to_index(position);
                        if let Some(c) = grid.get_cell_mut(index) {
                            c.particle = Particle::Water { solute: solute - 1 }
                        }
                        grid.dissolve_particles(index, i);
                        return;
                    }
                }
                None
            }
            None => Some(i),
        },
        Err(_) => None,
    };

    let _index_top = match grid.get_neighbor_index(position, (0, -1)) {
        Ok(i) => match grid.get_cell(i) {
            Some(c) => {
                if 0 < solute {
                    if c.particle == Particle::Salt {
                        let index = grid.to_index(position);
                        if let Some(c) = grid.get_cell_mut(index) {
                            c.particle = Particle::Water { solute: solute - 1 }
                        }
                        grid.dissolve_particles(index, i);
                        return;
                    }
                }
                None
            }
            None => Some(i),
        },
        Err(_) => None,
    };

    let _index_top_right = match grid.get_neighbor_index(position, (1, -1)) {
        Ok(i) => match grid.get_cell(i) {
            Some(c) => {
                if 0 < solute {
                    if c.particle == Particle::Salt {
                        let index = grid.to_index(position);
                        if let Some(c) = grid.get_cell_mut(index) {
                            c.particle = Particle::Water { solute: solute - 1 }
                        }
                        grid.dissolve_particles(index, i);
                        return;
                    }
                }
                None
            }
            None => Some(i),
        },
        Err(_) => None,
    };

    let index_left = match grid.get_neighbor_index(position, (-1, 0)) {
        Ok(i) => match grid.get_cell(i) {
            Some(c) => {
                if 0 < solute {
                    if c.particle == Particle::Salt {
                        let index = grid.to_index(position);
                        if let Some(c) = grid.get_cell_mut(index) {
                            c.particle = Particle::Water { solute: solute - 1 }
                        }
                        grid.dissolve_particles(index, i);
                        return;
                    }
                }
                None
            }
            None => Some(i),
        },
        Err(_) => None,
    };

    let index_right = match grid.get_neighbor_index(position, (1, 0)) {
        Ok(i) => match grid.get_cell(i) {
            Some(c) => {
                if 0 < solute {
                    if c.particle == Particle::Salt {
                        let index = grid.to_index(position);
                        if let Some(c) = grid.get_cell_mut(index) {
                            c.particle = Particle::Water { solute: solute - 1 }
                        }
                        grid.dissolve_particles(index, i);
                        return;
                    }
                }
                None
            }
            None => Some(i),
        },
        Err(_) => None,
    };

    let index_bottom_left = match grid.get_neighbor_index(position, (-1, 1)) {
        Ok(i) => match grid.get_cell(i) {
            Some(c) => {
                if 0 < solute {
                    if c.particle == Particle::Salt {
                        let index = grid.to_index(position);
                        if let Some(c) = grid.get_cell_mut(index) {
                            c.particle = Particle::Water { solute: solute - 1 }
                        }
                        grid.dissolve_particles(index, i);
                        return;
                    }
                }
                None
            }
            None => Some(i),
        },
        Err(_) => None,
    };

    let index_bottom = match grid.get_neighbor_index(position, (0, 1)) {
        Ok(i) => match grid.get_cell(i) {
            Some(c) => {
                if 0 < solute {
                    if c.particle == Particle::Salt {
                        let index = grid.to_index(position);
                        if let Some(c) = grid.get_cell_mut(index) {
                            c.particle = Particle::Water { solute: solute - 1 }
                        }
                        grid.dissolve_particles(index, i);
                        return;
                    }
                }
                None
            }
            None => Some(i),
        },
        Err(_) => None,
    };

    let index_bottom_right = match grid.get_neighbor_index(position, (1, 1)) {
        Ok(i) => match grid.get_cell(i) {
            Some(c) => {
                if 0 < solute {
                    if c.particle == Particle::Salt {
                        let index = grid.to_index(position);
                        if let Some(c) = grid.get_cell_mut(index) {
                            c.particle = Particle::Water { solute: solute - 1 }
                        }
                        grid.dissolve_particles(index, i);
                        return;
                    }
                }
                None
            }
            None => Some(i),
        },
        Err(_) => None,
    };

    let index = match (
        index_left,
        index_bottom_left,
        index_bottom,
        index_bottom_right,
        index_right,
    ) {
        (None, None, None, None, None) => None,
        (_, _, Some(i), _, _) => Some(i),
        (_, None, None, Some(i), _) => Some(i),
        (_, Some(i), None, None, _) => Some(i),
        (_, Some(l), None, Some(r), _) => match grid.water_direction() {
            ParticleHorizontalDirection::Left => Some(l),
            ParticleHorizontalDirection::Right => Some(r),
        },
        (None, None, None, None, Some(i)) => Some(i),
        (Some(i), None, None, None, None) => Some(i),
        (Some(l), None, None, None, Some(r)) => match grid.water_direction() {
            ParticleHorizontalDirection::Left => Some(l),
            ParticleHorizontalDirection::Right => Some(r),
        },
    };

    if let Some(index) = index {
        grid.swap_particles(grid.to_index(position), index);
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
        assert_eq!(None, *g.get_cell(0));
        assert_eq!(Some(Cell::new(Particle::new_water())), *g.get_cell(1));
        assert_eq!(None, *g.get_cell(2));

        g.update_grid();
        assert_eq!(None, *g.get_cell(0));
        assert_eq!(None, *g.get_cell(1));
        assert_eq!(Some(Cell::new(Particle::new_water())), *g.get_cell(2));
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

        assert_eq!(None, *g.get_cell(0));
        assert_eq!(None, *g.get_cell(1));
        assert_eq!(None, *g.get_cell(2));
        assert_eq!(Some(Cell::new(Particle::Sand)), *g.get_cell(3));
        assert_eq!(None, *g.get_cell(4));
        assert_eq!(Some(Cell::new(Particle::new_water())), *g.get_cell(5));
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

        assert_eq!(None, *g.get_cell(0));
        assert_eq!(None, *g.get_cell(1));
        assert_eq!(None, *g.get_cell(2));
        assert_eq!(Some(Cell::new(Particle::new_water())), *g.get_cell(3));
        assert_eq!(None, *g.get_cell(4));
        assert_eq!(Some(Cell::new(Particle::Sand)), *g.get_cell(5));
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

        assert_eq!(None, *g.get_cell(0));
        assert_eq!(None, *g.get_cell(1));
        assert_eq!(None, *g.get_cell(2));
        assert_eq!(None, *g.get_cell(3));
        assert_eq!(None, *g.get_cell(4));
        assert_eq!(Some(Cell::new(Particle::new_water())), *g.get_cell(5));
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

        assert_eq!(None, *g.get_cell(0));
        assert_eq!(None, *g.get_cell(1));
        assert_eq!(None, *g.get_cell(2));
        assert_eq!(Some(Cell::new(Particle::new_water())), *g.get_cell(3));
        assert_eq!(None, *g.get_cell(4));
        assert_eq!(None, *g.get_cell(5));
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

        assert_eq!(None, *g.get_cell(0));
        assert_eq!(None, *g.get_cell(1));
        assert_eq!(Some(Cell::new(Particle::Sand)), *g.get_cell(2));
        assert_eq!(Some(Cell::new(Particle::new_water())), *g.get_cell(3));
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

        assert_eq!(None, *g.get_cell(0));
        assert_eq!(None, *g.get_cell(1));
        assert_eq!(Some(Cell::new(Particle::new_water())), *g.get_cell(2));
        assert_eq!(Some(Cell::new(Particle::Sand)), *g.get_cell(3));
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

        assert_eq!(None, *g.get_cell(0));
        assert_eq!(None, *g.get_cell(1));
        assert_eq!(None, *g.get_cell(2));
        assert_eq!(Some(Cell::new(Particle::new_water())), *g.get_cell(3));
        assert_eq!(Some(Cell::new(Particle::Sand)), *g.get_cell(4));
        assert_eq!(None, *g.get_cell(5));
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

        assert_eq!(None, *g.get_cell(0));
        assert_eq!(None, *g.get_cell(1));
        assert_eq!(None, *g.get_cell(2));
        assert_eq!(None, *g.get_cell(3));
        assert_eq!(Some(Cell::new(Particle::Sand)), *g.get_cell(4));
        assert_eq!(Some(Cell::new(Particle::new_water())), *g.get_cell(5));
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

        assert_eq!(Some(Cell::new(Particle::new_water())), *g.get_cell(0));
        assert_eq!(Some(Cell::new(Particle::new_water())), *g.get_cell(1));
        assert_eq!(None, *g.get_cell(2));
        assert_eq!(None, *g.get_cell(3));

        let mut g = Grid::new_with_rand(
            4,
            1,
            Some(|| ParticleHorizontalDirection::Right),
            Some(|| RowUpdateDirection::Forward),
        );

        g.spawn_particle(1, 0, Particle::new_water());
        g.spawn_particle(2, 0, Particle::new_water());

        g.update_grid();

        assert_eq!(Some(Cell::new(Particle::new_water())), *g.get_cell(0));
        assert_eq!(None, *g.get_cell(1));
        assert_eq!(None, *g.get_cell(2));
        assert_eq!(Some(Cell::new(Particle::new_water())), *g.get_cell(3));
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

        assert_eq!(None, *g.get_cell(0));
        assert_eq!(None, *g.get_cell(1));
        assert_eq!(Some(Cell::new(Particle::new_water())), *g.get_cell(2));
        assert_eq!(Some(Cell::new(Particle::new_water())), *g.get_cell(3));

        let mut g = Grid::new_with_rand(
            4,
            1,
            Some(|| ParticleHorizontalDirection::Left),
            Some(|| RowUpdateDirection::Reverse),
        );

        g.spawn_particle(1, 0, Particle::new_water());
        g.spawn_particle(2, 0, Particle::new_water());

        g.update_grid();

        assert_eq!(Some(Cell::new(Particle::new_water())), *g.get_cell(0));
        assert_eq!(None, *g.get_cell(1));
        assert_eq!(None, *g.get_cell(2));
        assert_eq!(Some(Cell::new(Particle::new_water())), *g.get_cell(3));
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
                None,
                Some(Cell::new(Particle::new_water())),
                None,
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::Sand)),
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
                None,
                Some(Cell::new(Particle::new_water())),
                None,
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::Sand)),
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
                None,
                Some(Cell::new(Particle::new_water())),
                None,
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::Sand)),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_sand_should_sink_in_water_but_water_should_not_climb_sands() {
        /*
         * -s- -> -s- -> -w-
         * -s-    -w-    -s-
         * -w-    -s-    -s-
         */
        let mut g = Grid::new_with_rand(1, 3, Some(|| ParticleHorizontalDirection::Right), None);

        g.spawn_particle(0, 0, Particle::Sand);
        g.spawn_particle(0, 1, Particle::Sand);
        g.spawn_particle(0, 2, Particle::new_water());

        assert_eq!(
            vec![
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::new_water())),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::new_water())),
                Some(Cell::new(Particle::Sand)),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Some(Cell::new(Particle::new_water())),
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::Sand)),
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
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::Water { solute: 2 },)),
                None
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_update_grid_water_dissolve_neighboring_salts() {
        /*
         * for each neighbor:
         * S-- -> ---
         * -w-    ww-
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

                assert_eq!(
                    Particle::Water { solute: 2 },
                    g.get_cell(4).clone().unwrap().particle
                );

                for y in 0..3 {
                    for x in 0..3 {
                        if (x, y) == (1, 1) {
                            continue;
                        }
                        assert_eq!(None, g.get_cell(g.to_index((x, y))).clone());
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
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Water { solute: 3 })),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                None,
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Water { solute: 2 })),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                None,
                None,
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Water { solute: 1 })),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                None,
                None,
                None,
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Water { solute: 0 })),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
            ],
            *g.get_cells()
        );

        for _ in 0..6 {
            g.update_grid();
            assert_eq!(
                vec![
                    None,
                    None,
                    None,
                    Some(Cell::new(Particle::Salt)),
                    Some(Cell::new(Particle::Water { solute: 0 })),
                    Some(Cell::new(Particle::Salt)),
                    Some(Cell::new(Particle::Salt)),
                    Some(Cell::new(Particle::Salt)),
                    Some(Cell::new(Particle::Salt)),
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
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Water { solute: 3 })),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                None,
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Water { solute: 2 })),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                None,
                None,
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Water { solute: 1 })),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                None,
                None,
                None,
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Water { solute: 0 })),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                None,
                None,
                None,
                Some(Cell::new(Particle::Water { solute: 0 })),
                Some(Cell::new(Particle::Salt)),
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
                None,
                Some(Cell::new(Particle::Water { solute: 0 })),
                None,
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
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
                None,
                Some(Cell::new(Particle::Water { solute: 0 })),
                None,
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
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
                None,
                Some(Cell::new(Particle::Water { solute: 0 })),
                None,
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_salt_should_sink_in_water_but_water_should_not_climb_salts() {
        /*
         * -S- -> -S- -> -w-
         * -S-    -w-    -S-
         * -w-    -S-    -S-
         */
        let mut g = Grid::new_with_rand(1, 3, Some(|| ParticleHorizontalDirection::Right), None);

        g.spawn_particle(0, 0, Particle::Salt);
        g.spawn_particle(0, 1, Particle::Salt);
        g.spawn_particle(0, 2, Particle::Water { solute: 0 });

        assert_eq!(
            vec![
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Water { solute: 0 })),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Water { solute: 0 })),
                Some(Cell::new(Particle::Salt)),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Some(Cell::new(Particle::Water { solute: 0 })),
                Some(Cell::new(Particle::Salt)),
                Some(Cell::new(Particle::Salt)),
            ],
            *g.get_cells()
        );
    }
}
