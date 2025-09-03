use bevy::{
    asset::RenderAssetUsages,
    color::{palettes::css, Color, ColorToPacked},
    ecs::component::Component,
    image::Image,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use rand::random_range;

// TODO: remove all unwrap and expects

enum GridError {
    OutOfBound,
}

pub const BACKGROUND_COLOR: bevy::prelude::Color = Color::srgb(0.82, 0.93, 1.);

#[derive(Clone, PartialEq, Debug)]
pub enum Particle {
    Sand,
    Water,
    Salt,
}

enum ParticleOperation {
    Swap(usize),
    Dissolve(Particle),
}

#[derive(Clone, Debug)]
enum ParticleHorizontalDirection {
    Left = -1,
    Right = 1,
}

enum RowUpdateDirection {
    Forward = 0,
    Reverse = 1,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Cell {
    particle: Particle,
    simulated: bool,
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

trait GridAccess {
    fn water_direction(&self) -> ParticleHorizontalDirection;
    fn get_neighbor_index(&self, x: usize, y: usize, xn: i32, yn: i32) -> Result<usize, GridError>;
    fn get_cell(&self, index: usize) -> &Option<Cell>;
}

impl GridAccess for Grid {
    fn get_cell(&self, index: usize) -> &Option<Cell> {
        &self.cells[index]
    }

    fn get_neighbor_index(&self, x: usize, y: usize, xn: i32, yn: i32) -> Result<usize, GridError> {
        let neighbor_index = (y as i32 + yn) * self.width as i32 + (x as i32 + xn);

        if (0 <= y as i32 + yn)
            && ((y as i32 + yn) < self.height as i32)
            && ((x as i32 + xn) < self.width as i32)
            && (0 <= x as i32 + xn)
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

impl Particle {
    fn find_particle_next_location<T: GridAccess>(
        &self,
        grid: &T,
        x: usize,
        y: usize,
    ) -> Option<ParticleOperation> {
        match self {
            Particle::Sand => Particle::find_sand_particle_next_location(grid, x, y),
            Particle::Water => Particle::find_water_particle_next_location(grid, x, y),
            Particle::Salt => Particle::find_salt_particle_next_location(grid, x, y),
        }
    }

    fn find_sand_particle_next_location<T: GridAccess>(
        grid: &T,
        x: usize,
        y: usize,
    ) -> Option<ParticleOperation> {
        let index_bottom = match grid.get_neighbor_index(x, y, 0, 1) {
            Ok(i) => match grid.get_cell(i) {
                Some(p) => match p.particle {
                    Particle::Water => match p.simulated {
                        true => None,
                        false => Some(i),
                    },
                    _ => None,
                },
                None => Some(i),
            },
            Err(_) => None,
        };

        let index_bottom_right = match grid.get_neighbor_index(x, y, 1, 1) {
            Ok(i) => match grid.get_cell(i) {
                Some(p) => match p.particle {
                    Particle::Water => match p.simulated {
                        true => None,
                        false => Some(i),
                    },
                    _ => None,
                },
                None => Some(i),
            },
            Err(_) => None,
        };

        let index_bottom_left = match grid.get_neighbor_index(x, y, -1, 1) {
            Ok(i) => match grid.get_cell(i) {
                Some(p) => match p.particle {
                    Particle::Water => match p.simulated {
                        true => None,
                        false => Some(i),
                    },
                    _ => None,
                },
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

        match index {
            Some(i) => Some(ParticleOperation::Swap(i)),
            None => None,
        }
    }

    fn find_water_particle_next_location<T: GridAccess>(
        grid: &T,
        x: usize,
        y: usize,
    ) -> Option<ParticleOperation> {
        let index_bottom = match grid.get_neighbor_index(x, y, 0, 1) {
            Ok(i) => match grid.get_cell(i) {
                Some(_) => None,
                None => Some(i),
            },
            Err(_) => None,
        };

        let index_right = match grid.get_neighbor_index(x, y, 1, 0) {
            Ok(i) => match grid.get_cell(i) {
                Some(_) => None,
                None => Some(i),
            },
            Err(_) => None,
        };

        let index_left = match grid.get_neighbor_index(x, y, -1, 0) {
            Ok(i) => match grid.get_cell(i) {
                Some(_) => None,
                None => Some(i),
            },
            Err(_) => None,
        };

        let index_bottom_right = match grid.get_neighbor_index(x, y, 1, 1) {
            Ok(i) => match grid.get_cell(i) {
                Some(_) => None,
                None => Some(i),
            },
            Err(_) => None,
        };

        let index_bottom_left = match grid.get_neighbor_index(x, y, -1, 1) {
            Ok(i) => match grid.get_cell(i) {
                Some(_) => None,
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

        match index {
            Some(i) => Some(ParticleOperation::Swap(i)),
            None => None,
        }
    }

    fn find_salt_particle_next_location<T: GridAccess>(
        grid: &T,
        x: usize,
        y: usize,
    ) -> Option<ParticleOperation> {
        let neighboring_water =
            (-1..=1)
                .flat_map(|y| (-1..=1).map(move |x| (y, x)))
                .fold(false, |acc, (xo, yo)| {
                    let n = match grid.get_neighbor_index(x, y, xo, yo) {
                        Ok(i) => match grid.get_cell(i) {
                            Some(p) => match p.particle {
                                Particle::Water => true,
                                _ => false,
                            },
                            None => false,
                        },
                        Err(_) => false,
                    };
                    acc || n
                });
        if neighboring_water {
            return Some(ParticleOperation::Dissolve(Particle::Water));
        }
        let index_bottom = match grid.get_neighbor_index(x, y, 0, 1) {
            Ok(i) => match grid.get_cell(i) {
                Some(_) => None,
                None => Some(i),
            },
            Err(_) => None,
        };

        let index_bottom_right = match grid.get_neighbor_index(x, y, 1, 1) {
            Ok(i) => match grid.get_cell(i) {
                Some(_) => None,
                None => Some(i),
            },
            Err(_) => None,
        };

        let index_bottom_left = match grid.get_neighbor_index(x, y, -1, 1) {
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

        match index {
            Some(i) => Some(ParticleOperation::Swap(i)),
            None => None,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Particle::Sand => Color::srgb(0.76, 0.70, 0.50),
            Particle::Water => Color::srgb(0.05, 0.53, 0.80),
            Particle::Salt => Color::srgb(1.00, 1.00, 1.00),
        }
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
        };
        if let Some(p) = &mut self.cells[next_location_index] {
            p.simulated = true;
        };
    }

    fn disolve_particles(&mut self, index: usize, particle: Particle) {
        if let Some(p) = &mut self.cells[index] {
            p.simulated = true;
            p.particle = particle
        };
    }

    fn clear_all_simulated_field(&mut self) {
        self.cells.iter_mut().for_each(|x| {
            if let Some(x) = x {
                x.simulated = false
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
                            self.swap_particles(index, next_location_index)
                        }
                        ParticleOperation::Dissolve(particle) => {
                            self.disolve_particles(index, particle)
                        }
                    }
                }
            }
        };
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

    pub fn spawn_brush(&mut self, x: usize, y: usize, size: usize, particle: Particle) {
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
        g.spawn_brush(0, 0, 1, Particle::Sand);
        assert_eq!(Some(Cell::new(Particle::Sand)), g.cells[0]);

        let mut g = Grid::new(2, 2);
        g.spawn_brush(0, 0, 2, Particle::Sand);
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
