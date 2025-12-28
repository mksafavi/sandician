use std::fmt;

use bevy::{
    asset::RenderAssetUsages,
    color::{Color, ColorToPacked, palettes::css},
    ecs::component::Component,
    image::Image,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use super::particles::particle::{Particle, ParticleKind};

pub enum GridError {
    OutOfBound,
}

pub const BACKGROUND_COLOR: bevy::prelude::Color = Color::srgb(0.82, 0.93, 1.);

#[derive(Clone, PartialEq, Debug)]
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

#[derive(Debug)]
pub struct Random {
    row_update_direction: fn(r: &mut Random) -> RowUpdateDirection,
    particle_seed: fn(r: &mut Random) -> u8,
    particle_seed_with_cycle: fn(&mut Random) -> u8,
    horizontal_velocity_probability: fn(r: &mut Random) -> i16,
    vertical_velocity_probability: fn(r: &mut Random) -> i16,
    rng: fastrand::Rng,
    cycle: u32,
}

#[derive(Clone, Debug, PartialEq)]
struct Window {
    start: (usize, usize),
    end: (usize, usize),
    active: bool,
}

impl Window {
    fn new(start: (usize, usize), end: (usize, usize)) -> Self {
        Self {
            start,
            end,
            active: false,
        }
    }

    fn activate(&mut self) {
        self.active = true;
    }

    fn deactivate(&mut self) {
        self.active = false;
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn in_window(&self, (x, y): (usize, usize)) -> bool {
        (self.start.0 <= x && x <= self.end.0) && (self.start.1 <= y && y <= self.end.1)
    }
}

#[derive(Clone, Debug)]
pub struct WindowGrid {
    windows: Vec<Window>,
    width: usize,
    height: usize,
}

#[derive(Component, Debug)]
pub struct Grid {
    cells: Vec<Cell>,
    width: usize,
    height: usize,
    cycle: u32,
    draw_cycle: u32,
    random: Random,
    initial_particle_velocity: (i16, i16),
    window_grid: WindowGrid,
}

pub trait GridAccess {
    fn particle_seed(&mut self) -> u8;
    fn horizontal_velocity_probability(&mut self) -> i16;
    fn vertical_velocity_probability(&mut self) -> i16;
    fn get_neighbor_index(
        &self,
        position: (usize, usize),
        offset: (i32, i32),
    ) -> Result<usize, GridError>;
    fn get_neighbor_position(
        &self,
        position: (usize, usize),
        offset: (i32, i32),
    ) -> Result<(usize, usize), GridError>;
    fn get_cell(&self, index: usize) -> &Cell;
    fn get_cell_mut(&mut self, index: usize) -> &mut Cell;
    fn get_cells(&self) -> &Vec<Cell>;
    fn to_index(&self, position: (usize, usize)) -> usize;
    fn swap_particles(&mut self, index: usize, next_location_index: usize);
    fn is_empty(&self, position: (usize, usize), offset: (i32, i32)) -> Option<usize>;
    fn is_simulated(&self, c: &Cell) -> bool;
    fn cycle(&self) -> u32;
    fn get_particle_initial_velocity(&self) -> (i16, i16);
    fn activate_window(&mut self, position: (usize, usize));
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.particle {
            Some(p) => match p.kind {
                ParticleKind::Sand(_) => write!(f, "s"),
                ParticleKind::Water(_) => write!(f, "w"),
                ParticleKind::Salt(_) => write!(f, "S"),
                ParticleKind::Rock(_) => write!(f, "r"),
                ParticleKind::Drain(_) => write!(f, "d"),
                ParticleKind::Tap(_) => write!(f, "t"),
                ParticleKind::Acid(_) => write!(f, "a"),
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

    fn get_neighbor_position(
        &self,
        (x, y): (usize, usize),
        (ox, oy): (i32, i32),
    ) -> Result<(usize, usize), GridError> {
        let y = y as i32;
        let x = x as i32;
        if (0 <= y + oy)
            && ((y + oy) < self.height as i32)
            && ((x + ox) < self.width as i32)
            && (0 <= x + ox)
        {
            Ok(((x + ox) as usize, (y + oy) as usize))
        } else {
            Err(GridError::OutOfBound)
        }
    }

    fn get_neighbor_index(
        &self,
        position: (usize, usize),
        offset: (i32, i32),
    ) -> Result<usize, GridError> {
        Ok(self.to_index(self.get_neighbor_position(position, offset)?))
    }

    fn horizontal_velocity_probability(&mut self) -> i16 {
        (self.random.horizontal_velocity_probability)(&mut self.random)
    }

    fn particle_seed(&mut self) -> u8 {
        (self.random.particle_seed_with_cycle)(&mut self.random)
    }

    fn vertical_velocity_probability(&mut self) -> i16 {
        (self.random.vertical_velocity_probability)(&mut self.random)
    }

    fn get_cells(&self) -> &Vec<Cell> {
        &self.cells
    }

    fn swap_particles(&mut self, index: usize, next_location_index: usize) {
        self.cells.swap(index, next_location_index);
        self.cells[index].cycle = self.cycle;
        self.cells[next_location_index].cycle = self.cycle;
        self.activate_window((
            index % self.width,
            (index - index % self.width) / self.width,
        ));
        self.activate_window((
            next_location_index % self.width,
            (next_location_index - next_location_index % self.width) / self.width,
        ));
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

    fn get_particle_initial_velocity(&self) -> (i16, i16) {
        self.initial_particle_velocity
    }

    fn activate_window(&mut self, (x, y): (usize, usize)) {
        for yo in -1..=1 {
            for xo in -1..=1 {
                let position = ((x as i32 + xo) as usize, (y as i32 + yo) as usize);
                if let Some(w) = self.window_grid.get_window_mut(position) {
                    w.activate();
                }
            }
        }
    }
}

impl Random {
    fn new() -> Self {
        Self {
            horizontal_velocity_probability: Random::random_horizontal_velocity_probability,
            row_update_direction: Random::random_row_update_direction,
            particle_seed: Random::random_particle_seed,
            particle_seed_with_cycle: Random::random_particle_seed_with_cycle,
            vertical_velocity_probability: Random::random_vertical_velocity_probability,
            rng: fastrand::Rng::new(),
            cycle: 0,
        }
    }

    fn random_horizontal_velocity_probability(r: &mut Random) -> i16 {
        r.rng.i16(..)
    }
    fn random_row_update_direction(r: &mut Random) -> RowUpdateDirection {
        match r.rng.i32(..).is_positive() {
            true => RowUpdateDirection::Forward,
            false => RowUpdateDirection::Reverse,
        }
    }

    fn random_particle_seed(r: &mut Random) -> u8 {
        r.rng.u8(..)
    }

    fn random_particle_seed_with_cycle(r: &mut Random) -> u8 {
        ((r.particle_seed)(r) / 2) + (r.cycle as u8 / 2)
    }

    fn random_vertical_velocity_probability(r: &mut Random) -> i16 {
        r.rng.i16(0..=i16::MAX)
    }
}

impl WindowGrid {
    fn new((width, height): (usize, usize), window_size: (usize, usize)) -> Self {
        let windows = (0..height / window_size.1)
            .flat_map(|y| {
                (0..width / window_size.0).map(move |x| {
                    let s = (x * window_size.0, y * window_size.1);
                    let e = (s.0 + window_size.0 - 1, s.1 + window_size.1 - 1);
                    Window::new(s, e)
                })
            })
            .collect();
        Self {
            windows,
            width: window_size.0,
            height: window_size.1,
        }
    }

    fn get_window_mut(&mut self, position: (usize, usize)) -> Option<&mut Window> {
        for w in &mut self.windows {
            if w.in_window(position) {
                return Some(w);
            }
        }
        None
    }
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            cells: (0..width * height).map(|_| Cell::empty()).collect(),
            width,
            height,
            cycle: 0,
            draw_cycle: 0,
            random: Random::new(),
            initial_particle_velocity: (0, i16::MAX),
            window_grid: WindowGrid::new((width, height), (width, height)),
        }
    }

    pub fn spawn_particle(&mut self, (x, y): (usize, usize), particle: Particle) {
        if y < self.height && x < self.width {
            let index = self.to_index((x, y));
            if self.cells[index].particle.is_none() {
                self.cells[index] = Cell::new(particle).with_cycle(self.cycle);
                self.activate_window((x, y));
            }
        }
    }

    pub fn despawn_particle(&mut self, (x, y): (usize, usize)) {
        if y < self.height && x < self.width {
            let index = self.to_index((x, y));
            self.cells[index] = Cell::empty().with_cycle(self.cycle);
            self.activate_window((x, y));
        }
    }

    fn increment_cycle(&mut self) {
        self.cycle = self.cycle.wrapping_add(1);
        self.random.cycle = self.cycle;
    }

    pub fn update_grid(&mut self) {
        let window_grid = self.window_grid.clone();
        for w in &mut self.window_grid.windows {
            w.deactivate();
        }
        self.increment_cycle();
        for w in window_grid.windows {
            if w.is_active() {
                for y in (w.start.1..=w.end.1).rev() {
                    let x_direction = (self.random.row_update_direction)(&mut self.random);
                    for x in w.start.0..=w.end.0 {
                        let x = match x_direction {
                            RowUpdateDirection::Forward => x,
                            RowUpdateDirection::Reverse => w.end.0 + w.start.0 - x,
                        };
                        let c = self.get_cell(self.to_index((x, y)));
                        if !self.is_simulated(c) && c.particle.is_some() {
                            Particle::update(self, (x, y));
                        };
                    }
                }
            }
        }
    }

    pub fn clear_grid(&mut self) {
        self.cells.iter_mut().for_each(|c| {
            c.particle = None;
            c.cycle = self.cycle;
        });
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
        kind: Option<&ParticleKind>,
    ) {
        for position in Self::circle_brush(position, size) {
            match kind {
                Some(k) => {
                    let seed = (self.random.particle_seed_with_cycle)(&mut self.random);
                    self.spawn_particle(
                        position,
                        Particle::from(k.clone())
                            .with_seed(seed)
                            .with_velocity(self.initial_particle_velocity),
                    )
                }
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
    pub fn with_rand_horizontal_velocity_probability(
        mut self,
        particle_direction: fn(&mut Random) -> i16,
    ) -> Self {
        self.random.horizontal_velocity_probability = particle_direction;
        self
    }

    #[allow(dead_code)]
    pub fn with_rand_row_update_direction(
        mut self,
        row_update_direction: fn(&mut Random) -> RowUpdateDirection,
    ) -> Self {
        self.random.row_update_direction = row_update_direction;
        self
    }

    #[allow(dead_code)]
    pub fn with_rand_seed(mut self, particle_seed: fn(r: &mut Random) -> u8) -> Self {
        self.random.particle_seed = particle_seed;
        self
    }

    #[allow(dead_code)]
    pub fn with_rand_seed_with_cycle(
        mut self,
        particle_seed_with_cycle: fn(r: &mut Random) -> u8,
    ) -> Self {
        self.random.particle_seed_with_cycle = particle_seed_with_cycle;
        self
    }

    #[allow(dead_code)]
    pub fn with_rand_vertical_velocity_probability(
        mut self,
        veritical_velocity_probability: fn(r: &mut Random) -> i16,
    ) -> Self {
        self.random.vertical_velocity_probability = veritical_velocity_probability;
        self
    }

    #[allow(dead_code)]
    pub fn with_initial_particle_velocity(mut self, initial_particle_velocity: (i16, i16)) -> Self {
        self.initial_particle_velocity = initial_particle_velocity;
        self
    }

    #[allow(dead_code)]
    pub fn with_window_size(mut self, window_size: (usize, usize)) -> Self {
        self.window_grid = WindowGrid::new((self.width, self.height), window_size);
        self
    }
}

#[cfg(test)]
mod tests {
    use bevy::color::{Gray, Hsva};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::component::{
        macros::assert_color_srgb_eq,
        particles::{
            acid::Acid, drain::Drain, rock::Rock, salt::Salt, sand::Sand, tap::Tap, water::Water,
        },
    };

    #[test]
    fn test_create_grid() {
        let g = Grid::new(2, 3);
        assert_eq!(6, g.get_cells().len());
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
            *g.get_cells()
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
            *g.get_cells()
        );
    }

    #[test]
    fn test_grid_spawn_particle_to_non_empty_location_silently_fails() {
        let mut g = Grid::new(2, 3);
        g.spawn_particle((0, 0), Particle::from(Sand::new()));

        g.spawn_particle((0, 0), Particle::from(Water::new()));

        assert_eq!(Some(Particle::from(Sand::new())), g.get_cell(0).particle);
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
            *g.get_cells()
        );
    }

    #[test]
    fn test_grid_despawn_particle_empties_the_cell_particle() {
        let mut g = Grid::new(1, 1);
        g.spawn_particle((0, 0), Particle::from(Sand::new()));
        assert_eq!(Cell::new(Particle::from(Sand::new())), *g.get_cell(0));

        g.despawn_particle((0, 0));
        assert_eq!(Cell::empty(), *g.get_cell(0));
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
            *g.get_cells()
        );
    }

    #[test]
    fn test_grid_despawn_particle_to_grid_despawns_with_current_cycle() {
        let mut g = Grid::new(1, 1);
        g.spawn_particle((0, 0), Particle::from(Sand::new()));

        assert_eq!(
            vec![Cell::new(Particle::from(Sand::new())),],
            *g.get_cells()
        );

        g.update_grid();

        g.despawn_particle((0, 0));

        assert_eq!(vec![Cell::empty().with_cycle(1)], *g.get_cells());
    }

    #[test]
    fn test_spawn_particles_brush_sets_a_random_seed_to_particles() {
        let mut g = Grid::new(1, 1).with_rand_seed(|_| 255);
        (0..255).for_each(|_| g.update_grid());
        g.spawn_brush((0, 0), 1, Some(&ParticleKind::from(Sand::new())));
        assert_eq!(
            vec![Cell::new(Particle::from(Sand::new()).with_seed(254)).with_cycle(255),],
            *g.get_cells()
        );
    }

    #[test]
    fn test_spawn_particles_brush_sets_initial_velocity_to_particles() {
        let mut g = Grid::new(1, 1)
            .with_rand_seed(|_| 255)
            .with_initial_particle_velocity((111, 222));

        g.spawn_brush((0, 0), 1, Some(&ParticleKind::from(Sand::new())));

        let particle = Particle::from(Sand::new());
        assert_eq!(
            vec![Cell::new(particle.clone().with_velocity((111, 222)))],
            *g.get_cells()
        );
    }

    #[test]
    fn test_spawn_particles_brush_size_one() {
        /*
         * ---
         * -s-
         * ---
         */
        let mut g = Grid::new(3, 3).with_rand_seed(|_| 255);
        g.spawn_brush((1, 1), 1, Some(&ParticleKind::from(Sand::new())));

        let particle = Particle::from(Sand::new());
        assert_eq!(
            vec![
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::new(particle.clone()),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_spawn_particles_brush_size_two() {
        /*
         * -s-
         * sss
         * -s-
         */
        let mut g = Grid::new(3, 3).with_rand_seed(|_| 255);
        g.spawn_brush((1, 1), 2, Some(&ParticleKind::from(Sand::new())));

        let particle = Particle::from(Sand::new());
        assert_eq!(
            vec![
                Cell::empty(),
                Cell::new(particle.clone()),
                Cell::empty(),
                Cell::new(particle.clone()),
                Cell::new(particle.clone()),
                Cell::new(particle.clone()),
                Cell::empty(),
                Cell::new(particle.clone()),
                Cell::empty(),
            ],
            *g.get_cells()
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
        let mut g = Grid::new(5, 5).with_rand_seed(|_| 255);
        g.spawn_brush((2, 2), 4, Some(&ParticleKind::from(Sand::new())));

        let particle = Particle::from(Sand::new());
        assert_eq!(
            vec![
                Cell::empty(),
                Cell::empty(),
                Cell::new(particle.clone()),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::new(particle.clone()),
                Cell::new(particle.clone()),
                Cell::new(particle.clone()),
                Cell::empty(),
                Cell::new(particle.clone()),
                Cell::new(particle.clone()),
                Cell::new(particle.clone()),
                Cell::new(particle.clone()),
                Cell::new(particle.clone()),
                Cell::empty(),
                Cell::new(particle.clone()),
                Cell::new(particle.clone()),
                Cell::new(particle.clone()),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::new(particle.clone()),
                Cell::empty(),
                Cell::empty(),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_despawn_particles_brush() {
        /*
         * --- -> -s- -> ---
         * ---    sss    ---
         * ---    -s-    ---
         */
        let mut g = Grid::new(3, 3).with_rand_seed(|_| 255);

        g.spawn_brush((1, 1), 2, Some(&ParticleKind::from(Sand::new())));

        let particle = Particle::from(Sand::new());
        assert_eq!(
            vec![
                Cell::empty(),
                Cell::new(particle.clone()),
                Cell::empty(),
                Cell::new(particle.clone()),
                Cell::new(particle.clone()),
                Cell::new(particle.clone()),
                Cell::empty(),
                Cell::new(particle.clone()),
                Cell::empty(),
            ],
            *g.get_cells()
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
            *g.get_cells()
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
        g.despawn_particle((0, 0));
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

        assert_color_srgb_eq!(
            Color::hsva(126.00, 1.0, 0.9, 1.00),
            Particle::from(Acid::new()).color()
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
    fn test_particles_can_have_10_percent_color_value_variation() {
        assert_color_srgb_eq!(
            Color::hsva(43.20, 0.34, 0.71, 1.00),
            Particle::from(Sand::new()).with_seed(0).color()
        );
        assert_color_srgb_eq!(
            Color::hsva(43.20, 0.34, 0.76, 1.00),
            Particle::from(Sand::new()).with_seed(127).color()
        );
        assert_color_srgb_eq!(
            Color::hsva(43.20, 0.34, 0.81, 1.00),
            Particle::from(Sand::new()).with_seed(255).color()
        );
    }

    #[test]
    fn test_cell_string_names() {
        assert_eq!("-", Cell::empty().to_string());
        assert_eq!("s", Cell::new(Particle::from(Sand::new())).to_string());
        assert_eq!("S", Cell::new(Particle::from(Salt::new())).to_string());
        assert_eq!("w", Cell::new(Particle::from(Water::new())).to_string());
        assert_eq!("r", Cell::new(Particle::from(Rock::new())).to_string());
        assert_eq!("d", Cell::new(Particle::from(Drain::new())).to_string());
        assert_eq!("t", Cell::new(Particle::from(Tap::new())).to_string());
        assert_eq!("a", Cell::new(Particle::from(Acid::new())).to_string());
    }

    #[test]
    fn test_the_current_cycle_affects_half_of_the_particle_seed_value() {
        let mut g = Grid::new(1, 1).with_rand_seed(|_| 0);
        assert_eq!(0, g.particle_seed());

        let mut g = Grid::new(1, 1).with_rand_seed(|_| 255);
        assert_eq!(127, g.particle_seed());

        let mut g = Grid::new(1, 1).with_rand_seed(|_| 0);
        (0..255).for_each(|_| g.update_grid());
        assert_eq!(127, g.particle_seed());

        let mut g = Grid::new(1, 1).with_rand_seed(|_| 255);
        (0..255).for_each(|_| g.update_grid());
        assert_eq!(254, g.particle_seed());

        let mut g = Grid::new(1, 1).with_rand_seed(|_| 0);
        (0..256).for_each(|_| g.update_grid());
        assert_eq!(0, g.particle_seed(), "cycle is moduloed by 256");
    }

    #[test]
    fn test_clear_grid_sets_all_cells_to_empty() {
        /*
         *  ss -> --
         *  ss    --
         */
        let mut g = Grid::new(2, 2);
        g.spawn_particle((0, 0), Particle::from(Sand::new()));
        g.spawn_particle((1, 0), Particle::from(Sand::new()));
        g.spawn_particle((0, 1), Particle::from(Sand::new()));
        g.spawn_particle((1, 1), Particle::from(Sand::new()));

        assert_eq!(
            vec![
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())),
                Cell::new(Particle::from(Sand::new())),
            ],
            *g.get_cells()
        );

        g.update_grid();
        g.clear_grid();

        assert_eq!(
            vec![
                Cell::empty().with_cycle(1),
                Cell::empty().with_cycle(1),
                Cell::empty().with_cycle(1),
                Cell::empty().with_cycle(1)
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_grid_update_cycle_overflows_and_wraps_to_zero() {
        let mut g = Grid::new(2, 2);
        g.cycle = u32::MAX;

        g.update_grid();

        assert_eq!(0, g.cycle);
    }
}

#[cfg(test)]
mod random {
    use crate::component::grid::RowUpdateDirection;

    use super::Random;

    const TEST_ITERATIONS: i32 = 100000;

    #[test]
    fn test_random_particle_horizontal_velocity() {
        let mut r = Random::new();
        for _ in 0..TEST_ITERATIONS {
            let sample = (r.horizontal_velocity_probability)(&mut r);
            assert!(
                (i16::MIN..=i16::MAX).contains(&sample),
                "sample {:?} not in range",
                sample
            );
        }
    }

    #[test]
    fn test_random_particle_vertical_velocity() {
        let mut r = Random::new();
        for _ in 0..TEST_ITERATIONS {
            let sample = (r.vertical_velocity_probability)(&mut r);
            assert!(
                (0..=i16::MAX).contains(&sample),
                "sample {} not in range",
                sample
            );
        }
    }

    #[test]
    fn test_random_row_update_direction() {
        let mut r = Random::new();
        for _ in 0..TEST_ITERATIONS {
            let sample = (r.row_update_direction)(&mut r);
            assert!(
                [RowUpdateDirection::Forward, RowUpdateDirection::Reverse].contains(&sample),
                "sample {:?} not in range",
                sample
            );
        }
    }

    #[test]
    fn test_random_particle_seed() {
        let mut r = Random::new();
        for _ in 0..TEST_ITERATIONS {
            let sample = (r.particle_seed)(&mut r);
            assert!(
                (0..=u8::MAX).contains(&sample),
                "sample {:?} not in range",
                sample
            );
        }
    }

    #[test]
    fn test_random_particle_seed_with_cycle() {
        let mut r = Random::new();
        r.cycle = 0;
        for _ in 0..TEST_ITERATIONS {
            let sample = (r.particle_seed_with_cycle)(&mut r);
            assert!(
                (0..=u8::MAX / 2).contains(&sample),
                "sample {:?} not in range",
                sample
            );
        }
    }
}

#[cfg(test)]
mod windowing {

    use pretty_assertions::assert_eq;

    use crate::component::{
        grid::{Cell, Grid, GridAccess, Window},
        particles::{particle::Particle, rock::Rock, sand::Sand},
    };

    #[test]
    fn test_grid_set_default_window_to_the_whole_grid() {
        let g = Grid::new(3, 3);

        assert_eq!(vec![Window::new((0, 0), (2, 2))], g.window_grid.windows);
    }

    #[test]
    fn test_grid_split_windows_by_the_window_size() {
        let g = Grid::new(4, 4).with_window_size((2, 2));

        assert_eq!(
            vec![
                Window::new((0, 0), (1, 1)),
                Window::new((2, 0), (3, 1)),
                Window::new((0, 2), (1, 3)),
                Window::new((2, 2), (3, 3)),
            ],
            g.window_grid.windows
        );
    }

    #[test]
    fn test_grid_split_windows_by_the_window_size_into_rectangles() {
        let g = Grid::new(6, 4).with_window_size((3, 2));

        assert_eq!(
            vec![
                Window::new((0, 0), (2, 1)),
                Window::new((3, 0), (5, 1)),
                Window::new((0, 2), (2, 3)),
                Window::new((3, 2), (5, 3)),
            ],
            g.window_grid.windows
        );
    }

    #[test]
    fn test_get_mutable_particle_window_from_position() {
        let mut g = Grid::new(4, 4).with_window_size((2, 2));

        assert_eq!(
            Some(&mut Window::new((0, 0), (1, 1))),
            g.window_grid.get_window_mut((0, 0))
        );
        assert_eq!(
            Some(&mut Window::new((0, 0), (1, 1))),
            g.window_grid.get_window_mut((1, 0))
        );
        assert_eq!(
            Some(&mut Window::new((0, 0), (1, 1))),
            g.window_grid.get_window_mut((0, 1))
        );
        assert_eq!(
            Some(&mut Window::new((0, 0), (1, 1))),
            g.window_grid.get_window_mut((1, 1))
        );

        assert_eq!(
            Some(&mut Window::new((2, 2), (3, 3))),
            g.window_grid.get_window_mut((2, 2))
        );
        assert_eq!(
            Some(&mut Window::new((2, 2), (3, 3))),
            g.window_grid.get_window_mut((3, 2))
        );
        assert_eq!(
            Some(&mut Window::new((2, 2), (3, 3))),
            g.window_grid.get_window_mut((2, 3))
        );
        assert_eq!(
            Some(&mut Window::new((2, 2), (3, 3))),
            g.window_grid.get_window_mut((3, 3))
        );
    }

    #[test]
    fn test_spawning_particle_in_grid_sets_the_window_as_active() {
        let mut g = Grid::new(4, 4).with_window_size((2, 2));

        assert_eq!(
            vec![false, false, false, false],
            g.window_grid
                .windows
                .iter()
                .map(|w| w.is_active())
                .collect::<Vec<_>>()
        );

        g.spawn_particle((0, 0), Particle::from(Rock::new()));

        g.spawn_particle((3, 3), Particle::from(Rock::new()));

        assert_eq!(
            vec![true, false, false, true],
            g.window_grid
                .windows
                .iter()
                .map(|w| w.is_active())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_despawning_particle_in_grid_sets_the_window_as_active() {
        let mut g = Grid::new(4, 4).with_window_size((2, 2));

        g.spawn_particle((0, 0), Particle::from(Rock::new()));

        assert_eq!(
            vec![true, false, false, false],
            g.window_grid
                .windows
                .iter()
                .map(|w| w.is_active())
                .collect::<Vec<_>>()
        );

        g.update_grid();

        assert_eq!(
            vec![false, false, false, false],
            g.window_grid
                .windows
                .iter()
                .map(|w| w.is_active())
                .collect::<Vec<_>>()
        );

        g.despawn_particle((0, 0));

        assert_eq!(
            vec![true, false, false, false],
            g.window_grid
                .windows
                .iter()
                .map(|w| w.is_active())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_mark_window_as_deactive_when_nothing_changes_in_that_window() {
        let mut g = Grid::new(4, 4).with_window_size((2, 2));

        g.spawn_particle((0, 0), Particle::from(Rock::new()));

        assert_eq!(
            vec![true, false, false, false],
            g.window_grid
                .windows
                .iter()
                .map(|w| w.is_active())
                .collect::<Vec<_>>()
        );

        g.update_grid();

        assert_eq!(
            vec![false, false, false, false],
            g.window_grid
                .windows
                .iter()
                .map(|w| w.is_active())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_mark_window_as_active_when_swap_particles() {
        let mut g = Grid::new(4, 4)
            .with_initial_particle_velocity((0, 0))
            .with_window_size((2, 2))
            .with_rand_vertical_velocity_probability(|_| 0);

        g.spawn_particle((0, 0), Particle::from(Sand::new()).with_velocity((0, 0)));

        assert_eq!(
            vec![true, false, false, false],
            g.window_grid
                .windows
                .iter()
                .map(|w| w.is_active())
                .collect::<Vec<_>>()
        );

        g.update_grid();

        assert_eq!(
            vec![true, false, true, false],
            g.window_grid
                .windows
                .iter()
                .map(|w| w.is_active())
                .collect::<Vec<_>>(),
            "also activates the neighboring bottom window"
        );

        g.update_grid();

        assert_eq!(
            vec![true, false, true, false],
            g.window_grid
                .windows
                .iter()
                .map(|w| w.is_active())
                .collect::<Vec<_>>()
        );

        g.update_grid();

        assert_eq!(
            vec![true, false, true, false],
            g.window_grid
                .windows
                .iter()
                .map(|w| w.is_active())
                .collect::<Vec<_>>(),
            "also activates the neighboring top window"
        );

        for _ in 0..4 {
            g.update_grid(); // drain velocity
        }

        assert_eq!(
            vec![false, false, false, false],
            g.window_grid
                .windows
                .iter()
                .map(|w| w.is_active())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_update_particle_on_second_window() {
        let mut g = Grid::new(4, 2)
            .with_window_size((2, 2))
            .with_rand_vertical_velocity_probability(|_| 0);

        g.spawn_particle((3, 0), Particle::from(Sand::new()));
        g.update_grid();

        assert_eq!(
            vec![
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty().with_cycle(1),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::new(Particle::from(Sand::new())).with_cycle(1),
            ],
            *g.get_cells()
        );
    }

    #[test]
    fn test_activate_window_should_activate_neighboring_windows() {
        let mut g = Grid::new(3, 3)
            .with_window_size((1, 1))
            .with_rand_vertical_velocity_probability(|_| 0);

        assert_eq!(
            vec![
                false, false, false, false, false, false, false, false, false
            ],
            g.window_grid
                .windows
                .iter()
                .map(|w| w.is_active())
                .collect::<Vec<_>>()
        );

        g.spawn_particle((1, 1), Particle::from(Sand::new()));

        assert_eq!(
            vec![true, true, true, true, true, true, true, true, true],
            g.window_grid
                .windows
                .iter()
                .map(|w| w.is_active())
                .collect::<Vec<_>>()
        );
    }
}
