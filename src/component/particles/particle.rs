use core::fmt;

use bevy::prelude::Color;

use crate::component::grid::{GridAccess, ParticleHorizontalDirection};

use super::{salt::Salt, sand::Sand, water::Water};

#[derive(Clone, PartialEq, Debug)]
pub enum Particle {
    Sand(Sand),
    Water(Water),
    Salt(Salt),
    Rock,
}

pub trait Updatable {
    fn update<T: GridAccess>(&self, grid: &mut T, position: (usize, usize));
}

impl Particle {
    pub fn update<T: GridAccess>(&self, grid: &mut T, position: (usize, usize)) {
        match self {
            Particle::Sand(sand) => sand.update(grid, position),
            Particle::Water(water) => water.update(grid, position),
            Particle::Salt(salt) => salt.update(grid, position),
            Particle::Rock => (),
        };
    }

    pub fn color(&self) -> Color {
        match self {
            Particle::Sand(..) => Color::hsva(43.20, 0.34, 0.76, 1.00),
            Particle::Water(water) => match water.solute {
                0 => Color::hsva(201.60, 0.60, 0.80, 1.00),
                _ => Color::hsva(201.60, 1.00, 0.80, 1.00),
            },
            Particle::Salt(..) => Color::hsva(0.00, 0.00, 1.00, 1.00),
            Particle::Rock => Color::hsva(28.0, 0.25, 0.30, 1.00),
        }
    }

    pub fn new_sand() -> Particle {
        Particle::Sand(Sand::new())
    }

    pub fn new_salt() -> Particle {
        Particle::Salt(Salt::new())
    }

    pub fn new_water() -> Particle {
        Particle::Water(Water::new())
    }

    pub fn new_water_with_solute(solute: u8) -> Particle {
        Particle::Water(Water::new_with_solute(solute))
    }
}

impl fmt::Display for Particle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Particle::Sand(..) => "sand",
            Particle::Water(..) => "water",
            Particle::Salt(..) => "salt",
            Particle::Rock => "rock",
        };
        write!(f, "{s}")
    }
}

pub fn gravity<T: GridAccess>(grid: &mut T, position: (usize, usize)) -> bool {
    if let Some(index) = grid.is_empty(position, (0, 1)) {
        grid.swap_particles(grid.to_index(position), index);
        return true;
    }

    if let Some(index) = match (
        grid.is_empty(position, (-1, 1)),
        grid.is_empty(position, (1, 1)),
    ) {
        (None, None) => None,
        (None, Some(r)) => Some(r),
        (Some(l), None) => Some(l),
        (Some(l), Some(r)) => match grid.particle_direction() {
            ParticleHorizontalDirection::Left => Some(l),
            ParticleHorizontalDirection::Right => Some(r),
        },
    } {
        grid.swap_particles(grid.to_index(position), index);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_string_names() {
        assert_eq!("sand", Particle::new_sand().to_string());
        assert_eq!("salt", Particle::new_salt().to_string());
        assert_eq!("water", Particle::new_water().to_string());
        assert_eq!("rock", Particle::Rock.to_string());
    }
}
