use std::fmt;

use bevy::{
    asset::RenderAssetUsages,
    color::{Color, ColorToPacked, palettes::css},
    ecs::component::Component,
    image::Image,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use rand::random_range;

use super::particles::particle::Particle;

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
    pub particle: Option<Particle>,
    pub cycle: u32,
}

impl Cell {
    pub fn new(particle: Particle) -> Self {
        Self {
            particle: Some(particle),
            cycle: 0,
        }
    }

    pub fn empty() -> Self {
        Self {
            particle: None,
            cycle: 0,
        }
    }

    pub fn with_cycle(mut self, cycle: u32) -> Self {
        self.cycle = cycle;
        self
    }
}

#[derive(Component, Clone, PartialEq, Debug)]
pub struct Grid {
    cells: Vec<Cell>,
    width: usize,
    height: usize,
    particle_direction: fn() -> ParticleHorizontalDirection,
    row_update_direction: fn() -> RowUpdateDirection,
    cycle: u32,
    draw_cycle: u32,
}

pub trait GridAccess {
    fn particle_direction(&self) -> ParticleHorizontalDirection;
    fn get_neighbor_index(
        &self,
        position: (usize, usize),
        offset: (i32, i32),
    ) -> Result<usize, GridError>;
    fn get_cell(&self, index: usize) -> &Cell;
    fn get_cell_mut(&mut self, index: usize) -> &mut Cell;
    fn get_cells(&self) -> &Vec<Cell>;
    fn to_index(&self, position: (usize, usize)) -> usize;
    fn swap_particles(&mut self, index: usize, next_location_index: usize);
    fn dissolve_particles(&mut self, index: usize, next_location_index: usize);
    fn is_empty(&self, position: (usize, usize), offset: (i32, i32)) -> Option<usize>;
    fn is_simulated(&self, c: &Cell) -> bool;
    fn cycle(&self) -> u32;
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.particle {
            Some(p) => match p {
                Particle::Sand(_) => write!(f, "s"),
                Particle::Water(_) => write!(f, "w"),
                Particle::Salt(_) => write!(f, "S"),
                Particle::Rock(_) => write!(f, "r"),
                Particle::Drain(_) => write!(f, "d"),
                Particle::Tap(_) => write!(f, "t"),
            },
            None => write!(f, "-"),
        }
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = writeln!(f, "{}", ".".repeat(self.width + 2));
        for j in 0..self.height {
            let _ = write!(f, ".");
            for i in 0..self.width {
                let c = self.get_cell(self.to_index((i, j)));
                let _ = write!(f, "{c}");
            }
            let _ = writeln!(f, ".");
        }
        writeln!(f, "{}", ".".repeat(self.width + 2))
    }
}

impl GridAccess for Grid {
    fn get_cell(&self, index: usize) -> &Cell {
        &self.cells[index]
    }

    fn get_cell_mut(&mut self, index: usize) -> &mut Cell {
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

    fn particle_direction(&self) -> ParticleHorizontalDirection {
        (self.particle_direction)()
    }

    fn get_cells(&self) -> &Vec<Cell> {
        &self.cells
    }

    fn swap_particles(&mut self, index: usize, next_location_index: usize) {
        self.cells.swap(index, next_location_index);
        self.cells[index].cycle = self.cycle;
        self.cells[next_location_index].cycle = self.cycle;
    }

    fn dissolve_particles(&mut self, index: usize, next_location_index: usize) {
        self.cells[next_location_index].particle = None;
        self.cells[index].cycle = self.cycle;
        self.cells[next_location_index].cycle = self.cycle;
    }

    fn is_empty(&self, position: (usize, usize), offset: (i32, i32)) -> Option<usize> {
        match self.get_neighbor_index(position, offset) {
            Ok(i) => match self.get_cell(i).particle {
                Some(_) => None,
                None => Some(i),
            },
            Err(_) => None,
        }
    }

    fn cycle(&self) -> u32 {
        self.cycle
    }

    fn is_simulated(&self, c: &Cell) -> bool {
        self.cycle() <= c.cycle
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
            cells: (0..width * height).map(|_| Cell::empty()).collect(),
            width,
            height,
            particle_direction: random_water_direction,
            row_update_direction: random_row_update_direction,
            cycle: 0,
            draw_cycle: 0,
        }
    }

    pub fn spawn_particle(&mut self, (x, y): (usize, usize), particle: Particle) {
        if y < self.height && x < self.width {
            let index = self.to_index((x, y));
            if self.cells[index].particle.is_none() {
                self.cells[index] = Cell::new(particle).with_cycle(self.cycle);
            }
        }
    }

