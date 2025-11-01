use core::fmt;

use bevy::prelude::Color;

use crate::component::grid::{GridAccess, ParticleHorizontalDirection};

use super::{drain::Drain, salt::Salt, sand::Sand, tap::Tap, water::Water};

#[derive(Clone, PartialEq, Debug)]
pub enum Particle {
    Sand(Sand),
    Water(Water),
    Salt(Salt),
    Rock,
    Drain(Drain),
    Tap(Tap),
}

pub trait Updatable {
    fn update<T: GridAccess>(&mut self, grid: &mut T, position: (usize, usize));
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

impl From<Drain> for Particle {
    fn from(drain: Drain) -> Self {
        Self::Drain(drain)
    }
}

impl From<Tap> for Particle {
    fn from(tap: Tap) -> Self {
        Self::Tap(tap)
    }
}

impl Particle {
    pub fn update<T: GridAccess>(&mut self, grid: &mut T, position: (usize, usize)) {
        match self {
            Particle::Sand(sand) => sand.update(grid, position),
            Particle::Water(water) => water.update(grid, position),
            Particle::Salt(salt) => salt.update(grid, position),
            Particle::Rock => (),
            Particle::Drain(drain) => drain.update(grid, position),
            Particle::Tap(tap) => tap.update(grid, position),
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
            Particle::Drain(..) => Color::hsva(0.0, 0.0, 0.10, 1.00),
            Particle::Tap(..) => Color::hsva(190.0, 0.4, 0.75, 1.00),
        }
    }

    fn weight(&self) -> u8 {
        match self {
            Particle::Sand(sand) => sand.weight,
            Particle::Water(water) => water.weight,
            Particle::Salt(salt) => salt.weight,
            Particle::Rock => u8::MAX,
            Particle::Drain(..) => u8::MAX,
            Particle::Tap(..) => u8::MAX,
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
            Particle::Drain(..) => "drain",
            Particle::Tap(..) => "tap",
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

    if let Ok(index_n) = grid.get_neighbor_index(position, (0, 1)) {
        let cell = grid.get_cell(index_n);
        match &cell.particle {
            Some(p) => {
                if !grid.is_simulated(cell) && p.weight() < weight {
                    grid.swap_particles(index, index_n);
                    return true;
                }
            }
            None => {
                grid.swap_particles(index, index_n);
                return true;
            }
        };
    }

    let bottom_left = match grid.get_neighbor_index(position, (-1, 1)) {
        Ok(index_n) => match &grid.get_cell(index_n).particle {
            Some(p) => {
                if p.weight() < weight {
                    Some(index_n)
                } else {
                    None
                }
            }
            None => Some(index_n),
        },
        Err(_) => None,
    };

    let bottom_right = match grid.get_neighbor_index(position, (1, 1)) {
        Ok(index_n) => match &grid.get_cell(index_n).particle {
            Some(p) => {
                if p.weight() < weight {
                    Some(index_n)
                } else {
                    None
                }
            }
            None => Some(index_n),
        },
        Err(_) => None,
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
        grid.swap_particles(index, index_n);
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
        assert_eq!("sand", Particle::from(Sand::new()).to_string());
        assert_eq!("salt", Particle::from(Salt::new()).to_string());
        assert_eq!("water", Particle::from(Water::new()).to_string());
        assert_eq!("rock", Particle::Rock.to_string());
        assert_eq!("drain", Particle::from(Drain::new()).to_string());
        assert_eq!("tap", Particle::from(Tap::new()).to_string());
    }
}
