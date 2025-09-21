use bevy::{
    asset::RenderAssetUsages,
    color::{Color, ColorToPacked, palettes::css},
    ecs::component::Component,
    image::Image,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use rand::random_range;

use super::particles::particle::Particle;

// TODO: remove all unwrap and expects

pub enum GridError {
    OutOfBound,
}

pub const BACKGROUND_COLOR: bevy::prelude::Color = Color::srgb(0.82, 0.93, 1.);

#[derive(Clone, Debug)]
pub enum ParticleHorizontalDirection {
    Left = -1,
    Right = 1,
}

pub enum RowUpdateDirection {
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
    fn get_cell_mut(&mut self, index: usize) -> &mut Option<Cell>;
    fn get_cells(&self) -> &Vec<Option<Cell>>;
    fn to_index(&self, position: (usize, usize)) -> usize;
    fn swap_particles(&mut self, index: usize, next_location_index: usize);
    fn dissolve_particles(&mut self, index: usize, next_location_index: usize);
    fn is_empty(&self, position: (usize, usize), offset: (i32, i32)) -> Option<usize>;
}

impl GridAccess for Grid {
    fn get_cell(&self, index: usize) -> &Option<Cell> {
        &self.cells[index]
    }

    fn get_cell_mut(&mut self, index: usize) -> &mut Option<Cell> {
        &mut self.cells[index]
    }

    fn to_index(&self, (x, y): (usize, usize)) -> usize {
        y * self.width + x
    }

    fn get_neighbor_index(
        &self,
        (x, y): (usize, usize),
        (ox, oy): (i32, i32),
    ) -> Result<usize, GridError> {
        let y = y as i32;
        let x = x as i32;
        if (0 <= y + oy)
            && ((y + oy) < self.height as i32)
            && ((x + ox) < self.width as i32)
            && (0 <= x + ox)
        {
            Ok(self.to_index(((x + ox) as usize, (y + oy) as usize)))
        } else {
            Err(GridError::OutOfBound)
        }
    }

    fn water_direction(&self) -> ParticleHorizontalDirection {
        (self.water_direction)()
    }

    fn get_cells(&self) -> &Vec<Option<Cell>> {
        &self.cells
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

    fn dissolve_particles(&mut self, index: usize, next_location_index: usize) {
        if let Some(p) = &mut self.cells[index] {
            p.simulated = true;
        }
        if self.cells[next_location_index].is_some() {
            self.cells[next_location_index] = None;
        }
    }

    fn is_empty(&self, position: (usize, usize), offset: (i32, i32)) -> Option<usize> {
        match self.get_neighbor_index(position, offset) {
            Ok(i) => match self.get_cell(i) {
                Some(_) => None,
                None => Some(i),
            },
            Err(_) => None,
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
            let index = self.to_index((x, y));
            if self.cells[index].is_none() {
                self.cells[index] = Some(Cell::new(particle));
            }
        }
    }

    fn clear_all_simulated_field(&mut self) {
        self.cells.iter_mut().for_each(|x| {
            if let Some(x) = x {
                x.simulated = false;
            }
        });
    }

    pub fn update_grid(&mut self) {
        for y in (0..self.height).rev() {
            let x_direction = (self.row_update_direction)();
            for x in 0..self.width {
                let x = match x_direction {
                    RowUpdateDirection::Forward => x,
                    RowUpdateDirection::Reverse => self.width - 1 - x,
                };
                if let Some(p) = self.get_cell(self.to_index((x, y))) {
                    if !p.simulated {
                        p.particle.clone().update(self, (x, y)); // TODO: is there any other way to handle this double borrow instead of clone?
                    }
                }
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

    #[allow(dead_code)]
    pub fn new_with_rand(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::macros::assert_color_srgb_eq;

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

        g.spawn_particle(1, 1, Particle::new_water());

        match &g.cells[0] {
            Some(p) => assert_eq!(Particle::Sand, p.particle),
            None => panic!(),
        }

        match &g.cells[3] {
            Some(p) => assert_eq!(Particle::new_water(), p.particle),
            None => panic!(),
        }
    }

    #[test]
    fn test_grid_spawn_particle_to_non_empty_location_silently_fails() {
        let mut g = Grid::new(2, 3);
        g.spawn_particle(0, 0, Particle::Sand);

        g.spawn_particle(0, 0, Particle::new_water());

        match &g.cells[0] {
            Some(p) => assert_eq!(Particle::Sand, p.particle),
            None => panic!(),
        }
    }
    #[test]
    fn test_grid_spawn_particle_out_of_grid_bound_silently_fails() {
        let mut g = Grid::new(2, 3);
        g.spawn_particle(0, 3, Particle::Sand);
        g.spawn_particle(2, 0, Particle::new_water());

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

        g.spawn_particle(1, 0, Particle::new_water());
        g.cells[0] = None;
        g.draw_grid(&mut image);
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(0, 0).unwrap());
        assert_color_srgb_eq!(
            Particle::new_water().color(),
            image.get_color_at(1, 0).unwrap()
        );
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(0, 1).unwrap());
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(1, 1).unwrap());
    }

    #[test]
    fn test_get_particle_color() {
        assert_color_srgb_eq!(Color::hsva(43.20, 0.34, 0.76, 1.00), Particle::Sand.color());
        assert_color_srgb_eq!(
            Color::hsva(201.60, 1.00, 0.80, 1.00),
            Particle::new_water().color()
        );
        assert_color_srgb_eq!(Color::hsva(0.00, 0.00, 1.00, 1.00), Particle::Salt.color());
    }

    #[test]
    fn test_water_particle_gets_lighter_color_when_it_cannot_dissolve_anymore_salt() {
        for s in 1..=3 {
            assert_color_srgb_eq!(
                Color::hsva(201.60, 1.00, 0.80, 1.00),
                Particle::Water { solute: s }.color()
            );
        }
        assert_color_srgb_eq!(
            Color::hsva(201.60, 0.60, 0.80, 1.00),
            Particle::Water { solute: 0 }.color()
        );
    }
}
