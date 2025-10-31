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

impl From<Sand> for Particle {
    fn from(sand: Sand) -> Self {
        Self::Sand(sand)
    }
}

impl From<Salt> for Particle {
    fn from(salt: Salt) -> Self {
        Self::Salt(salt)
    }
}

impl From<Water> for Particle {
    fn from(water: Water) -> Self {
        Self::Water(water)
    }
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

    fn weight(&self) -> u8 {
        match self {
            Particle::Sand(sand) => sand.weight,
            Particle::Water(water) => water.weight,
            Particle::Salt(salt) => salt.weight,
            Particle::Rock => u8::MAX,
        }
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
    let index = grid.to_index(position);
    let weight = match &grid.get_cell(index).particle {
        Some(p) => p.weight(),
        None => u8::MAX,
    };

    let bottom = if let Ok(index_n) = grid.get_neighbor_index(position, (0, 1)) {
        (
            match &grid.get_cell(index_n).particle {
                Some(p) => {
                    if !grid.is_simulated(grid.get_cell(index_n)) {
                        p.weight()
                    } else {
                        u8::MAX
                    }
                }
                None => u8::MIN,
            },
            index_n,
        )
    } else {
        (u8::MAX, 0)
    };

    if bottom.0 < weight {
        grid.swap_particles(index, bottom.1);
        return true;
    }

    let bottom_left = if let Ok(index_n) = grid.get_neighbor_index(position, (-1, 1)) {
        let w = match &grid.get_cell(index_n).particle {
            Some(p) => p.weight(),
            None => u8::MIN,
        };
        if w < weight { Some(index_n) } else { None }
    } else {
        None
    };

    let bottom_right = if let Ok(index_n) = grid.get_neighbor_index(position, (1, 1)) {
        let w = match &grid.get_cell(index_n).particle {
            Some(p) => p.weight(),
            None => u8::MIN,
        };
        if w < weight { Some(index_n) } else { None }
    } else {
        None
    };

    if let Some(index_n) = match (bottom_left, bottom_right) {
        (None, None) => None,
        (None, Some(r)) => Some(r),
        (Some(l), None) => Some(l),
        (Some(l), Some(r)) => match grid.particle_direction() {
            ParticleHorizontalDirection::Left => Some(l),
            ParticleHorizontalDirection::Right => Some(r),
        },
    } {
        if !grid.is_simulated(grid.get_cell(index_n)) {
            grid.swap_particles(index, index_n);
            true
        } else {
            false
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_string_names() {
        assert_eq!("sand", Particle::from(Sand::new()).to_string());
        assert_eq!("salt", Particle::from(Salt::new()).to_string());
        assert_eq!("water", Particle::from(Water::new()).to_string());
        assert_eq!("rock", Particle::Rock.to_string());
    }
}