    pub fn despawn_particle(&mut self, (x, y): (usize, usize)) {
        if y < self.height && x < self.width {
            let index = self.to_index((x, y));
            self.cells[index] = Cell::empty().with_cycle(self.cycle);
        }
    }

    pub fn update_grid(&mut self) {
        self.cycle = self.cycle.wrapping_add(1);
        for y in (0..self.height).rev() {
            let x_direction = (self.row_update_direction)();
            for x in 0..self.width {
                let x = match x_direction {
                    RowUpdateDirection::Forward => x,
                    RowUpdateDirection::Reverse => self.width - 1 - x,
                };
                let c = self.get_cell(self.to_index((x, y)));
                if !self.is_simulated(c) {
                    if let Some(p) = &c.particle {
                        p.clone().update(self, (x, y)); // TODO: is there any other way to handle this double borrow instead of clone?
                    };
                }
            }
        }
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

    pub fn draw_grid(&mut self, image: &mut Image) {
        for (index, cell) in self.cells.iter().enumerate() {
            if self.draw_cycle <= cell.cycle {
                let x: u32 = index as u32 % self.width as u32;
                let y: u32 = (index as u32 - x) / self.width as u32;
                let _ = match &cell.particle {
                    Some(p) => image.set_color_at(x, y, p.color()),
                    _ => image.set_color_at(x, y, BACKGROUND_COLOR),
                };
            }
        }
        self.draw_cycle = self.cycle;
    }

    pub fn spawn_brush(
        &mut self,
        position: (usize, usize),
        size: usize,
        particle: Option<Particle>,
    ) {
        for position in Self::circle_brush(position, size) {
            match particle.clone() {
                Some(p) => self.spawn_particle(position, p),
                None => self.despawn_particle(position),
            }
        }
    }

    fn circle_brush((x, y): (usize, usize), size: usize) -> impl Iterator<Item = (usize, usize)> {
        let radius = size as i32 / 2;
        ((-radius)..=(radius)).flat_map(move |j| {
            ((-radius)..=(radius)).filter_map(move |i| {
                if (i * i) + (j * j) <= (radius * radius) {
                    Some(((x as i32 + i) as usize, (y as i32 + j) as usize))
                } else {
                    None
                }
            })
        })
    }

    #[allow(dead_code)]
    pub fn new_with_rand(
        width: usize,
        height: usize,
        particle_direction: Option<fn() -> ParticleHorizontalDirection>,
        row_update_direction: Option<fn() -> RowUpdateDirection>,
    ) -> Self {
        let mut g = Self::new(width, height);
        g.particle_direction = match particle_direction {
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
    use bevy::color::{Gray, Hsva};

    use super::*;
    use crate::component::{
        macros::assert_color_srgb_eq,
        particles::{drain::Drain, rock::Rock, salt::Salt, sand::Sand, tap::Tap, water::Water},
    };

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
        g.spawn_particle((0, 0), Particle::from(Sand::new()));

        g.spawn_particle((1, 1), Particle::from(Water::new()));

        assert_eq!(
            vec![
                Cell::new(Particle::from(Sand::new())),
                Cell::empty(),
                Cell::empty(),
                Cell::new(Particle::from(Water::new())),
                Cell::empty(),
                Cell::empty(),
            ],
            g.cells
        );
    }

    #[test]
    fn test_grid_spawn_particle_to_grid_spawns_with_current_cycle() {
        let mut g = Grid::new(1, 2);
        g.spawn_particle((0, 0), Particle::from(Sand::new()));

        g.update_grid();

        g.spawn_particle((0, 0), Particle::from(Sand::new()));

        assert_eq!(
            vec![
                Cell::new(Particle::from(Sand::new())).with_cycle(1),
                Cell::new(Particle::from(Sand::new())).with_cycle(1),
            ],
            g.cells
        );
    }

    #[test]
    fn test_grid_spawn_particle_to_non_empty_location_silently_fails() {
        let mut g = Grid::new(2, 3);
        g.spawn_particle((0, 0), Particle::from(Sand::new()));

        g.spawn_particle((0, 0), Particle::from(Water::new()));

        assert_eq!(Some(Particle::from(Sand::new())), g.cells[0].particle);
    }

    #[test]
    fn test_grid_spawn_particle_out_of_grid_bound_silently_fails() {
        let mut g = Grid::new(2, 3);
        g.spawn_particle((0, 3), Particle::from(Sand::new()));
        g.spawn_particle((2, 0), Particle::from(Water::new()));

        assert_eq!(
            vec![
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
            ],
            g.cells
        );
    }

    #[test]
    fn test_grid_despawn_particle_empties_the_cell_particle() {
        let mut g = Grid::new(1, 1);
        g.spawn_particle((0, 0), Particle::from(Sand::new()));
        assert_eq!(Cell::new(Particle::from(Sand::new())), g.cells[0]);

        g.despawn_particle((0, 0));
        assert_eq!(Cell::empty(), g.cells[0]);
    }

    #[test]
    fn test_grid_despawn_particle_out_of_grid_bound_silently_fails() {
        let mut g = Grid::new(2, 3);
        g.spawn_particle((0, 0), Particle::from(Sand::new()));
        g.spawn_particle((1, 0), Particle::from(Sand::new()));
        g.spawn_particle((0, 1), Particle::from(Sand::new()));
        g.spawn_particle((1, 1), Particle::from(Sand::new()));
        g.spawn_particle((0, 2), Particle::from(Sand::new()));
        g.spawn_particle((1, 2), Particle::from(Sand::new()));

        g.despawn_particle((0, 3));
        g.despawn_particle((2, 0));

        assert_eq!(
            vec![
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())),
            ],
            g.cells
        );
    }

