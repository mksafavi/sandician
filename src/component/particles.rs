use super::grid::{GridAccess, ParticleHorizontalDirection, ParticleOperation};
use bevy::prelude::Color;

#[derive(Clone, PartialEq, Debug)]
pub enum Particle {
    Sand,
    Water,
    Salt,
}

impl Particle {
    pub fn find_particle_next_location<T: GridAccess>(
        &self,
        grid: &T,
        x: usize,
        y: usize,
    ) -> Option<ParticleOperation> {
        match self {
            Particle::Sand => Particle::find_sand_particle_next_location(grid, (x, y)),
            Particle::Water => Particle::find_water_particle_next_location(grid, (x, y)),
            Particle::Salt => Particle::find_salt_particle_next_location(grid, (x, y)),
        }
    }

    fn find_sand_particle_next_location<T: GridAccess>(
        grid: &T,
        position: (usize, usize),
    ) -> Option<ParticleOperation> {
        let index_bottom = match grid.get_neighbor_index(position, (0, 1)) {
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

        let index_bottom_right = match grid.get_neighbor_index(position, (1, 1)) {
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

        let index_bottom_left = match grid.get_neighbor_index(position, (-1, 1)) {
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

        index.map(ParticleOperation::Swap)
    }

    fn find_water_particle_next_location<T: GridAccess>(
        grid: &T,
        position: (usize, usize),
    ) -> Option<ParticleOperation> {
        let index_bottom = match grid.get_neighbor_index(position, (0, 1)) {
            Ok(i) => match grid.get_cell(i) {
                Some(_) => None,
                None => Some(i),
            },
            Err(_) => None,
        };

        let index_right = match grid.get_neighbor_index(position, (1, 0)) {
            Ok(i) => match grid.get_cell(i) {
                Some(_) => None,
                None => Some(i),
            },
            Err(_) => None,
        };

        let index_left = match grid.get_neighbor_index(position, (-1, 0)) {
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

        index.map(ParticleOperation::Swap)
    }

    fn find_salt_particle_next_location<T: GridAccess>(
        grid: &T,
        position: (usize, usize),
    ) -> Option<ParticleOperation> {
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

    pub fn color(&self) -> Color {
        match self {
            Particle::Sand => Color::srgb(0.76, 0.70, 0.50),
            Particle::Water => Color::srgb(0.05, 0.53, 0.80),
            Particle::Salt => Color::srgb(1.00, 1.00, 1.00),
        }
    }
}
