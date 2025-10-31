use super::particle::{self, Particle};
use crate::component::grid::{GridAccess, ParticleHorizontalDirection};

#[derive(Clone, PartialEq, Debug)]
pub struct Water {
    pub weight: u8,
    pub solute: u8,
}

impl Default for Water {
    fn default() -> Self {
        Self::new()
    }
}

impl Water {
    pub fn new() -> Self {
        Self::new_with_solute(3)
    }

    pub fn new_with_solute(solute: u8) -> Self {
        Self { weight: 1, solute }
    }
}

impl particle::Updatable for Water {
    fn update<T: GridAccess>(&self, grid: &mut T, position: (usize, usize)) {
        if dissolve_salt(grid, self.solute, position) {
            return;
        }

        if particle::gravity(grid, position) {
            return;
        }

        slide_water(grid, position);
    }
}

fn dissolve_salt<T: GridAccess>(grid: &mut T, solute: u8, position: (usize, usize)) -> bool {
    for y in -1..=1 {
        for x in -1..=1 {
            if let Ok(i) = grid.get_neighbor_index(position, (x, y)) {
                if let Some(Particle::Salt(..)) = grid.get_cell(i).particle {
                    if 0 < solute {
                        let index = grid.to_index(position);
                        grid.get_cell_mut(index).particle = Some({
                            let solute = solute - 1;
                            Particle::from(Water::new_with_solute(solute))
                        });
                        grid.dissolve_particles(index, i);
                        return true;
                    }
                }
            };
        }
    }
    false
}

