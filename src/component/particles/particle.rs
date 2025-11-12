use core::fmt;

use bevy::prelude::Color;

use crate::component::grid::{GridAccess, ParticleHorizontalDirection};

use super::{drain::Drain, rock::Rock, salt::Salt, sand::Sand, tap::Tap, water::Water};

#[derive(Clone, PartialEq, Debug)]
pub enum Particle {
    Sand(Sand),
    Water(Water),
    Salt(Salt),
    Rock(Rock),
    Drain(Drain),
    Tap(Tap),
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

impl From<Rock> for Particle {
    fn from(rock: Rock) -> Self {
        Self::Rock(rock)
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
    pub fn update<T: GridAccess>(&self, grid: &mut T, position: (usize, usize)) {
        if gravity(grid, position) {
            return;
        }
        if flow(grid, position) {
            return;
        }
        match self {
            Particle::Sand(..) => (),
            Particle::Water(water) => water.update(grid, position),
            Particle::Salt(..) => (),
            Particle::Rock(..) => (),
            Particle::Drain(drain) => drain.update(grid, position),
            Particle::Tap(tap) => tap.update(grid, position),
        };
    }

    pub fn color(&self) -> Color {
        match self {
            Particle::Sand(..) => Color::hsva(43.20, 0.34, 0.76, 1.00),
            Particle::Water(water) => Color::hsva(
                201.60,
                1.0 - (3 - water.solvant_capacity) as f32 * 0.1,
                0.80,
                1.00,
            ),
            Particle::Salt(..) => Color::hsva(0.00, 0.00, 1.00, 1.00),
            Particle::Rock(..) => Color::hsva(28.0, 0.25, 0.30, 1.00),
            Particle::Drain(..) => Color::hsva(0.0, 0.0, 0.10, 1.00),
            Particle::Tap(..) => Color::hsva(190.0, 0.4, 0.75, 1.00),
        }
    }

    fn weight(&self) -> u8 {
        match self {
            Particle::Sand(sand) => sand.weight,
            Particle::Water(water) => water.weight + (3 - water.solvant_capacity),
            Particle::Salt(salt) => salt.weight,
            Particle::Rock(..) => u8::MIN,
            Particle::Drain(..) => u8::MIN,
            Particle::Tap(..) => u8::MIN,
        }
    }

    fn viscosity(&self) -> u8 {
        match self {
            Particle::Water(water) => u8::MIN + (3 - water.solvant_capacity),
            _ => u8::MAX,
        }
    }
}

impl fmt::Display for Particle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Particle::Sand(..) => "sand",
            Particle::Water(..) => "water",
            Particle::Salt(..) => "salt",
            Particle::Rock(..) => "rock",
            Particle::Drain(..) => "drain",
            Particle::Tap(..) => "tap",
        };
        write!(f, "{s}")
    }
}

fn flow<T: GridAccess>(grid: &mut T, position: (usize, usize)) -> bool {
    let index = grid.to_index(position);
    let viscosity = match &grid.get_cell(index).particle {
        Some(p) => p.viscosity(),
        None => u8::MAX,
    };

    if viscosity == u8::MAX {
        return false;
    }

    let index_left = match grid.get_neighbor_index(position, (-1, 0)) {
        Ok(i) => {
            let c = grid.get_cell(i);
            match &c.particle {
                Some(p) => {
                    if p.viscosity() < viscosity {
                        Some(i)
                    } else {
                        None
                    }
                }
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
                Some(p) => {
                    if p.viscosity() < viscosity {
                        Some(i)
                    } else {
                        None
                    }
                }
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

fn gravity<T: GridAccess>(grid: &mut T, position: (usize, usize)) -> bool {
    let index = grid.to_index(position);
    let weight = match &grid.get_cell(index).particle {
        Some(p) => p.weight(),
        None => u8::MIN,
    };

    if weight == u8::MIN {
        return false;
    }

    if let Ok(index_n) = grid.get_neighbor_index(position, (0, 1)) {
        let cell = grid.get_cell(index_n);
        match &cell.particle {
            Some(p) => {
                if !grid.is_simulated(cell) && p.weight() < weight && p.weight() != u8::MIN {
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
                if p.weight() < weight && p.weight() != u8::MIN {
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
                if p.weight() < weight && p.weight() != u8::MIN {
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
        assert_eq!("rock", Particle::from(Rock::new()).to_string());
        assert_eq!("drain", Particle::from(Drain::new()).to_string());
        assert_eq!("tap", Particle::from(Tap::new()).to_string());
    }
}
