use bevy::{
    asset::RenderAssetUsages,
    color::{palettes::css, Color, ColorToPacked},
    ecs::component::Component,
    image::Image,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use rand::random_range;

use super::particles::Particle;

// TODO: remove all unwrap and expects

pub enum GridError {
    OutOfBound,
}

pub const BACKGROUND_COLOR: bevy::prelude::Color = Color::srgb(0.82, 0.93, 1.);

pub enum ParticleOperation {
    Swap(usize),
    Dissolve(Particle),
}

#[derive(Clone, Debug)]
pub enum ParticleHorizontalDirection {
    Left = -1,
    Right = 1,
}

enum RowUpdateDirection {
    Forward = 0,
    Reverse = 1,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Cell {
    pub particle: Particle,
    pub simulated: bool,
}

impl Cell {
    pub fn new(particle: Particle) -> Self {
        Self {
            particle,
            simulated: false,
        }
    }
}

#[derive(Component, Clone, PartialEq, Debug)]
pub struct Grid {
    cells: Vec<Option<Cell>>,
    width: usize,
    height: usize,
    water_direction: fn() -> ParticleHorizontalDirection,
    row_update_direction: fn() -> RowUpdateDirection,
}

pub trait GridAccess {
    fn water_direction(&self) -> ParticleHorizontalDirection;
    fn get_neighbor_index(
        &self,
        position: (usize, usize),
        offset: (i32, i32),
    ) -> Result<usize, GridError>;
    fn get_cell(&self, index: usize) -> &Option<Cell>;
}

impl GridAccess for Grid {
    fn get_cell(&self, index: usize) -> &Option<Cell> {
        &self.cells[index]
    }

    fn get_neighbor_index(
        &self,
        (x, y): (usize, usize),
        (ox, oy): (i32, i32),
    ) -> Result<usize, GridError> {
        let neighbor_index = (y as i32 + oy) * self.width as i32 + (x as i32 + ox);
        if (0 <= y as i32 + oy)
            && ((y as i32 + oy) < self.height as i32)
            && ((x as i32 + ox) < self.width as i32)
            && (0 <= x as i32 + ox)
        {
            Ok(neighbor_index as usize)
        } else {
            Err(GridError::OutOfBound)
        }
    }

    fn water_direction(&self) -> ParticleHorizontalDirection {
        (self.water_direction)()
    }
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        fn random_water_direction() -> ParticleHorizontalDirection {
            match random_range(0..=1) {
                0 => ParticleHorizontalDirection::Left,
                _ => ParticleHorizontalDirection::Right,
            }
        }
        fn random_row_update_direction() -> RowUpdateDirection {
            match random_range(0..=1) {
                0 => RowUpdateDirection::Forward,
                _ => RowUpdateDirection::Reverse,
            }
        }
        Self {
            cells: (0..width * height).map(|_| None).collect(),
            width,
            height,
            water_direction: random_water_direction,
            row_update_direction: random_row_update_direction,
        }
    }

    pub fn spawn_particle(&mut self, x: usize, y: usize, particle: Particle) {
        if y < self.height && x < self.width {
            let index = self.width * y + x;
            if self.cells[index].is_none() {
                self.cells[index] = Some(Cell::new(particle));
            }
        }
    }

    fn swap_particles(&mut self, index: usize, next_location_index: usize) {
        self.cells.swap(index, next_location_index);
        if let Some(p) = &mut self.cells[index] {
            p.simulated = true;
        }
        if let Some(p) = &mut self.cells[next_location_index] {
            p.simulated = true;
        }
    }

    fn disolve_particles(&mut self, index: usize, particle: Particle) {
        if let Some(p) = &mut self.cells[index] {
            p.simulated = true;
            p.particle = particle;
        }
    }

    fn clear_all_simulated_field(&mut self) {
        self.cells.iter_mut().for_each(|x| {
            if let Some(x) = x {
                x.simulated = false;
            }
        });
    }

    fn update_particles(&mut self, x: usize, y: usize) {
        let index = y * self.width + x;
        if let Some(p) = &self.cells[index] {
            if !p.simulated {
                let next_operation = p.particle.find_particle_next_location(self, x, y);
                if let Some(next_operation) = next_operation {
                    match next_operation {
                        ParticleOperation::Swap(next_location_index) => {
                            self.swap_particles(index, next_location_index);
                        }
                        ParticleOperation::Dissolve(particle) => {
                            self.disolve_particles(index, particle);
                        }
                    }
                }
            }
        }
    }

    pub fn update_grid(&mut self) {
        for y in (0..self.height).rev() {
            let x_direction = (self.row_update_direction)();
            for x in 0..self.width {
                let x = match x_direction {
                    RowUpdateDirection::Forward => x,
                    RowUpdateDirection::Reverse => self.width - 1 - x,
                };
                self.update_particles(x, y);
            }
        }
        self.clear_all_simulated_field();
    }

    pub fn create_output_frame(width: usize, height: usize) -> Image {
        Image::new_fill(
            Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &(css::BLACK.to_u8_array()),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
    }

    pub fn draw_grid(&self, image: &mut Image) {
        for (index, particle) in self.cells.iter().enumerate() {
            let x: u32 = index as u32 % self.width as u32;
            let y: u32 = (index as u32 - x) / self.width as u32;
            let _ = match particle {
                Some(p) => image.set_color_at(x, y, p.particle.color()),
                _ => image.set_color_at(x, y, BACKGROUND_COLOR),
            };
        }
    }

    pub fn spawn_brush(&mut self, (x, y): (usize, usize), size: usize, particle: Particle) {
        for j in 0..size {
            for i in 0..size {
                self.spawn_particle(x + i, y + j, particle.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::component::macros::assert_color_srgb_eq;

    use super::*;

    impl Grid {
        fn new_with_rand(
            width: usize,
            height: usize,
            water_direction: Option<fn() -> ParticleHorizontalDirection>,
            row_update_direction: Option<fn() -> RowUpdateDirection>,
        ) -> Self {
            let mut g = Self::new(width, height);
            g.water_direction = match water_direction {
                Some(f) => f,
                None => || ParticleHorizontalDirection::Right,
            };

            g.row_update_direction = match row_update_direction {
                Some(f) => f,
                None => || RowUpdateDirection::Forward,
            };
            g
        }
    }

    #[test]
    fn test_create_grid() {
        let g = Grid::new(2, 3);
        assert_eq!(6, g.cells.len());
        assert_eq!(2, g.width);
        assert_eq!(3, g.height);
    }

    #[test]
    fn test_grid_spawn_particle_to_grid() {
        let mut g = Grid::new(2, 3);
        g.spawn_particle(0, 0, Particle::Sand);

        g.spawn_particle(1, 1, Particle::Water);

        match &g.cells[0] {
            Some(p) => assert_eq!(Particle::Sand, p.particle),
            None => panic!(),
        }

        match &g.cells[3] {
            Some(p) => assert_eq!(Particle::Water, p.particle),
            None => panic!(),
        }
    }

    #[test]
    fn test_grid_spawn_particle_to_non_empty_location_silently_fails() {
        let mut g = Grid::new(2, 3);
        g.spawn_particle(0, 0, Particle::Sand);

        g.spawn_particle(0, 0, Particle::Water);

        match &g.cells[0] {
            Some(p) => assert_eq!(Particle::Sand, p.particle),
            None => panic!(),
        }
    }
    #[test]
    fn test_grid_spawn_particle_out_of_grid_bound_silently_fails() {
        let mut g = Grid::new(2, 3);
        g.spawn_particle(0, 3, Particle::Sand);
        g.spawn_particle(2, 0, Particle::Water);

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(None, g.cells[3]);
        assert_eq!(None, g.cells[4]);
        assert_eq!(None, g.cells[5]);
    }

    #[test]
    fn test_spawn_particles_brush() {
        let mut g = Grid::new(2, 2);
        g.spawn_brush((0, 0), 1, Particle::Sand);
        assert_eq!(Some(Cell::new(Particle::Sand)), g.cells[0]);

        let mut g = Grid::new(2, 2);
        g.spawn_brush((0, 0), 2, Particle::Sand);
        assert_eq!(
            vec![
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::Sand)),
            ],
            g.cells
        );
    }

    #[test]
    fn test_update_grid_sand_falling_down_to_last_row_stays_there() {
        let mut g = Grid::new(2, 2);
        g.spawn_particle(0, 1, Particle::Sand);

        g.update_grid(); /* should stay at the last line*/
        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(Some(Cell::new(Particle::Sand)), g.cells[2]);
        assert_eq!(None, g.cells[3]);
    }

    #[test]
    fn test_update_grid_sand_falls_down_when_bottom_cell_is_empty() {
        let mut g = Grid::new(2, 2);

        g.spawn_particle(0, 0, Particle::Sand);

        assert_eq!(Some(Cell::new(Particle::Sand)), g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(None, g.cells[3]);

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(Some(Cell::new(Particle::Sand)), g.cells[2]);
        assert_eq!(None, g.cells[3]);
    }

    #[test]
    fn test_update_grid_sand_falls_bottom_right_when_bottom_cell_is_full_and_bottom_left_is_wall_and_bottom_right_is_empty(
    ) {
        let mut g = Grid::new(2, 2);

        g.spawn_particle(0, 0, Particle::Sand);

        g.spawn_particle(0, 1, Particle::Sand);

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(Some(Cell::new(Particle::Sand)), g.cells[2]);
        assert_eq!(Some(Cell::new(Particle::Sand)), g.cells[3]);
    }

    #[test]
    fn test_update_grid_sand_falls_bottom_left_when_bottom_cell_is_full_and_bottom_right_is_wall_and_bottom_left_is_empty(
    ) {
        let mut g = Grid::new(2, 2);

        g.spawn_particle(1, 0, Particle::Sand);

        g.spawn_particle(1, 1, Particle::Sand);

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(Some(Cell::new(Particle::Sand)), g.cells[2]);
        assert_eq!(Some(Cell::new(Particle::Sand)), g.cells[3]);
    }

    #[test]
    fn test_update_grid_sand_falls_bottom_left_or_bottom_right_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_for_testing_forced_left(
    ) {
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Left), None);

        g.spawn_particle(1, 0, Particle::Sand);

        g.spawn_particle(1, 1, Particle::Sand);

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(Some(Cell::new(Particle::Sand)), g.cells[3]);
        assert_eq!(Some(Cell::new(Particle::Sand)), g.cells[4]);
        assert_eq!(None, g.cells[5]);
    }

    #[test]
    fn test_update_grid_sand_falls_bottom_left_or_bottom_right_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_for_testing_forced_right(
    ) {
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Right), None);

        g.spawn_particle(1, 0, Particle::Sand);

        g.spawn_particle(1, 1, Particle::Sand);

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(None, g.cells[3]);
        assert_eq!(Some(Cell::new(Particle::Sand)), g.cells[4]);
        assert_eq!(Some(Cell::new(Particle::Sand)), g.cells[5]);
    }

    #[test]
    fn test_update_grid_salt_falling_down_to_last_row_stays_there() {
        let mut g = Grid::new(2, 2);
        g.spawn_particle(0, 1, Particle::Salt);

        g.update_grid(); /* should stay at the last line*/
        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(Some(Cell::new(Particle::Salt)), g.cells[2]);
        assert_eq!(None, g.cells[3]);
    }

    #[test]
    fn test_update_grid_salt_falls_down_when_bottom_cell_is_empty() {
        let mut g = Grid::new(2, 2);

        g.spawn_particle(0, 0, Particle::Salt);

        assert_eq!(Some(Cell::new(Particle::Salt)), g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(None, g.cells[3]);

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(Some(Cell::new(Particle::Salt)), g.cells[2]);
        assert_eq!(None, g.cells[3]);
    }

    #[test]
    fn test_update_grid_salt_falls_bottom_right_when_bottom_cell_is_full_and_bottom_left_is_wall_and_bottom_right_is_empty(
    ) {
        let mut g = Grid::new(2, 2);

        g.spawn_particle(0, 0, Particle::Salt);

        g.spawn_particle(0, 1, Particle::Salt);

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(Some(Cell::new(Particle::Salt)), g.cells[2]);
        assert_eq!(Some(Cell::new(Particle::Salt)), g.cells[3]);
    }

    #[test]
    fn test_update_grid_salt_falls_bottom_left_when_bottom_cell_is_full_and_bottom_right_is_wall_and_bottom_left_is_empty(
    ) {
        let mut g = Grid::new(2, 2);

        g.spawn_particle(1, 0, Particle::Salt);

        g.spawn_particle(1, 1, Particle::Salt);

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(Some(Cell::new(Particle::Salt)), g.cells[2]);
        assert_eq!(Some(Cell::new(Particle::Salt)), g.cells[3]);
    }

    #[test]
    fn test_update_grid_salt_dissolves_when_touches_water() {
        for y in 0..3 {
            for x in 0..3 {
                if (x, y) == (1, 1) {
                    continue;
                }
                let mut g = Grid::new(3, 3);
                g.spawn_particle(1, 1, Particle::Salt);
                g.spawn_particle(x, y, Particle::Water);

                g.update_grid();

                assert_eq!(Some(Cell::new(Particle::Water)), g.cells[4]);
            }
        }
    }

    #[test]
    fn test_update_grid_salt_falls_bottom_left_or_bottom_right_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_for_testing_forced_left(
    ) {
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Left), None);

        g.spawn_particle(1, 0, Particle::Salt);

        g.spawn_particle(1, 1, Particle::Salt);

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(Some(Cell::new(Particle::Salt)), g.cells[3]);
        assert_eq!(Some(Cell::new(Particle::Salt)), g.cells[4]);
        assert_eq!(None, g.cells[5]);
    }

    #[test]
    fn test_update_grid_salt_falls_bottom_left_or_bottom_right_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_for_testing_forced_right(
    ) {
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Right), None);

        g.spawn_particle(1, 0, Particle::Salt);

        g.spawn_particle(1, 1, Particle::Salt);

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(None, g.cells[3]);
        assert_eq!(Some(Cell::new(Particle::Salt)), g.cells[4]);
        assert_eq!(Some(Cell::new(Particle::Salt)), g.cells[5]);
    }

    #[test]
    fn test_update_grid_water_moves_right_when_bottom_cell_and_left_is_full_and_right_cell_is_empty(
    ) {
        let mut g = Grid::new(3, 2);

        g.spawn_particle(0, 1, Particle::Sand);

        g.spawn_particle(1, 1, Particle::Water);

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(Some(Cell::new(Particle::Sand)), g.cells[3]);
        assert_eq!(None, g.cells[4]);
        assert_eq!(Some(Cell::new(Particle::Water)), g.cells[5]);
    }

    #[test]
    fn test_update_grid_water_moves_left_when_bottom_cell_and_right_is_full_and_left_cell_is_empty()
    {
        let mut g = Grid::new(3, 2);

        g.spawn_particle(1, 1, Particle::Water);

        g.spawn_particle(2, 1, Particle::Sand);

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(Some(Cell::new(Particle::Water)), g.cells[3]);
        assert_eq!(None, g.cells[4]);
        assert_eq!(Some(Cell::new(Particle::Sand)), g.cells[5]);
    }

    #[test]
    fn test_update_grid_water_moves_left_or_right_when_bottom_cell_is_empty_and_both_right_and_left_are_empty_forced_right(
    ) {
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Right), None);

        g.spawn_particle(1, 1, Particle::Water);

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(None, g.cells[3]);
        assert_eq!(None, g.cells[4]);
        assert_eq!(Some(Cell::new(Particle::Water)), g.cells[5]);
    }

    #[test]
    fn test_update_grid_water_moves_left_or_right_when_bottom_cell_is_empty_and_both_right_and_left_are_empty_forced_left(
    ) {
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Left), None);

        g.spawn_particle(1, 1, Particle::Water);

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(Some(Cell::new(Particle::Water)), g.cells[3]);
        assert_eq!(None, g.cells[4]);
        assert_eq!(None, g.cells[5]);
    }

    #[test]
    fn test_updating_rows_in_forward_order_creates_a_left_bias_on_water() {
        /*
         * updating in forward: -ww- => ww-- or w--w
         */
        let mut g = Grid::new_with_rand(
            4,
            1,
            Some(|| ParticleHorizontalDirection::Left),
            Some(|| RowUpdateDirection::Forward),
        );

        g.spawn_particle(1, 0, Particle::Water);
        g.spawn_particle(2, 0, Particle::Water);

        g.update_grid();

        assert_eq!(Some(Cell::new(Particle::Water)), g.cells[0]);
        assert_eq!(Some(Cell::new(Particle::Water)), g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(None, g.cells[3]);

        let mut g = Grid::new_with_rand(
            4,
            1,
            Some(|| ParticleHorizontalDirection::Right),
            Some(|| RowUpdateDirection::Forward),
        );

        g.spawn_particle(1, 0, Particle::Water);
        g.spawn_particle(2, 0, Particle::Water);

        g.update_grid();

        assert_eq!(Some(Cell::new(Particle::Water)), g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(Some(Cell::new(Particle::Water)), g.cells[3]);
    }

    #[test]
    fn test_updating_rows_in_reverse_order_creates_a_right_bias_on_water() {
        /*
         * updating in reverse: -ww- => --ww or w--w
         */
        let mut g = Grid::new_with_rand(
            4,
            1,
            Some(|| ParticleHorizontalDirection::Right),
            Some(|| RowUpdateDirection::Reverse),
        );

        g.spawn_particle(1, 0, Particle::Water);
        g.spawn_particle(2, 0, Particle::Water);

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(Some(Cell::new(Particle::Water)), g.cells[2]);
        assert_eq!(Some(Cell::new(Particle::Water)), g.cells[3]);

        let mut g = Grid::new_with_rand(
            4,
            1,
            Some(|| ParticleHorizontalDirection::Left),
            Some(|| RowUpdateDirection::Reverse),
        );

        g.spawn_particle(1, 0, Particle::Water);
        g.spawn_particle(2, 0, Particle::Water);

        g.update_grid();

        assert_eq!(Some(Cell::new(Particle::Water)), g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(Some(Cell::new(Particle::Water)), g.cells[3]);
    }

    #[test]
    fn test_sand_should_sink_to_bottom_in_water() {
        let mut g = Grid::new(3, 2);

        g.spawn_particle(1, 0, Particle::Sand);
        g.spawn_particle(0, 1, Particle::Sand);
        g.spawn_particle(1, 1, Particle::Water);
        g.spawn_particle(2, 1, Particle::Sand);

        g.update_grid();

        assert_eq!(
            vec![
                None,
                Some(Cell::new(Particle::Water)),
                None,
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::Sand)),
            ],
            g.cells
        );
    }

    #[test]
    fn test_sand_should_sink_to_bottom_left_in_water() {
        let mut g = Grid::new(3, 2);

        g.spawn_particle(1, 0, Particle::Sand);
        g.spawn_particle(0, 1, Particle::Water);
        g.spawn_particle(1, 1, Particle::Sand);
        g.spawn_particle(2, 1, Particle::Sand);

        g.update_grid();

        assert_eq!(
            vec![
                None,
                Some(Cell::new(Particle::Water)),
                None,
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::Sand)),
            ],
            g.cells
        );
    }

    #[test]
    fn test_sand_should_sink_to_bottom_right_in_water() {
        let mut g = Grid::new(3, 2);

        g.spawn_particle(1, 0, Particle::Sand);
        g.spawn_particle(0, 1, Particle::Sand);
        g.spawn_particle(1, 1, Particle::Sand);
        g.spawn_particle(2, 1, Particle::Water);

        g.update_grid();

        assert_eq!(
            vec![
                None,
                Some(Cell::new(Particle::Water)),
                None,
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::Sand)),
            ],
            g.cells
        );
    }

    #[test]
    fn test_sand_should_sink_in_water_but_water_shouldnot_climb_sands() {
        let mut g = Grid::new_with_rand(1, 3, Some(|| ParticleHorizontalDirection::Right), None);

        g.spawn_particle(0, 0, Particle::Sand);
        g.spawn_particle(0, 1, Particle::Sand);
        g.spawn_particle(0, 2, Particle::Water);

        assert_eq!(
            vec![
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::Water)),
            ],
            g.cells
        );

        g.update_grid();

        assert_eq!(
            vec![
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::Water)),
                Some(Cell::new(Particle::Sand)),
            ],
            g.cells
        );

        g.update_grid();

        assert_eq!(
            vec![
                Some(Cell::new(Particle::Water)),
                Some(Cell::new(Particle::Sand)),
                Some(Cell::new(Particle::Sand)),
            ],
            g.cells
        );
    }

    #[test]
    fn test_draw_grid() {
        let mut g = Grid::new(2, 2);
        let mut image = Grid::create_output_frame(2, 2);
        g.draw_grid(&mut image);
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(0, 0).unwrap());
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(1, 0).unwrap());
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(0, 1).unwrap());
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(1, 1).unwrap());

        g.spawn_particle(0, 0, Particle::Sand);
        g.draw_grid(&mut image);
        assert_color_srgb_eq!(
            Particle::Sand.color(),
            image.get_color_at(0, 0).unwrap(),
            0.1
        );
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(1, 0).unwrap());
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(0, 1).unwrap());
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(1, 1).unwrap());

        g.spawn_particle(1, 0, Particle::Water);
        g.cells[0] = None;
        g.draw_grid(&mut image);
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(0, 0).unwrap());
        assert_color_srgb_eq!(Particle::Water.color(), image.get_color_at(1, 0).unwrap());
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(0, 1).unwrap());
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(1, 1).unwrap());
    }

    #[test]
    fn test_get_particle_color() {
        assert_color_srgb_eq!(Color::srgb(0.76, 0.70, 0.50), Particle::Sand.color());
        assert_color_srgb_eq!(Color::srgb(0.05, 0.53, 0.80), Particle::Water.color());
        assert_color_srgb_eq!(Color::srgb(1.00, 1.00, 1.00), Particle::Salt.color());
    }
}