fn slide_water<T: GridAccess>(grid: &mut T, position: (usize, usize)) -> bool {
    let index_left = match grid.get_neighbor_index(position, (-1, 0)) {
        Ok(i) => {
            let c = grid.get_cell(i);
            match &c.particle {
                Some(_) => None,
                None => match grid.get_neighbor_index(position, (-2, 0)) {
                    Ok(ii) => {
                        let c = grid.get_cell(ii);
                        match &c.particle {
                            Some(_) => Some(i),
                            None => Some(ii),
                        }
                    }
                    Err(_) => Some(i),
                },
            }
        }
        Err(_) => None,
    };

    let index_right = match grid.get_neighbor_index(position, (1, 0)) {
        Ok(i) => {
            let c = grid.get_cell(i);
            match &c.particle {
                Some(_) => None,
                None => match grid.get_neighbor_index(position, (2, 0)) {
                    Ok(ii) => {
                        let c = grid.get_cell(ii);
                        match &c.particle {
                            Some(_) => Some(i),
                            None => Some(ii),
                        }
                    }
                    Err(_) => Some(i),
                },
            }
        }
        Err(_) => None,
    };

    let index = match (index_left, index_right) {
        (None, None) => None,
        (None, Some(i)) => Some(i),
        (Some(i), None) => Some(i),
        (Some(l), Some(r)) => match grid.particle_direction() {
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
        particles::{particle::Particle, salt::Salt, sand::Sand},
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
        g.spawn_particle((0, 0), Particle::from(Water::new()));

        g.update_grid();
        assert_eq!(Cell::empty().with_cycle(1), *g.get_cell(0));
        assert_eq!(
            Cell::new(Particle::from(Water::new())).with_cycle(1),
            *g.get_cell(1)
        );
        assert_eq!(Cell::empty(), *g.get_cell(2));

        g.update_grid();
        assert_eq!(Cell::empty().with_cycle(1), *g.get_cell(0));
        assert_eq!(Cell::empty().with_cycle(2), *g.get_cell(1));
        assert_eq!(
            Cell::new(Particle::from(Water::new())).with_cycle(2),
            *g.get_cell(2)
        );
    }

    #[test]
    fn test_update_grid_water_moves_right_when_bottom_cell_and_left_are_full() {
        /*
         * --- -> ---
         * sw-    s-w
         */
        let mut g = Grid::new(3, 2);
        g.spawn_particle((0, 1), Particle::from(Sand::new()));
        g.spawn_particle((1, 1), Particle::from(Water::new()));

        g.update_grid();

        assert_eq!(Cell::empty(), *g.get_cell(0));
        assert_eq!(Cell::empty(), *g.get_cell(1));
        assert_eq!(Cell::empty(), *g.get_cell(2));
        assert_eq!(Cell::new(Particle::from(Sand::new())), *g.get_cell(3));
        assert_eq!(Cell::empty().with_cycle(1), *g.get_cell(4));
        assert_eq!(
            Cell::new(Particle::from(Water::new())).with_cycle(1),
            *g.get_cell(5)
        );
    }

    #[test]
    fn test_update_grid_water_moves_left_when_bottom_cell_and_right_are_full() {
        /*
         * --- -> ---
         * -ws    w-s
         */
        let mut g = Grid::new(3, 2);
        g.spawn_particle((1, 1), Particle::from(Water::new()));
        g.spawn_particle((2, 1), Particle::from(Sand::new()));

        g.update_grid();

        assert_eq!(Cell::empty(), *g.get_cell(0));
        assert_eq!(Cell::empty(), *g.get_cell(1));
        assert_eq!(Cell::empty(), *g.get_cell(2));
        assert_eq!(
            Cell::new(Particle::from(Water::new())).with_cycle(1),
            *g.get_cell(3)
        );
        assert_eq!(Cell::empty().with_cycle(1), *g.get_cell(4));
        assert_eq!(Cell::new(Particle::from(Sand::new())), *g.get_cell(5));
    }

    #[test]
    fn test_update_grid_water_moves_left_or_right_when_both_right_and_left_are_empty_forced_right()
    {
        /*
         * --- -> ---
         * -w-    --w
         */
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Right), None);

        g.spawn_particle((1, 1), Particle::from(Water::new()));

        g.update_grid();

        assert_eq!(Cell::empty(), *g.get_cell(0));
        assert_eq!(Cell::empty(), *g.get_cell(1));
        assert_eq!(Cell::empty(), *g.get_cell(2));
        assert_eq!(Cell::empty(), *g.get_cell(3));
        assert_eq!(Cell::empty().with_cycle(1), *g.get_cell(4));
        assert_eq!(
            Cell::new(Particle::from(Water::new())).with_cycle(1),
            *g.get_cell(5)
        );
    }

    #[test]
    fn test_update_grid_water_moves_left_or_right_when_both_right_and_left_are_empty_forced_left() {
        /*
         * --- -> ---
         * -w-    w--
         */
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Left), None);

        g.spawn_particle((1, 1), Particle::from(Water::new()));

        g.update_grid();

        assert_eq!(Cell::empty(), *g.get_cell(0));
        assert_eq!(Cell::empty(), *g.get_cell(1));
        assert_eq!(Cell::empty(), *g.get_cell(2));
        assert_eq!(
            Cell::new(Particle::from(Water::new())).with_cycle(1),
            *g.get_cell(3)
        );
        assert_eq!(Cell::empty().with_cycle(1), *g.get_cell(4));
        assert_eq!(Cell::empty(), *g.get_cell(5));
    }

    #[test]
    fn test_water_can_slide_two_cells_to_right() {
        /*
         * --- -> ---
         * w--    --w
         */
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Right), None);

        g.spawn_particle((0, 1), Particle::from(Water::new()));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty().with_cycle(1),
                Cell::empty(),
                Cell::new(Particle::from(Water::new())).with_cycle(1),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_water_can_slide_two_cells_to_left() {
        /*
         * --- -> ---
         * --w    w--
         */
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Left), None);

        g.spawn_particle((2, 1), Particle::from(Water::new()));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::new(Particle::from(Water::new())).with_cycle(1),
                Cell::empty(),
                Cell::empty().with_cycle(1),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_update_grid_water_falls_to_bottom_right_when_bottom_cell_and_bottom_left_are_full_and_bottom_right_is_empty()
     {
        /*
         * w- -> --
         * s-    sw
         */
        let mut g = Grid::new(2, 2);

        g.spawn_particle((0, 0), Particle::from(Water::new()));
        g.spawn_particle((0, 1), Particle::from(Sand::new()));

        g.update_grid();

        assert_eq!(Cell::empty().with_cycle(1), *g.get_cell(0));
        assert_eq!(Cell::empty(), *g.get_cell(1));
        assert_eq!(Cell::new(Particle::from(Sand::new())), *g.get_cell(2));
        assert_eq!(
            Cell::new(Particle::from(Water::new())).with_cycle(1),
            *g.get_cell(3)
        );
    }

    #[test]
    fn test_update_grid_water_falls_bottom_left_when_bottom_cell_and_bottom_right_are_full_and_bottom_left_is_empty()
     {
        /*
         * -w -> --
         * -s    ws
         */
        let mut g = Grid::new(2, 2);

        g.spawn_particle((1, 0), Particle::from(Water::new()));
        g.spawn_particle((1, 1), Particle::from(Sand::new()));

        g.update_grid();

        assert_eq!(Cell::empty(), *g.get_cell(0));
        assert_eq!(Cell::empty().with_cycle(1), *g.get_cell(1));
        assert_eq!(
            Cell::new(Particle::from(Water::new())).with_cycle(1),
            *g.get_cell(2)
        );
        assert_eq!(Cell::new(Particle::from(Sand::new())), *g.get_cell(3));
    }

    #[test]
    fn test_update_grid_water_falls_bottom_left_or_bottom_right_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_forced_left()
     {
        /*
         * -w- -> ---
         * -s-    ws-
         */
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Left), None);

        g.spawn_particle((1, 0), Particle::from(Water::new()));
        g.spawn_particle((1, 1), Particle::from(Sand::new()));

        g.update_grid();

        assert_eq!(Cell::empty(), *g.get_cell(0));
        assert_eq!(Cell::empty().with_cycle(1), *g.get_cell(1));
        assert_eq!(Cell::empty(), *g.get_cell(2));
        assert_eq!(
            Cell::new(Particle::from(Water::new())).with_cycle(1),
            *g.get_cell(3)
        );
        assert_eq!(Cell::new(Particle::from(Sand::new())), *g.get_cell(4));
        assert_eq!(Cell::empty(), *g.get_cell(5));
    }

    #[test]
    fn test_update_grid_water_falls_bottom_left_or_bottom_right_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_forced_right()
     {
        /*
         * -w- -> ---
         * -s-    -sw
         */
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Right), None);

        g.spawn_particle((1, 0), Particle::from(Water::new()));
        g.spawn_particle((1, 1), Particle::from(Sand::new()));

        g.update_grid();

        assert_eq!(Cell::empty(), *g.get_cell(0));
        assert_eq!(Cell::empty().with_cycle(1), *g.get_cell(1));
        assert_eq!(Cell::empty(), *g.get_cell(2));
        assert_eq!(Cell::empty(), *g.get_cell(3));
        assert_eq!(Cell::new(Particle::from(Sand::new())), *g.get_cell(4));
        assert_eq!(
            Cell::new(Particle::from(Water::new())).with_cycle(1),
            *g.get_cell(5)
        );
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

        g.spawn_particle((1, 0), Particle::from(Water::new()));
        g.spawn_particle((2, 0), Particle::from(Water::new()));

        g.update_grid();

        assert_eq!(
            Cell::new(Particle::from(Water::new())).with_cycle(1),
            *g.get_cell(0)
        );
        assert_eq!(
            Cell::new(Particle::from(Water::new())).with_cycle(1),
            *g.get_cell(1)
        );
        assert_eq!(Cell::empty().with_cycle(1), *g.get_cell(2));
        assert_eq!(Cell::empty(), *g.get_cell(3));

        let mut g = Grid::new_with_rand(
            4,
            1,
            Some(|| ParticleHorizontalDirection::Right),
            Some(|| RowUpdateDirection::Forward),
        );

        g.spawn_particle((1, 0), Particle::from(Water::new()));
        g.spawn_particle((2, 0), Particle::from(Water::new()));

        g.update_grid();

        assert_eq!(
            Cell::new(Particle::from(Water::new())).with_cycle(1),
            *g.get_cell(0)
        );
        assert_eq!(Cell::empty().with_cycle(1), *g.get_cell(1));
        assert_eq!(Cell::empty().with_cycle(1), *g.get_cell(2));
        assert_eq!(
            Cell::new(Particle::from(Water::new())).with_cycle(1),
            *g.get_cell(3)
        );
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

        g.spawn_particle((1, 0), Particle::from(Water::new()));
        g.spawn_particle((2, 0), Particle::from(Water::new()));

        g.update_grid();

        assert_eq!(Cell::empty(), *g.get_cell(0));
        assert_eq!(Cell::empty().with_cycle(1), *g.get_cell(1));
        assert_eq!(
            Cell::new(Particle::from(Water::new())).with_cycle(1),
            *g.get_cell(2)
        );
        assert_eq!(
            Cell::new(Particle::from(Water::new())).with_cycle(1),
            *g.get_cell(3)
        );

        let mut g = Grid::new_with_rand(
            4,
            1,
            Some(|| ParticleHorizontalDirection::Left),
            Some(|| RowUpdateDirection::Reverse),
        );

        g.spawn_particle((1, 0), Particle::from(Water::new()));
        g.spawn_particle((2, 0), Particle::from(Water::new()));

        g.update_grid();

        assert_eq!(
            Cell::new(Particle::from(Water::new())).with_cycle(1),
            *g.get_cell(0)
        );
        assert_eq!(Cell::empty().with_cycle(1), *g.get_cell(1));
        assert_eq!(Cell::empty().with_cycle(1), *g.get_cell(2));
        assert_eq!(
            Cell::new(Particle::from(Water::new())).with_cycle(1),
            *g.get_cell(3)
        );
    }

    #[test]
    fn test_sand_should_sink_to_bottom_in_water() {
        /*
         * -s- -> -w-
         * sws    sss
         */

        let mut g = Grid::new(3, 2);

        g.spawn_particle((1, 0), Particle::from(Sand::new()));
        g.spawn_particle((0, 1), Particle::from(Sand::new()));
        g.spawn_particle((1, 1), Particle::from(Water::new()));
        g.spawn_particle((2, 1), Particle::from(Sand::new()));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty(),
                Cell::new(Particle::from(Water::new())).with_cycle(1),
                Cell::empty(),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())).with_cycle(1),
                Cell::new(Particle::from(Sand::new())),
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

        g.spawn_particle((1, 0), Particle::from(Sand::new()));
        g.spawn_particle((0, 1), Particle::from(Water::new()));
        g.spawn_particle((1, 1), Particle::from(Sand::new()));
        g.spawn_particle((2, 1), Particle::from(Sand::new()));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty(),
                Cell::new(Particle::from(Water::new())).with_cycle(1),
                Cell::empty(),
                Cell::new(Particle::from(Sand::new())).with_cycle(1),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())),
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

        g.spawn_particle((1, 0), Particle::from(Sand::new()));
        g.spawn_particle((0, 1), Particle::from(Sand::new()));
        g.spawn_particle((1, 1), Particle::from(Sand::new()));
        g.spawn_particle((2, 1), Particle::from(Water::new()));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty(),
                Cell::new(Particle::from(Water::new())).with_cycle(1),
                Cell::empty(),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())).with_cycle(1),
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

        g.spawn_particle((0, 0), Particle::from(Sand::new()));
        g.spawn_particle((0, 1), Particle::from(Sand::new()));
        g.spawn_particle((0, 2), Particle::from(Water::new()));

        assert_eq!(
            vec![
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Water::new())),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Water::new())).with_cycle(1),
                Cell::new(Particle::from(Sand::new())).with_cycle(1),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Water::new())).with_cycle(2),
                Cell::new(Particle::from(Sand::new())).with_cycle(2),
                Cell::new(Particle::from(Sand::new())).with_cycle(1),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_allow_sand_to_sink_diagonally_even_if_the_destination_cell_is_simulated() {
        /*
         * sss -> wws
         * sss -> wss
         * www    sss
         * www    www
         */
        let mut g = Grid::new_with_rand(3, 4, Some(|| ParticleHorizontalDirection::Right), None);

        g.spawn_particle((0, 0), Particle::from(Sand::new()));
        g.spawn_particle((1, 0), Particle::from(Sand::new()));
        g.spawn_particle((2, 0), Particle::from(Sand::new()));

        g.spawn_particle((0, 1), Particle::from(Sand::new()));
        g.spawn_particle((1, 1), Particle::from(Sand::new()));
        g.spawn_particle((2, 1), Particle::from(Sand::new()));

        g.spawn_particle((0, 2), Particle::from(Water::new()));
        g.spawn_particle((1, 2), Particle::from(Water::new()));
        g.spawn_particle((2, 2), Particle::from(Water::new()));

        g.spawn_particle((0, 3), Particle::from(Water::new()));
        g.spawn_particle((1, 3), Particle::from(Water::new()));
        g.spawn_particle((2, 3), Particle::from(Water::new()));

        g.update_grid();
        assert_eq!(
            vec![
                Cell::new(Particle::from(Water::new())).with_cycle(1),
                Cell::new(Particle::from(Water::new())).with_cycle(1),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Water::new())).with_cycle(1),
                Cell::new(Particle::from(Sand::new())).with_cycle(1),
                Cell::new(Particle::from(Sand::new())).with_cycle(1),
                Cell::new(Particle::from(Sand::new())).with_cycle(1),
                Cell::new(Particle::from(Sand::new())).with_cycle(1),
                Cell::new(Particle::from(Sand::new())).with_cycle(1),
                Cell::new(Particle::from(Water::new())),
                Cell::new(Particle::from(Water::new())),
                Cell::new(Particle::from(Water::new())),
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
        g.spawn_particle((0, 0), Particle::from(Sand::new()));
        g.spawn_particle((0, 1), Particle::from(Water::new()));
        g.spawn_particle((0, 2), Particle::from(Salt::new()));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Water::new_with_solute(2))).with_cycle(1),
                Cell::empty().with_cycle(1),
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
                g.spawn_particle((1, 1), Particle::from(Water::new()));
                g.spawn_particle((x, y), Particle::from(Salt::new()));

                g.update_grid();

                assert_eq!(
                    Some(Particle::from(Water::new_with_solute(2))),
                    g.get_cell(4).particle
                );

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
        let mut g = Grid::new(1, 9);
        g.spawn_particle((0, 8), Particle::from(Water::new()));
        for y in 0..8 {
            g.spawn_particle((0, y), Particle::from(Salt::new()));
        }

        assert_eq!(
            vec![
                Cell::new(Particle::from(Salt::new())),
                Cell::new(Particle::from(Salt::new())),
                Cell::new(Particle::from(Salt::new())),
                Cell::new(Particle::from(Salt::new())),
                Cell::new(Particle::from(Salt::new())),
                Cell::new(Particle::from(Salt::new())),
                Cell::new(Particle::from(Salt::new())),
                Cell::new(Particle::from(Salt::new())),
                Cell::new(Particle::from(Water::new_with_solute(3))),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty().with_cycle(1),
                Cell::new(Particle::from(Salt::new())).with_cycle(1),
                Cell::new(Particle::from(Salt::new())).with_cycle(1),
                Cell::new(Particle::from(Salt::new())).with_cycle(1),
                Cell::new(Particle::from(Salt::new())).with_cycle(1),
                Cell::new(Particle::from(Salt::new())).with_cycle(1),
                Cell::new(Particle::from(Salt::new())).with_cycle(1),
                Cell::new(Particle::from(Salt::new())).with_cycle(1),
                Cell::new(Particle::from(Water::new_with_solute(2))).with_cycle(1),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty().with_cycle(1),
                Cell::empty().with_cycle(2),
                Cell::new(Particle::from(Salt::new())).with_cycle(2),
                Cell::new(Particle::from(Salt::new())).with_cycle(2),
                Cell::new(Particle::from(Salt::new())).with_cycle(2),
                Cell::new(Particle::from(Salt::new())).with_cycle(2),
                Cell::new(Particle::from(Salt::new())).with_cycle(2),
                Cell::new(Particle::from(Salt::new())).with_cycle(2),
                Cell::new(Particle::from(Water::new_with_solute(1))).with_cycle(2),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty().with_cycle(1),
                Cell::empty().with_cycle(2),
                Cell::empty().with_cycle(3),
                Cell::new(Particle::from(Salt::new())).with_cycle(3),
                Cell::new(Particle::from(Salt::new())).with_cycle(3),
                Cell::new(Particle::from(Salt::new())).with_cycle(3),
                Cell::new(Particle::from(Salt::new())).with_cycle(3),
                Cell::new(Particle::from(Salt::new())).with_cycle(3),
                Cell::new(Particle::from(Water::new_with_solute(0))).with_cycle(3),
            ],
            *g.get_cells()
        );

        for _ in 0..6 {
            g.update_grid();
        }

        assert_eq!(
            vec![
                Cell::empty().with_cycle(1),
                Cell::empty().with_cycle(2),
                Cell::empty().with_cycle(3),
                Cell::new(Particle::from(Water::new_with_solute(0))).with_cycle(8),
                Cell::new(Particle::from(Salt::new())).with_cycle(8),
                Cell::new(Particle::from(Salt::new())).with_cycle(7),
                Cell::new(Particle::from(Salt::new())).with_cycle(6),
                Cell::new(Particle::from(Salt::new())).with_cycle(5),
                Cell::new(Particle::from(Salt::new())).with_cycle(4),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_update_grid_salt_sink_in_water_when_capacity_is_zero() {
        let mut g = Grid::new(1, 5);

        g.spawn_particle((0, 0), Particle::from(Salt::new()));
        g.spawn_particle((0, 1), Particle::from(Salt::new()));
        g.spawn_particle((0, 2), Particle::from(Salt::new()));
        g.spawn_particle((0, 3), Particle::from(Salt::new()));
        g.spawn_particle((0, 4), Particle::from(Water::new()));

        assert_eq!(
            vec![
                Cell::new(Particle::from(Salt::new())),
                Cell::new(Particle::from(Salt::new())),
                Cell::new(Particle::from(Salt::new())),
                Cell::new(Particle::from(Salt::new())),
                Cell::new(Particle::from(Water::new_with_solute(3))),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty().with_cycle(1),
                Cell::new(Particle::from(Salt::new())).with_cycle(1),
                Cell::new(Particle::from(Salt::new())).with_cycle(1),
                Cell::new(Particle::from(Salt::new())).with_cycle(1),
                Cell::new(Particle::from(Water::new_with_solute(2))).with_cycle(1),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty().with_cycle(1),
                Cell::empty().with_cycle(2),
                Cell::new(Particle::from(Salt::new())).with_cycle(2),
                Cell::new(Particle::from(Salt::new())).with_cycle(2),
                Cell::new(Particle::from(Water::new_with_solute(1))).with_cycle(2),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty().with_cycle(1),
                Cell::empty().with_cycle(2),
                Cell::empty().with_cycle(3),
                Cell::new(Particle::from(Salt::new())).with_cycle(3),
                Cell::new(Particle::from(Water::new_with_solute(0))).with_cycle(3),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty().with_cycle(1),
                Cell::empty().with_cycle(2),
                Cell::empty().with_cycle(3),
                Cell::new(Particle::from(Water::new_with_solute(0))).with_cycle(4),
                Cell::new(Particle::from(Salt::new())).with_cycle(4),
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

        g.spawn_particle((1, 0), Particle::from(Salt::new()));
        g.spawn_particle((0, 1), Particle::from(Salt::new()));
        g.spawn_particle((1, 1), Particle::from(Water::new_with_solute(0)));
        g.spawn_particle((2, 1), Particle::from(Salt::new()));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty(),
                Cell::new(Particle::from(Water::new_with_solute(0))).with_cycle(1),
                Cell::empty(),
                Cell::new(Particle::from(Salt::new())),
                Cell::new(Particle::from(Salt::new())).with_cycle(1),
                Cell::new(Particle::from(Salt::new())),
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

        g.spawn_particle((1, 0), Particle::from(Salt::new()));
        g.spawn_particle((0, 1), Particle::from(Water::new_with_solute(0)));
        g.spawn_particle((1, 1), Particle::from(Salt::new()));
        g.spawn_particle((2, 1), Particle::from(Salt::new()));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty(),
                Cell::new(Particle::from(Water::new_with_solute(0))).with_cycle(1),
                Cell::empty(),
                Cell::new(Particle::from(Salt::new())).with_cycle(1),
                Cell::new(Particle::from(Salt::new())),
                Cell::new(Particle::from(Salt::new())),
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

        g.spawn_particle((1, 0), Particle::from(Salt::new()));
        g.spawn_particle((0, 1), Particle::from(Salt::new()));
        g.spawn_particle((1, 1), Particle::from(Salt::new()));
        g.spawn_particle((2, 1), Particle::from(Water::new_with_solute(0)));

        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty(),
                Cell::new(Particle::from(Water::new_with_solute(0))).with_cycle(1),
                Cell::empty(),
                Cell::new(Particle::from(Salt::new())),
                Cell::new(Particle::from(Salt::new())),
                Cell::new(Particle::from(Salt::new())).with_cycle(1),
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

        g.spawn_particle((0, 0), Particle::from(Salt::new()));
        g.spawn_particle((0, 1), Particle::from(Salt::new()));
        g.spawn_particle((0, 2), Particle::from(Water::new_with_solute(0)));

        assert_eq!(
            vec![
                Cell::new(Particle::from(Salt::new())),
                Cell::new(Particle::from(Salt::new())),
                Cell::new(Particle::from(Water::new_with_solute(0))),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Salt::new())),
                Cell::new(Particle::from(Water::new_with_solute(0))).with_cycle(1),
                Cell::new(Particle::from(Salt::new())).with_cycle(1),
            ],
            *g.get_cells()
        );

        g.update_grid();

        assert_eq!(
            vec![
                Cell::new(Particle::from(Water::new_with_solute(0))).with_cycle(2),
                Cell::new(Particle::from(Salt::new())).with_cycle(2),
                Cell::new(Particle::from(Salt::new())).with_cycle(1),
            ],
            *g.get_cells()
        );
    }
}