    #[test]
    fn test_grid_despawn_particle_to_grid_despawns_with_current_cycle() {
        let mut g = Grid::new(1, 1);
        g.spawn_particle((0, 0), Particle::from(Sand::new()));

        assert_eq!(vec![Cell::new(Particle::from(Sand::new())),], g.cells);

        g.update_grid();

        g.despawn_particle((0, 0));

        assert_eq!(vec![Cell::empty().with_cycle(1)], g.cells);
    }

    #[test]
    fn test_spawn_particles_brush_size_one() {
        /*
         * ---
         * -s-
         * ---
         */
        let mut g = Grid::new(3, 3);
        g.spawn_brush((1, 1), 1, Some(Particle::from(Sand::new())));
        assert_eq!(
            vec![
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::new(Particle::from(Sand::new())),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
            ],
            g.cells
        );
    }

    #[test]
    fn test_spawn_particles_brush_size_two() {
        /*
         * -s-
         * sss
         * -s-
         */
        let mut g = Grid::new(3, 3);
        g.spawn_brush((1, 1), 2, Some(Particle::from(Sand::new())));
        assert_eq!(
            vec![
                Cell::empty(),
                Cell::new(Particle::from(Sand::new())),
                Cell::empty(),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())),
                Cell::empty(),
                Cell::new(Particle::from(Sand::new())),
                Cell::empty(),
            ],
            g.cells
        );
    }

    #[test]
    fn test_spawn_particles_brush_size_four() {
        /*
         * --s--
         * -sss-
         * sssss
         * -sss-
         * --s--
         */
        let mut g = Grid::new(5, 5);
        g.spawn_brush((2, 2), 4, Some(Particle::from(Sand::new())));
        assert_eq!(
            vec![
                Cell::empty(),
                Cell::empty(),
                Cell::new(Particle::from(Sand::new())),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())),
                Cell::empty(),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())),
                Cell::empty(),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::new(Particle::from(Sand::new())),
                Cell::empty(),
                Cell::empty(),
            ],
            g.cells
        );
    }

    #[test]
    fn test_despawn_particles_brush() {
        /*
         * --- -> -s- -> ---
         * ---    sss    ---
         * ---    -s-    ---
         */
        let mut g = Grid::new(3, 3);

        g.spawn_brush((1, 1), 2, Some(Particle::from(Sand::new())));

        assert_eq!(
            vec![
                Cell::empty(),
                Cell::new(Particle::from(Sand::new())),
                Cell::empty(),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())),
                Cell::empty(),
                Cell::new(Particle::from(Sand::new())),
                Cell::empty(),
            ],
            g.cells
        );

        {
            let this = &mut g;
            let position = (1, 1);
            this.spawn_brush(position, 2, None);
        };

        assert_eq!(
            vec![
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
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

        g.spawn_particle((0, 0), Particle::from(Sand::new()));
        g.draw_grid(&mut image);
        assert_color_srgb_eq!(
            Particle::from(Sand::new()).color(),
            image.get_color_at(0, 0).unwrap(),
            0.1
        );
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(1, 0).unwrap());
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(0, 1).unwrap());
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(1, 1).unwrap());

        g.spawn_particle((1, 0), Particle::from(Water::new()));
        g.cells[0].particle = None;
        g.draw_grid(&mut image);
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(0, 0).unwrap());
        assert_color_srgb_eq!(
            Particle::from(Water::new()).color(),
            image.get_color_at(1, 0).unwrap()
        );
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(0, 1).unwrap());
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(1, 1).unwrap());
    }

    #[test]
    fn test_draw_grid_only_redraw_changed_cells() {
        let mut g = Grid::new(2, 2);
        g.spawn_particle((0, 0), Particle::from(Sand::new()));

        /*
         * draw_cycle: 0
         * cycles:
         *  0 0
         *  0 0
         */
        let mut image = Grid::create_output_frame(2, 2);
        g.draw_grid(&mut image);

        assert_color_srgb_eq!(
            Particle::from(Sand::new()).color(),
            image.get_color_at(0, 0).unwrap()
        );
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(1, 0).unwrap());
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(0, 1).unwrap());
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(1, 1).unwrap());

        g.update_grid();
        /*
         * draw_cycle: 0
         * cycles:
         *  1 0
         *  1 0
         */
        let mut image = Grid::create_output_frame(2, 2);
        g.draw_grid(&mut image);

        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(0, 0).unwrap());
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(1, 0).unwrap());
        assert_color_srgb_eq!(
            Particle::from(Sand::new()).color(),
            image.get_color_at(0, 1).unwrap()
        );
        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(1, 1).unwrap());

        g.update_grid();
        /*
         * draw_cycle: 1
         * cycles:
         *  1 0
         *  1 0
         */
        let mut image = Grid::create_output_frame(2, 2);
        g.draw_grid(&mut image);

        assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(0, 0).unwrap());
        assert_color_srgb_eq!(Color::Hsva(Hsva::BLACK), image.get_color_at(1, 0).unwrap());
        assert_color_srgb_eq!(
            Particle::from(Sand::new()).color(),
            image.get_color_at(0, 1).unwrap()
        );
        assert_color_srgb_eq!(Color::Hsva(Hsva::BLACK), image.get_color_at(1, 1).unwrap());

        g.update_grid();
        /*
         * draw_cycle: 2
         * cycles:
         *  1 0
         *  1 0
         */
        let mut image = Grid::create_output_frame(2, 2);
        g.draw_grid(&mut image);

        assert_color_srgb_eq!(Color::Hsva(Hsva::BLACK), image.get_color_at(0, 0).unwrap());
        assert_color_srgb_eq!(Color::Hsva(Hsva::BLACK), image.get_color_at(1, 0).unwrap());
        assert_color_srgb_eq!(Color::Hsva(Hsva::BLACK), image.get_color_at(0, 1).unwrap());
        assert_color_srgb_eq!(Color::Hsva(Hsva::BLACK), image.get_color_at(1, 1).unwrap());
    }

    #[test]
    fn test_get_particle_color() {
        assert_color_srgb_eq!(
            Color::hsva(43.20, 0.34, 0.76, 1.00),
            Particle::from(Sand::new()).color()
        );
        assert_color_srgb_eq!(
            Color::hsva(201.60, 1.00, 0.80, 1.00),
            Particle::from(Water::new()).color()
        );
        assert_color_srgb_eq!(
            Color::hsva(0.00, 0.00, 1.00, 1.00),
            Particle::from(Salt::new()).color()
        );
        assert_color_srgb_eq!(
            Color::hsva(28.0, 0.25, 0.30, 1.00),
            Particle::from(Rock::new()).color()
        );

        assert_color_srgb_eq!(
            Color::hsva(0.0, 0.0, 0.10, 1.00),
            Particle::from(Drain::new()).color()
        );

        assert_color_srgb_eq!(
            Color::hsva(190.0, 0.40, 0.75, 1.00),
            Particle::from(Tap::new()).color()
        );
    }

    #[test]
    fn test_water_particle_gets_lighter_color_with_more_dissolved_salt_particles() {
        for (c, s) in (0..=3).zip([0.7, 0.8, 0.9, 1.00]) {
            assert_color_srgb_eq!(
                Color::hsva(201.60, s, 0.80, 1.00),
                Particle::from(Water::with_capacity(c)).color()
            );
        }
    }

    #[test]
    fn test_cell_string_names() {
        assert_eq!("-",Cell::empty().to_string());
        assert_eq!("s",Cell::new( Particle::from(Sand::new())).to_string());
        assert_eq!("S",Cell::new( Particle::from(Salt::new())).to_string());
        assert_eq!("w",Cell::new( Particle::from(Water::new())).to_string());
        assert_eq!("r",Cell::new( Particle::from(Rock::new())).to_string());
        assert_eq!("d",Cell::new( Particle::from(Drain::new())).to_string());
        assert_eq!("t",Cell::new( Particle::from(Tap::new())).to_string());
    }
}
